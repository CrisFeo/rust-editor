use crate::*;
use ropey::Rope;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Default)]
pub struct Pipe {
  command: Rope,
}

impl Pipe {
  pub fn switch_to() -> UpdateCommand {
    UpdateCommand::Switch(Box::new(Self::default()))
  }
}

impl Mode for Pipe {
  fn update(
    &mut self,
    buffer: &mut Buffer,
    _registry: &mut Registry,
    _window: &mut Window,
    modifiers: Modifiers,
    key: Key,
  ) -> UpdateCommand {
    use crate::key::Key::*;
    match key {
      Char('q') if modifiers.control => return Normal::switch_to(),
      Esc => return Normal::switch_to(),
      Backspace => {
        let len = self.command.len_chars();
        if len > 0 {
          self.command.remove(len - 1..len);
        }
      }
      Char(ch) => {
        let len = self.command.len_chars();
        self.command.insert_char(len, ch);
      }
      Enter => {
        let results = pipe_selections_thru_script(
          &self.command,
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
      }
      _ => {}
    }
    UpdateCommand::None
  }

  fn status(&self) -> CowStr {
    format!("pipe > {}", self.command).into()
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
  Ok(results)
}
