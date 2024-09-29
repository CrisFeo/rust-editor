use crate::*;
use ropey::Rope;

#[derive(Debug, Clone)]
pub struct Filter {
  pub command: Rope,
  pub reject: bool,
}

impl Filter {
  pub fn switch_to(reject: bool) -> UpdateCommand {
    let mode = Self {
      command: Rope::new(),
      reject,
    };
    UpdateCommand::Switch(Box::new(mode))
  }
}

impl Mode for Filter {
  fn update(
    &mut self,
    buffer: &mut Buffer,
    _window: &mut Window,
    _modifiers: Modifiers,
    key: Key,
  ) -> UpdateCommand {
    use crate::key::Key::*;
    match key {
      Esc => {
        buffer.preview_selections = None;
        return Normal::switch_to();
      }
      Backspace => {
        let len = self.command.len_chars();
        if len > 0 {
          self.command.remove(len - 1..len);
          update_preview(self, buffer);
        }
      }
      Enter => {
        let command = self.command.to_string();
        let selections = match self.reject {
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
        buffer.preview_selections = None;
        return Normal::switch_to();
      }
      Char(ch) => {
        let len = self.command.len_chars();
        self.command.insert_char(len, ch);
        update_preview(self, buffer);
      }
      _ => {}
    }
    UpdateCommand::None
  }

  fn status(&self) -> String {
    format!("filter > {}", self.command)
  }
}

fn update_preview(mode: &Filter, buffer: &mut Buffer) {
  let command = mode.command.to_string();
  let selections = match mode.reject {
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

fn accept(contents: &Rope, selections: &[Selection], pattern: &str) -> Option<Vec<Selection>> {
  if pattern.is_empty() {
    return None;
  }
  let regex = Regex::new(pattern)?;
  let mut new_selections = vec![];
  for selection in selections.iter() {
    let result = regex.find(
      contents,
      selection.start(),
      selection.end(),
    ).next();
    if result.is_some() {
      new_selections.push(*selection);
    }
  }
  Some(new_selections)
}

fn reject(contents: &Rope, selections: &[Selection], pattern: &str) -> Option<Vec<Selection>> {
  if pattern.is_empty() {
    return None;
  }
  let regex = Regex::new(pattern)?;
  let mut new_selections = vec![];
  for selection in selections.iter() {
    let result = regex.find(
      contents,
      selection.start(),
      selection.end(),
    ).next();
    if result.is_none() {
      new_selections.push(*selection);
    }
  }
  Some(new_selections)
}
