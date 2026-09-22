use crate::*;

const HELP_TEXT: &str = "
`q` switch to meta mode
`v` switch to view mode

`d` delete selection
`a` insert before active anchor (insert mode)
`x` store selection content to register (default target: 'clipboard')
`X` load selection content from register (default target: 'clipboard')
`z` undo
`Z` redo
`r` pipe each selection through a command (pipe mode)

`h` move active anchor left
`j` move active anchor down
`k` move active anchor up
`l` move active anchor right
`g` move current cursor to matching regex (seek mode)
`b` swap active and passive anchor
`n` collapse selection to active anchor

`u` select entire buffer
`t` make next selection primary
`T` make previous selection primary
`y` drop primary selection
`Y` drop all selections besides primary
`s` split selections (split mode)
`f` filter selection (filter mode)
";

mode!(Normal, "normal", HELP_TEXT, |key, ctx: ModeContext| {
  use crate::key::Key::*;
  match key {
    // Meta actions
    Char('q') => return vec![Meta::switch_to()],
    Char('v') => return vec![Viewport::switch_to()],

    // Content modifications
    Char('d') => {
      ctx.buffer.apply_operations(&[Op::RemoveAll]);
      ctx.buffer.history.commit();
    }
    Char('a') => return vec![mode::Insert::switch_to()],
    Char('x') => {
      let name = take_register_target(ctx.registry, "clipboard");
      ctx.registry.set(&name, Register::Content(copy(ctx.buffer)))
    }
    Char('X') => {
      let name = take_register_target(ctx.registry, "clipboard");
      if let Some(Register::Content(contents)) = ctx.registry.get(&name) {
        paste(ctx.buffer, contents);
      }
    }
    Char('z') => undo(ctx.buffer),
    Char('Z') => redo(ctx.buffer),
    Char('r') => return vec![Pipe::switch_to()],

    // Anchor movements
    Char('h') => ctx.buffer.apply_operations(&[Op::MoveByChar(-1)]),
    Char('j') => ctx.buffer.apply_operations(&[Op::MoveByLine(1)]),
    Char('k') => ctx.buffer.apply_operations(&[Op::MoveByLine(-1)]),
    Char('l') => ctx.buffer.apply_operations(&[Op::MoveByChar(1)]),
    Char('g') => return vec![SeekDirection::switch_to()],
    Char('b') => ctx.buffer.apply_operations(&[Op::Swap]),
    Char('n') => ctx.buffer.apply_operations(&[Op::Collapse]),

    // Selection manipulation
    Char('u') => ctx.buffer.set_selections(vec![Selection::new_at_end(
      0,
      ctx.buffer.contents.len_chars(),
    )]),
    Char('t') => {
      ctx.buffer.primary_selection =
        wrap_add(ctx.buffer.selections.len(), ctx.buffer.primary_selection, 1)
    }
    Char('T') => {
      ctx.buffer.primary_selection = wrap_add(
        ctx.buffer.selections.len(),
        ctx.buffer.primary_selection,
        -1,
      );
    }
    Char('y') => ctx
      .buffer
      .set_selections(vec![*ctx.buffer.primary_selection()]),
    Char('Y') => {
      let selections = ctx
        .buffer
        .selections
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != ctx.buffer.primary_selection)
        .map(|(_, &v)| v)
        .collect();
      ctx.buffer.set_selections(selections);
    }
    Char('s') => return vec![SplitType::switch_to()],
    Char('f') => return vec![FilterType::switch_to()],
    _ => {}
  }
  vec![]
});

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

pub fn copy(buffer: &mut Buffer) -> Vec<String> {
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
  contents
}

pub fn paste(buffer: &mut Buffer, contents: &[String]) {
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

pub fn undo(buffer: &mut Buffer) {
  let Some(changes) = buffer.history.backward() else {
    return;
  };
  let selections = changes.apply(&mut buffer.contents);
  buffer.set_selections(selections);
}

pub fn redo(buffer: &mut Buffer) {
  let Some(changes) = buffer.history.forward() else {
    return;
  };
  let selections = changes.apply(&mut buffer.contents);
  buffer.set_selections(selections);
}

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
