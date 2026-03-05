use crate::*;
use std::collections::VecDeque;

#[derive(Default)]
pub struct Recorder {
  keys: Option<VecDeque<Key>>,
}

impl Recorder {
  pub fn take(&mut self) -> Option<impl Iterator<Item=Key>> {
    let keys = self.keys.take()?;
    Some(keys.into_iter())
  }

  pub fn add(&mut self, keys: Vec<Key>) {
    if let Some(ref mut existing_keys) = self.keys {
      for key in keys.iter().rev() {
        existing_keys.push_front(*key);
      }
    } else {
      self.keys = Some(keys.into());
    }
  }

}
