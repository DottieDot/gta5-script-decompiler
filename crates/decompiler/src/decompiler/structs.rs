use crate::disassembler::{Instruction, InstructionInfo};

pub struct DecompiledItem<'a, T> {
  pub instructions: &'a [InstructionInfo],
  pub item:         T
}

pub struct Function {
  pub name: String
}

pub enum Statement {}

pub enum Expression {}
