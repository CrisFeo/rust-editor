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

  pub fn apply(&self, contents: &mut Rope) -> Selection {
    match self {
      Change::Addition(begin, content) => {
        // TODO there might be a better way to insert this change into the target rope
        contents.insert(*begin, &content.to_string());
        Selection::new_at_end(*begin, begin + content.len_chars().saturating_sub(1))
      },
      Change::Removal(begin, content) => {
        contents.remove(*begin..begin+content.len_chars());
        Selection::new_at_end(*begin, *begin)
      },
    }
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

  pub fn apply(&self, contents: &mut Rope) -> Vec<Selection> {
    let mut selections: Vec<Selection> = Vec::new();
    for change in self.0.iter() {
      let selection = change.apply(contents);
      for selection in selections.iter_mut() {
        selection.adjust(contents, Some(change));
      }
      selections.push(selection);
      merge_overlapping_selections(&mut selections);
    }
    selections
  }

  pub fn push(&mut self, change: Change) -> &Change {
    self.0.push(change);
    self.0.last().expect("should be able to retrieve last change after push")
  }
}
