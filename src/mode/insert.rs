use crate::*;

pub struct Insert;

impl Insert {
  pub fn switch_to() -> UpdateCommand {
    let mode = Self;
    UpdateCommand::SwitchMode(Box::new(mode))
  }
}

impl Mode for Insert {
  fn update(&mut self, ctx: ModeContext, key: Key) -> Vec<UpdateCommand> {
    use crate::key::Key::*;
    match key {
      Esc => {
        ctx.buffer.history.commit();
        return vec![Normal::switch_to()];
      }
      Backspace => ctx.buffer.apply_operations(&[Op::Remove]),
      Tab => ctx.buffer.apply_operations(&[Op::InsertStr("  ")]),
      Enter => ctx.buffer.apply_operations(&[Op::InsertChar('\n')]),
      Char(ch) => ctx.buffer.apply_operations(&[Op::InsertChar(ch)]),
      _ => {}
    }
    vec![]
  }

  fn status(&self) -> CowStr<'_> {
    "insert".into()
  }
}
