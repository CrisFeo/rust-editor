use crate::*;
use std::collections::HashMap;

pub enum Register {
  Content(Vec<String>),
  //Selections(Vec<Selection>, usize),
  Regex(Regex),
  //Input(Rope),
}

#[derive(Default)]
pub struct Registry(HashMap<String, Register>);

impl Registry {
  pub fn set(&mut self, key: &str, value: Register) {
    self.0.insert(key.to_string(), value);
  }

  pub fn get(&mut self, key: &str) -> Option<&Register> {
    self.0.get(key)
  }
}
