use crate::*;

type HandlerFn = dyn for<'a> Fn(Key, ModeContext<'a>) -> Vec<UpdateCommand>;

pub struct MicroMode {
  name: &'static str,
  handler: Box<HandlerFn>,
}

impl MicroMode {
  pub fn switch_to<T: Fn(Key, ModeContext<'_>) -> Vec<UpdateCommand> + 'static>(
    name: &'static str,
    handler: T,
  ) -> UpdateCommand {
    let handler = Box::new(handler);
    let mode = Self { name, handler };
    UpdateCommand::SwitchMode(Box::new(mode))
  }
}

impl Mode for MicroMode {
  fn update(&mut self, ctx: ModeContext, key: Key) -> Vec<UpdateCommand> {
    (self.handler)(key, ctx)
  }

  fn status(&self) -> CowStr<'_> {
    self.name.into()
  }
}
