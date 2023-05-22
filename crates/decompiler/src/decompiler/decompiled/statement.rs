use crate::{
  decompiler::{control_flow::CaseValue, StackEntryInfo},
  disassembler::InstructionInfo
};

#[derive(Debug)]
pub enum Statement<'i, 'b> {
  Nop,
  Assign {
    destination: StackEntryInfo<'i>,
    source:      StackEntryInfo<'i>
  },
  Return {
    values: Vec<StackEntryInfo<'i>>
  },
  Throw {
    value: StackEntryInfo<'i>
  },
  FunctionCall {
    args:             Vec<StackEntryInfo<'i>>,
    function_address: usize
  },
  NativeCall {
    args:        Vec<StackEntryInfo<'i>>,
    native_hash: u64
  },
  If {
    condition: StackEntryInfo<'i>,
    then:      Vec<StatementInfo<'i, 'b>>
  },
  IfElse {
    condition: StackEntryInfo<'i>,
    then:      Vec<StatementInfo<'i, 'b>>,
    els:       Vec<StatementInfo<'i, 'b>>
  },
  WhileLoop {
    condition: StackEntryInfo<'i>,
    body:      Vec<StatementInfo<'i, 'b>>
  },
  Switch {
    condition: StackEntryInfo<'i>,
    cases:     Vec<(Vec<StatementInfo<'i, 'b>>, Vec<CaseValue>)>
  },
  StringCopy {
    destination: StackEntryInfo<'i>,
    string:      StackEntryInfo<'i>,
    max_length:  usize
  },
  IntToString {
    destination: StackEntryInfo<'i>,
    int:         StackEntryInfo<'i>,
    max_length:  usize
  },
  StringConcat {
    destination: StackEntryInfo<'i>,
    string:      StackEntryInfo<'i>,
    max_length:  usize
  },
  StringIntConcat {
    destination: StackEntryInfo<'i>,
    int:         StackEntryInfo<'i>,
    max_length:  usize
  },
  MemCopy {
    destination: StackEntryInfo<'i>,
    source:      Vec<StackEntryInfo<'i>>,
    buffer_size: StackEntryInfo<'i>,
    count:       usize
  },
  Break,
  Continue
}

#[derive(Debug)]
pub struct StatementInfo<'input, 'bytes> {
  pub instructions: &'input [InstructionInfo<'bytes>],
  pub statement:    Statement<'input, 'bytes>
}
