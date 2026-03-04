use rust_editor::*;
use std::panic::{catch_unwind, resume_unwind};
use std::process::exit;
use std::collections::VecDeque;

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
      let mut ramp = [
        Some(Color::Black),
        Some(Color::DarkGrey),
        Some(Color::Grey),
        Some(Color::White),
      ];
      if option_env!("THEME") == Some("light") {
        ramp.reverse();
      }
      Theme {
        default_face: (None, None),
        selection_primary_face: (ramp[1], ramp[0]),
        selection_secondary_face: (ramp[1], ramp[0]),
        cursor_primary_face: (ramp[3], ramp[0]),
        cursor_secondary_face: (ramp[2], ramp[0]),
        status_face: (ramp[0], ramp[3]),
        new_line_char: '¬',
        end_of_file_char: 'Ω',
      }
    };
    let mut view = View::create(theme);
    let mut window = Window::new(view.buffer_size());
    let mut macro_keys: Option<VecDeque<Key>> = None;
    loop {
      view.render(mode.as_ref(), &buffer, &window);
      let key = {
        if let Some(ref mut keys) = macro_keys {
          if let Some(key) = keys.pop_front() {
            key
          } else {
            view.poll()
          }
        } else {
          view.poll()
        }
      };
      let result = mode.update(&mut buffer, &mut registry, &mut window, key);
      match result {
        UpdateCommand::SwitchMode(next_mode) => mode = next_mode,
        UpdateCommand::SendKeys(keys) => {
          if let Some(ref mut macro_keys) = macro_keys {
            for key in keys.iter().rev() {
              macro_keys.push_front(*key);
            }
          } else {
            macro_keys = Some(keys.into());
          }
        },
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
