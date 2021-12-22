use ropey::Rope;
use crate::{
  window::Window,
  mode::Mode,
  buffer::Buffer,
  regex::Regex,
  selection::{
    Side,
    Selection,
  },
  key::{
    Key,
    Modifiers,
  },
};

#[derive(Debug, Copy, Clone)]
pub struct SearchSettings {
  pub reverse: bool,
}

pub fn update_mode_search(
  settings: SearchSettings,
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
      let selections = match settings.reverse {
        true  => reverse(&buffer.contents, &buffer.selections, &command),
        false => forward(&buffer.contents, &buffer.selections, &command),
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
  return Some(Mode::Search(settings));
}

fn forward(contents: &Rope, selections: &Vec<Selection>, pattern: &str) -> Vec<Selection> {
  let pattern = format!("(?ms){}", pattern);
  let regex = Regex::new(&pattern).unwrap();
  let mut new_selections = vec![];
  for selection in selections.iter() {
    let search_selection = Selection::new_at_end(
      selection.cursor().saturating_add(1),
      contents.len_chars()
    );
    let new_selection = match search_selection.scan(&regex, &contents).first() {
      Some((start, end)) => move_cursor(*selection, *start, *end),
      None => *selection,
    };
    new_selections.push(new_selection);
  }
  new_selections
}

fn reverse(contents: &Rope, selections: &Vec<Selection>, pattern: &str) -> Vec<Selection> {
  let pattern = format!("(?ms){}", pattern);
  let regex = Regex::new(&pattern).unwrap();
  let mut new_selections = vec![];
  for selection in selections.iter() {
    let search_selection = Selection::new_at_end(
      0,
      selection.cursor().saturating_sub(1),
    );
    let new_selection = match search_selection.scan(&regex, &contents).last() {
      Some((start, end)) => move_cursor(*selection, *start, *end),
      None => *selection,
    };
    new_selections.push(new_selection);
  }
  new_selections
}

fn move_cursor(selection: Selection, start: usize, end: usize) -> Selection {
  match selection.side() {
    Side::Start => {
      if end > selection.end() {
        Selection::new_at_end(selection.end(), end)
      } else {
        Selection::new_at_start(start, selection.end())
      }
    },
    Side::End => {
      if start < selection.start() {
        Selection::new_at_start(start, selection.start())
      } else {
        Selection::new_at_end(selection.start(), end)
      }
    },
  }
}
