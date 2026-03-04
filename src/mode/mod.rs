mod filter;
mod insert;
mod normal;
mod pipe;
mod seek;
mod split;
mod target;

pub use filter::*;
pub use insert::*;
pub use normal::*;
pub use pipe::*;
pub use seek::*;
pub use split::*;
pub use target::*;

use crate::*;

pub enum UpdateCommand {
  Switch(Box<dyn Mode>),
  Macro(Vec<Key>),
  Quit,
  None,
}

pub trait Mode {
  fn update(
    &mut self,
    buffer: &mut Buffer,
    registry: &mut Registry,
    window: &mut Window,
    key: Key,
  ) -> UpdateCommand;

  fn status(&self) -> CowStr<'_>;

  fn preview_selections(&self) -> Option<&Vec<Selection>> {
    None
  }
}
