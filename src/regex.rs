use ropey::Rope;
use hyperscan::prelude::*;

pub struct Regex {
  db: StreamingDatabase,
  scratch: Scratch,
}

impl Regex {
  pub fn new(pattern: &str) -> Regex {
    let pattern = pattern!{pattern; SOM_LEFTMOST | MULTILINE | DOTALL};
    let db = pattern.build().unwrap();
    let scratch = db.alloc_scratch().unwrap();
    Regex {
      db,
      scratch,
    }
  }

  pub fn scan(&mut self, contents: &Rope, start: usize, end: usize) -> Vec<(usize, usize)> {
    let stream = self.db.open_stream().unwrap();
    let mut results = vec![];
    let start_byte = contents.char_to_byte(start);
    let end_byte = contents.char_to_byte(end);
    let bytes = contents.bytes_at(start_byte);
    let mut count = 0;
    let mut callback = |_, from, to, _| {
      let from = start_byte.saturating_add(from as usize);
      let start = contents.byte_to_char(from);
      let to = start_byte.saturating_add(to as usize);
      let end = contents.byte_to_char(to).saturating_sub(1);
      results.push((start, end));
      Matching::Continue
    };
    for b in bytes {
      if count > end_byte - start_byte {
        break;
      }
      stream.scan([b], &self.scratch, &mut callback).unwrap();
      count += 1;
    }
    stream.close(&self.scratch, callback).unwrap();
    results
  }
}

