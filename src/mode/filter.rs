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
  editor: MiniEditor,
  preview: ModeResult,
}

impl Filter {
  pub fn switch_to(reject: bool) -> UpdateCommand {
    let mode = Self {
      reject,
      editor: Default::default(),
      preview: ModeResult::Empty,
    };
    UpdateCommand::SwitchMode(Box::new(mode))
  }
}

impl Mode for Filter {
  fn update(
    &mut self,
    buffer: &mut Buffer,
    _registry: &mut Registry,
    _window: &mut Window,
    key: Key,
  ) -> Vec<UpdateCommand> {
    match self.editor.update(key) {
      MiniEditorCommand::Cancel => return vec![Normal::switch_to()],
      MiniEditorCommand::Update => update_preview(self, buffer),
      MiniEditorCommand::Submit => {
        let command = self.editor.value.to_string();
        let result = match self.reject {
          true => reject(&buffer.contents, &buffer.selections, &command),
          false => accept(&buffer.contents, &buffer.selections, &command),
        };
        if let ModeResult::Ok(selections) = result {
          buffer.primary_selection = selections.len().saturating_sub(1);
          buffer.set_selections(selections);
        }
        return vec![Normal::switch_to()];
      },
      MiniEditorCommand::None => { },
    }
    vec![]
  }

  fn status(&self) -> CowStr<'_> {
    let match_indicator = match &self.preview {
      ModeResult::Error => "[!]",
      ModeResult::Empty => "[_]",
      ModeResult::Ok(x) if x.is_empty() => "[_]",
      ModeResult::Ok(_) => "[*]",
    };
    format!("filter {} > {}", match_indicator, self.editor.value).into()
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
  let command = mode.editor.value.to_string();
  let result = match mode.reject {
    true => reject(&buffer.contents, &buffer.selections, &command),
    false => accept(&buffer.contents, &buffer.selections, &command),
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
