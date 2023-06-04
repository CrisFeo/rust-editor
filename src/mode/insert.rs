use crate::{
  buffer::Buffer,
  key::{Key, Modifiers},
  mode::Mode,
  selection::Op,
  window::Window,
};

pub fn update_mode_insert(
  buffer: &mut Buffer,
  _window: &mut Window,
  _modifiers: Modifiers,
  key: Key,
) -> Option<Mode> {
  use crate::key::Key::*;
  match key {
    Esc => return Some(Mode::Normal),
    Backspace => buffer.apply_operations(&[Op::Remove]),
    Tab => buffer.apply_operations(&[Op::Insert(' '), Op::Insert(' ')]),
    Enter => buffer.apply_operations(&[Op::Insert('\n')]),
    Char(ch) => buffer.apply_operations(&[Op::Insert(ch)]),
    _ => {}
  }
  return Some(Mode::Insert);
}
