use crate::*;

const HELP_TEXT: &str = "
`Q` exit editor
`w` write buffer to file
`o` open file by name (open mode)
`c` close current buffer
`h` switch to previous buffer view
`l` switch to next buffer view
";

mode!(Meta, "meta", HELP_TEXT, |key, ctx| {
  use crate::Key::*;
  match key {
    Char('Q') => vec![UpdateCommand::Quit],
    Char('w') => save(ctx),
    Char('o') => vec![Open::switch_to()],
    Char('c') => vec![Normal::switch_to(), UpdateCommand::Close],
    Char('h') => vec![Normal::switch_to(), UpdateCommand::ViewPrev],
    Char('l') => vec![Normal::switch_to(), UpdateCommand::ViewNext],
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
