use ropey::Rope;
use crate::{
  mode::Mode,
  selection::{
    Op,
    Selection,
  },
};

#[derive(Clone)]
pub struct Buffer {
  pub mode: Mode,
  pub command: Rope,
  pub contents: Rope,
  pub selections: Vec<Selection>,
  pub primary_selection: usize,
}

impl Buffer {
  pub fn new(contents: Rope) -> Buffer {
    Buffer {
      mode: Mode::Normal,
      command: Rope::new(),
      contents: contents,
      selections: vec![Selection::new_at_end(0, 0)],
      primary_selection: 0,
    }
  }

  pub fn primary_selection(&self) -> &Selection {
    self.selections.get(self.primary_selection).unwrap()
  }

  fn adjust_selections(&mut self) {
    self.selections.sort_by(|a, b| a.start().partial_cmp(&b.start()).unwrap());
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
        let selection = self.selections.get_mut(i).unwrap();
        let change = selection.apply(&mut self.contents, *op);
        for j in i+1..self.selections.len() {
          let next_selection = self.selections.get_mut(j).unwrap();
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
