use crate::*;
use ropey::Rope;

#[derive(Debug, Clone)]
pub struct Search {
  pub command: Rope,
  pub reverse: bool,
}

impl Search {
  pub fn switch_to(reverse: bool) -> UpdateCommand {
    let mode = Self {
      command: Rope::new(),
      reverse,
    };
    UpdateCommand::Switch(Box::new(mode))
  }
}

impl Mode for Search {
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
        let selections = match self.reverse {
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
    format!("search > {}", self.command)
  }
}

fn update_preview(mode: &Search, buffer: &mut Buffer) {
  let command = mode.command.to_string();
  let selections = match mode.reverse {
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
  buffer.preview_selections = selections;
}

fn forward(contents: &Rope, selections: &[Selection], pattern: &str) -> Option<Vec<Selection>> {
  if pattern.is_empty() {
    return None;
  }
  let regex = Regex::new(pattern)?;
  let mut new_selections = vec![];
  for selection in selections.iter() {
    let contents_end = contents.len_chars().saturating_sub(1);
    let result = regex
      .find(
        contents,
        selection.cursor().saturating_add(1).min(contents_end),
        contents_end,
      )
      .next();
    let new_selection = match result {
      Some((start, end)) => move_cursor(*selection, start, end),
      None => *selection,
    };
    new_selections.push(new_selection);
  }
  Some(new_selections)
}

fn reverse(contents: &Rope, selections: &[Selection], pattern: &str) -> Option<Vec<Selection>> {
  if pattern.is_empty() {
    return None;
  }
  let regex = Regex::new(pattern)?;
  let mut new_selections = vec![];
  for selection in selections.iter() {
    let result = regex
      .find(contents, 0, selection.cursor().saturating_sub(1).max(0))
      .last();
    let new_selection = match result {
      Some((start, end)) => move_cursor(*selection, start, end),
      None => *selection,
    };
    new_selections.push(new_selection);
  }
  Some(new_selections)
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
