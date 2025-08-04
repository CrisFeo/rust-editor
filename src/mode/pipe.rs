use crate::*;
use ropey::Rope;

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
        let command = self.command.to_string();
        // TODO pipe each selection through this shell command
        for selection in buffer.current.selections.iter() {
          let content = buffer.current.contents.slice();
        }
        buffer.apply_operations(&[Op::Collapse]);
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
