use crate::*;
use ropey::{Rope, RopeSlice};
use std::cmp::Ordering;
use std::mem::swap;

#[derive(Debug, Copy, Clone)]
pub enum Op<'a> {
  Swap,
  Collapse,
  MoveByChar(isize),
  MoveByLine(isize),
  InsertChar(char),
  InsertStr(&'a str),
  Remove,
  RemoveAll,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Side {
  Start,
  End,
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

  pub fn cursor_line(&self, contents: &Rope) -> usize {
    if contents.len_chars() == 0 {
      return 0;
    }
    let cursor = self.cursor();
    if cursor > contents.len_chars() {
      return contents.len_lines();
    }
    contents.char_to_line(cursor)
  }

  pub fn slice<'a>(&self, contents: &'a Rope) -> RopeSlice<'a> {
    let end = self.end().min(contents.len_chars() - 1);
    let range = self.start()..=end;
    contents.slice(range)
  }

  pub fn try_merge(&self, other: &Self) -> Option<Selection> {
    if other.start < self.start {
      other.try_merge(self)
    } else if self.end >= other.start {
      Some(Selection::new_at_end(self.start, self.end.max(other.end)))
    } else {
      None
    }
  }

  pub fn apply_operation(&mut self, contents: &mut Rope, op: Op) -> Option<Change> {
    let change = match op {
      Op::Swap => self.swap(),
      Op::Collapse => self.collapse(),
      Op::MoveByChar(delta) => self.move_by_char(contents, delta),
      Op::MoveByLine(delta) => self.move_by_line(contents, delta),
      Op::InsertChar(value) => self.insert_char(contents, value),
      Op::InsertStr(value) => self.insert_str(contents, value),
      Op::Remove => self.remove(contents),
      Op::RemoveAll => self.remove_all(contents),
    };
    self.adjust(contents, change.as_ref());
    change
  }

  pub fn adjust(&mut self, contents: &Rope, change: Option<&Change>) {
    let max = contents.len_chars();
    match change {
      None => {
        self.start = self.start.min(max);
        self.end = self.end.min(max);
      }
      Some(Change::Addition(begin, content)) => {
        let delta = content.len_chars() as isize;
        if self.start >= *begin {
          self.start = step(max, self.start, delta);
        }
        if self.end >= *begin {
          self.end = step(max, self.end, delta);
        }
      }
      Some(Change::Removal(begin, content)) => {
        let delta = content.len_chars();
        let end = begin.saturating_add(delta);
        let delta = -(delta as isize);
        if self.start >= end {
          self.start = step(max, self.start, delta);
        } else if self.start >= *begin {
          self.start = *begin;
        }
        if self.end >= end {
          self.end = step(max, self.end, delta);
        } else if self.end >= *begin {
          self.end = *begin;
        }
      }
    }
    match self.start.cmp(&self.end) {
      Ordering::Less => {}
      Ordering::Equal => self.side = Side::End,
      Ordering::Greater => {
        swap(&mut self.start, &mut self.end);
        self.side = match self.side {
          Side::Start => Side::End,
          Side::End => Side::Start,
        };
      }
    }
  }

  fn swap(&mut self) -> Option<Change> {
    match self.side {
      Side::Start => {
        self.side = Side::End;
      }
      Side::End => {
        self.side = Side::Start;
      }
    }
    self.last_line_offset = None;
    None
  }

  fn collapse(&mut self) -> Option<Change> {
    match self.side {
      Side::Start => {
        self.end = self.start;
      }
      Side::End => {
        self.start = self.end;
      }
    }
    None
  }

  fn move_by_char(&mut self, contents: &Rope, delta: isize) -> Option<Change> {
    let max = contents.len_chars();
    match self.side {
      Side::Start => self.start = step(max, self.start, delta),
      Side::End => self.end = step(max, self.end, delta),
    };
    self.last_line_offset = None;
    None
  }

  fn move_by_line(&mut self, contents: &Rope, delta: isize) -> Option<Change> {
    let max = contents.len_lines();
    let cursor = self.cursor();
    let line = self.cursor_line(contents);
    let new_line = step(max, line, delta);
    if new_line >= contents.len_lines() {
      return None;
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
    None
  }

  fn insert_char(&mut self, contents: &mut Rope, value: char) -> Option<Change> {
    let cursor = self.cursor();
    let change = Change::Addition(cursor, value.to_string().into());
    contents.insert_char(cursor, value);
    self.last_line_offset = None;
    Some(change)
  }

  fn insert_str(&mut self, contents: &mut Rope, value: &str) -> Option<Change> {
    let cursor = self.cursor();
    let change = Change::Addition(cursor, value.into());
    contents.insert(cursor, value);
    self.last_line_offset = None;
    Some(change)
  }

  fn remove(&mut self, contents: &mut Rope) -> Option<Change> {
    let cursor = self.cursor();
    if cursor == 0 {
      return None;
    }
    let begin = cursor.saturating_sub(1);
    let range = begin..cursor;
    let content = contents.slice(range.clone()).into();
    let change = Change::Removal(begin, content);
    contents.remove(range);
    self.last_line_offset = None;
    Some(change)
  }

  fn remove_all(&mut self, contents: &mut Rope) -> Option<Change> {
    let max = contents.len_chars();
    if self.start >= max {
      return None;
    }
    let range = self.start..=self.end.min(max.saturating_sub(1));
    let content = contents.slice(range.clone()).into();
    let change = Change::Removal(self.start, content);
    contents.remove(range);
    self.last_line_offset = None;
    Some(change)
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

pub fn merge_overlapping_selections(selections: &mut Vec<Selection>) {
  selections.sort_by(|a, b| {
    a.start()
      .partial_cmp(&b.start())
      .expect("selection bounds should be comparable")
  });
  *selections = {
    let mut selections_iter = selections.drain(..);
    let mut selections = vec![];
    let mut current = selections_iter.next();
    while let Some(current_value) = current {
      let mut next = selections_iter.next();
      if let Some(next_value) = next {
        if let Some(merged_value) = current_value.try_merge(&next_value) {
          next = Some(merged_value);
        } else {
          selections.push(current_value);
        }
      } else {
        selections.push(current_value);
      }
      current = next;
    }
    selections
  };
}
