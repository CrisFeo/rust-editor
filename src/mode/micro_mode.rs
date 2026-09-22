use crate::*;


pub struct MicroMode {
  name: &'static str,
  handler: Box<dyn for<'a> Fn(Key, ModeContext<'a>) -> Vec<UpdateCommand>>,
}

impl MicroMode {
  pub fn switch_to<'a>(name: &'static str, handler: impl Fn(Key, ModeContext<'a>) -> Vec<UpdateCommand> + 'static) -> UpdateCommand {
    let handler = Box::new(handler);
    let mode = Self {
      name,
      handler,
    };
    UpdateCommand::SwitchMode(Box::new(mode))
  }
}

impl Mode for MicroMode {
  fn update(
    &mut self,
    ctx: ModeContext,
    key: Key,
  ) -> Vec<UpdateCommand> {
    (self.handler)(key, ctx)
  }

  fn status(&self) -> CowStr<'_> {
    self.name.into()
  }
}
