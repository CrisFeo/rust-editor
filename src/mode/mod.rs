mod filter;
mod insert;
mod meta;
mod normal;
mod open;
mod pipe;
mod seek;
mod split;
mod target;
mod viewport;

pub use filter::*;
pub use insert::*;
pub use meta::*;
pub use normal::*;
pub use open::*;
pub use pipe::*;
pub use seek::*;
pub use split::*;
pub use target::*;
pub use viewport::*;

use crate::*;

pub enum UpdateCommand {
  SwitchMode(Box<dyn Mode>),
  SendKeys(Vec<Key>),
  ViewPrev,
  ViewNext,
  Open(String),
  Close,
  Quit,
}

pub struct ModeContext<'a> {
  pub toast: &'a mut Toast,
  pub buffer: &'a mut Buffer,
  pub window: &'a mut Window,
  pub registry: &'a mut Registry,
}

pub trait Mode {
  fn update(&mut self, context: ModeContext, key: Key) -> Vec<UpdateCommand>;

  fn status(&self) -> CowStr<'_>;

  fn preview_selections(&self) -> Option<&Vec<Selection>> {
    None
  }
}

#[macro_export]
macro_rules! mode {
  ($name:ident, $status: expr, $help: expr, $update:expr) => {
    pub struct $name;

    impl $name {
      pub fn switch_to() -> UpdateCommand {
        UpdateCommand::SwitchMode(Box::new($name))
      }
    }

    impl Mode for $name {
      fn update(&mut self, ctx: ModeContext, key: Key) -> Vec<UpdateCommand> {
        if matches!(key, Key::Char('?')) {
          return help($help);
        }
        $update(key, ctx)
      }

      fn status(&self) -> CowStr<'_> {
        $status.into()
      }
    }
  };
}

pub fn help(content: &str) -> Vec<UpdateCommand> {
  let content = content
    .trim()
    .replace("<", "<LT>")
    .replace(">", "<GT>")
    .replace("\n", "<RET>");
  let input = format!("uda{content}<RET><ESC>n");
  let keys = Key::from_input(&input);
  vec![
    UpdateCommand::Open("/tmp/help.txt".to_string()),
    Normal::switch_to(),
    UpdateCommand::SendKeys(keys),
  ]
}
