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
}

impl View {
  pub fn new() -> Self {
    View {
      screen: Screen::new(),
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
      .take(height.saturating_sub(1))
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
        let mut bg = Color::Black;
        let mut fg = Color::White;
        if is_selection {
          bg = {
            if is_primary {
              if is_cursor {
                Color::Blue
              } else {
                Color::White
              }
            } else {
              if is_cursor {
                Color::White
              } else {
                Color::Grey
              }
            }
          };
          fg = Color::Black;
        }
        let ch = match ch {
          '\n' => ' ',
          ch => ch,
        };
        self.screen.draw(row, col, ch, bg, fg);
      }
    }
    if let Some(selection) = current_selection {
      if selection.cursor() == buffer.contents.len_chars() {
        let buffer_end = window.to_scroll_position(&buffer.contents, buffer.contents.len_chars());
        if let Some((row, col)) = buffer_end {
          self.screen.draw(row, col, ' ', Color::Blue, Color::Black);
        }
      }
    }
    self.screen.present();
  }
}
