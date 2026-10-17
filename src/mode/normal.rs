use crate::*;

const PLAYBACK_REGISTER: &str = " ";

const HELP_TEXT: &str = "
`q` switch to meta mode
`v` switch to view mode
`x` switch to clipboard mode
`z` switch to history mode

`t` set target register for next command
` ` play keys from register (default target: ' ')

`d` delete selection
`a` insert before active anchor (insert mode)
`r` pipe each selection through a command (pipe mode)

`h` move active anchor left
`j` move active anchor down
`k` move active anchor up
`l` move active anchor right
`Left`  move active anchor left
`Down`  move active anchor down
`Up`    move active anchor up
`Right` move active anchor right
`g` move current cursor to matching regex (seek mode)
`b` swap active and passive anchor
`n` collapse selection to active anchor

`u` select entire buffer
`y` modify selections (selection mode)
`s` split selections (split mode)
`f` filter selection (filter mode)
";

mode!(Normal, "normal", HELP_TEXT, |key, ctx: ModeContext| {
  use crate::key::Key::*;
  match key {
    // Meta actions
    Char('q') => return vec![Meta::switch_to()],
    Char('v') => return vec![Viewport::switch_to()],
    Char('x') => return vec![Clipboard::switch_to()],
    Char('z') => return vec![HistoryControl::switch_to()],

    // Macros
    Char('t') => return vec![Target::switch_to()],
    Char(' ') => return playback(ctx),

    // Content modifications
    Char('d') => delete_selection(ctx.buffer),
    Char('a') => return vec![Insert::switch_to()],
    Char('r') => return vec![Pipe::switch_to()],

    // Anchor movements
    Char('h') => ctx.buffer.apply_operations(&[Op::MoveByChar(-1)]),
    Char('j') => ctx.buffer.apply_operations(&[Op::MoveByLine(1)]),
    Char('k') => ctx.buffer.apply_operations(&[Op::MoveByLine(-1)]),
    Char('l') => ctx.buffer.apply_operations(&[Op::MoveByChar(1)]),
    Left      => ctx.buffer.apply_operations(&[Op::MoveByChar(-1)]),
    Down      => ctx.buffer.apply_operations(&[Op::MoveByLine(1)]),
    Up        => ctx.buffer.apply_operations(&[Op::MoveByLine(-1)]),
    Right     => ctx.buffer.apply_operations(&[Op::MoveByChar(1)]),
    Char('g') => return vec![SeekDirection::switch_to()],
    Char('b') => ctx.buffer.apply_operations(&[Op::Swap]),
    Char('n') => ctx.buffer.apply_operations(&[Op::Collapse]),

    // Selection manipulation
    Char('u') => select_entire_buffer(ctx.buffer),
    Char('y') => return vec![SelectionControl::switch_to()],
    Char('s') => return vec![SplitType::switch_to()],
    Char('f') => return vec![FilterType::switch_to()],
    _ => {}
  }
  vec![]
});

pub fn take_register_target(registry: &mut Registry, default: &str) -> String {
  let Some(target) = registry.get("target") else {
    return default.to_string();
  };
  let Register::Content(target) = target;
  let Some(target) = target.first() else {
    return default.to_string();
  };
  let target = target.to_string();
  registry.del("target");
  target
}

fn playback(ctx: ModeContext) -> Vec<UpdateCommand> {
  let name = take_register_target(ctx.registry, PLAYBACK_REGISTER);
  let Some(register) = ctx.registry.get(&name) else {
    return vec![];
  };
  let Register::Content(contents) = register;
  let Some(contents) = contents.first() else {
    return vec![];
  };
  let keys = Key::from_input(contents);
  vec![UpdateCommand::SendKeys(keys)]
}

fn delete_selection(buffer: &mut Buffer) {
  buffer.apply_operations(&[Op::RemoveAll]);
  buffer.history.commit();
}

fn select_entire_buffer(buffer: &mut Buffer) {
  let selections = vec![Selection::new_at_end(0, buffer.contents.len_chars())];
  buffer.set_selections(selections);
}
