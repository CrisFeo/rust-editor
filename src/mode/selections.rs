use crate::*;

const HELP_TEXT: &str = "
`h` make previous selection primary
`j` drop primary selection
`k` drop all selections besides primary
`l` make next selection primary
";

mode!(
  SelectionControl,
  "selections",
  HELP_TEXT,
  |key, ctx: ModeContext| {
    use crate::key::Key::*;
    match key {
      Char('h') => cycle_primary_selection(ctx.buffer, -1),
      Char('j') => drop_primary_selection(ctx.buffer),
      Char('k') => solo_primary_selection(ctx.buffer),
      Char('l') => cycle_primary_selection(ctx.buffer, 1),
      _ => return vec![Normal::switch_to()],
    };
    vec![]
  }
);

fn drop_primary_selection(buffer: &mut Buffer) {
  let selections = buffer
    .selections
    .iter()
    .enumerate()
    .filter(|&(i, _)| i != buffer.primary_selection)
    .map(|(_, &v)| v)
    .collect();
  buffer.set_selections(selections);
}

fn solo_primary_selection(buffer: &mut Buffer) {
  let selections = vec![*buffer.primary_selection()];
  buffer.set_selections(selections);
}

fn cycle_primary_selection(buffer: &mut Buffer, delta: isize) {
  buffer.primary_selection = wrap_add(buffer.selections.len(), buffer.primary_selection, delta)
}

fn wrap_add(domain: usize, value: usize, delta: isize) -> usize {
  let value = (value as isize) + delta;
  let value = if value < 0 {
    let value = -value as usize;
    (value / domain + 1) * domain - value
  } else {
    value as usize
  };
  value % domain
}
