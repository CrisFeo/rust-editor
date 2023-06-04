use crate::key::{Key, Modifiers};
use crossterm::{
  cursor,
  event::{read, Event, KeyCode, KeyModifiers},
  execute, queue,
  style::{Color, Print, SetBackgroundColor, SetForegroundColor},
  terminal,
};
use gag::Hold;
use std::io::{self, BufWriter, Stdout, Write};

#[derive(Debug, Copy, Clone, PartialEq)]
struct Cell(char, Color, Color);

pub struct Screen {
  _held_stderr: Hold,
  output: BufWriter<Stdout>,
  buffer: Vec<Option<Cell>>,
  width: usize,
  height: usize,
  current_cursor: (usize, usize),
  current_bg: Color,
  current_fg: Color,
}

impl Screen {
  pub fn new() -> Self {
    let held_stderr = gag::Hold::stderr().expect("should gag stderr");
    let mut output = BufWriter::with_capacity(1 << 14, io::stdout());
    execute!(output, terminal::EnterAlternateScreen).expect("should enter alternate screen");
    terminal::enable_raw_mode().expect("should enable raw mode");
    queue!(output, cursor::Hide).expect("should hide cursor");
    queue!(output, cursor::MoveTo(0, 0)).expect("should move cursor");
    queue!(output, SetBackgroundColor(Color::Black)).expect("should set background color");
    queue!(output, SetForegroundColor(Color::White)).expect("should set foreground color");
    output.flush().expect("should flush queued output");
    let (width, height) = {
      let (width, height) = terminal::size().expect("should retrieve terminal size");
      (width as usize, height as usize)
    };
    let mut buffer = Vec::with_capacity(width * height);
    for _ in 0..(width * height) {
      buffer.push(Some(Cell(' ', Color::Black, Color::White)));
    }
    Screen {
      _held_stderr: held_stderr,
      output,
      buffer,
      width,
      height,
      current_cursor: (0, 0),
      current_bg: Color::Black,
      current_fg: Color::White,
    }
  }

  pub fn size(&self) -> (usize, usize) {
    (self.width, self.height)
  }

  pub fn poll(&mut self) -> (Modifiers, Key) {
    loop {
      match read().expect("should read input") {
        Event::Key(event) => {
          let modifiers = Modifiers {
            control: event.modifiers.contains(KeyModifiers::CONTROL),
            shift: event.modifiers.contains(KeyModifiers::SHIFT),
            alt: event.modifiers.contains(KeyModifiers::ALT),
          };
          let key = match event.code {
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Enter => Key::Enter,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::Tab => Key::Tab,
            KeyCode::BackTab => Key::BackTab,
            KeyCode::Delete => Key::Delete,
            KeyCode::Insert => Key::Insert,
            KeyCode::F(n) => Key::F(n),
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Null => Key::Null,
            KeyCode::Esc => Key::Esc,
          };
          return (modifiers, key);
        }
        Event::Resize(width, height) => {
          self.width = width as usize;
          self.height = height as usize;
          self.buffer.resize(self.width * self.height, None);
          let blank = Cell(' ', Color::Black, Color::White);
          for i in 0..self.buffer.len() {
            self.buffer[i] = Some(blank);
          }
        }
        _ => {}
      }
    }
  }

  pub fn clear(&mut self) {
    let blank = Cell(' ', Color::Black, Color::White);
    for cell in &mut self.buffer {
      if let Some(c) = cell {
        if *c == blank {
          *cell = None;
        } else {
          *cell = Some(blank);
        }
      }
    }
  }

  pub fn draw(&mut self, row: usize, col: usize, ch: char, bg: Color, fg: Color) {
    if row >= self.height || col >= self.width {
      return;
    }
    self.buffer[row * self.width + col] = Some(Cell(ch, bg, fg));
    if col < self.width {
      self.current_cursor = (row, col + 1);
    } else {
      self.current_cursor = (row + 1, col);
    }
  }

  fn set_cursor(&mut self, row: usize, col: usize) {
    if (row, col) != self.current_cursor {
      queue!(self.output, cursor::MoveTo(col as u16, row as u16)).expect("should move cursor");
      self.current_cursor = (row, col);
    }
  }

  fn set_bg(&mut self, color: Color) {
    if color != self.current_bg {
      queue!(self.output, SetBackgroundColor(color)).expect("should set background color");
      self.current_bg = color;
    }
  }

  fn set_fg(&mut self, color: Color) {
    if color != self.current_fg {
      queue!(self.output, SetForegroundColor(color)).expect("should set foreground color");
      self.current_fg = color;
    }
  }

  pub fn present(&mut self) {
    for row in 0..self.height {
      for col in 0..self.width {
        if let Some(Cell(ch, bg, fg)) = self.buffer[row * self.width + col] {
          self.set_cursor(row, col);
          self.set_bg(bg);
          self.set_fg(fg);
          queue!(self.output, Print(ch)).expect("should queue printing character");
        }
      }
    }
    self.output.flush().expect("should flush queued output");
  }
}

impl Drop for Screen {
  fn drop(&mut self) {
    terminal::disable_raw_mode().expect("should disable raw mode");
    execute!(self.output, terminal::LeaveAlternateScreen).expect("should leave alternate screen");
  }
}
