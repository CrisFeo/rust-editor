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

#[derive(Debug, Copy, Clone)]
pub struct FilterSettings {
  pub reject: bool,
}

pub fn update_mode_filter(
  settings: FilterSettings,
  buffer: &mut Buffer,
  _window: &mut Window,
  _modifiers: Modifiers,
  key: Key
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
      let selections = match settings.reject {
        true  => reject(&buffer.contents, &buffer.selections, &command),
        false => accept(&buffer.contents, &buffer.selections, &command),
      };
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
  return Some(Mode::Filter(settings));
}

fn accept(contents: &Rope, selections: &Vec<Selection>, pattern: &str) -> Vec<Selection> {
  let pattern = format!("(?ms){}", pattern);
  let regex = Regex::new(&pattern).unwrap();
  let mut new_selections = vec![];
  for selection in selections.iter() {
    if !selection.scan(&regex, &contents).is_empty() {
      new_selections.push(*selection);
    }
  }
  new_selections
}

fn reject(contents: &Rope, selections: &Vec<Selection>, pattern: &str) -> Vec<Selection> {
  let pattern = format!("(?ms){}", pattern);
  let regex = Regex::new(&pattern).unwrap();
  let mut new_selections = vec![];
  for selection in selections.iter() {
    if selection.scan(&regex, &contents).is_empty() {
      new_selections.push(*selection);
    }
  }
  new_selections
}
