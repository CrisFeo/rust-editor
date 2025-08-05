use crate::*;
use crossterm::event::{read, Event, KeyCode, KeyModifiers};
use crossterm::style::{Print, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, queue, terminal};
use gag::Hold;
use std::io::{self, BufWriter, Stdout, Write};

impl From<Color> for crossterm::style::Color {
  fn from(c: Color) -> Self {
    (c.0, c.1, c.2).into()
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Cell {
  Unknown,
  Unchanged,
  Changed(char, Color, Color),
}

pub struct Screen {
  _held_stderr: Hold,
  output: BufWriter<Stdout>,
  buffer: Vec<Cell>,
  width: usize,
  height: usize,
  current_cursor: (usize, usize),
  default_bg: Color,
  default_fg: Color,
  current_bg: Color,
  current_fg: Color,
}

impl Screen {
  pub fn create(bg: Color, fg: Color) -> Self {
    let held_stderr = gag::Hold::stderr().expect("should gag stderr");
    let mut output = BufWriter::with_capacity(1 << 14, io::stdout());
    execute!(output, terminal::EnterAlternateScreen).expect("should enter alternate screen");
    terminal::enable_raw_mode().expect("should enable raw mode");
    queue!(output, terminal::Clear(ClearType::All)).expect("should clear screen");
    queue!(output, cursor::Hide).expect("should hide cursor");
    queue!(output, cursor::MoveTo(0, 0)).expect("should move cursor when setting up");
    queue!(output, SetBackgroundColor(bg.into()))
      .expect("should set background color when setting up");
    queue!(output, SetForegroundColor(fg.into()))
      .expect("should set foreground color when setting up");
    output
      .flush()
      .expect("should flush queued output when setting up");
    let (width, height) = {
      let (width, height) = terminal::size().expect("should retrieve terminal size");
      (width as usize, height as usize)
    };
    let mut buffer = Vec::with_capacity(width * height);
    for _ in 0..(width * height) {
      buffer.push(Cell::Unknown);
    }
    Self {
      _held_stderr: held_stderr,
      output,
      buffer,
      width,
      height,
      current_cursor: (0, 0),
      default_bg: bg,
      default_fg: fg,
      current_bg: bg,
      current_fg: fg,
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
          self.buffer.resize(self.width * self.height, Cell::Unknown);
        }
        _ => {}
      }
    }
  }

  pub fn clear(&mut self) {
    let blank = Cell::Changed(' ', self.default_bg, self.default_fg);
    for cell in &mut self.buffer {
      if *cell == blank {
        *cell = Cell::Unchanged;
      } else {
        *cell = blank;
      }
    }
  }

  pub fn draw(&mut self, row: usize, col: usize, ch: char, bg: Color, fg: Color) {
    if row >= self.height || col >= self.width {
      return;
    }
    self.buffer[row * self.width + col] = Cell::Changed(ch, bg, fg);
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
      queue!(self.output, SetBackgroundColor(color.into())).expect("should set background color");
      self.current_bg = color;
    }
  }

  fn set_fg(&mut self, color: Color) {
    if color != self.current_fg {
      queue!(self.output, SetForegroundColor(color.into())).expect("should set foreground color");
      self.current_fg = color;
    }
  }

  pub fn present(&mut self) {
    for row in 0..self.height {
      for col in 0..self.width {
        match self.buffer[row * self.width + col] {
          Cell::Unknown => {
            self.set_cursor(row, col);
            self.set_bg(self.default_bg);
            self.set_fg(self.default_fg);
            queue!(self.output, Print(' ')).expect("should queue printing character");
          }
          Cell::Unchanged => {}
          Cell::Changed(ch, bg, fg) => {
            self.set_cursor(row, col);
            self.set_bg(bg);
            self.set_fg(fg);
            queue!(self.output, Print(ch)).expect("should queue printing character");
          }
        }
      }
    }
    self
      .output
      .flush()
      .expect("should flush queued output when presenting");
  }
}

impl Drop for Screen {
  fn drop(&mut self) {
    execute!(self.output, cursor::Show).expect("should show cursor");
    terminal::disable_raw_mode().expect("should disable raw mode");
    execute!(self.output, terminal::LeaveAlternateScreen).expect("should leave alternate screen");
  }
}
