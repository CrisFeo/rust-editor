use crate::*;

#[derive(Debug, Clone)]
pub struct Target {
  editor: MiniEditor,
}

impl Target {
  pub fn switch_to() -> UpdateCommand {
    let mode = Self {
      editor: Default::default(),
    };
    UpdateCommand::SwitchMode(Box::new(mode))
  }
}

impl Mode for Target {
  fn update(&mut self, ctx: ModeContext, key: Key) -> Vec<UpdateCommand> {
    match self.editor.update(key) {
      MiniEditorCommand::Cancel => return vec![Normal::switch_to()],
      MiniEditorCommand::Submit => {
        let name = self.editor.value.to_string();
        let value = Register::Content(vec![name]);
        ctx.registry.set("target", value);
        return vec![Normal::switch_to()];
      }
      MiniEditorCommand::Update => {}
      MiniEditorCommand::None => {}
    }
    vec![]
  }

  fn status(&self) -> CowStr<'_> {
    format!("target > {}", self.editor.value).into()
  }
}
