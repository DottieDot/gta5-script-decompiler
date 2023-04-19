use super::Instruction;

#[derive(Debug)]
pub struct InstructionInfo<'input> {
  pub instruction: Instruction,
  pub pos:         usize,
  pub bytes:       &'input [u8]
}
