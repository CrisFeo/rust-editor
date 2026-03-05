use crate::*;
use ropey::Rope;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Default)]
pub struct Pipe {
  editor: MiniEditor,
}

impl Pipe {
  pub fn switch_to() -> UpdateCommand {
    UpdateCommand::SwitchMode(Box::new(Self::default()))
  }
}

impl Mode for Pipe {
  fn update(
    &mut self,
    buffer: &mut Buffer,
    _registry: &mut Registry,
    _window: &mut Window,
    key: Key,
  ) -> UpdateCommand {
    match self.editor.update(key) {
      MiniEditorCommand::Cancel => return Normal::switch_to(),
      MiniEditorCommand::Submit => {
        let results = pipe_selections_thru_script(
          &self.editor.value,
          &buffer.contents,
          &buffer.selections,
        );
        let mut results = match results {
          Ok(results) => results,
          Err(error) => return Normal::switch_to_with_toast(error),
        };
        let mut selections = Vec::with_capacity(results.len());
        for i in 0..results.len() {
          let (selection, output) = results
            .get_mut(i)
            .expect("should be able to retrieve selection at index less than length when piping");
          let change_a = selection.apply_operation(&mut buffer.contents, Op::RemoveAll);
          let change_b = selection.apply_operation(&mut buffer.contents, Op::InsertStr(output));
          selections.push(*selection);
          for j in i + 1..results.len() {
            let (next_selection, _) = results
              .get_mut(j)
              .expect("should be able to retrieve selection at index less than length when adjusting selections after applying operation during pipe");
            next_selection.adjust(&buffer.contents, change_a.as_ref());
            next_selection.adjust(&buffer.contents, change_b.as_ref());
          }
          change_a.map(|c| buffer.history.record(c));
          change_b.map(|c| buffer.history.record(c));
        }
        buffer.history.commit();
        buffer.set_selections(selections);
        return Normal::switch_to();
      },
      MiniEditorCommand::Update => {},
      MiniEditorCommand::None => { },
    }
    UpdateCommand::None
  }

  fn status(&self) -> CowStr<'_> {
    format!("pipe > {}", self.editor.value).into()
  }

  fn preview_selections(&self) -> Option<&Vec<Selection>> {
    None
  }
}

fn pipe_selections_thru_script(
  script: &Rope,
  contents: &Rope,
  selections: &[Selection],
) -> Result<Vec<(Selection, String)>, String> {
  leave_controlled_terminal(&mut std::io::stdout(), true);
  let mut command = Command::new("sh");
  command.arg("-c");
  command.arg(script.to_string());
  command.stdin(Stdio::piped());
  command.stdout(Stdio::piped());
  command.stderr(Stdio::piped());
  let mut results = Vec::with_capacity(selections.len());
  for selection in selections.iter() {
    let mut child = match command.spawn() {
      Ok(child) => child,
      Err(e) => return Err(format!("failed to spawn child process: {e:?}")),
    };
    let mut stdin = match child.stdin.take() {
      Some(stdin) => stdin,
      None => return Err("failed to open child process stdin".into()),
    };
    let input = selection.slice(contents);
    let input = input.bytes().collect::<Vec<u8>>();
    if let Err(e) = stdin.write_all(&input) {
      return Err(format!("failed to write to child process stdin: {e:?}"));
    }
    drop(stdin);
    let output = match child.wait_with_output() {
      Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
      Err(e) => return Err(format!("failed to read child process stdout: {e:?}")),
    };
    results.push((*selection, output));
  }
  enter_controlled_terminal(&mut std::io::stdout());
  Ok(results)
}
