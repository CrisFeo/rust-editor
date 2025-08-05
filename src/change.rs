use ropey::Rope;

#[derive(Debug, PartialEq)]
pub enum Change {
  Addition(usize, Rope),
  Removal(usize, Rope),
}

impl Change {
  pub fn invert(&self) -> Self {
    // Note that cloning ropes like we do here is O(1)
    match self {
      Change::Addition(index, content) => Change::Removal(*index, content.clone()),
      Change::Removal(index, content) => Change::Addition(*index, content.clone()),
    }
  }

  pub fn apply(&self, contents: &mut Rope) {
    // TODO there might be a better way to insert this change into the target rope
    match self {
      Change::Addition(i, c) => contents.insert(*i, &c.to_string()),
      Change::Removal(i, c) => contents.remove(*i..i+c.len_chars()),
    }
  }
}
