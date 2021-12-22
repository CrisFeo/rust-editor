mod normal;
mod insert;
mod split;
mod search;
mod filter;

pub use normal::*;
pub use insert::*;
pub use split::*;
pub use search::*;
pub use filter::*;

use crate::{
  window::Window,
  buffer::Buffer,
  key::{
    Key,
    Modifiers,
  },
};

#[derive(Debug, Copy, Clone)]
pub enum Mode {
  Normal,
  Insert,
  Split(SplitSettings),
  Search(SearchSettings),
  Filter(FilterSettings),
}

pub fn update_mode(
  buffer: &mut Buffer,
  window: &mut Window,
  modifiers: Modifiers,
  key: Key
) -> Option<Mode> {
  match &buffer.mode {
    Mode::Normal => update_mode_normal(buffer, window, modifiers, key),
    Mode::Insert => update_mode_insert(buffer, window, modifiers, key),
    Mode::Split(settings) => update_mode_split(*settings, buffer, window, modifiers, key),
    Mode::Search(settings) => update_mode_search(*settings, buffer, window, modifiers, key),
    Mode::Filter(settings) => update_mode_filter(*settings, buffer, window, modifiers, key),
  }
}
