use crate::{
  buffer::Buffer,
  key::{Key, Modifiers},
  mode::Mode,
  selection::Selection,
  window::Window,
};
use regex::Regex;
use ropey::Rope;

#[derive(Debug, Copy, Clone)]
pub struct FilterSettings {
  pub reject: bool,
}

pub fn update_mode_filter(
  settings: FilterSettings,
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
        .expect("command should always be set in filter mode");
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
        .expect("command should always be set in filter mode")
        .to_string();
      let selections = match settings.reject {
        true => reject(
          &buffer.current.contents,
          &buffer.current.selections,
          &command,
        ),
        false => accept(
          &buffer.current.contents,
          &buffer.current.selections,
          &command,
        ),
      };
      if let Some(selections) = selections {
        buffer.current.primary_selection = selections.len().saturating_sub(1);
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
        .expect("command should always be set in filter mode");
      let len = c.len_chars();
      c.insert_char(len, ch);
      update_preview(settings, buffer);
    }
    _ => {}
  }
  return Some(Mode::Filter(settings));
}

fn update_preview(settings: FilterSettings, buffer: &mut Buffer) {
  let command = buffer
    .command
    .as_mut()
    .expect("command should always be set in filter mode")
    .to_string();
  let selections = match settings.reject {
    true => reject(
      &buffer.current.contents,
      &buffer.current.selections,
      &command,
    ),
    false => accept(
      &buffer.current.contents,
      &buffer.current.selections,
      &command,
    ),
  };
  buffer.preview_selections = selections;
}

fn accept(contents: &Rope, selections: &Vec<Selection>, pattern: &str) -> Option<Vec<Selection>> {
  if pattern.is_empty() {
    return None;
  }
  let pattern = format!("(?ms){}", pattern);
  match Regex::new(&pattern) {
    Ok(regex) => {
      let mut new_selections = vec![];
      for selection in selections.iter() {
        if !selection.scan(&regex, &contents).is_empty() {
          new_selections.push(*selection);
        }
      }
      Some(new_selections)
    }
    Err(_) => None,
  }
}

fn reject(contents: &Rope, selections: &Vec<Selection>, pattern: &str) -> Option<Vec<Selection>> {
  if pattern.is_empty() {
    return None;
  }
  let pattern = format!("(?ms){}", pattern);
  match Regex::new(&pattern) {
    Ok(regex) => {
      let mut new_selections = vec![];
      for selection in selections.iter() {
        if selection.scan(&regex, &contents).is_empty() {
          new_selections.push(*selection);
        }
      }
      Some(new_selections)
    }
    Err(_) => None,
  }
}
