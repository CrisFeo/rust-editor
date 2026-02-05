pub struct Modifiers {
  pub control: bool,
  pub shift: bool,
  pub alt: bool,
}

pub enum Key {
  Char(char),
  Tab,
  Esc,
  Enter,
  Backspace,
  Up,
  Down,
  Left,
  Right,
}
