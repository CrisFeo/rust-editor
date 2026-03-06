mod filter;
mod insert;
mod normal;
mod open;
mod pipe;
mod seek;
mod split;
mod target;

pub use filter::*;
pub use insert::*;
pub use normal::*;
pub use open::*;
pub use pipe::*;
pub use seek::*;
pub use split::*;
pub use target::*;

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

pub trait Mode {
  fn update(
    &mut self,
    buffer: &mut Buffer,
    registry: &mut Registry,
    window: &mut Window,
    key: Key,
  ) -> Vec<UpdateCommand>;

  fn status(&self) -> CowStr<'_>;

  fn preview_selections(&self) -> Option<&Vec<Selection>> {
    None
  }
}
