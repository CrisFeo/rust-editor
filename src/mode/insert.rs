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
    modifiers: Modifiers,
    key: Key,
  ) -> UpdateCommand {
    use crate::key::Key::*;
    match key {
      Char('q') if modifiers.control => return Normal::switch_to(),
      Esc => return Normal::switch_to(),
      Backspace => buffer.apply_operations(&[Op::Remove]),
      Tab => buffer.apply_operations(&[Op::InsertChar(' '), Op::InsertChar(' ')]),
      Enter => buffer.apply_operations(&[Op::InsertChar('\n')]),
      Char(ch) => buffer.apply_operations(&[Op::InsertChar(ch)]),
      _ => {}
    }
    UpdateCommand::None
  }

  fn status<'a>(&'a self) -> CowStr<'a> {
    "insert".into()
  }

  fn preview_selections(&self) -> Option<&Vec<Selection>> {
    None
  }
}
