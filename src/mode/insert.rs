use crate::*;

pub struct Insert;

impl Insert {
  pub fn switch_to() -> UpdateCommand {
    let mode = Self;
    UpdateCommand::Switch(Box::new(mode))
  }
}

impl Mode for Insert {
  fn update(
    &mut self,
    buffer: &mut Buffer,
    _window: &mut Window,
    _modifiers: Modifiers,
    key: Key,
  ) -> UpdateCommand {
    use crate::key::Key::*;
    match key {
      Esc => {
        buffer.push_snapshot();
        return Normal::switch_to();
      }
      Backspace => buffer.apply_operations(&[Op::Remove]),
      Tab => buffer.apply_operations(&[Op::Insert(' '), Op::Insert(' ')]),
      Enter => buffer.apply_operations(&[Op::Insert('\n')]),
      Char(ch) => buffer.apply_operations(&[Op::Insert(ch)]),
      _ => {}
    }
    UpdateCommand::None
  }

  fn status(&self) -> String {
    "insert".to_string()
  }
}
