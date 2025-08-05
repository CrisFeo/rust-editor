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
        let script = self.command.to_string();
        let mut command = Command::new("sh");
        command.arg("-c");
        command.arg(script);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let mut error = None;
        let mut results = Vec::with_capacity(buffer.current.selections.len());
        for selection in buffer.current.selections.iter() {
          let mut child = match command.spawn() {
            Ok(child) => child,
            Err(e) => {
              let msg = format!("failed to spawn child process: {:?}", e);
              error = Some(msg);
              break;
            }
          };
          let mut stdin = match child.stdin.take() {
            Some(stdin) => stdin,
            None => {
              error = Some("failed to open child process stdin".into());
              break;
            }
          };
          let input = selection.slice(&buffer.current.contents);
          let input = input.bytes().collect::<Vec<u8>>();
          if let Err(e) = stdin.write_all(&input) {
            let msg = format!("failed to write to child process stdin: {:?}", e);
            error = Some(msg);
            break;
          }
          drop(stdin);
          let output = match child.wait_with_output() {
            Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
            Err(e) => {
              let msg = format!("failed to read child process stdout: {:?}", e);
              error = Some(msg);
              break;
            }
          };
          results.push((*selection, output));
        }
        if let Some(error) = error {
          return Normal::switch_to_with_toast(error);
        }
        let mut selections = Vec::with_capacity(results.len());
        for i in 0..results.len() {
          let (selection, output) = results
            .get_mut(i)
            .expect("should be able to retrieve selection at index less than length when piping");
          let change_a = selection.apply(&mut buffer.current.contents, Op::RemoveAll);
          let change_b = selection.apply(&mut buffer.current.contents, Op::InsertStr(output));
          selections.push(*selection);
          for j in i + 1..results.len() {
            let (next_selection, _) = results
              .get_mut(j)
              .expect("should be able to retrieve selection at index less than length when adjusting selections after applying operation during pipe");
            next_selection.adjust(&buffer.current.contents, change_a);
            next_selection.adjust(&buffer.current.contents, change_b);
          }
        }
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
