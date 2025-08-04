mod filter;
mod insert;
mod normal;
mod pipe;
mod seek;
mod split;

pub use filter::*;
pub use insert::*;
pub use normal::*;
pub use pipe::*;
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
    registry: &mut Registry,
    window: &mut Window,
    modifiers: Modifiers,
    key: Key,
  ) -> UpdateCommand;

  fn status(&self) -> CowStr;

  fn preview_selections(&self) -> Option<&Vec<Selection>> {
    None
  }
}
