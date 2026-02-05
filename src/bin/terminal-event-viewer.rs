use rust_editor::*;
use std::panic::{catch_unwind, resume_unwind};

fn main() {
  let result = catch_unwind(|| {
    let mut t = Terminal::new();
    t.goto(0, 0);
    t.write("terminal event viewer ('q' to exit)");
    t.goto(0, 1);
    t.flush();
    loop {
      let c = t.poll();
      if c == Event::Char('q') {
        break;
      }
      t.write(format!("{c:?}\r\n"));
      t.flush();
    }
  });
  if let Err(e) = result {
    resume_unwind(e);
  }

}
