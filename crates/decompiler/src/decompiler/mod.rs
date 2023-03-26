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
  let mut it = instructions.iter().enumerate().peekable();

  while let Some((start, instr)) = it.next() {
    if let Instruction::Enter { arg_count, .. } = instr.instruction {
      let mut last_leave: Option<(usize, u8)> = None;
      loop {
        let next = it.peek();

        match next {
          Some((
            end,
            InstructionInfo {
              instruction: Instruction::Leave { return_count, .. },
              ..
            }
          )) => {
            last_leave = Some((*end, *return_count));
            it.next();
          }
          Some((
            _,
            InstructionInfo {
              instruction: Instruction::Enter { .. },
              ..
            }
          ))
          | None => break,
          _ => {
            it.next();
          }
        }
      }

      if let Some((end, return_count)) = last_leave {
        result.push(Function {
          name:         format!("func_{}", result.len()),
          parameters:   arg_count as u32,
          return_count: return_count as u32,
          instructions: &instructions[start..=end]
        })
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
