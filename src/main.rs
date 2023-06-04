extern crate crossterm;
extern crate gag;
extern crate regex;
extern crate ropey;

mod buffer;
mod key;
mod mode;
mod screen;
mod selection;
mod view;
mod window;

use std::process::exit;

use crate::{buffer::Buffer, mode::update_mode, view::View, window::Window};

fn main() {
  let filename = std::env::args().nth(1);
  let result = std::panic::catch_unwind(|| {
    let buffer = match filename {
      Some(filename) => Buffer::new_from_file(filename),
      None => Ok(Buffer::new()),
    };
    let mut buffer = match buffer {
      Ok(buffer) => buffer,
      Err(e) => {
        eprintln!("{}", e);
        exit(1);
      },
    };
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
