use crate::{decompiler::stack_entry::StackEntry, disassembler::InstructionInfo};

#[derive(Debug)]
pub enum Statement {
  Nop,
  Assign {
    destination: StackEntry,
    source:      StackEntry
  },
  Return {
    values: Vec<StackEntry>
  },
  Throw {
    value: StackEntry
  },
  FunctionCall {
    args:             Vec<StackEntry>,
    function_address: usize
  },
  NativeCall {
    args:        Vec<StackEntry>,
    native_hash: u64
  }
}

#[derive(Debug)]
pub struct StatementInfo<'input, 'bytes> {
  pub instructions: &'input [InstructionInfo<'bytes>],
  pub statement:    Statement
}
