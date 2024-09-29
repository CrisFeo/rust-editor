mod filter;
mod insert;
mod normal;
mod search;
mod split;

pub use filter::*;
pub use insert::*;
pub use normal::*;
pub use search::*;
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

  fn status(&self) -> String;
}
