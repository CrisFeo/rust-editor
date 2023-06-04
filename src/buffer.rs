use crate::{
  mode::Mode,
  selection::{Op, Selection},
};
use ropey::Rope;
use std::{
  fs::{OpenOptions, File},
  io::{ErrorKind, Result, BufReader, BufWriter},
};

#[derive(Clone)]
pub struct Buffer {
  pub filename: Option<String>,
  pub contents: Rope,
  pub mode: Mode,
  pub command: Option<Rope>,
  pub selections: Vec<Selection>,
  pub primary_selection: usize,
  pub preview_selections: Option<Vec<Selection>>,
}

impl Buffer {
  pub fn new() -> Buffer {
    Buffer {
      filename: None,
      contents: Rope::new(),
      mode: Mode::Normal,
      command: None,
      selections: vec![Selection::new_at_end(0, 0)],
      primary_selection: 0,
      preview_selections: None,
    }
  }

  pub fn new_from_file(filename: String) -> Result<Buffer> {
    let file = OpenOptions::new().read(true).open(&filename);
    let contents = match file {
      Ok(file) => {
        let reader = BufReader::new(file);
        Rope::from_reader(reader)
      },
      Err(e) if e.kind() == ErrorKind::NotFound => Ok(Rope::new()),
      Err(e) => Err(e),
    }?;
    Ok(Buffer {
      filename: Some(filename),
      contents,
      mode: Mode::Normal,
      command: None,
      selections: vec![Selection::new_at_end(0, 0)],
      primary_selection: 0,
      preview_selections: None,
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
      match self.contents.write_to(writer) {
        Ok(_) => {}
        Err(e) => {
          eprintln!("{}", e);
          return;
        }
      };
    }
  }

  pub fn primary_selection(&self) -> &Selection {
    self
      .selections
      .get(self.primary_selection)
      .expect("selections should always contain a primary selection")
  }

  fn adjust_selections(&mut self) {
    self.selections.sort_by(|a, b| {
      a.start()
        .partial_cmp(&b.start())
        .expect("selection bounds should be comparable")
    });
    self.selections = {
      let mut selections = vec![];
      let mut selections_iter = self.selections.drain(..);
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
    if self.primary_selection >= self.selections.len() {
      self.primary_selection = self.selections.len().saturating_sub(1);
    }
  }

  pub fn apply_operations(&mut self, ops: &[Op]) {
    for op in ops.iter() {
      for i in 0..self.selections.len() {
        let selection = self
          .selections
          .get_mut(i)
          .expect("should be able to retrieve selection at index less than length");
        let change = selection.apply(&mut self.contents, *op);
        for j in i + 1..self.selections.len() {
          let next_selection = self
            .selections
            .get_mut(j)
            .expect("should be able to retrieve selection at index less than length");
          next_selection.adjust(&self.contents, change);
        }
      }
    }
    self.adjust_selections();
  }

  pub fn set_selections(&mut self, selections: Vec<Selection>) {
    if selections.len() == 0 {
      return;
    }
    self.selections = selections;
    self.adjust_selections();
  }
}
