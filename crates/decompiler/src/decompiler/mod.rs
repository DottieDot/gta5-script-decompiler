use crate::{
  disassembler::{Instruction, InstructionInfo},
  formatters::AssemblyFormatter
};

use self::function::Function;

mod function;
mod function_graph;
mod stack_entry;

fn find_functions<'bytes, 'input: 'bytes>(
  instructions: &'input [InstructionInfo]
) -> Vec<Function<'input, 'bytes>> {
  let mut result = vec![];
  let mut it = instructions.iter().enumerate();

  while let Some((start, instr)) = it.next() {
    if let Instruction::Enter { arg_count, .. } = instr.instruction {
      'leave_loop: while let Some((end, instr)) = it.next() {
        if let Instruction::Leave { return_count, .. } = instr.instruction {
          result.push(Function {
            name:         result.len().to_string(),
            parameters:   arg_count as u32,
            return_count: return_count as u32,
            instructions: &instructions[start..=end]
          });
          break 'leave_loop;
        }
      }
    }
  }

  result
}

pub fn decompile(instructions: &[InstructionInfo]) {
  let functions = find_functions(instructions);
}

pub fn function_dot_string(
  instructions: &[InstructionInfo],
  function: usize,
  formatter: AssemblyFormatter
) -> String {
  let functions = find_functions(instructions);
  functions[function].dot_string(formatter)
}
