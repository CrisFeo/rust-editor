use gag::{
  Hold,
};
use std::io::{
  self,
  Stdout,
  BufWriter,
  Write,
};
use crossterm::{
  style::{
    Color,
    SetForegroundColor,
    SetBackgroundColor,
  },
  cursor,
  terminal,
  execute,
  queue,
  event::{
    Event,
    read,
    KeyCode,
    KeyModifiers,
  },
};
use crate::key::{
  Key,
  Modifiers,
};

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
    let held_stderr = gag::Hold::stderr().unwrap();
    let mut output = BufWriter::with_capacity(1 << 14, io::stdout());
    execute!(output, terminal::EnterAlternateScreen).unwrap();
    terminal::enable_raw_mode().unwrap();
    queue!(output, cursor::Hide).unwrap();
    queue!(output, cursor::MoveTo(0, 0)).unwrap();
    queue!(output, SetBackgroundColor(Color::Black)).unwrap();
    queue!(output, SetForegroundColor(Color::White)).unwrap();
    output.flush().unwrap();
    let (width, height) = {
      let (width, height) = terminal::size().unwrap();
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
      match read().unwrap() {
        Event::Key(event) => {
          let modifiers = Modifiers {
            control: event.modifiers.contains(KeyModifiers::CONTROL),
            shift:   event.modifiers.contains(KeyModifiers::SHIFT),
            alt:     event.modifiers.contains(KeyModifiers::ALT),
          };
          let key = match event.code {
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Enter     => Key::Enter,
            KeyCode::Left      => Key::Left,
            KeyCode::Right     => Key::Right,
            KeyCode::Up        => Key::Up,
            KeyCode::Down      => Key::Down,
            KeyCode::Home      => Key::Home,
            KeyCode::End       => Key::End,
            KeyCode::PageUp    => Key::PageUp,
            KeyCode::PageDown  => Key::PageDown,
            KeyCode::Tab       => Key::Tab,
            KeyCode::BackTab   => Key::BackTab,
            KeyCode::Delete    => Key::Delete,
            KeyCode::Insert    => Key::Insert,
            KeyCode::F(n)      => Key::F(n),
            KeyCode::Char(c)   => Key::Char(c),
            KeyCode::Null      => Key::Null,
            KeyCode::Esc       => Key::Esc,
          };
          return (modifiers, key);
        },
        Event::Resize(width, height) => {
          self.width = width as usize;
          self.height = height as usize;
          self.buffer.resize(self.width * self.height, None);
          for i in 0..self.buffer.len() {
            self.buffer[i] = Some(Cell(' ', Color::Black, Color::White));
          }
        },
        _ => { },
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
  }

  fn set_cursor(&mut self, row: usize, col: usize) {
    if (row, col) != self.current_cursor {
      queue!(self.output, cursor::MoveTo(col as u16, row as u16)).unwrap();
      self.current_cursor = (row, col);
    }
  }

  fn set_bg(&mut self, color: Color) {
    if color != self.current_bg {
      queue!(self.output, SetBackgroundColor(color)).unwrap();
      self.current_bg = color;
    }
  }

  fn set_fg(&mut self, color: Color) {
    if color != self.current_fg {
      queue!(self.output, SetForegroundColor(color)).unwrap();
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
          write!(self.output, "{}", ch).unwrap();
        }
      }
    }
    self.output.flush().unwrap();
  }

}

impl Drop for Screen {
  fn drop(&mut self) {
    terminal::disable_raw_mode().unwrap();
    execute!(self.output, terminal::LeaveAlternateScreen).unwrap();
  }
}
