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

use crate::{
  buffer::Buffer,
  key::{Key, Modifiers},
  window::Window,
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
  key: Key,
) -> Option<Mode> {
  match &buffer.mode {
    Mode::Normal => update_mode_normal(buffer, window, modifiers, key),
    Mode::Insert => update_mode_insert(buffer, window, modifiers, key),
    Mode::Split(settings) => update_mode_split(*settings, buffer, window, modifiers, key),
    Mode::Search(settings) => update_mode_search(*settings, buffer, window, modifiers, key),
    Mode::Filter(settings) => update_mode_filter(*settings, buffer, window, modifiers, key),
  }
}
