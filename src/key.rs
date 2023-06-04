pub struct Modifiers {
  pub control: bool,
  pub shift: bool,
  pub alt: bool,
}

pub enum Key {
  Backspace,
  Enter,
  Left,
  Right,
  Up,
  Down,
  Home,
  End,
  PageUp,
  PageDown,
  Tab,
  BackTab,
  Delete,
  Insert,
  F(u8),
  Char(char),
  Null,
  Esc,
}
