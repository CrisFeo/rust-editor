use ropey::Rope;
use crate::{
  window::Window,
  mode::Mode,
  buffer::Buffer,
  regex::Regex,
  selection::Selection,
  key::{
    Key,
    Modifiers,
  },
};

pub fn update_mode_filter(
  buffer: &mut Buffer,
  _window: &mut Window,
  _modifiers: Modifiers, key: Key
) -> Option<Mode> {
  use crate::key::Key::*;
  match key {
    Esc => {
      buffer.command = Rope::new();
      return Some(Mode::Normal);
    },
    Backspace => {
      let c = &mut buffer.command;
      let len = c.len_chars();
      c.remove(len-1..len);
    },
    Enter => {
      let command = buffer.command.chars().collect::<String>();
      buffer.command = Rope::new();
      let selections = filter(&buffer.contents, &buffer.selections, &command);
      buffer.primary_selection = selections.len().saturating_sub(1);
      buffer.set_selections(selections);
      return Some(Mode::Normal);
    },
    Char(ch) => {
      let c = &mut buffer.command;
      let len = c.len_chars();
      c.insert_char(len, ch);
    },
    _          => { },
  }
  return Some(Mode::Filter);
}

fn filter(contents: &Rope, selections: &Vec<Selection>, pattern: &str) -> Vec<Selection> {
  let mut regex = Regex::new(pattern);
  let mut new_selections = vec![];
  for selection in selections.iter() {
    for (start, end) in regex.scan(&contents, selection.start(), selection.end()) {
      new_selections.push(Selection::new(start, end));
    }
  }
  new_selections
}

