mod structs;

use structs::{DecompiledItem, Function};

use peg::{Parse, ParseElem, ParseSlice, RuleResult};

use crate::disassembler::{Instruction, InstructionInfo};

peg::parser! {
  grammar decompiler<'a>() for Instructions<'a> {
    rule function() -> DecompiledItem<'a, Function>
      = instr:$([InstructionInfo { instruction: Instruction::Enter { .. }, .. }])  {
        if let Instruction::Enter { name, .. } = &instr[0].instruction {
          DecompiledItem::<'a, _> {
            instructions: instr,
            item: Function {
              name: name.clone().unwrap_or_else(|| "tmp".to_owned())
            }
          }
        } else {
          panic!("this shouldn't happen")
        }
      }
  }
}

struct Instructions<'a> {
  instructions: &'a [InstructionInfo]
}

impl<'a> Parse for Instructions<'a> {
  type PositionRepr = usize;

  fn start<'input>(&'input self) -> usize {
    0
  }

  fn is_eof<'input>(&'input self, p: usize) -> bool {
    p == self.instructions.len()
  }

  fn position_repr<'input>(&'input self, p: usize) -> Self::PositionRepr {
    p
  }
}

impl<'a, 'b: 'a> ParseElem<'b> for Instructions<'a> {
  type Element = &'a InstructionInfo;

  fn parse_elem<'input>(&'input self, pos: usize) -> peg::RuleResult<Self::Element> {
    if pos < self.instructions.len() {
      RuleResult::Matched(pos + 1, &self.instructions[pos])
    } else {
      RuleResult::Failed
    }
  }
}

impl<'a: 'input, 'input> ParseSlice<'input> for Instructions<'a> {
  type Slice = &'a [InstructionInfo];

  fn parse_slice(&'input self, p1: usize, p2: usize) -> Self::Slice {
    &self.instructions[p1..p2]
  }
}
