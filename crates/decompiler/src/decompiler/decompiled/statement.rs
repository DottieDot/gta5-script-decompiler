use crate::{decompiler::stack_entry::StackEntry, disassembler::InstructionInfo};

#[derive(Debug)]
pub enum Statement<'i, 'b> {
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
  },
  If {
    condition: StackEntry,
    then:      Vec<StatementInfo<'i, 'b>>
  },
  IfElse {
    condition: StackEntry,
    then:      Vec<StatementInfo<'i, 'b>>,
    els:       Vec<StatementInfo<'i, 'b>>
  },
  WhileLoop {
    condition: StackEntry,
    body:      Vec<StatementInfo<'i, 'b>>
  },
  Switch {
    condition: StackEntry
  },
  Break,
  Continue
}

#[derive(Debug)]
pub struct StatementInfo<'input, 'bytes> {
  pub instructions: &'input [InstructionInfo<'bytes>],
  pub statement:    Statement<'input, 'bytes>
}
