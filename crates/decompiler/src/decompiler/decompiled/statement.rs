use crate::{
  decompiler::{function_graph::CaseValue, StackEntryInfo},
  disassembler::InstructionInfo
};

#[derive(Debug)]
pub enum Statement<'i, 'b> {
  Nop,
  Assign {
    destination: StackEntryInfo,
    source:      StackEntryInfo
  },
  Return {
    values: Vec<StackEntryInfo>
  },
  Throw {
    value: StackEntryInfo
  },
  FunctionCall {
    args:             Vec<StackEntryInfo>,
    function_address: usize
  },
  NativeCall {
    args:        Vec<StackEntryInfo>,
    native_hash: u64
  },
  If {
    condition: StackEntryInfo,
    then:      Vec<StatementInfo<'i, 'b>>
  },
  IfElse {
    condition: StackEntryInfo,
    then:      Vec<StatementInfo<'i, 'b>>,
    els:       Vec<StatementInfo<'i, 'b>>
  },
  WhileLoop {
    condition: StackEntryInfo,
    body:      Vec<StatementInfo<'i, 'b>>
  },
  Switch {
    condition: StackEntryInfo,
    cases:     Vec<(Vec<StatementInfo<'i, 'b>>, Vec<CaseValue>)>
  },
  Break,
  Continue
}

#[derive(Debug)]
pub struct StatementInfo<'input, 'bytes> {
  pub instructions: &'input [InstructionInfo<'bytes>],
  pub statement:    Statement<'input, 'bytes>
}
