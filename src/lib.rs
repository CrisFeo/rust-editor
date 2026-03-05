mod buffer;
mod change;
mod color;
mod recorder;
mod history;
mod key;
mod mode;
mod regex;
mod registry;
mod screen;
mod selection;
mod view;
mod window;

pub use buffer::*;
pub use change::*;
pub use color::*;
pub use recorder::*;
pub use history::*;
pub use key::*;
pub use mode::*;
pub use regex::*;
pub use registry::*;
pub use screen::*;
pub use selection::*;
pub use view::*;
pub use window::*;

pub type CowStr<'a> = std::borrow::Cow<'a, str>;
