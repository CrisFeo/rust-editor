use crate::*;

#[derive(Debug, Clone)]
pub struct Target {
  editor: MiniEditor,
}

impl Target {
  pub fn switch_to() -> UpdateCommand {
    let mode = Self { editor: Default::default() };
    UpdateCommand::SwitchMode(Box::new(mode))
  }
}

impl Mode for Target {
  fn update(
    &mut self,
    _buffer: &mut Buffer,
    registry: &mut Registry,
    _window: &mut Window,
    key: Key,
  ) -> UpdateCommand {
    match self.editor.update(key) {
      MiniEditorCommand::Cancel => return Normal::switch_to(),
      MiniEditorCommand::Submit => {
        let name = self.editor.value.to_string();
        let value = Register::Content(vec![name]);
        registry.set("target", value);
        return Normal::switch_to();
      },
      MiniEditorCommand::Update => {},
      MiniEditorCommand::None => { },
    }
    UpdateCommand::None
  }

  fn status(&self) -> CowStr<'_> {
    format!("target > {}", self.editor.value).into()
  }
}
