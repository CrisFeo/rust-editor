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
}
