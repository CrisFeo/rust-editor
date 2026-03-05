use crate::*;

#[derive(Default)]
pub struct Normal {
  toast: Option<String>,
}

impl Normal {
  pub fn switch_to() -> UpdateCommand {
    UpdateCommand::SwitchMode(Box::new(Self::default()))
  }

  pub fn switch_to_with_toast(toast: impl Into<String>) -> UpdateCommand {
    UpdateCommand::SwitchMode(Box::new(Self {
      toast: Some(toast.into()),
    }))
  }
}

impl Mode for Normal {
  fn update(
    &mut self,
    buffer: &mut Buffer,
    registry: &mut Registry,
    window: &mut Window,
    key: Key,
  ) -> Vec<UpdateCommand> {
    use crate::key::Key::*;
    self.toast = None;
    match key {
      // Meta actions
      Char('Q') => return vec![UpdateCommand::Quit],
      Char('W') => {
        if buffer.filename.is_some() {
          self.toast = if buffer.save() {
            Some("file saved!".into())
          } else {
            Some("error: could not save file".into())
          };
        } else {
          self.toast = Some("scratch buffers cannot be saved".into());
        }
      }
      Char('O') => return vec![Open::switch_to()],
      Char('C') => return vec![UpdateCommand::Close],
      Char(' ') => {
        let name = take_register_target(registry).unwrap_or_else(|| "playback".to_string());
        if let Some(Register::Content(contents)) = registry.get(&name) {
          if let Some(contents) = contents.first() {
            let keys = Key::from_input(contents);
            return vec![UpdateCommand::SendKeys(keys)];
          }
        }
      },

      // Registry actions
      Char('e') => return vec![Target::switch_to()],

      // Content modifications
      Char('d') => {
        buffer.apply_operations(&[Op::RemoveAll]);
        buffer.history.commit();
      }
      Char('a') => return vec![mode::Insert::switch_to()],
      Char('x') => {
        let name = take_register_target(registry).unwrap_or_else(|| "clipboard".to_string());
        registry.set(&name, Register::Content(copy(buffer)))
      }
      Char('X') => {
        let name = take_register_target(registry).unwrap_or_else(|| "clipboard".to_string());
        if let Some(Register::Content(contents)) = registry.get(&name) {
          paste(buffer, contents);
        }
      }
      Char('z') => undo(buffer),
      Char('Z') => redo(buffer),
      Char('r') => return vec![Pipe::switch_to()],

      // Anchor movements
      Char('h') => buffer.apply_operations(&[Op::MoveByChar(-1), Op::Collapse]),
      Char('j') => buffer.apply_operations(&[Op::MoveByLine(1), Op::Collapse]),
      Char('k') => buffer.apply_operations(&[Op::MoveByLine(-1), Op::Collapse]),
      Char('l') => buffer.apply_operations(&[Op::MoveByChar(1), Op::Collapse]),
      Char('H') => buffer.apply_operations(&[Op::MoveByChar(-1)]),
      Char('J') => buffer.apply_operations(&[Op::MoveByLine(1)]),
      Char('K') => buffer.apply_operations(&[Op::MoveByLine(-1)]),
      Char('L') => buffer.apply_operations(&[Op::MoveByChar(1)]),
      Char('g') => return vec![Seek::switch_to(false)],
      Char('G') => return vec![Seek::switch_to(true)],
      Char('b') => buffer.apply_operations(&[Op::Swap]),
      Char('n') => buffer.apply_operations(&[Op::Collapse]),
      Char('p') => move_by_window_page(buffer, window, 1),
      Char('P') => move_by_window_page(buffer, window, -1),

      // Selection manipulation
      Char('u') => {
        buffer.set_selections(vec![Selection::new_at_end(0, buffer.contents.len_chars())])
      }
      Char('t') => {
        buffer.primary_selection = wrap_add(buffer.selections.len(), buffer.primary_selection, 1)
      }
      Char('T') => {
        buffer.primary_selection = wrap_add(buffer.selections.len(), buffer.primary_selection, -1);
      }
      Char('y') => buffer.set_selections(vec![*buffer.primary_selection()]),
      Char('Y') => {
        let selections = buffer
          .selections
          .iter()
          .enumerate()
          .filter(|&(i, _)| i != buffer.primary_selection)
          .map(|(_, &v)| v)
          .collect();
        buffer.set_selections(selections);
      }
      Char('s') => return vec![Split::switch_to(false)],
      Char('S') => return vec![Split::switch_to(true)],
      Char('f') => return vec![Filter::switch_to(false)],
      Char('F') => return vec![Filter::switch_to(true)],

      // View controls
      Char('v') => center(buffer, window),
      Up => {
        window.keep_cursor_visible = false;
        window.scroll_top = window.scroll_top.saturating_sub(1);
      }
      Down => {
        window.keep_cursor_visible = false;
        window.scroll_top = window.scroll_top.saturating_add(1);
      }
      Left => {
        window.keep_cursor_visible = false;
        window.scroll_left = window.scroll_left.saturating_sub(1);
      }
      Right => {
        window.keep_cursor_visible = false;
        window.scroll_left = window.scroll_left.saturating_add(1);
      }

      _ => {}
    }
    vec![]
  }

  fn status(&self) -> CowStr<'_> {
    match &self.toast {
      Some(toast) => toast.into(),
      None => "normal".into(),
    }
  }
}

fn move_by_window_page(buffer: &mut Buffer, window: &mut Window, delta: isize) {
  buffer.apply_operations(
    &[
      vec![Op::MoveByLine(delta); window.height / 2],
      vec![Op::Collapse],
    ]
    .concat(),
  );
  center(buffer, window);
}

fn center(buffer: &Buffer, window: &mut Window) {
  window.scroll_top = buffer
    .primary_selection()
    .cursor_line(&buffer.contents)
    .saturating_sub(window.height / 2);
}

fn wrap_add(domain: usize, value: usize, delta: isize) -> usize {
  let value = (value as isize) + delta;
  let value = if value < 0 {
    let value = -value as usize;
    (value / domain + 1) * domain - value
  } else {
    value as usize
  };
  value % domain
}

pub fn copy(buffer: &mut Buffer) -> Vec<String> {
  let mut contents = Vec::with_capacity(buffer.selections.len());
  for i in 0..buffer.selections.len() {
    let i = (buffer.primary_selection + i) % buffer.selections.len();
    let selection = buffer
      .selections
      .get(i)
      .expect("should be able to retrieve selection at index less than length when copying");
    let content = selection.slice(&buffer.contents);
    contents.push(content.into());
  }
  contents
}

pub fn paste(buffer: &mut Buffer, contents: &[String]) {
  for selection_i in 0..buffer.selections.len() {
    let content_i = selection_i % contents.len();
    let selection_i = (buffer.primary_selection + selection_i) % buffer.selections.len();
    let selection = buffer
      .selections
      .get_mut(selection_i)
      .expect("should be able to retrieve selection at index less than length when pasting");
    let content = contents
      .get(content_i)
      .expect("should be able to retrieve content at index less than length when pasting");
    let change = selection.apply_operation(&mut buffer.contents, Op::InsertStr(content));
    for j in selection_i + 1..buffer.selections.len() {
      let next_selection = buffer
        .selections
        .get_mut(j)
        .expect("should be able to retrieve selection at index less than length when adjusting selections after applying operation");
      next_selection.adjust(&buffer.contents, change.as_ref());
    }
    change.map(|c| buffer.history.record(c));
  }
  buffer.history.commit();
  buffer.cleanup_overlaps();
}

pub fn undo(buffer: &mut Buffer) {
  let Some(changes) = buffer.history.backward() else {
    return;
  };
  let selections = changes.apply(&mut buffer.contents);
  buffer.set_selections(selections);
}

pub fn redo(buffer: &mut Buffer) {
  let Some(changes) = buffer.history.forward() else {
    return;
  };
  let selections = changes.apply(&mut buffer.contents);
  buffer.set_selections(selections);
}

pub fn take_register_target(registry: &mut Registry) -> Option<String> {
  match registry.get("target") {
    Some(Register::Content(target)) => match target.as_slice() {
      [name, ..] => {
        let name = name.to_string();
        registry.del("target");
        Some(name)
      }
      _ => None,
    },
    _ => None,
  }
}
