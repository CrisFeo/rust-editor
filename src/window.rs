use ropey::Rope;

pub struct Window {
  pub keep_cursor_visible: bool,
  pub scroll_top: usize,
  pub scroll_left: usize,
  pub width: usize,
  pub height: usize,
}

impl Window {
  pub fn new(size: (usize, usize)) -> Self {
    Self {
      keep_cursor_visible: true,
      scroll_top: 0,
      scroll_left: 0,
      width: size.0,
      height: size.1,
    }
  }

  pub fn set_size(&mut self, size: (usize, usize)) {
    self.width = size.0;
    self.height = size.1;
  }

  pub fn to_scroll_position(&self, contents: &Rope, index: usize) -> Option<(usize, usize)> {
    let row = contents.char_to_line(index);
    if row < self.scroll_top {
      return None;
    }
    let col = index.saturating_sub(contents.line_to_char(row));
    if col < self.scroll_left {
      return None;
    }
    let row = row - self.scroll_top;
    let col = col - self.scroll_left;
    Some((row, col))
  }

  pub fn to_index(&self, contents: &Rope, row: usize, col: usize) -> usize {
    let row = self.scroll_top + row;
    let col = self.scroll_left + col;
    if row >= contents.len_lines() {
      return contents.len_chars();
    }
    contents.line_to_char(row) + col
  }

  pub fn scroll_into_view(&mut self, contents: &Rope, index: usize) {
    if contents.len_chars() == 0 {
      self.scroll_top = 0;
      self.scroll_left = 0;
      return;
    }
    let index = index.min(contents.len_chars());
    let row = contents.char_to_line(index);
    if row < self.scroll_top {
      self.scroll_top = row;
    }
    if row >= self.scroll_top.saturating_add(self.height) {
      self.scroll_top = row.saturating_sub(self.height.saturating_sub(1));
    }
    let col = index.saturating_sub(contents.line_to_char(row));
    if col < self.scroll_left {
      self.scroll_left = col;
    }
    if col >= self.scroll_left.saturating_add(self.width) {
      self.scroll_left = col.saturating_sub(self.width.saturating_sub(1));
    }
  }
}
