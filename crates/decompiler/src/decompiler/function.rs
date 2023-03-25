use crate::disassembler::InstructionInfo;

pub struct Function<'input, 'bytes> {
  pub name:         String,
  pub parameters:   u32,
  pub return_count: u32,
  pub instructions: &'input [InstructionInfo<'bytes>]
}
