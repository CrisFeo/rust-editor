extern crate gag;
extern crate ropey;
extern crate crossterm;

mod screen;
mod view;
mod window;
mod buffer;
mod selection;
mod mode;
mod key;
mod regex;

use std::{
  env,
};
use std::fs::{
  File,
};
use std::io::{
  BufReader,
};
use ropey::{
  Rope,
};
use view::{
  View,
};
use window::{
  Window,
};
use buffer::{
  Buffer,
};
use mode::{
  Mode,
  update_mode_normal,
  update_mode_insert,
  update_mode_filter,
  update_mode_reject,
};

fn main() {
  let filename = env::args().nth(1);
  let contents = if let Some(filename) = filename {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    Rope::from_reader(reader).unwrap()
  } else {
    Rope::from_str("abcdefghijk\nhello\na\ndoggydog\n\n\nsplorteeeeeee")
  };
  let result = std::panic::catch_unwind(|| {
    let mut buffer = Buffer::new(contents);
    let mut view = View::new();
    let mut window = Window::new();
    loop {
      view.render(&buffer, &window);
      let (modifiers, key) = view.poll();
      let new_mode = match &buffer.mode {
        Mode::Normal => update_mode_normal(&mut buffer, &mut window, modifiers, key),
        Mode::Insert => update_mode_insert(&mut buffer, &mut window, modifiers, key),
        Mode::Filter => update_mode_filter(&mut buffer, &mut window, modifiers, key),
        Mode::Reject => update_mode_reject(&mut buffer, &mut window, modifiers, key),
      };
      if let Some(mode) = new_mode {
        buffer.mode = mode;
      } else {
        break;
      }
      window.set_size(view.buffer_size());
      window.scroll_into_view(&buffer.contents, buffer.primary_selection().cursor());
    }
  });
  if let Err(e) = result {
    std::panic::resume_unwind(e);
  }
}
