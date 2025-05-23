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

#[derive(Clone)]
pub struct Buffer {
  pub current: Snapshot,
  pub filename: Option<String>,
  history: History,
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
      history: Default::default(),
      filename: None,
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
      history: Default::default(),
      filename: Some(filename),
    })
  }

  pub fn save(&self) {
    if let Some(filename) = &self.filename {
      let file = match File::create(filename) {
        Ok(file) => file,
        Err(e) => {
          eprintln!("{}", e);
          return;
        }
      };
      let writer = BufWriter::new(file);
      match self.current.contents.write_to(writer) {
        Ok(_) => {}
        Err(e) => eprintln!("{}", e),
      };
    }
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
    self.adjust_selections();
  }

  pub fn undo(&mut self) {
    if let Some(snapshot) = self.history.back(&self.current) {
      self.current = snapshot;
    };
  }

  pub fn redo(&mut self) {
    if let Some(snapshot) = self.history.forward() {
      self.current = snapshot;
    };
  }

  pub fn push_history(&mut self) {
    self.history.push(self.current.clone());
  }

  pub fn apply_operations(&mut self, ops: &[Op]) {
    for op in ops.iter() {
      for i in 0..self.current.selections.len() {
        let selection = self.current.selections.get_mut(i).expect(
          "should be able to retrieve selection at index less than length when applying operation",
        );
        let change = selection.apply(&mut self.current.contents, *op);
        for j in i + 1..self.current.selections.len() {
          let next_selection = self
            .current
            .selections
            .get_mut(j)
            .expect("should be able to retrieve selection at index less than length when adjusting selections after applying operation");
          next_selection.adjust(&self.current.contents, change);
        }
      }
    }
    self.adjust_selections();
  }

  fn adjust_selections(&mut self) {
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
