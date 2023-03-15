use crate::disassembler::InstructionInfo;

pub struct Function<'input> {
  name:         String,
  instructions: &'input [InstructionInfo]
}
