use crate::{
  mode::Mode,
  window::Window,
  buffer::Buffer,
  selection::{
    Op,
    Selection,
  },
  key::{
    Key,
    Modifiers,
  },
};

pub fn update_mode_normal(
  buffer: &mut Buffer,
  window: &mut Window,
  modifiers: Modifiers, key: Key
) -> Option<Mode> {
  use crate::key::Key::*;
  match key {
    Char('q') if modifiers.control => return None,
    Char('h') => buffer.apply_operations(&[Op::MoveByChar(-1), Op::Collapse]),
    Char('j') => buffer.apply_operations(&[Op::MoveByLine(1), Op::Collapse]),
    Char('k') => buffer.apply_operations(&[Op::MoveByLine(-1), Op::Collapse]),
    Char('l') => buffer.apply_operations(&[Op::MoveByChar(1), Op::Collapse]),
    Char('H') => buffer.apply_operations(&[Op::MoveByChar(-1)]),
    Char('J') => buffer.apply_operations(&[Op::MoveByLine(1)]),
    Char('K') => buffer.apply_operations(&[Op::MoveByLine(-1)]),
    Char('L') => buffer.apply_operations(&[Op::MoveByChar(1)]),
    Char(';') => buffer.apply_operations(&[Op::Swap]),
    Char(':') => buffer.apply_operations(&[Op::Collapse]),
    Char('d') => buffer.apply_operations(&[Op::RemoveAll]),
    Char('[') => buffer.primary_selection = wrap_add(buffer.selections.len(), buffer.primary_selection, -1),
    Char(']') => buffer.primary_selection = wrap_add(buffer.selections.len(), buffer.primary_selection, 1),
    Char('\\') => buffer.set_selections(vec![*buffer.selections.get(buffer.primary_selection).unwrap()]),
    Char('|') => {
      let selections = buffer.selections
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != buffer.primary_selection)
        .map(|(_, &v)| v)
        .collect();
      buffer.set_selections(selections);
    },
    Char('f') => buffer.set_selections(vec![Selection::new(0, buffer.contents.len_chars())]),
    Char('s') => return Some(Mode::Filter),
    Char('S') => return Some(Mode::Reject),
    Char('i') => {
      buffer.apply_operations(&[Op::Collapse]);
      return Some(Mode::Insert)
      },
    Char('a') => {
      buffer.apply_operations(&[Op::MoveByChar(1), Op::Collapse]);
      return Some(Mode::Insert);
    },
    Char('c') => {
      buffer.apply_operations(&[Op::RemoveAll]);
      return Some(Mode::Insert);
    },
    Up => window.scroll_top = window.scroll_top.saturating_sub(1),
    Down => window.scroll_top = window.scroll_top.saturating_add(1),
    Left => window.scroll_left = window.scroll_left.saturating_sub(1),
    Right => window.scroll_left = window.scroll_left.saturating_add(1),
    _ => { },
  }
  return Some(Mode::Normal);
}

fn wrap_add(domain: usize, value: usize, delta: isize) -> usize {
  let value = (value as isize) + delta;
  let value = if value < 0 {
    let value = -value as usize;
    (value / domain + 1) * domain - value
  } else {
    value as usize
  };
  value % domain
}
