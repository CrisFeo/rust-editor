use crate::*;

pub struct Theme {
  pub new_line_char: char,
  pub end_of_file_char: char,
  pub default_face: (Option<Color>, Option<Color>),
  pub selection_primary_face: (Option<Color>, Option<Color>),
  pub selection_secondary_face: (Option<Color>, Option<Color>),
  pub cursor_primary_face: (Option<Color>, Option<Color>),
  pub cursor_secondary_face: (Option<Color>, Option<Color>),
  pub status_face: (Option<Color>, Option<Color>),
}

pub struct View {
  screen: Screen,
  theme: Theme,
}

impl View {
  pub fn create(theme: Theme) -> Self {
    Self {
      screen: Screen::create(),
      theme,
    }
  }

  pub fn buffer_size(&self) -> (usize, usize) {
    let (width, height) = self.screen.size();
    (width, height.saturating_sub(1))
  }

  pub fn poll(&mut self) -> Event {
    self.screen.poll()
  }

  pub fn render(&mut self, mode: &dyn Mode, buffer: &Buffer, window: &Window) {
    self.screen.clear();
    let (width, height) = self.screen.size();
    let (selections, primary_selection) = match mode.preview_selections() {
      Some(selections) => (selections, None),
      None => (&buffer.selections, Some(buffer.primary_selection())),
    };
    // render buffer contents
    {
      let mut selection_iter = selections.iter();
      let mut current_selection = selection_iter.next();
      let start_index = window.to_index(&buffer.contents, 0, 0);
      while let Some(selection) = current_selection {
        if selection.end() >= start_index {
          break;
        }
        current_selection = selection_iter.next();
      }
      let lines = buffer
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
          let index = window.to_index(&buffer.contents, row, col);
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
                self.theme.new_line_char
              }
            }
            ch => ch,
          };
          self.screen.draw(row, col, ch, bg, fg);
        }
      }
      let buffer_end = window.to_scroll_position(&buffer.contents, buffer.contents.len_chars());
      if let Some((row, col)) = buffer_end {
        let index = buffer.contents.len_chars();
        let mut style = None;
        for selection in selections.iter() {
          let (is_selection, is_primary, is_cursor) =
            Self::properties(index, Some(selection), primary_selection);
          if is_selection {
            style = Some(self.style(is_selection, is_primary, is_cursor));
          }
        }
        if let Some((bg, fg)) = style {
          self
            .screen
            .draw(row, col, self.theme.end_of_file_char, bg, fg);
        }
      }
    }
    // render status bar
    {
      let status_left = mode.status();
      let status_left_size = status_left.chars().count();
      let status_right = {
        let cursor_location = match primary_selection {
          Some(primary_selection) => {
            let cursor = primary_selection.cursor();
            let row = primary_selection.cursor_line(&buffer.contents);
            let col = cursor.saturating_sub(buffer.contents.line_to_char(row));
            format!(" {row}:{col}")
          }
          None => "".to_string(),
        };
        let buffer_name = match &buffer.filename {
          Some(filename) => {
            let cursor_location_size = cursor_location.chars().count();
            let available_size = width.saturating_sub(status_left_size + cursor_location_size);
            let required_size = filename.chars().count() + 1;
            if required_size <= available_size {
              format!(" {filename}")
            } else {
              let start = required_size + 1 - available_size;
              let name = filename.chars().skip(start).collect::<String>();
              format!(" …{name}")
            }
          }
          None => "".to_string(),
        };
        format!("{cursor_location}{buffer_name}")
      };
      let status_right_size = status_right.chars().count();
      let status_min_size = status_left_size + status_right_size;
      let status = if status_min_size <= width {
        let status_gap_size = width - status_min_size;
        let status_gap = (0..status_gap_size).map(|_| " ").collect::<String>();
        format!("{status_left}{status_gap}{status_right}")
      } else if status_left_size < width {
        let status_gap_size = width - status_left_size;
        let status_gap = (0..status_gap_size).map(|_| " ").collect::<String>();
        format!("{status_left}{status_gap}")
      } else {
        (0..width).map(|_| " ").collect::<String>()
      };
      for (i, ch) in status.chars().enumerate() {
        self.screen.draw(
          height.saturating_sub(1),
          i,
          ch,
          self.theme.status_face.0,
          self.theme.status_face.1,
        );
      }
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

  fn style(&self, is_selection: bool, is_primary: bool, is_cursor: bool) -> (Option<Color>, Option<Color>) {
    let mut face = self.theme.default_face;
    if is_selection {
      if is_primary {
        if is_cursor {
          face = self.theme.cursor_primary_face;
        } else {
          face = self.theme.selection_primary_face;
        };
      } else if is_cursor {
        face = self.theme.cursor_secondary_face;
      } else {
        face = self.theme.selection_secondary_face;
      }
    }
    face
  }
}
