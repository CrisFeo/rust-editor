extern crate gag;
extern crate ropey;
extern crate regex;
extern crate crossterm;

mod screen;
mod view;
mod window;
mod buffer;
mod selection;
mod mode;
mod key;

use std::{
  env,
  fs::File,
  io::BufReader,
};
use ropey::Rope;
use crate::{
  view::View,
  window::Window,
  buffer::Buffer,
  mode::update_mode,
};

fn main() {
  let filename = env::args().nth(1);
  let contents = if let Some(filename) = filename {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    Rope::from_reader(reader).unwrap()
  } else {
    Rope::new()
    //Rope::from_str("abcdefghijk\nhello\na\ndoggydog\n\n\nsplorteeeeeee")
  };
  let result = std::panic::catch_unwind(|| {
    let mut buffer = Buffer::new(contents);
    let mut view = View::new();
    let mut window = Window::new();
    loop {
      view.render(&buffer, &window);
      let (modifiers, key) = view.poll();
      match update_mode(&mut buffer, &mut window, modifiers, key) {
        Some(new_mode) => buffer.mode = new_mode,
        None => break,
      };
      window.set_size(view.buffer_size());
      window.scroll_into_view(&buffer.contents, buffer.primary_selection().cursor());
    }
  });
  if let Err(e) = result {
    std::panic::resume_unwind(e);
  }
}
