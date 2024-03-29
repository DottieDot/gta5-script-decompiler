use crate::disassembler::{Instruction, InstructionInfo};

mod cfg_reducer;
mod control_flow;
pub mod decompiled;
mod decompiler_data;
mod function;
mod function_graph;
mod script_globals;
mod script_statics;
mod stack;
mod stack_entry;
mod value_type;

pub use control_flow::*;
pub use decompiler_data::*;
pub use function::*;
pub use script_globals::*;
pub use script_statics::*;
pub use stack_entry::*;
pub use value_type::*;

fn find_functions<'bytes, 'input: 'bytes>(
  instructions: &'input [InstructionInfo]
) -> Vec<Function<'input, 'bytes>> {
  let mut result = vec![];
  let mut it = instructions.iter().enumerate().peekable();

  while let Some((start, instr)) = it.next() {
    if let Instruction::Enter {
      arg_count,
      frame_size,
      ..
    } = instr.instruction
    {
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
        result.push(Function::new(FunctionInfo {
          name:         format!("func_{}", result.len()),
          location:     instructions[start].pos,
          parameters:   arg_count as u32,
          returns:      return_count as u32,
          locals:       frame_size as u32 - arg_count as u32 - 2,
          instructions: &instructions[start..=end]
        }))
      }
    }
  }

  result
}

pub fn get_functions<'i: 'b, 'b>(instructions: &'i [InstructionInfo<'b>]) -> Vec<Function<'i, 'b>> {
  find_functions(instructions)
}
