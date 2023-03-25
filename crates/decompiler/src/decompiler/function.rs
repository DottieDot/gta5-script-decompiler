use crate::{disassembler::InstructionInfo, formatters::AssemblyFormatter};

use super::function_graph::FunctionGraph;

pub struct Function<'input, 'bytes> {
  pub name:         String,
  pub parameters:   u32,
  pub return_count: u32,
  pub instructions: &'input [InstructionInfo<'bytes>]
}

impl<'input, 'bytes> Function<'input, 'bytes> {
  // Temporary
  pub fn dot_string(&self, formatter: AssemblyFormatter) -> String {
    let graph = FunctionGraph::generate(self);
    graph.to_dot_string(self, formatter)
  }
}
