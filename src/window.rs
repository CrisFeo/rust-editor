use ropey::Rope;

pub struct Window {
  pub scroll_top: usize,
  pub scroll_left: usize,
  pub width: usize,
  pub height: usize,
}

impl Window {
  pub fn new() -> Window {
    Window {
      scroll_top: 0,
      scroll_left: 0,
      width: 0,
      height: 0,
    }
  }

  pub fn set_size(&mut self, size: (usize, usize)) {
    let (width, height) = size;
    self.width = width;
    self.height = height;
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

  pub fn from_scroll_position(&self, contents: &Rope, row: usize, col: usize) -> usize {
    let row = self.scroll_top + row;
    let col = self.scroll_left + col;
    contents.line_to_char(row) + col
  }

  pub fn scroll_into_view(&mut self, contents: &Rope, index: usize) {
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

