use crate::*;

#[derive(Clone)]
enum State {
  Empty,
  Tracking(Vec<Snapshot>),
  Viewing {
    snapshots: Vec<Snapshot>,
    current: Snapshot,
    index: usize,
  },
}

#[derive(Clone)]
pub struct History {
  state: Option<State>,
}

impl Default for History {
  fn default() -> Self {
    Self { state: Some(State::Empty) }
  }
}

impl History {
  pub fn back(&mut self, current: &Snapshot) -> Option<Snapshot> {
    let mut result = None;
    let state = self.state.take().expect("history state cannot be updated recursively");
    self.state = Some(match state {
      State::Empty => State::Empty,
      State::Tracking(snapshots) => {
        let current = current.clone();
        let index = snapshots.len() - 1;
        result = Some(snapshots[index].clone());
        State::Viewing {
          snapshots,
          current,
          index,
        }
      },
      State::Viewing { snapshots, current, index } => {
        let index = index.saturating_sub(1);
        result = Some(snapshots[index].clone());
        State::Viewing {
          snapshots,
          current,
          index,
        }
      },
    });
    result
  }

  pub fn forward(&mut self) -> Option<Snapshot> {
    let mut result = None;
    let state = self.state.take().expect("history state cannot be updated recursively");
    self.state = Some(match state {
      State::Empty => State::Empty,
      State::Tracking(snapshots) => State::Tracking(snapshots),
      State::Viewing { snapshots, current, index } => {
        let index = index + 1;
        if index == snapshots.len() {
          result = Some(current);
          State::Tracking(snapshots)
        } else {
          let snapshot = snapshots[index].clone();
          result = Some(snapshot);
          State::Viewing {
            snapshots,
            current,
            index,
          }
        }
      },
    });
    result
  }

  pub fn push(&mut self, current: Snapshot) {
    let state = self.state.take().expect("history state cannot be updated recursively");
    self.state = Some(match state {
      State::Empty => {
        State::Tracking(vec![current])
      },
      State::Tracking(mut snapshots) => {
        snapshots.push(current);
        State::Tracking(snapshots)
      },
      State::Viewing { mut snapshots, current, index } => {
        snapshots.truncate(index + 1);
        snapshots.push(current.clone());
        State::Viewing {
          snapshots,
          current,
          index,
        }
      },
    });
  }
}

/*
#[derive(Clone, Default)]
pub struct History {
  snapshots: Vec<Snapshot>,
  pointer: Option<(usize, Snapshot)>,
}

impl History {
  pub fn back(&mut self, current: &Snapshot) -> Option<Snapshot> {
    if self.snapshots.is_empty() {
      return None;
    }
    match self.pointer.take() {
      Some((0, _)) => None,
      Some((index, pending)) => {
        let index = index - 1;
        self.pointer = Some((index, pending));
        let snapshot = self.snapshots[index].clone();
        Some(snapshot)
      },
      None => {
        let index = self.snapshots.len() - 1;
        self.pointer = Some((index, current.clone()));
        let snapshot = self.snapshots[index].clone();
        Some(snapshot)
      },
    }
  }

  pub fn forward(&mut self) -> Option<Snapshot> {
    if self.snapshots.is_empty() {
      return None;
    }
    match self.pointer.take() {
      Some((index, pending)) => {
        let index = index + 1;
        if index == self.snapshots.len() {
          self.pointer = None;
          Some(pending)
        } else {
          self.pointer = Some((index, pending));
          let snapshot = self.snapshots[index].clone();
          Some(snapshot)
        }
      },
      None => None,
    }
  }

  pub fn push(&mut self, current: Snapshot) {
		if let Some((index, _)) = self.pointer {
      self.snapshots.truncate(index + 1);
      self.pointer = None;
    }
    self.snapshots.push(current.clone());
  }
}
*/
