use crate::*;

const HELP_TEXT: &str = "
`Q` exit editor
`w` write buffer to file
`o` open file by name (open mode)
`c` close current buffer
`[` switch to previous buffer view
`]` switch to next buffer view
` ` play keys from register (default target: 'playback')
`e` set target register for next command
";

mode!(Meta, "meta", HELP_TEXT, |key, ctx| {
  use crate::Key::*;
  match key {
    Char('Q') => vec![UpdateCommand::Quit],
    Char('w') => save(ctx),
    Char('o') => vec![Open::switch_to()],
    Char('c') => vec![Normal::switch_to(), UpdateCommand::Close],
    Char('[') => vec![Normal::switch_to(), UpdateCommand::ViewPrev],
    Char(']') => vec![Normal::switch_to(), UpdateCommand::ViewNext],
    Char(' ') => playback(ctx),
    Char('e') => vec![Target::switch_to()],
    _ => vec![Normal::switch_to()],
  }
});

fn save(ctx: ModeContext) -> Vec<UpdateCommand> {
  if ctx.buffer.filename.is_none() {
    ctx.toast.status = Some("scratch buffers cannot be saved".into());
  } else {
    if ctx.buffer.save() {
      ctx.toast.status = Some("file saved!".into());
    } else {
      ctx.toast.status = Some("error: could not save file".into());
    }
  }
  vec![Normal::switch_to()]
}

fn playback(ctx: ModeContext) -> Vec<UpdateCommand> {
  let name = take_register_target(ctx.registry, "playback");
  let Some(register) = ctx.registry.get(&name) else {
    return vec![];
  };
  let Register::Content(contents) = register;
  let Some(contents) = contents.first() else {
    return vec![];
  };
  let keys = Key::from_input(contents);
  vec![Normal::switch_to(), UpdateCommand::SendKeys(keys)]
}
