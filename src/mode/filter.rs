use crate::*;
use ropey::Rope;

#[derive(Debug, Clone)]
enum ModeResult {
  Empty,
  Error,
  Ok(Vec<Selection>),
}

#[derive(Debug, Clone)]
pub struct Filter {
  reject: bool,
  command: Rope,
  preview: ModeResult,
}

impl Filter {
  pub fn switch_to(reject: bool) -> UpdateCommand {
    let mode = Self {
      reject,
      command: Rope::new(),
      preview: ModeResult::Empty,
    };
    UpdateCommand::Switch(Box::new(mode))
  }
}

impl Mode for Filter {
  fn update(
    &mut self,
    buffer: &mut Buffer,
    _registry: &mut Registry,
    _window: &mut Window,
    modifiers: Modifiers,
    key: Key,
  ) -> UpdateCommand {
    use crate::key::Key::*;
    match key {
      Char('q') if modifiers.control => return Normal::switch_to(),
      Esc => return Normal::switch_to(),
      Backspace => {
        let len = self.command.len_chars();
        if len > 0 {
          self.command.remove(len - 1..len);
          update_preview(self, buffer);
        }
      }
      Enter => {
        let command = self.command.to_string();
        let result = match self.reject {
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
        if let ModeResult::Ok(selections) = result {
          buffer.current.primary_selection = selections.len().saturating_sub(1);
          buffer.set_selections(selections);
        }
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

  fn status<'a>(&'a self) -> CowStr<'a> {
    let match_indicator = match &self.preview {
      ModeResult::Error => "[!]",
      ModeResult::Empty => "[_]",
      ModeResult::Ok(x) if x.is_empty() => "[_]",
      ModeResult::Ok(_) => "[*]",
    };
    format!("filter {} > {}", match_indicator, self.command).into()
  }

  fn preview_selections(&self) -> Option<&Vec<Selection>> {
    match &self.preview {
      ModeResult::Error => None,
      ModeResult::Empty => None,
      ModeResult::Ok(x) if x.is_empty() => None,
      ModeResult::Ok(x) => Some(x),
    }
  }
}

fn update_preview(mode: &mut Filter, buffer: &Buffer) {
  let command = mode.command.to_string();
  let result = match mode.reject {
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
  mode.preview = result;
}

fn accept(contents: &Rope, selections: &[Selection], pattern: &str) -> ModeResult {
  if pattern.is_empty() {
    return ModeResult::Empty;
  }
  let Some(regex) = Regex::new(pattern) else {
    return ModeResult::Error;
  };
  let mut new_selections = vec![];
  for selection in selections.iter() {
    let result = regex
      .find(contents, selection.start(), selection.end())
      .next();
    if result.is_some() {
      new_selections.push(*selection);
    }
  }
  ModeResult::Ok(new_selections)
}

fn reject(contents: &Rope, selections: &[Selection], pattern: &str) -> ModeResult {
  if pattern.is_empty() {
    return ModeResult::Empty;
  }
  let Some(regex) = Regex::new(pattern) else {
    return ModeResult::Error;
  };
  let mut new_selections = vec![];
  for selection in selections.iter() {
    let result = regex
      .find(contents, selection.start(), selection.end())
      .next();
    if result.is_none() {
      new_selections.push(*selection);
    }
  }
  ModeResult::Ok(new_selections)
}
