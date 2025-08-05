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
      // Meta
      Char('q') if modifiers.control => return UpdateCommand::Quit,
      Char('w') if modifiers.control => {
        self.toast = if buffer.save() {
          Some("file saved!".into())
        } else {
          Some("error: could not save file".into())
        };
      }

      // Anchors
      Char('h') => buffer.apply_operations(&[Op::MoveByChar(-1), Op::Collapse]),
      Char('j') => buffer.apply_operations(&[Op::MoveByLine(1), Op::Collapse]),
      Char('k') => buffer.apply_operations(&[Op::MoveByLine(-1), Op::Collapse]),
      Char('l') => buffer.apply_operations(&[Op::MoveByChar(1), Op::Collapse]),
      Char('H') => buffer.apply_operations(&[Op::MoveByChar(-1)]),
      Char('J') => buffer.apply_operations(&[Op::MoveByLine(1)]),
      Char('K') => buffer.apply_operations(&[Op::MoveByLine(-1)]),
      Char('L') => buffer.apply_operations(&[Op::MoveByChar(1)]),
      Char('b') => buffer.apply_operations(&[Op::Swap]),
      Char('n') => buffer.apply_operations(&[Op::Collapse]),
      Char('p') => move_by_window_page(buffer, window, 1),
      Char('P') => move_by_window_page(buffer, window, -1),

      // Selections
      Char('u') => buffer.set_selections(vec![Selection::new_at_end(
        0,
        buffer.current.contents.len_chars(),
      )]),
      Char('y') => {
        buffer.current.primary_selection = wrap_add(
          buffer.current.selections.len(),
          buffer.current.primary_selection,
          1,
        )
      }
      Char('Y') => {
        buffer.current.primary_selection = wrap_add(
          buffer.current.selections.len(),
          buffer.current.primary_selection,
          -1,
        );
      }
      Char('t') => buffer.set_selections(vec![*buffer.primary_selection()]),
      Char('T') => {
        let selections = buffer
          .current
          .selections
          .iter()
          .enumerate()
          .filter(|&(i, _)| i != buffer.current.primary_selection)
          .map(|(_, &v)| v)
          .collect();
        buffer.set_selections(selections);
      }
      Char('s') => return Split::switch_to(false),
      Char('S') => return Split::switch_to(true),
      Char('g') => return Seek::switch_to(false),
      Char('G') => return Seek::switch_to(true),
      Char('f') => return Filter::switch_to(false),
      Char('F') => return Filter::switch_to(true),

      // Modification
      Char('d') => {
        buffer.apply_operations(&[Op::RemoveAll]);
      }
      Char('a') => {
        buffer.apply_operations(&[Op::Collapse]);
        return mode::Insert::switch_to();
      }
      Char('q') => registry.set("clipboard", Register::Content(buffer.copy())),
      Char('Q') => {
        if let Some(Register::Content(contents)) = registry.get("clipboard") {
          buffer.paste(contents);
        }
      }
      Char('z') => buffer.undo(),
      Char('Z') => buffer.redo(),
      Char('r') => {
        return Pipe::switch_to();
      }
      // View
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
    .cursor_line(&buffer.current.contents)
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
