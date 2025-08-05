use crate::*;

#[derive(Default)]
pub struct History {
  prev: Vec<Change>,
  next: Vec<Change>,
}

impl History {
  pub fn push(&mut self, edit: Change) -> &Change {
    self.prev.push(edit);
    self.next.clear();
    self.prev.last().expect("should be able to retrieve last edit during push")
  }

  pub fn backward(&mut self) -> Option<&Change> {
    let edit = self.prev.pop()?;
    self.next.push(edit.invert());
    self.next.last()
  }

  pub fn forward(&mut self) -> Option<&Change> {
    let edit = self.next.pop()?;
    self.prev.push(edit.invert());
    self.prev.last()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use ropey::Rope;

  #[test]
  fn kitchen_sink() {
    let mut contents = Rope::new();
    let mut a = |e: &Change| {
      e.apply(&mut contents);
      contents.clone()
    };
    let mut h = History::default();
    assert_eq!(a(h.push(Change::Addition(0, "hello world".into()))), "hello world");
    assert_eq!(a(h.push(Change::Removal(6, "world".into()))), "hello ");
    assert_eq!(a(h.push(Change::Addition(6, "there".into()))), "hello there");
    assert_eq!(a(h.push(Change::Addition(11, " yall".into()))), "hello there yall");
    assert_eq!(a(h.backward().unwrap()), "hello there");
    assert_eq!(a(h.backward().unwrap()), "hello ");
    assert_eq!(a(h.backward().unwrap()), "hello world");
    assert_eq!(a(h.backward().unwrap()), "");
    assert_eq!(h.backward(), None);
    assert_eq!(a(h.forward().unwrap()), "hello world");
    assert_eq!(a(h.forward().unwrap()), "hello ");
    assert_eq!(a(h.forward().unwrap()), "hello there");
    assert_eq!(a(h.forward().unwrap()), "hello there yall");
    assert_eq!(h.forward(), None);
    assert_eq!(a(h.backward().unwrap()), "hello there");
    assert_eq!(a(h.backward().unwrap()), "hello ");
    assert_eq!(a(h.push(Change::Addition(6, "friends".into()))), "hello friends");
    assert_eq!(a(h.push(Change::Addition(13, " and countrymen".into()))), "hello friends and countrymen");
    assert_eq!(a(h.backward().unwrap()), "hello friends");
    assert_eq!(a(h.forward().unwrap()), "hello friends and countrymen");
  }
}
