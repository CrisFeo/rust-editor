use rust_editor::*;
use std::panic::{catch_unwind, resume_unwind};
use std::process::exit;

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
        eprintln!("{e}");
        exit(1);
      }
    };
    let mut registry = Registry::default();
    let mut mode: Box<dyn Mode> = Box::new(Normal::default());
    let theme = {
      let ramp_0 = Color(255, 255, 255);
      let ramp_1 = Color(188, 188, 188);
      let ramp_2 = Color(120, 120, 120);
      let ramp_3 = Color(0, 0, 0);
      Theme {
        default_face: (ramp_0, ramp_3),
        selection_primary_face: (ramp_1, ramp_3),
        selection_secondary_face: (ramp_1, ramp_3),
        cursor_primary_face: (ramp_3, ramp_0),
        cursor_secondary_face: (ramp_2, ramp_0),
        status_face: (ramp_0, ramp_2),
        new_line_char: '¬',
        end_of_file_char: 'Ω',
      }
    };
    let mut view = View::create(theme);
    let mut window = Window::default();
    loop {
      view.render(mode.as_ref(), &buffer, &window);
      let key = view.poll();
      let result = mode.update(&mut buffer, &mut registry, &mut window, key);
      match result {
        UpdateCommand::Switch(next_mode) => mode = next_mode,
        UpdateCommand::None => {}
        UpdateCommand::Quit => break,
      }
      window.set_size(view.buffer_size());
      if window.keep_cursor_visible {
        let target_cursor = mode
          .preview_selections()
          .and_then(|ps| ps.first())
          .unwrap_or(buffer.primary_selection())
          .cursor();
        window.scroll_into_view(&buffer.contents, target_cursor);
      }
      window.keep_cursor_visible = true;
    }
  });
  if let Err(e) = result {
    resume_unwind(e);
  }
}
