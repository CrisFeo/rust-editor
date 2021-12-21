mod normal;
mod insert;
mod filter;
mod reject;

pub use normal::*;
pub use insert::*;
pub use filter::*;
pub use reject::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Mode {
  Normal,
  Insert,
  Filter,
  Reject,
}

