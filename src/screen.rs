use crate::*;
use crossterm::event::{
  read, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyCode, KeyModifiers, MouseEventKind,
};
use crossterm::style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, queue, terminal};
use gag::Hold;
use std::io::{self, BufWriter, Stdout, Write};

impl From<Color> for crossterm::style::Color {
  fn from(c: Color) -> Self {
    match c {
      Color::DarkGrey => crossterm::style::Color::DarkGrey,
      Color::Black => crossterm::style::Color::Black,
      Color::Red => crossterm::style::Color::Red,
      Color::DarkRed => crossterm::style::Color::DarkRed,
      Color::Green => crossterm::style::Color::Green,
      Color::DarkGreen => crossterm::style::Color::DarkGreen,
      Color::Yellow => crossterm::style::Color::Yellow,
      Color::DarkYellow => crossterm::style::Color::DarkYellow,
      Color::Blue => crossterm::style::Color::Blue,
      Color::DarkBlue => crossterm::style::Color::DarkBlue,
      Color::Magenta => crossterm::style::Color::Magenta,
      Color::DarkMagenta => crossterm::style::Color::DarkMagenta,
      Color::Cyan => crossterm::style::Color::Cyan,
      Color::DarkCyan => crossterm::style::Color::DarkCyan,
      Color::White => crossterm::style::Color::White,
      Color::Grey => crossterm::style::Color::Grey,
      Color::Rgb(r, g, b) => (r, g, b).into(),
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Cell {
  Unknown,
  Unchanged,
  Changed(char, Option<Color>, Option<Color>),
}

pub enum Event {
  Redraw,
  Key(Key),
}

pub struct Screen {
  _held_stderr: Hold,
  output: BufWriter<Stdout>,
  buffer: Vec<Cell>,
  width: usize,
  height: usize,
  current_cursor: (usize, usize),
  current_bg: Option<Color>,
  current_fg: Option<Color>,
}

impl Screen {
  pub fn create() -> Self {
    let held_stderr = gag::Hold::stderr().expect("should gag stderr");
    let mut output = BufWriter::with_capacity(1 << 14, io::stdout());
    execute!(output, terminal::EnterAlternateScreen).expect("should enter alternate screen");
    terminal::enable_raw_mode().expect("should enable raw mode");
    queue!(output, terminal::Clear(ClearType::All)).expect("should clear screen");
    queue!(output, cursor::Hide).expect("should hide cursor");
    queue!(output, cursor::MoveTo(0, 0)).expect("should move cursor when setting up");
    queue!(output, ResetColor)
      .expect("should reset colors when setting up");
    queue!(output, EnableMouseCapture).expect("should enable mouse when setting up");
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
      current_bg: None,
      current_fg: None,
    }
  }

  pub fn size(&self) -> (usize, usize) {
    (self.width, self.height)
  }

  pub fn poll(&mut self) -> Event {
    loop {
      let event = read().expect("should read input");
      match event {
        CrosstermEvent::Mouse(event) => match event.kind {
          MouseEventKind::ScrollUp => return Event::Key(Key::Up),
          MouseEventKind::ScrollDown => return Event::Key(Key::Down),
          MouseEventKind::ScrollLeft => return Event::Key(Key::Left),
          MouseEventKind::ScrollRight => return Event::Key(Key::Right),
          _ => {}
        },
        CrosstermEvent::Key(event) => match event.code {
          KeyCode::Backspace => return Event::Key(Key::Backspace),
          KeyCode::Enter => return Event::Key(Key::Enter),
          KeyCode::Left => return Event::Key(Key::Left),
          KeyCode::Right => return Event::Key(Key::Right),
          KeyCode::Up => return Event::Key(Key::Up),
          KeyCode::Down => return Event::Key(Key::Down),
          KeyCode::Tab => return Event::Key(Key::Tab),
          KeyCode::Esc => return Event::Key(Key::Esc),
          KeyCode::Char('z') if event.modifiers & KeyModifiers::CONTROL == KeyModifiers::CONTROL => {
            self.suspend();
            return Event::Redraw;
          },
          KeyCode::Char(c) => return Event::Key(Key::Char(c)),
          _ => {}
        },
        CrosstermEvent::Resize(width, height) => {
          self.width = width as usize;
          self.height = height as usize;
          self.buffer.resize(self.width * self.height, Cell::Unknown);
          return Event::Redraw;
        }
        _ => {}
      }
    }
  }

  pub fn suspend(&mut self) {
    execute!(self.output, DisableMouseCapture).expect("should disable mouse");
    execute!(self.output, cursor::Show).expect("should show cursor");
    terminal::disable_raw_mode().expect("should disable raw mode");
    execute!(self.output, terminal::LeaveAlternateScreen).expect("should leave alternate screen");
    unsafe { libc::raise(libc::SIGTSTP) };
    execute!(self.output, terminal::EnterAlternateScreen).expect("should enter alternate screen");
    terminal::enable_raw_mode().expect("should disable raw mode");
    execute!(self.output, cursor::Hide).expect("should hide cursor");
    execute!(self.output, EnableMouseCapture).expect("should enable mouse");
    let size = terminal::window_size().expect("should retrieve size after resume");
    self.width = size.columns as usize;
    self.height = size.rows as usize;
    self.buffer.resize(self.width * self.height, Cell::Unknown);
  }

  pub fn clear(&mut self) {
    let blank = Cell::Changed(' ', None, None);
    for cell in &mut self.buffer {
      if *cell == blank {
        *cell = Cell::Unchanged;
      } else {
        *cell = blank;
      }
    }
  }

  pub fn draw(&mut self, row: usize, col: usize, ch: char, bg: Option<Color>, fg: Option<Color>) {
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

  fn set_bg(&mut self, color: Option<Color>) {
    if color != self.current_bg {
      if let Some(color) = color {
        queue!(self.output, SetBackgroundColor(color.into())).expect("should set background color");
      } else {
        queue!(self.output, ResetColor).expect("should reset colors");
        if let Some(color) = self.current_fg {
          queue!(self.output, SetForegroundColor(color.into())).expect("should reset other color");
        }
      }
      self.current_bg = color;
    }
  }

  fn set_fg(&mut self, color: Option<Color>) {
    if color != self.current_fg {
      if let Some(color) = color {
        queue!(self.output, SetForegroundColor(color.into())).expect("should set foreground color");
      } else {
        queue!(self.output, ResetColor).expect("should reset colors");
        if let Some(color) = self.current_bg {
          queue!(self.output, SetBackgroundColor(color.into())).expect("should reset other color");
        }
      }
      self.current_fg = color;
    }
  }

  pub fn present(&mut self) {
    for row in 0..self.height {
      for col in 0..self.width {
        match self.buffer[row * self.width + col] {
          Cell::Unknown => {
            self.set_cursor(row, col);
            self.set_bg(None);
            self.set_fg(None);
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
    execute!(self.output, DisableMouseCapture).expect("should disable mouse");
    execute!(self.output, cursor::Show).expect("should show cursor");
    terminal::disable_raw_mode().expect("should disable raw mode");
    execute!(self.output, terminal::LeaveAlternateScreen).expect("should leave alternate screen");
  }
}
