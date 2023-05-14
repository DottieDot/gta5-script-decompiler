use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::LinkedValueType;

#[derive(Default)]
pub struct ScriptGlobals {
  globals: HashMap<usize, Rc<RefCell<LinkedValueType>>>
}

impl ScriptGlobals {
  pub fn get_global(&mut self, global: usize) -> &Rc<RefCell<LinkedValueType>> {
    self
      .globals
      .entry(global)
      .or_insert_with(|| LinkedValueType::new_primitive(super::Primitives::Unknown).make_shared())
  }
}
