use crate::*;
use ropey::Rope;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, ErrorKind, Result};

pub struct Buffer {
  pub filename: Option<String>,
  pub contents: Rope,
  pub selections: Vec<Selection>,
  pub primary_selection: usize,
  pub history: History,
}

impl Buffer {
  pub fn new_scratch() -> Self {
    Self {
      filename: None,
      contents: Rope::new(),
      selections: vec![Selection::new_at_end(0, 0)],
      primary_selection: 0,
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
    Ok(Self {
      filename: Some(filename),
      contents,
      selections: vec![Selection::new_at_end(0, 0)],
      primary_selection: 0,
      history: Default::default(),
    })
  }

  pub fn save(&self) -> bool {
    if let Some(filename) = &self.filename {
      let file = match File::create(filename) {
        Ok(file) => file,
        Err(e) => {
          eprintln!("{e}");
          return false;
        }
      };
      let writer = BufWriter::new(file);
      if let Err(e) = self.contents.write_to(writer) {
        eprintln!("{e}");
        return false;
      };
    }
    true
  }

  pub fn primary_selection(&self) -> &Selection {
    self
      .selections
      .get(self.primary_selection)
      .expect("selections should always contain a primary selection")
  }

  pub fn set_selections(&mut self, selections: Vec<Selection>) {
    if selections.is_empty() {
      return;
    }
    self.selections = selections;
    self.cleanup_overlaps();
  }

  pub fn apply_operations(&mut self, ops: &[Op]) {
    for op in ops.iter() {
      for i in 0..self.selections.len() {
        let selection = self.selections.get_mut(i).expect(
          "should be able to retrieve selection at index less than length when applying operation",
        );
        let change = selection.apply_operation(&mut self.contents, *op);
        for j in i + 1..self.selections.len() {
          let next_selection = self
            .selections
            .get_mut(j)
            .expect("should be able to retrieve selection at index less than length when adjusting selections after applying operation");
          next_selection.adjust(&self.contents, change.as_ref());
        }
        change.map(|c| self.history.record(c));
      }
    }
    self.cleanup_overlaps();
  }

  pub fn cleanup_overlaps(&mut self) {
    merge_overlapping_selections(&mut self.selections);
    if self.primary_selection >= self.selections.len() {
      self.primary_selection = self.selections.len().saturating_sub(1);
    }
  }
}
