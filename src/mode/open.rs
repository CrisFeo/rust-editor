use crate::*;

#[derive(Debug, Clone)]
pub struct Open {
  editor: MiniEditor,
}

impl Open {
  pub fn switch_to() -> UpdateCommand {
    let mode = Self { editor: Default::default() };
    UpdateCommand::SwitchMode(Box::new(mode))
  }
}

impl Mode for Open {
  fn update(
    &mut self,
    _buffer: &mut Buffer,
    _registry: &mut Registry,
    _window: &mut Window,
    key: Key,
  ) -> Vec<UpdateCommand> {
    match self.editor.update(key) {
      MiniEditorCommand::Cancel => return vec![Normal::switch_to()],
      MiniEditorCommand::Submit => return vec![
        Normal::switch_to(),
        UpdateCommand::Open(self.editor.value.to_string())
      ],
      MiniEditorCommand::Update => {},
      MiniEditorCommand::None => { },
    }
    vec![]
  }

  fn status(&self) -> CowStr<'_> {
    format!("open > {}", self.editor.value).into()
  }
}
