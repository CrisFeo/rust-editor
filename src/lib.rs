mod buffer;
mod history;
mod key;
mod mode;
mod regex;
mod screen;
mod selection;
mod view;
mod window;

pub use buffer::*;
pub use history::*;
pub use key::*;
pub use mode::*;
pub use regex::*;
pub use screen::*;
pub use selection::*;
pub use view::*;
pub use window::*;


pub type CowStr<'a> = std::borrow::Cow<'a, str>;
