use std::{cell::RefCell, rc::Rc};

use crate::decompiler::LinkedValueType;

use super::StatementInfo;

#[derive(Debug)]
pub struct DecompiledFunction<'input, 'bytes> {
  pub name:       String,
  pub params:     Vec<Rc<RefCell<LinkedValueType>>>,
  pub locals:     Vec<Rc<RefCell<LinkedValueType>>>,
  pub returns:    Option<Rc<RefCell<LinkedValueType>>>,
  pub statements: Vec<StatementInfo<'input, 'bytes>>
}
