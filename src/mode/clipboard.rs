use crate::*;

const CLIPBOARD_REGISTER: &str = "x";

const HELP_TEXT: &str = "
`k` store selection content to register (default target: 'x')
`j` load selection content from register (default target: 'x')
";

mode!(
  Clipboard,
  "clipboard",
  HELP_TEXT,
  |key, ctx: ModeContext| {
    use crate::key::Key::*;
    match key {
      Char('k') => copy(ctx.registry, ctx.buffer),
      Char('j') => paste(ctx.registry, ctx.buffer),
      _ => {}
    };
    vec![Normal::switch_to()]
  }
);

fn copy(registry: &mut Registry, buffer: &Buffer) {
  let mut contents = Vec::with_capacity(buffer.selections.len());
  for i in 0..buffer.selections.len() {
    let i = (buffer.primary_selection + i) % buffer.selections.len();
    let selection = buffer
      .selections
      .get(i)
      .expect("should be able to retrieve selection at index less than length when copying");
    let content = selection.slice(&buffer.contents);
    contents.push(content.into());
  }
  let register_name = take_register_target(registry, CLIPBOARD_REGISTER);
  registry.set(&register_name, Register::Content(contents));
}

fn paste(registry: &mut Registry, buffer: &mut Buffer) {
  let name = take_register_target(registry, CLIPBOARD_REGISTER);
  let Some(Register::Content(contents)) = registry.get(&name) else {
    return;
  };
  for selection_i in 0..buffer.selections.len() {
    let content_i = selection_i % contents.len();
    let selection_i = (buffer.primary_selection + selection_i) % buffer.selections.len();
    let selection = buffer
      .selections
      .get_mut(selection_i)
      .expect("should be able to retrieve selection at index less than length when pasting");
    let content = contents
      .get(content_i)
      .expect("should be able to retrieve content at index less than length when pasting");
    let change = selection.apply_operation(&mut buffer.contents, Op::InsertStr(content));
    for j in selection_i + 1..buffer.selections.len() {
      let next_selection = buffer
        .selections
        .get_mut(j)
        .expect("should be able to retrieve selection at index less than length when adjusting selections after applying operation");
      next_selection.adjust(&buffer.contents, change.as_ref());
    }
    change.map(|c| buffer.history.record(c));
  }
  buffer.history.commit();
  buffer.cleanup_overlaps();
}
