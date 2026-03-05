use crate::*;

pub struct View {
  pub buffer: Buffer,
  pub window: Window,
  pub mode: Box<dyn Mode>,
}

impl View {
  pub fn update(
    &mut self,
    registry: &mut Registry,
    recorder: &mut Recorder,
    key: Key,
  ) -> UpdateCommand {
    let result = self.mode.update(&mut self.buffer, registry, &mut self.window, key);
    match result {
      UpdateCommand::SwitchMode(next_mode) => self.mode = next_mode,
      UpdateCommand::SendKeys(keys) => recorder.add(keys),
      x => return x,
    }
    UpdateCommand::None
  }
}

#[derive(Default)]
pub struct Views {
  entries: Vec<View>,
  selected: usize,
}

impl Views {
  pub fn current(&mut self) -> Option<&mut View> {
    self.entries.get_mut(self.selected)
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

  pub fn next(&mut self) {
    let current = self.selected % self.entries.len();
    self.selected = current.wrapping_add(1) % self.entries.len();
  }

  pub fn previous(&mut self) {
    let current = self.selected % self.entries.len();
    self.selected = current.wrapping_sub(1) % self.entries.len();
  }
}

