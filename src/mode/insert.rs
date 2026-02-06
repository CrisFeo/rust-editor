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
    _registry: &mut Registry,
    _window: &mut Window,
    key: Key,
  ) -> UpdateCommand {
    use crate::key::Key::*;
    match key {
      Esc => {
        buffer.history.commit();
        return Normal::switch_to();
      }
      Backspace => buffer.apply_operations(&[Op::Remove]),
      Tab => buffer.apply_operations(&[Op::InsertStr("  ")]),
      Enter => buffer.apply_operations(&[Op::InsertChar('\n')]),
      Char(ch) => buffer.apply_operations(&[Op::InsertChar(ch)]),
      _ => {}
    }
    UpdateCommand::None
  }

  fn status(&self) -> CowStr<'_> {
    "insert".into()
  }
}
