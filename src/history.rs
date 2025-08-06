use crate::*;

#[derive(Default)]
pub struct History {
  pending: Option<Changes>,
  prev: Vec<Changes>,
  next: Vec<Changes>,
}

impl History {
  pub fn record(&mut self, change: Change) -> &Change {
    let pending = self.pending.get_or_insert_default();
    pending.push(change)
  }

  pub fn commit(&mut self) {
    let Some(pending) = self.pending.take() else {
      return;
    };
    self.prev.push(pending);
    self.next.clear();
  }

  pub fn backward(&mut self) -> Option<&Changes> {
    self.commit();
    let change = self.prev.pop()?;
    self.next.push(change.invert());
    self.next.last()
  }

  pub fn forward(&mut self) -> Option<&Changes> {
    self.commit();
    self.pending.take();
    let change = self.next.pop()?;
    self.prev.push(change.invert());
    self.prev.last()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use ropey::Rope;

  #[test]
  fn kitchen_sink() {
    let mut c = Rope::new();
    let mut h = History::default();
    test_record(&mut c, h.record(add(0, "hello world")), "hello world");
    test_record(&mut c, h.record(del(6, "world")), "hello ");
    h.commit();
    test_record(&mut c, h.record(add(6, "there")), "hello there");
    h.commit();
    test_record(&mut c, h.record(add(11, " yall")), "hello there yall");
    test_seek(&mut c, h.backward().unwrap(), "hello there");
    test_record(&mut c, h.record(del(9, "re")), "hello the");
    test_seek(&mut c, h.backward().unwrap(), "hello there");
    test_seek(&mut c, h.backward().unwrap(), "hello ");
    test_seek(&mut c, h.backward().unwrap(), "");
    assert_eq!(h.backward(), None);
    test_seek(&mut c, h.forward().unwrap(), "hello ");
    test_seek(&mut c, h.forward().unwrap(), "hello there");
    test_seek(&mut c, h.forward().unwrap(), "hello the");
    assert_eq!(h.forward(), None);
    test_seek(&mut c, h.backward().unwrap(), "hello there");
    test_seek(&mut c, h.backward().unwrap(), "hello ");
    test_record(&mut c, h.record(add(6, "friends")), "hello friends");
    test_record(&mut c, h.record(add(13, " and countrymen")), "hello friends and countrymen");
    h.commit();
    test_seek(&mut c, h.backward().unwrap(), "hello ");
    test_seek(&mut c, h.forward().unwrap(), "hello friends and countrymen");
  }

  fn add(index: usize, content: &str) -> Change {
    Change::Addition(index, content.into())
  }
  fn del(index: usize, content: &str) -> Change {
    Change::Removal(index, content.into())
  }

  fn test_record(contents: &mut Rope, change: &Change, expected: &str) {
    change.apply(contents);
    assert_eq!(contents, expected);
  }

  fn test_seek(contents: &mut Rope, changes: &Changes, expected: &str) {
    changes.apply(contents);
    assert_eq!(contents, expected);
  }
}
