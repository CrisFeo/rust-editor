use crate::*;
use ropey::Rope;

#[derive(Debug, PartialEq)]
pub enum Change {
  Addition(usize, Rope),
  Removal(usize, Rope),
}

impl Change {
  pub fn invert(self) -> Self {
    match self {
      Change::Addition(index, content) => Change::Removal(index, content),
      Change::Removal(index, content) => Change::Addition(index, content),
    }
  }

  pub fn apply(&self, contents: &mut Rope) {
    match self {
      // TODO there might be a better way to insert this change into the target rope
      Change::Addition(i, c) => contents.insert(*i, &c.to_string()),
      Change::Removal(i, c) => contents.remove(*i..i+c.len_chars()),
    }
  }

  pub fn selections(&self) -> Vec<Selection> {
    // TODO
    Vec::new()
  }
}

#[derive(Debug, Default, PartialEq)]
pub struct Changes(Vec<Change>);

impl Changes {
  pub fn invert(self) -> Self {
    let mut changes = Vec::with_capacity(self.0.len());
    for change in self.0.into_iter().rev() {
      changes.push(change.invert());
    }
    Self(changes)
  }

  pub fn apply(&self, contents: &mut Rope) {
    for change in self.0.iter() {
      change.apply(contents);
    }
  }

  pub fn push(&mut self, change: Change) -> &Change {
    self.0.push(change);
    self.0.last().expect("should be able to retrieve last change after push")
  }
}
