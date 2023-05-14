use std::{cell::RefCell, rc::Rc};

use super::LinkedValueType;

pub struct ScriptStatics {
  statics: Vec<Rc<RefCell<LinkedValueType>>>
}

impl ScriptStatics {
  pub fn new(static_count: usize) -> Self {
    Self {
      statics: (0..static_count)
        .map(|_| LinkedValueType::new_primitive(super::Primitives::Unknown).make_shared())
        .collect()
    }
  }

  pub fn get_static(&self, static_index: usize) -> Option<&Rc<RefCell<LinkedValueType>>> {
    self.statics.get(static_index)
  }
}
