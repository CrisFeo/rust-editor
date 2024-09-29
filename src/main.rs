use rust_editor::*;
use std::process::exit;
use std::panic::{catch_unwind, resume_unwind};

fn main() {
  let filename = std::env::args().nth(1);
  let result = catch_unwind(|| {
    let buffer = match filename {
      Some(filename) => Buffer::new_from_file(filename),
      None => Ok(Buffer::new_scratch()),
    };
    let mut buffer = match buffer {
      Ok(buffer) => buffer,
      Err(e) => {
        eprintln!("{}", e);
        exit(1);
      }
    };
    let mut mode: Box<dyn Mode> = Box::new(Normal);
    let mut view = View::create();
    let mut window = Window::default();
    loop {
      view.render(mode.as_ref(), &buffer, &window);
      let (modifiers, key) = view.poll();
      let result = mode.update(&mut buffer, &mut window, modifiers, key);
      match result {
        UpdateCommand::Switch(next_mode) => mode = next_mode,
        UpdateCommand::None => {},
        UpdateCommand::Quit => break,
      }
      window.set_size(view.buffer_size());
      window.scroll_into_view(
        &buffer.current.contents,
        buffer.primary_selection().cursor(),
      );
    }
  });
  if let Err(e) = result {
    resume_unwind(e);
  }
}
