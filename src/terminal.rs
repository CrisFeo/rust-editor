use std::io::{self, BufWriter, Error, StdinLock, StdoutLock, Read, Write};

#[derive(Debug,PartialEq,Eq)]
pub enum Event {
  Ignored,
  Char(char),
  Tab,
  Esc,
  Enter,
  Backspace,
  Up,
  Down,
  Left,
  Right,
}

struct StdinIterator<'a> {
  stdin: StdinLock<'a>,
  extra_char: Option<u8>,
}

impl<'a> StdinIterator<'a> {
  fn new(stdin: StdinLock<'a>) -> Self {
    Self {
      stdin,
      extra_char: None,
    }
  }
}

impl<'a> Iterator for StdinIterator<'a> {
  type Item = Event;

  fn next(&mut self) -> Option<Event> {
    if let Some(c) = self.extra_char {
      self.extra_char = None;
      let stdin_iter = (&mut self.stdin)
        .bytes()
        .map(|r| r.expect("should be able to read next from stdin"));
      let event = parse(c, stdin_iter);
      return Some(event);
    }
    let mut b = [0; 2];
    let n = self
      .stdin
      .read(&mut b[..])
      .expect("should be able to read 2 bytes from stdin");
    let stdin_iter = (&mut self.stdin)
      .bytes()
      .map(|r| r.expect("should be able to read next from stdin"));
    match n {
      0 => None,
      1 => match b[0] {
        b'\x1B' => Some(Event::Esc),
        c => Some(parse(c, stdin_iter)),
      },
      2 => {
        let mut extra_iter = Some(b[1]).into_iter();
        let input = (&mut extra_iter).chain(stdin_iter);
        let event = parse(b[0], input);
        self.extra_char = extra_iter.next();
        Some(event)
      }
      _ => panic!("should not be able to read more than 2 bytes"),
    }
  }
}

fn parse(start: u8, mut rest: impl Iterator<Item = u8>) -> Event {
  match start {
    b'\n' => Event::Enter,
    b'\r' => Event::Enter,
    b'\t' => Event::Tab,
    b'\x7F' => Event::Backspace,
    b'\x1B' => {
      //let mut rest = rest.peekable();
      match rest.next() {
        Some(b'[') => match rest.next() {
          Some(b'A') => Event::Up,
          Some(b'B') => Event::Down,
          Some(b'D') => Event::Left,
          Some(b'C') => Event::Right,
          _ => Event::Ignored,
        },
        _ => Event::Ignored,
      }
    },
    b => {
      if b.is_ascii() {
        Event::Char(b as char)
      } else {
        let mut bytes = vec!(b);
        for _ in 0..4 {
          match rest.next() {
            Some(b) => {
              bytes.push(b);
              if let Ok(s) = str::from_utf8(&bytes) {
                let c = s
                  .chars()
                  .next()
                  .expect("should parse utf8 to a single character");
                return Event::Char(c);
              }
            },
            None => panic!("eof while parsing character"),
          }
        }
        panic!("failed to parse ut8 char from stdin");
      }
    }
  }
}

pub struct Terminal<'a> {
  old_termios: libc::termios,
  stdin: StdinIterator<'a>,
  stdout: BufWriter<StdoutLock<'a>>,
}

impl<'a> Drop for Terminal<'a> {
  fn drop(&mut self) {
    self.write("\u{001b}[2J"); // clear
    self.write("\u{001b}[?25h"); // show cursor
    self.write("\u{001b}[?1000l"); // disable mouse
    self.write("\u{001b}[?1049l"); // from alt screen
    self.flush();
    exit_raw_mode(&self.old_termios);
  }
}

impl<'a> Terminal<'a> {
  pub fn new() -> Self {
    let old_termios = enter_raw_mode();
    let stdin = io::stdin().lock();
    let stdin = StdinIterator::new(stdin);
    let stdout = io::stdout().lock();
    let stdout = BufWriter::new(stdout);
    let mut t = Self {
      old_termios,
      stdin,
      stdout,
    };
    t.write("\u{001b}[?1049h"); // to alt screen
    //t.write("\u{001b}[?1000h"); // enable mouse
    t.write("\u{001b}[?25l"); // hide cursor
    t.write("\u{001b}[2J"); // clear
    t.flush();
    t
  }

  pub fn goto(&mut self, x: usize, y: usize) {
    let r = (y + 1).to_string();
    let c = (x + 1).to_string();
    write!(
      self.stdout,
      "\u{001b}[{r};{c}H",
    ).expect("should be able to move cursor via stdout");
  }

  pub fn write(&mut self, value: impl AsRef<str>) {
    write!(
      self.stdout,
      "{}",
      value.as_ref(),
    ).expect("should be able to write character to stdout");
  }

  pub fn flush(&mut self) {
    self.stdout
      .flush()
      .expect("should be able to flush stdout");
  }

  pub fn poll(&mut self) -> Event {
    self.stdin.next().expect("should be able to poll event")
  }
}

fn enter_raw_mode() -> libc::termios {
  unsafe {
    let mut old_termios = std::mem::zeroed();
    let result = libc::tcgetattr(
      libc::STDOUT_FILENO,
      &mut old_termios,
    );
    if result == -1 {
      panic!("failed to get termios: {}", Error::last_os_error());
    }
    let mut raw_termios = old_termios;
    libc::cfmakeraw(&mut raw_termios);
    let result = libc::tcsetattr(
      libc::STDOUT_FILENO,
      libc::TCSANOW,
      &raw_termios,
    );
    if result == -1 {
      panic!("failed to set raw termios: {}", Error::last_os_error());
    }
    old_termios
  }
}

fn exit_raw_mode(old_termios: &libc::termios) {
  unsafe {
    let result = libc::tcsetattr(
      libc::STDOUT_FILENO,
      libc::TCSANOW,
      old_termios,
    );
    if result == -1 {
      panic!("failed to set old termios: {}", Error::last_os_error());
    }
  }
}
