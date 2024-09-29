use regex_cursor::engines::meta::Regex as MetaRegex;
use regex_cursor::regex_automata::util::syntax::Config as SyntaxConfig;
use regex_cursor::Input;
use ropey::Rope;

pub struct Regex(MetaRegex);

impl Regex {
  pub fn new(pattern: &str) -> Option<Self> {
    let regex = MetaRegex::builder()
      .syntax(
        SyntaxConfig::new()
          .multi_line(true)
          .dot_matches_new_line(true),
      )
      .build(pattern);
    match regex {
      Ok(regex) => Some(Self(regex)),
      Err(_) => None,
    }
  }

  pub fn find<'a>(
    &'a self,
    contents: &'a Rope,
    range_start: usize,
    range_end: usize,
  ) -> impl Iterator<Item = (usize, usize)> + 'a {
    let mut input = Input::new(contents);
    input.set_range(range_start..=range_end);
    self.0.find_iter(input).map(|m| {
      let start = contents.byte_to_char(m.start());
      let end = contents.byte_to_char(m.end());
      let end = end.saturating_sub(1).max(start);
      (start, end)
    })
  }
}
