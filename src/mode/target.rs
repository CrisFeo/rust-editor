use crate::*;
use ropey::Rope;

#[derive(Debug, Clone)]
pub struct Target {
  name: Rope,
}

impl Target {
  pub fn switch_to() -> UpdateCommand {
    let mode = Self { name: Rope::new() };
    UpdateCommand::SwitchMode(Box::new(mode))
  }
}

impl Mode for Target {
  fn update(
    &mut self,
    _buffer: &mut Buffer,
    registry: &mut Registry,
    _window: &mut Window,
    key: Key,
  ) -> UpdateCommand {
    use crate::key::Key::*;
    match key {
      Esc => return Normal::switch_to(),
      Backspace => {
        let len = self.name.len_chars();
        if len > 0 {
          self.name.remove(len - 1..len);
        }
      }
      Char(ch) => {
        let len = self.name.len_chars();
        self.name.insert_char(len, ch);
      }
      Enter => {
        let name = self.name.to_string();
        let value = Register::Content(vec![name]);
        registry.set("target", value);
        return Normal::switch_to();
      }
      _ => {}
    }
    UpdateCommand::None
  }

  fn status(&self) -> CowStr<'_> {
    format!("target > {}", self.name).into()
  }
}
