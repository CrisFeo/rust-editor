use crate::*;

const HELP_TEXT: &str = "
`h` undo
`j` redo
";

mode!(
  HistoryControl,
  "history",
  HELP_TEXT,
  |key, ctx: ModeContext| {
    use crate::key::Key::*;
    match key {
      Char('h') => undo(ctx.buffer),
      Char('l') => redo(ctx.buffer),
      _ => return vec![Normal::switch_to()],
    };
    vec![]
  }
);

fn undo(buffer: &mut Buffer) {
  let Some(changes) = buffer.history.backward() else {
    return;
  };
  let selections = changes.apply(&mut buffer.contents);
  buffer.set_selections(selections);
}

fn redo(buffer: &mut Buffer) {
  let Some(changes) = buffer.history.forward() else {
    return;
  };
  let selections = changes.apply(&mut buffer.contents);
  buffer.set_selections(selections);
}
