use crate::*;

const HELP_TEXT: &str = "
`v` center view on primary selection
`h` move view left one column
`j` move view down one row
`k` move view up one row
`l` move view right one column
`J` move active anchor down one page
`K` move active anchor up one page
";

mode!(Viewport, "view", HELP_TEXT, |key, ctx: ModeContext| {
  use crate::Key::*;
  match key {
    Char('v') => center(ctx.buffer, ctx.window),
    Char('h') => {
      ctx.window.keep_cursor_visible = false;
      ctx.window.scroll_left = ctx.window.scroll_left.saturating_sub(1);
    }
    Char('j') => {
      ctx.window.keep_cursor_visible = false;
      ctx.window.scroll_top = ctx.window.scroll_top.saturating_add(1);
    }
    Char('k') => {
      ctx.window.keep_cursor_visible = false;
      ctx.window.scroll_top = ctx.window.scroll_top.saturating_sub(1);
    }
    Char('l') => {
      ctx.window.keep_cursor_visible = false;
      ctx.window.scroll_left = ctx.window.scroll_left.saturating_add(1);
    }
    Char('J') => {
      move_by_window_page(ctx.buffer, ctx.window, 1);
    }
    Char('K') => {
      move_by_window_page(ctx.buffer, ctx.window, -1);
    }
    _ => return vec![Normal::switch_to()],
  }
  vec![]
});

fn center(buffer: &Buffer, window: &mut Window) {
  window.scroll_top = buffer
    .primary_selection()
    .cursor_line(&buffer.contents)
    .saturating_sub(window.height / 2);
}

fn move_by_window_page(buffer: &mut Buffer, window: &mut Window, delta: isize) {
  buffer.apply_operations(
    &[
      vec![Op::MoveByLine(delta); window.height / 2],
      vec![Op::Collapse],
    ]
    .concat(),
  );
  center(buffer, window);
}
