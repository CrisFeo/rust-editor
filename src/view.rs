use crate::*;

pub struct View {
  screen: Screen,
  accent_color: Color,
  ramp_0_color: Color,
  ramp_1_color: Color,
  ramp_2_color: Color,
  new_line_char: char,
  end_of_file_char: char,
}

impl View {
  pub fn create() -> Self {
    #[rustfmt::skip]
    let accent_color = Color(95,  135, 0  );
    let ramp_0_color = Color(0, 0, 0);
    let ramp_1_color = Color(78, 78, 78);
    let ramp_2_color = Color(188, 188, 188);
    Self {
      screen: Screen::create(ramp_0_color, ramp_2_color),
      accent_color,
      ramp_0_color,
      ramp_1_color,
      ramp_2_color,
      new_line_char: '¬',
      end_of_file_char: 'Ω',
    }
  }

  pub fn buffer_size(&self) -> (usize, usize) {
    let (width, height) = self.screen.size();
    (width, height.saturating_sub(1))
  }

  pub fn poll(&mut self) -> (Modifiers, Key) {
    self.screen.poll()
  }

  pub fn render(&mut self, mode: &dyn Mode, buffer: &Buffer, window: &Window) {
    self.screen.clear();
    let (width, height) = self.screen.size();
    let (mut selection_iter, primary_selection) = match &mode.preview_selections() {
      Some(selections) => (selections.iter(), None),
      None => (
        buffer.current.selections.iter(),
        Some(buffer.primary_selection()),
      ),
    };
    let mut current_selection = selection_iter.next();
    let start_index = window.to_index(&buffer.current.contents, 0, 0);
    while let Some(selection) = current_selection {
      if selection.end() >= start_index {
        break;
      }
      current_selection = selection_iter.next();
    }
    let lines = buffer
      .current
      .contents
      .get_lines_at(window.scroll_top)
      .into_iter()
      .flatten()
      //.take(height.saturating_sub(1))
      .take(height)
      .enumerate();
    for (row, line) in lines {
      let chars = line
        .get_chars_at(window.scroll_left)
        .into_iter()
        .flatten()
        .take(width)
        .enumerate();
      for (col, ch) in chars {
        let index = window.to_index(&buffer.current.contents, row, col);
        while let Some(selection) = current_selection {
          if index <= selection.end() {
            break;
          }
          current_selection = selection_iter.next();
        }
        let (is_selection, is_primary, is_cursor) =
          Self::properties(index, current_selection, primary_selection);
        let (bg, fg) = self.style(is_selection, is_primary, is_cursor);
        let ch = match ch {
          '\n' => {
            if !is_selection {
              ' '
            } else {
              self.new_line_char
            }
          }
          ch => ch,
        };
        self.screen.draw(row, col, ch, bg, fg);
      }
    }
    let buffer_end = window.to_scroll_position(
      &buffer.current.contents,
      buffer.current.contents.len_chars(),
    );
    if let Some((row, col)) = buffer_end {
      let index = buffer.current.contents.len_chars();
      let (is_selection, is_primary, is_cursor) =
        Self::properties(index, current_selection, primary_selection);
      if is_selection {
        let (bg, fg) = self.style(is_selection, is_primary, is_cursor);
        self.screen.draw(row, col, self.end_of_file_char, bg, fg);
      }
    }
    let status_left = mode.status();
    let status_right = {
      let cursor_location = match primary_selection {
        Some(primary_selection) => {
          let cursor = primary_selection.cursor();
          let row = buffer.current.contents.char_to_line(cursor);
          let col = cursor.saturating_sub(buffer.current.contents.line_to_char(row));
          format!(" {}:{}", row, col)
        }
        None => "".to_string(),
      };
      let buffer_name = match &buffer.filename {
        Some(filename) => format!(" {}", filename),
        None => "".to_string(),
      };
      format!("{}{}", cursor_location, buffer_name)
    };
    let status_gap_size = width - (status_left.len() + status_right.len());
    let status_gap = (0..status_gap_size).map(|_| " ").collect::<String>();
    let status = format!("{}{}{}", status_left, status_gap, status_right);
    for (i, ch) in status.chars().enumerate() {
      self.screen.draw(
        height.saturating_sub(1),
        i,
        ch,
        self.accent_color,
        self.ramp_0_color,
      );
    }
    self.screen.present();
  }

  fn properties(
    index: usize,
    current_selection: Option<&Selection>,
    primary_selection: Option<&Selection>,
  ) -> (bool, bool, bool) {
    let mut is_selection = false;
    let mut is_primary = false;
    let mut is_cursor = false;
    if let Some(selection) = current_selection {
      if index >= selection.start() && index <= selection.end() {
        is_selection = true;
      }
      if let Some(primary_selection) = primary_selection {
        if selection.start() == primary_selection.start() {
          is_primary = true;
        }
      }
      is_cursor = index == selection.cursor();
    }
    (is_selection, is_primary, is_cursor)
  }

  fn style(&self, is_selection: bool, is_primary: bool, is_cursor: bool) -> (Color, Color) {
    let mut bg = self.ramp_0_color;
    let mut fg = self.ramp_2_color;
    if is_selection {
      if is_primary {
        if is_cursor {
          bg = self.accent_color;
        } else {
          bg = self.ramp_2_color;
        };
        fg = self.ramp_0_color;
      } else if is_cursor {
        bg = self.ramp_2_color;
        fg = self.ramp_0_color;
      } else {
        bg = self.ramp_1_color;
        fg = self.ramp_2_color;
      }
    }
    (bg, fg)
  }
}
