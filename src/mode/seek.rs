use crate::*;
use ropey::Rope;

#[derive(Debug, Clone)]
enum ModeResult {
  Empty,
  Error,
  Same,
  Ok(Vec<Selection>),
}

#[derive(Debug, Clone)]
pub struct Seek {
  reverse: bool,
  command: Rope,
  preview: ModeResult,
}

impl Seek {
  pub fn switch_to(reverse: bool) -> UpdateCommand {
    let mode = Self {
      reverse,
      command: Rope::new(),
      preview: ModeResult::Empty,
    };
    UpdateCommand::Switch(Box::new(mode))
  }
}

impl Mode for Seek {
  fn update(
    &mut self,
    buffer: &mut Buffer,
    _window: &mut Window,
    _modifiers: Modifiers,
    key: Key,
  ) -> UpdateCommand {
    use crate::key::Key::*;
    match key {
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
        let result = match self.reverse {
          true => reverse(
            &buffer.current.contents,
            &buffer.current.selections,
            &command,
          ),
          false => forward(
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

  fn status<'a>(&self) -> CowStr<'a> {
    let match_indicator = match &self.preview {
      ModeResult::Error => "[!]",
      ModeResult::Empty => "[_]",
      ModeResult::Same => "[_]",
      ModeResult::Ok(_) => "[*]",
    };
    format!("seek {} > {}", match_indicator, self.command).into()
  }

  fn preview_selections(&self) -> Option<&Vec<Selection>> {
    match &self.preview {
      ModeResult::Error => None,
      ModeResult::Empty => None,
      ModeResult::Same => None,
      ModeResult::Ok(x) => Some(x),
    }
  }
}

fn update_preview(mode: &mut Seek, buffer: &Buffer) {
  let command = mode.command.to_string();
  let result = match mode.reverse {
    true => reverse(
      &buffer.current.contents,
      &buffer.current.selections,
      &command,
    ),
    false => forward(
      &buffer.current.contents,
      &buffer.current.selections,
      &command,
    ),
  };
  mode.preview = result;
}

fn forward(contents: &Rope, selections: &[Selection], pattern: &str) -> ModeResult {
  if pattern.is_empty() {
    return ModeResult::Empty;
  }
  let Some(regex) = Regex::new(pattern) else {
    return ModeResult::Error;
  };
  let mut changed = false;
  let mut new_selections = vec![];
  for selection in selections.iter() {
    let contents_end = contents.len_chars().saturating_sub(1);
    // begin the search from the character after the active anchor to allow
    // seeking to the next instance of the character under the cursor
    let start = selection.cursor().saturating_add(1).min(contents_end);
    let result = regex.find(contents, start, contents_end).next();
    let new_selection = match result {
      Some((start, end)) => {
        changed = true;
        move_cursor(*selection, start, end)
      }
      None => *selection,
    };
    new_selections.push(new_selection);
  }
  if changed {
    ModeResult::Ok(new_selections)
  } else {
    ModeResult::Same
  }
}

fn reverse(contents: &Rope, selections: &[Selection], pattern: &str) -> ModeResult {
  if pattern.is_empty() {
    return ModeResult::Empty;
  }
  let Some(regex) = Regex::new(pattern) else {
    return ModeResult::Error;
  };
  let mut changed = false;
  let mut new_selections = vec![];
  for selection in selections.iter() {
    // end the search a character before the active anchor to allow seeking to
    // the previous instance of the character under the cursor
    let end = selection.cursor().saturating_sub(1).max(0);
    let result = regex.find(contents, 0, end).last();
    let new_selection = match result {
      Some((start, end)) => {
        changed = true;
        move_cursor(*selection, start, end)
      }
      None => *selection,
    };
    new_selections.push(new_selection);
  }
  if changed {
    ModeResult::Ok(new_selections)
  } else {
    ModeResult::Same
  }
}

fn move_cursor(selection: Selection, start: usize, end: usize) -> Selection {
  // update the provided selection to use the new bounds taking into account
  // the seeked anchor "crossing over" the other
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
