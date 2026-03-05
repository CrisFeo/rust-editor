use crate::*;

pub struct View {
  pub buffer: Buffer,
  pub window: Window,
  pub mode: Box<dyn Mode>,
}

#[derive(Default)]
pub struct Views {
  entries: Vec<View>,
  selected: usize,
}

impl Views {
  pub fn current(&mut self) -> &mut View {
    self.entries.get_mut(self.selected).expect("should always have at least one view")
  }

  pub fn current_index(&self) -> usize {
    self.selected
  }

  pub fn add(&mut self, buffer: Buffer, window: Window) -> usize {
    let index = self.entries.len();
    let view = View {
      buffer,
      window,
      mode: Box::new(Normal::default()),
    };
    self.entries.push(view);
    index
  }

  pub fn del(&mut self, index: usize) {
    self.entries.remove(index);
    self.selected %= self.entries.len();
  }

  pub fn count(&self) -> usize {
    self.entries.len()
  }

  pub fn find(&mut self, filename: &str) -> Option<usize> {
    self.entries
      .iter()
      .enumerate()
      .find(|(_, e)| e.buffer.filename.as_deref() == Some(filename))
      .map(|(i, _)| i)
  }

  pub fn goto(&mut self, index: usize) {
    self.selected = index % self.entries.len();
  }

  pub fn next(&mut self) {
    let current = self.selected % self.entries.len();
    self.selected = current.wrapping_add(1) % self.entries.len();
  }

  pub fn prev(&mut self) {
    let current = self.selected % self.entries.len();
    self.selected = current.wrapping_sub(1) % self.entries.len();
  }
}

