use rust_editor::*;
use std::panic::{catch_unwind, resume_unwind};
use std::process::exit;

fn main() {
  let result = catch_unwind(|| {
    let filename = std::env::args().nth(1);
    let theme = load_theme();
    let mut ui = Ui::create(theme);
    let mut views = Views::default();
    let mut registry = Registry::default();
    let mut recorder = Recorder::default();
    views.add(
      load_initial_buffer(filename),
      Window::new(ui.buffer_size()),
    );
    'app_loop: loop {
      let Some(view) = views.current() else {
        break 'app_loop;
      };
      if let Some(keys) = recorder.take() {
        for key in keys {
          let command = view.update(&mut registry, &mut recorder, key);
          if let UpdateCommand::Quit = command {
            break 'app_loop
          }
        }
      } else {
        ui.render(view);
        let event = ui.poll();
        match event {
          Event::Key(key) => {
            let command = view.update(&mut registry, &mut recorder, key);
            if let UpdateCommand::Quit = command {
              break 'app_loop
            }
          },
          Event::Redraw => {
            view.window.set_size(ui.buffer_size());
          },
        }
      }
      if view.window.keep_cursor_visible {
        let target_cursor = view.mode
          .preview_selections()
          .and_then(|ps| ps.first())
          .unwrap_or(view.buffer.primary_selection())
          .cursor();
        view.window.scroll_into_view(&view.buffer.contents, target_cursor);
      }
      view.window.keep_cursor_visible = true;
    }
  });
  if let Err(e) = result {
    resume_unwind(e);
  }
}

fn load_initial_buffer(filename: Option<String>) -> Buffer {
  let buffer = match filename {
    Some(filename) => Buffer::new_from_file(filename),
    None => Ok(Buffer::new_scratch()),
  };
  match buffer {
    Ok(buffer) => buffer,
    Err(e) => {
      eprintln!("{e}");
      exit(1);
    }
  }
}

fn load_theme() -> Theme {
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
}
