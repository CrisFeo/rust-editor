mod filter;
mod insert;
mod normal;
mod seek;
mod split;

pub use filter::*;
pub use insert::*;
pub use normal::*;
pub use seek::*;
pub use split::*;

use crate::*;

pub enum UpdateCommand {
  Switch(Box<dyn Mode>),
  Quit,
  None,
}

pub trait Mode {
  fn update(
    &mut self,
    buffer: &mut Buffer,
    window: &mut Window,
    modifiers: Modifiers,
    key: Key,
  ) -> UpdateCommand;

  fn status<'a>(&self) -> CowStr<'a>;

  fn preview_selections(&self) -> Option<&Vec<Selection>>;
}
