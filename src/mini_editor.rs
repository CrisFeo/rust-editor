use crate::*;
use ropey::Rope;

pub enum MiniEditorCommand {
  None,
  Cancel,
  Update,
  Submit,
}

#[derive(Default, Debug, Clone)]
pub struct MiniEditor {
  pub value: Rope,
}

impl MiniEditor {
  pub fn update(&mut self, key: Key) -> MiniEditorCommand {
    match key {
      Key::Esc => MiniEditorCommand::Cancel,
      Key::Backspace => {
        let len = self.value.len_chars();
        if len > 0 {
          self.value.remove(len - 1..len);
          MiniEditorCommand::Update
        } else {
          MiniEditorCommand::None
        }
      },
      Key::Char(ch) => {
        let len = self.value.len_chars();
        self.value.insert_char(len, ch);
        MiniEditorCommand::Update
      },
      Key::Enter => MiniEditorCommand::Submit,
      _ => MiniEditorCommand::None,
    }
  }
}
