use crate::*;

#[derive(Default)]
pub struct Normal {
  toast: Option<String>,
}

impl Normal {
  pub fn switch_to() -> UpdateCommand {
    UpdateCommand::Switch(Box::new(Self::default()))
  }

  pub fn switch_to_with_toast(toast: impl Into<String>) -> UpdateCommand {
    UpdateCommand::Switch(Box::new(Self {
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
    modifiers: Modifiers,
    key: Key,
  ) -> UpdateCommand {
    use crate::key::Key::*;
    self.toast = None;
    match key {
      // Meta actions
      Char('q') if modifiers.control => return UpdateCommand::Quit,
      Char('w') if modifiers.control => {
        self.toast = if buffer.save() {
          Some("file saved!".into())
        } else {
          Some("error: could not save file".into())
        };
      }

      // Content modifications
      Char('d') => {
        buffer.apply_operations(&[Op::RemoveAll]);
        buffer.history.commit();
      }
      Char('a') => return mode::Insert::switch_to(),
      Char('q') => registry.set("clipboard", Register::Content(copy(buffer))),
      Char('Q') => {
        if let Some(Register::Content(contents)) = registry.get("clipboard") {
          paste(buffer, contents);
        }
      }
      Char('z') => undo(buffer),
      Char('Z') => redo(buffer),
      Char('r') => return Pipe::switch_to(),

      // Anchor movements
      Char('h') => buffer.apply_operations(&[Op::MoveByChar(-1)]),
      Char('j') => buffer.apply_operations(&[Op::MoveByLine(1)]),
      Char('k') => buffer.apply_operations(&[Op::MoveByLine(-1)]),
      Char('l') => buffer.apply_operations(&[Op::MoveByChar(1)]),
      Char('g') => return Seek::switch_to(false),
      Char('G') => return Seek::switch_to(true),
      Char('b') => buffer.apply_operations(&[Op::Swap]),
      Char('n') => buffer.apply_operations(&[Op::Collapse]),
      Char('p') => move_by_window_page(buffer, window, 1),
      Char('P') => move_by_window_page(buffer, window, -1),

      // Selection manipulation
      Char('u') => buffer.set_selections(vec![Selection::new_at_end(
        0,
        buffer.contents.len_chars(),
      )]),
      Char('t') => {
        buffer.primary_selection = wrap_add(
          buffer.selections.len(),
          buffer.primary_selection,
          1,
        )
      }
      Char('T') => {
        buffer.primary_selection = wrap_add(
          buffer.selections.len(),
          buffer.primary_selection,
          -1,
        );
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
      Char('s') => return Split::switch_to(false),
      Char('S') => return Split::switch_to(true),
      Char('f') => return Filter::switch_to(false),
      Char('F') => return Filter::switch_to(true),

      // View controls
      Char('v') => center(buffer, window),
      Up => window.scroll_top = window.scroll_top.saturating_sub(1),
      Down => window.scroll_top = window.scroll_top.saturating_add(1),
      Left => window.scroll_left = window.scroll_left.saturating_sub(1),
      Right => window.scroll_left = window.scroll_left.saturating_add(1),

      _ => {}
    }
    UpdateCommand::None
  }

  fn status(&self) -> CowStr {
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
  for content_i in 0..buffer.selections.len().min(contents.len()) {
    let selection_i =
      (buffer.primary_selection + content_i) % buffer.selections.len();
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
