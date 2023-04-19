use super::{Statement, StatementInfo};

#[derive(Debug)]
pub struct DecompiledFunction<'input, 'bytes> {
  pub name:       String,
  pub params:     usize,
  pub statements: Vec<StatementInfo<'input, 'bytes>>
}
