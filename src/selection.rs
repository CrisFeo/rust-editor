use regex::Regex;
use ropey::Rope;

#[derive(Debug, Copy, Clone)]
pub enum Op {
  Swap,
  Collapse,
  MoveByChar(isize),
  MoveByLine(isize),
  Insert(char),
  Remove,
  RemoveAll,
}

#[derive(Debug, Copy, Clone)]
pub enum Change {
  None,
  Addition(usize, usize),
  Removal(usize, usize),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Side {
  Start,
  End,
}

#[derive(Debug, Copy, Clone)]
pub struct Selection {
  start: usize,
  end: usize,
  side: Side,
  last_line_offset: Option<usize>,
}

impl Selection {
  pub fn new_at_start(start: usize, end: usize) -> Selection {
    Selection {
      start,
      end,
      side: Side::Start,
      last_line_offset: None,
    }
  }

  pub fn new_at_end(start: usize, end: usize) -> Selection {
    Selection {
      start,
      end,
      side: Side::End,
      last_line_offset: None,
    }
  }

  pub fn start(&self) -> usize {
    self.start
  }

  pub fn end(&self) -> usize {
    self.end
  }

  pub fn side(&self) -> Side {
    self.side
  }

  pub fn cursor(&self) -> usize {
    match self.side {
      Side::Start => self.start,
      Side::End => self.end,
    }
  }

  pub fn size(&self) -> usize {
    self.end.saturating_sub(self.start) + 1
  }

  pub fn scan(&self, regex: &Regex, contents: &Rope) -> Vec<(usize, usize)> {
    let start_byte = contents.char_to_byte(self.start);
    let end = usize::min(self.end.saturating_add(1), contents.len_chars().saturating_sub(1));
    let slice: std::borrow::Cow<str> = contents.slice(self.start..end).into();
    regex
      .find_iter(&slice)
      .map(|m| {
        let start = start_byte.saturating_add(m.start());
        let start = contents.byte_to_char(start);
        let end = start_byte.saturating_add(m.end());
        let mut end = contents.byte_to_char(end);
        if end > start {
          end = end.saturating_sub(1);
        }
        (start, end)
      })
      .collect()
  }

  pub fn try_merge(&self, other: &Self) -> Option<Selection> {
    if other.start < self.start {
      other.try_merge(self)
    } else {
      if self.end >= other.start {
        Some(Selection::new_at_end(self.start, self.end.max(other.end)))
      } else {
        None
      }
    }
  }

  pub fn apply(&mut self, contents: &mut Rope, op: Op) -> Change {
    let change = match op {
      Op::Swap => self.swap(),
      Op::Collapse => self.collapse(),
      Op::MoveByChar(delta) => self.move_by_char(&contents, delta),
      Op::MoveByLine(delta) => self.move_by_line(&contents, delta),
      Op::Insert(ch) => self.insert(contents, ch),
      Op::Remove => self.remove(contents),
      Op::RemoveAll => self.remove_all(contents),
    };
    self.adjust(&contents, change);
    change
  }

  pub fn adjust(&mut self, contents: &Rope, change: Change) {
    let max = contents.len_chars();
    match change {
      Change::None => {}
      Change::Addition(begin, delta) => {
        let delta = delta as isize;
        if self.start >= begin {
          self.start = step(max, self.start, delta);
        }
        if self.end >= begin {
          self.end = step(max, self.end, delta);
        }
      }
      Change::Removal(begin, delta) => {
        let end = begin.saturating_add(delta);
        let delta = -(delta as isize);
        if self.start >= end {
          self.start = step(max, self.start, delta);
        } else if self.start >= begin {
          self.start = begin;
        }
        if self.end >= end {
          self.end = step(max, self.end, delta);
        } else if self.end >= begin {
          self.end = begin;
        }
      }
    }
    if self.start == self.end {
      self.side = Side::End;
    } else if self.start > self.end {
      let tmp = self.start;
      self.start = self.end;
      self.end = tmp;
      self.side = match self.side {
        Side::Start => Side::End,
        Side::End => Side::Start,
      };
    }
  }

  fn swap(&mut self) -> Change {
    match self.side {
      Side::Start => {
        self.side = Side::End;
      }
      Side::End => {
        self.side = Side::Start;
      }
    }
    self.last_line_offset = None;
    Change::None
  }

  fn collapse(&mut self) -> Change {
    match self.side {
      Side::Start => {
        self.end = self.start;
      }
      Side::End => {
        self.start = self.end;
      }
    }
    Change::None
  }

  fn move_by_char(&mut self, contents: &Rope, delta: isize) -> Change {
    let max = contents.len_chars();
    match self.side {
      Side::Start => self.start = step(max, self.start, delta),
      Side::End => self.end = step(max, self.end, delta),
    };
    self.last_line_offset = None;
    Change::None
  }

  fn move_by_line(&mut self, contents: &Rope, delta: isize) -> Change {
    let max = contents.len_lines();
    let cursor = self.cursor();
    let line = contents.char_to_line(cursor);
    let new_line = step(max, line, delta);
    if new_line >= contents.len_lines() {
      return Change::None;
    }
    let line_begin = contents.line_to_char(line);
    let line_offset = {
      if let Some(last_line_offset) = self.last_line_offset {
        last_line_offset
      } else {
        let last_line_offset = cursor.saturating_sub(line_begin);
        self.last_line_offset = Some(last_line_offset);
        last_line_offset
      }
    };
    let new_line_begin = contents.line_to_char(new_line);
    let new_line_len = contents.line(new_line).len_chars();
    let new_line_offset = line_offset.min(new_line_len.saturating_sub(1));
    let new_cursor = new_line_begin.saturating_add(new_line_offset);
    match self.side {
      Side::Start => self.start = new_cursor,
      Side::End => self.end = new_cursor,
    };
    Change::None
  }

  fn insert(&mut self, contents: &mut Rope, ch: char) -> Change {
    let cursor = self.cursor();
    let change = Change::Addition(cursor, 1);
    contents.insert_char(cursor, ch);
    self.last_line_offset = None;
    change
  }

  fn remove(&mut self, contents: &mut Rope) -> Change {
    let cursor = self.cursor();
    if cursor == 0 {
      return Change::None;
    }
    let begin = cursor.saturating_sub(1);
    let change = Change::Removal(begin, 1);
    contents.remove(begin..cursor);
    self.last_line_offset = None;
    change
  }

  fn remove_all(&mut self, contents: &mut Rope) -> Change {
    let max = contents.len_chars();
    if self.start >= max {
      return Change::None;
    }
    let change = Change::Removal(self.start, self.size());
    let end = self.end.min(max.saturating_sub(1));
    contents.remove(self.start..=end);
    self.last_line_offset = None;
    change
  }
}

fn step(max: usize, value: usize, delta: isize) -> usize {
  let new_value = {
    if delta > 0 {
      value.saturating_add(delta as usize)
    } else {
      value.saturating_sub(-delta as usize)
    }
  };
  if new_value >= max {
    max
  } else {
    new_value
  }
}
