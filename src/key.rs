use crate::*;

#[derive(Debug, Copy, Clone, PartialEq)]
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

impl Key {
  pub fn to_input(&self) -> CowStr<'static> {
    match self {
      Self::Char(char) => match char {
        '<' => "<LT>".into(),
        '>' => "<GT>".into(),
        v => format!("{v}").into(),
      },
      Self::Tab => "<TAB>".into(),
      Self::Esc => "<ESC>".into(),
      Self::Enter => "<RET>".into(),
      Self::Backspace => "<BSP>".into(),
      Self::Up => "<UP>".into(),
      Self::Down => "<DWN>".into(),
      Self::Left => "<LFT>".into(),
      Self::Right => "<RGT>".into(),
    }
  }

  pub fn from_input(input: &str) -> Vec<Self> {
    let mut keys = Vec::new();
    let mut input = input;
    fn consume_if(input: &mut &str, token: &str) -> bool {
      if input.starts_with(token) {
        *input = &input[token.len()..];
        return true;
      }
      false
    }
    while !input.is_empty() {
      if consume_if(&mut input, "<LT>") {
          keys.push(Self::Char('<'));
      } else if consume_if(&mut input, "<GT>") {
          keys.push(Self::Char('>'));
      } else if consume_if(&mut input, "<TAB>") {
        keys.push(Self::Tab);
      } else if consume_if(&mut input, "<ESC>") {
        keys.push(Self::Esc);
      } else if consume_if(&mut input, "<RET>") {
        keys.push(Self::Enter);
      } else if consume_if(&mut input, "<BSP>") {
        keys.push(Self::Backspace);
      } else if consume_if(&mut input, "<UP>") {
        keys.push(Self::Up);
      } else if consume_if(&mut input, "<DWN>") {
        keys.push(Self::Down);
      } else if consume_if(&mut input, "<LFT>") {
        keys.push(Self::Left);
      } else if consume_if(&mut input, "<RGT>") {
        keys.push(Self::Right);
      } else if let Some(char) = input.chars().next() {
        keys.push(Self::Char(char));
        input = &input[1..];
      }
    }
    keys
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn to_input() {
    assert_eq!(Key::Char('a').to_input(), "a");
    assert_eq!(Key::Char('<').to_input(), "<LT>");
    assert_eq!(Key::Char('>').to_input(), "<GT>");
    assert_eq!(Key::Tab.to_input(), "<TAB>");
    assert_eq!(Key::Esc.to_input(), "<ESC>");
    assert_eq!(Key::Enter.to_input(), "<RET>");
    assert_eq!(Key::Backspace.to_input(), "<BSP>");
    assert_eq!(Key::Up.to_input(), "<UP>");
    assert_eq!(Key::Down.to_input(), "<DWN>");
    assert_eq!(Key::Left.to_input(), "<LFT>");
    assert_eq!(Key::Right.to_input(), "<RGT>");
  }

  #[test]
  fn from_input_simple() {
    assert_eq!(Key::from_input("a"), vec![Key::Char('a')]);
    assert_eq!(Key::from_input("<LT>"), vec![Key::Char('<')]);
    assert_eq!(Key::from_input("<GT>"), vec![Key::Char('>')]);
    assert_eq!(Key::from_input("<TAB>"), vec![Key::Tab]);
    assert_eq!(Key::from_input("<ESC>"), vec![Key::Esc]);
    assert_eq!(Key::from_input("<RET>"), vec![Key::Enter]);
    assert_eq!(Key::from_input("<BSP>"), vec![Key::Backspace]);
    assert_eq!(Key::from_input("<UP>"), vec![Key::Up]);
    assert_eq!(Key::from_input("<DWN>"), vec![Key::Down]);
    assert_eq!(Key::from_input("<LFT>"), vec![Key::Left]);
    assert_eq!(Key::from_input("<RGT>"), vec![Key::Right]);
  }

  #[test]
  fn from_input_multiple() {
    assert_eq!(
      Key::from_input("ahello<RET><ESC>"),
      vec![
        Key::Char('a'),
        Key::Char('h'),
        Key::Char('e'),
        Key::Char('l'),
        Key::Char('l'),
        Key::Char('o'),
        Key::Enter,
        Key::Esc,
      ],
    );
  }

  #[test]
  fn from_input_unrecognized_key() {
    assert_eq!(
      Key::from_input("a<wut><RET>"),
      vec![
        Key::Char('a'),
        Key::Char('<'),
        Key::Char('w'),
        Key::Char('u'),
        Key::Char('t'),
        Key::Char('>'),
        Key::Enter,
      ],
    );
  }
}
