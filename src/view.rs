use crossterm::style::{
  Color,
};
use crate::{
  screen::{
    Screen,
  },
  window::{
    Window,
  },
  buffer::{
    Buffer,
  },
  key::{
    Key,
    Modifiers,
  },
};

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
  pub fn new() -> Self {
    View {
      screen: Screen::new(),
      accent_color: Color::Rgb{ r: 95,  g: 135, b: 0   },
      ramp_0_color: Color::Rgb{ r: 0,   g: 0,   b: 0   },
      ramp_1_color: Color::Rgb{ r: 78,  g: 78,  b: 78  },
      ramp_2_color: Color::Rgb{ r: 188, g: 188, b: 188 },
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

  pub fn render(&mut self, buffer: &Buffer, window: &Window) {
    self.screen.clear();
    let (width, height) = self.screen.size();
    let primary_selection = buffer.primary_selection();
    let mut selection_iter = buffer.selections.iter();
    let mut current_selection = selection_iter.next();
    let start_index = window.from_scroll_position(&buffer.contents, 0, 0);
    while let Some(selection) = current_selection {
      if selection.end() >= start_index {
        break;
      }
      current_selection = selection_iter.next();
    }
    let lines = buffer.contents
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
        let index = window.from_scroll_position(&buffer.contents, row, col);
        let mut is_selection = false;
        let mut is_primary = false;
        let mut is_cursor = false;
        if let Some(selection) = current_selection {
          if index >= selection.start() && index <= selection.end() {
            is_selection = true;
          }
          if selection.start() == primary_selection.start() {
            is_primary = true;
          }
          is_cursor = index == selection.cursor();
          if index == selection.end() {
            current_selection = selection_iter.next();
          }
        }
        let mut bg = self.ramp_0_color;
        let mut fg = self.ramp_2_color;
        if is_selection {
          bg = {
            if is_primary {
              if is_cursor {
                self.accent_color
              } else {
                self.ramp_2_color
              }
            } else {
              if is_cursor {
                self.ramp_2_color
              } else {
                self.ramp_1_color
              }
            }
          };
          fg = self.ramp_0_color;
        }
        let ch = match ch {
          '\n' => {
            if !is_selection {
              ' '
            } else {
              self.new_line_char
            }
          },
          ch => ch,
        };
        self.screen.draw(row, col, ch, bg, fg);
      }
    }
    if let Some(selection) = current_selection {
      if selection.cursor() == buffer.contents.len_chars() {
        let buffer_end = window.to_scroll_position(&buffer.contents, buffer.contents.len_chars());
        if let Some((row, col)) = buffer_end {
          self.screen.draw(row, col, self.end_of_file_char, self.accent_color, self.ramp_0_color);
        }
      }
    }
    let cursor = primary_selection.cursor();
    let row = buffer.contents.char_to_line(cursor);
    let col = cursor.saturating_sub(buffer.contents.line_to_char(row));
    let status: Vec<char> = format!(" {}:{} ", row, col)
      .chars()
      .collect();
    for i in 0..status.len() {
      let start = width.saturating_sub(status.len());
      self.screen.draw(
        height.saturating_sub(1),
        start + i,
        status[i],
        self.accent_color,
        self.ramp_0_color
      );
    }
    self.screen.present();
  }
}
