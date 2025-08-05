use crate::*;
use ropey::Rope;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, ErrorKind, Result};

#[derive(Clone, PartialEq)]
pub struct Snapshot {
  pub contents: Rope,
  pub selections: Vec<Selection>,
  pub primary_selection: usize,
}

pub struct Buffer {
  pub current: Snapshot,
  pub filename: Option<String>,
  pub history: History,
}

impl Buffer {
  pub fn new_scratch() -> Self {
    let current = Snapshot {
      contents: Rope::new(),
      selections: vec![Selection::new_at_end(0, 0)],
      primary_selection: 0,
    };
    Self {
      current,
      filename: None,
      history: Default::default(),
    }
  }

  pub fn new_from_file(filename: String) -> Result<Self> {
    let file = OpenOptions::new().read(true).open(&filename);
    let contents = match file {
      Ok(file) => {
        let reader = BufReader::new(file);
        Rope::from_reader(reader)
      }
      Err(e) if e.kind() == ErrorKind::NotFound => Ok(Rope::new()),
      Err(e) => Err(e),
    }?;
    let current = Snapshot {
      contents,
      selections: vec![Selection::new_at_end(0, 0)],
      primary_selection: 0,
    };
    Ok(Self {
      current,
      filename: Some(filename),
      history: Default::default(),
    })
  }

  pub fn save(&self) -> bool {
    if let Some(filename) = &self.filename {
      let file = match File::create(filename) {
        Ok(file) => file,
        Err(e) => {
          eprintln!("{}", e);
          return false;
        }
      };
      let writer = BufWriter::new(file);
      match self.current.contents.write_to(writer) {
        Ok(_) => {}
        Err(e) => {
          eprintln!("{}", e);
          return false;
        }
      };
    }
    true
  }

  pub fn primary_selection(&self) -> &Selection {
    self
      .current
      .selections
      .get(self.current.primary_selection)
      .expect("selections should always contain a primary selection")
  }

  pub fn set_selections(&mut self, selections: Vec<Selection>) {
    if selections.is_empty() {
      return;
    }
    self.current.selections = selections;
    self.merge_overlapping_selections();
  }

  pub fn undo(&mut self) {
    let Some(changes) = self.history.backward() else {
      return;
    };
    changes.apply(&mut self.current.contents);
    // TODO replace existing selections with selected changes
    for selection in self.current.selections.iter_mut() {
      selection.adjust(&self.current.contents, &None);
    }
    self.merge_overlapping_selections();
  }

  pub fn redo(&mut self) {
    let Some(changes) = self.history.forward() else {
      return;
    };
    changes.apply(&mut self.current.contents);
    // TODO replace existing selections with selected changes
    for selection in self.current.selections.iter_mut() {
      selection.adjust(&self.current.contents, &None);
    }
    self.merge_overlapping_selections();
  }

  pub fn apply_operations(&mut self, ops: &[Op]) {
    for op in ops.iter() {
      for i in 0..self.current.selections.len() {
        let selection = self.current.selections.get_mut(i).expect(
          "should be able to retrieve selection at index less than length when applying operation",
        );
        let change = selection.apply_operation(&mut self.current.contents, *op);
        for j in i + 1..self.current.selections.len() {
          let next_selection = self
            .current
            .selections
            .get_mut(j)
            .expect("should be able to retrieve selection at index less than length when adjusting selections after applying operation");
          next_selection.adjust(&self.current.contents, &change);
        }
        change.map(|c| self.history.record(c));
      }
    }
    self.merge_overlapping_selections();
  }

  pub fn copy(&mut self) -> Vec<String> {
    let mut contents = Vec::with_capacity(self.current.selections.len());
    for i in 0..self.current.selections.len() {
      let i = (self.current.primary_selection + i) % self.current.selections.len();
      let selection = self
        .current
        .selections
        .get(i)
        .expect("should be able to retrieve selection at index less than length when copying");
      let content = selection.slice(&self.current.contents);
      contents.push(content.into());
    }
    contents
  }

  pub fn paste(&mut self, contents: &[String]) {
    for content_i in 0..self.current.selections.len().min(contents.len()) {
      let selection_i =
        (self.current.primary_selection + content_i) % self.current.selections.len();
      let selection = self
        .current
        .selections
        .get_mut(selection_i)
        .expect("should be able to retrieve selection at index less than length when pasting");
      let content = contents
        .get(content_i)
        .expect("should be able to retrieve content at index less than length when pasting");
      let change = selection.apply_operation(&mut self.current.contents, Op::InsertStr(content));
      for j in selection_i + 1..self.current.selections.len() {
        let next_selection = self
          .current
          .selections
          .get_mut(j)
          .expect("should be able to retrieve selection at index less than length when adjusting selections after applying operation");
        next_selection.adjust(&self.current.contents, &change);
      }
      change.map(|c| self.history.record(c));
    }
    self.history.commit();
    self.merge_overlapping_selections();
  }

  fn merge_overlapping_selections(&mut self) {
    self.current.selections.sort_by(|a, b| {
      a.start()
        .partial_cmp(&b.start())
        .expect("selection bounds should be comparable")
    });
    self.current.selections = {
      let mut selections = vec![];
      let mut selections_iter = self.current.selections.drain(..);
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
    if self.current.primary_selection >= self.current.selections.len() {
      self.current.primary_selection = self.current.selections.len().saturating_sub(1);
    }
  }
}
