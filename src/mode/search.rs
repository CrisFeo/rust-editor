use crate::{
  buffer::Buffer,
  key::{Key, Modifiers},
  mode::Mode,
  selection::{Selection, Side},
  window::Window,
};
use regex::Regex;
use ropey::Rope;

#[derive(Debug, Copy, Clone)]
pub struct SearchSettings {
  pub reverse: bool,
}

pub fn update_mode_search(
  settings: SearchSettings,
  buffer: &mut Buffer,
  _window: &mut Window,
  _modifiers: Modifiers,
  key: Key,
) -> Option<Mode> {
  use crate::key::Key::*;
  if let None = buffer.command {
    buffer.command = Some(Rope::new());
  }
  match key {
    Esc => {
      buffer.command = None;
      buffer.preview_selections = None;
      return Some(Mode::Normal);
    }
    Backspace => {
      let c = &mut buffer
        .command
        .as_mut()
        .expect("command should always be set in search mode");
      let len = c.len_chars();
      if len > 0 {
        c.remove(len - 1..len);
        update_preview(settings, buffer);
      }
    }
    Enter => {
      let command = buffer
        .command
        .as_mut()
        .expect("command should always be set in search mode")
        .to_string();
      let selections = match settings.reverse {
        true => reverse(&buffer.contents, &buffer.selections, &command),
        false => forward(&buffer.contents, &buffer.selections, &command),
      };
      if let Some(selections) = selections {
        buffer.primary_selection = selections.len().saturating_sub(1);
        buffer.set_selections(selections);
      }
      buffer.command = None;
      buffer.preview_selections = None;
      return Some(Mode::Normal);
    }
    Char(ch) => {
      let c = &mut buffer
        .command
        .as_mut()
        .expect("command should always be set in search mode");
      let len = c.len_chars();
      c.insert_char(len, ch);
      update_preview(settings, buffer);
    }
    _ => {}
  }
  return Some(Mode::Search(settings));
}

fn update_preview(settings: SearchSettings, buffer: &mut Buffer) {
  let command = buffer
    .command
    .as_mut()
    .expect("command should always be set in filter mode")
    .to_string();
  let selections = match settings.reverse {
    true => reverse(&buffer.contents, &buffer.selections, &command),
    false => forward(&buffer.contents, &buffer.selections, &command),
  };
  buffer.preview_selections = selections;
}

fn forward(contents: &Rope, selections: &Vec<Selection>, pattern: &str) -> Option<Vec<Selection>> {
  if pattern.is_empty() {
    return None
  }
  let pattern = format!("(?ms){}", pattern);
  match Regex::new(&pattern) {
    Ok(regex) => {
      let mut new_selections = vec![];
      for selection in selections.iter() {
        let search_selection =
          Selection::new_at_end(selection.cursor().saturating_add(1), contents.len_chars());
        let new_selection = match search_selection.scan(&regex, &contents).first() {
          Some((start, end)) => move_cursor(*selection, *start, *end),
          None => *selection,
        };
        new_selections.push(new_selection);
      }
      Some(new_selections)
    }
    Err(_) => None,
  }
}

fn reverse(contents: &Rope, selections: &Vec<Selection>, pattern: &str) -> Option<Vec<Selection>> {
  if pattern.is_empty() {
    return None
  }
  let pattern = format!("(?ms){}", pattern);
  match Regex::new(&pattern) {
    Ok(regex) => {
      let mut new_selections = vec![];
      for selection in selections.iter() {
        let search_selection = Selection::new_at_end(0, selection.cursor().saturating_sub(1));
        let new_selection = match search_selection.scan(&regex, &contents).last() {
          Some((start, end)) => move_cursor(*selection, *start, *end),
          None => *selection,
        };
        new_selections.push(new_selection);
      }
      Some(new_selections)
    }
    Err(_) => None,
  }
}

fn move_cursor(selection: Selection, start: usize, end: usize) -> Selection {
  match selection.side() {
    Side::Start => {
      if end > selection.end() {
        Selection::new_at_end(selection.end(), end)
      } else {
        Selection::new_at_start(start, selection.end())
      }
    }
    Side::End => {
      if start < selection.start() {
        Selection::new_at_start(start, selection.start())
      } else {
        Selection::new_at_end(selection.start(), end)
      }
    }
  }
}
