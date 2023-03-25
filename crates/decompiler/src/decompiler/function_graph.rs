use std::collections::{HashMap, HashSet, LinkedList};

use crate::{
  disassembler::{Instruction, InstructionInfo, SwitchCase},
  formatters::AssemblyFormatter
};

use super::function::Function;

#[derive(Clone, Copy)]
enum EdgeDest {
  Success(usize),
  Fallback(usize),
  Flow(usize)
}

impl EdgeDest {
  pub fn dest(self) -> usize {
    match self {
      EdgeDest::Success(dest) | EdgeDest::Fallback(dest) | EdgeDest::Flow(dest) => dest
    }
  }
}

#[derive(Default)]
pub struct FunctionGraph {
  nodes:           Vec<(usize, usize)>,
  parent_to_child: HashMap<usize, Vec<EdgeDest>>,
  child_to_parent: HashMap<usize, Vec<usize>>
}

impl FunctionGraph {
  pub fn generate(function: &Function) -> Self {
    let mut graph = FunctionGraph::default();
    let destinations = get_destinations(function.instructions);

    let mut current_index: Option<usize> = None;
    let mut current_node: Option<usize> = None;

    for (index, instruction) in function.instructions.iter().enumerate() {
      let cindex = *current_index.get_or_insert(index);
      let cnode = *current_node.get_or_insert(instruction.pos);
      let next = instruction.pos + instruction.bytes.len();

      match &instruction.instruction {
        Instruction::Jump { location } => {
          graph.add_join(cnode, *location as usize);
          graph.nodes.push((cindex, index));
          current_index = None;
          current_node = None;
        }
        Instruction::JumpZero { location }
        | Instruction::IfEqualJumpZero { location }
        | Instruction::IfNotEqualJumpZero { location }
        | Instruction::IfLowerThanJumpZero { location }
        | Instruction::IfGreaterThanJumpZero { location }
        | Instruction::IfLowerOrEqualJumpZero { location }
        | Instruction::IfGreaterOrEqualJumpZero { location } => {
          graph.add_conditional_jmp(cnode, *location as usize, next);
          graph.nodes.push((cindex, index));
          current_index = None;
          current_node = None;
        }
        Instruction::Switch { cases } => {
          graph.add_switch(cnode, cases, next);
          graph.nodes.push((cindex, index));
          current_index = None;
          current_node = None;
        }
        _ => {
          if destinations.contains(&next) {
            graph.add_jmp(cnode, next);
            graph.nodes.push((cindex, index));
            current_index = None;
            current_node = None;
          }
        }
      }
    }

    if let Some(index) = current_index {
      graph.nodes.push((index, function.instructions.len() - 1));
    }

    graph
  }

  pub fn to_dot_string(&self, function: &Function, formatter: AssemblyFormatter) -> String {
    let mut first = true;
    let mut diagram: LinkedList<String> = Default::default();
    diagram.push_back(r#"digraph{node[fontname="Consolas",fontcolor=black]"#.to_owned());
    for (index, end) in &self.nodes {
      let pos = function.instructions[*index].pos;
      let assembly = formatter.format(&function.instructions[*index..=*end], false);
      diagram.push_back(format!(
        "node_{pos}[label=\"{assembly}\\l\",shape=rectangle,color={color}]",
        assembly = assembly
          .trim_start_matches('\n')
          .replace("\n", "\\l")
          .replace("\t", "    "),
        color = {
          if first {
            first = false;
            "darkgreen"
          } else if self
            .parent_to_child
            .get(&pos)
            .map(|v| v.is_empty())
            .unwrap_or(true)
          {
            "red4"
          } else {
            "black"
          }
        }
      ));
    }

    for (origin, destinations) in &self.parent_to_child {
      for destination in destinations {
        diagram.push_back(format!(
          "node_{origin}->node_{dest}[color={color}]",
          dest = destination.dest(),
          color = {
            match destination {
              EdgeDest::Success(_) => "darkgreen",
              EdgeDest::Fallback(_) => "red4",
              EdgeDest::Flow(_) => "black"
            }
          }
        ));
      }
    }

    diagram.push_back("}".to_owned());

    diagram.into_iter().collect::<Vec<_>>().join("")
  }

  fn add_join(&mut self, origin: usize, destination: usize) {
    self.add_edge(origin, EdgeDest::Flow(destination));
  }

  fn add_jmp(&mut self, origin: usize, destination: usize) {
    self.add_edge(origin, EdgeDest::Flow(destination));
  }

  fn add_conditional_jmp(&mut self, origin: usize, destination: usize, next: usize) {
    self.add_edge(origin, EdgeDest::Fallback(next));
    self.add_edge(origin, EdgeDest::Success(destination));
  }

  fn add_switch(&mut self, origin: usize, case_destinations: &[SwitchCase], next: usize) {
    self.add_edge(origin, EdgeDest::Fallback(next));

    for destination in case_destinations {
      self.add_edge(origin, EdgeDest::Success(destination.location as usize))
    }
  }

  fn add_edge(&mut self, origin: usize, destination: EdgeDest) {
    self
      .parent_to_child
      .entry(origin)
      .or_default()
      .push(destination);
    self
      .child_to_parent
      .entry(destination.dest())
      .or_default()
      .push(origin);
  }
}

fn get_destinations(instructions: &[InstructionInfo]) -> HashSet<usize> {
  let mut result = HashSet::new();

  for instruction in instructions {
    match &instruction.instruction {
      Instruction::Jump { location }
      | Instruction::JumpZero { location }
      | Instruction::IfEqualJumpZero { location }
      | Instruction::IfNotEqualJumpZero { location }
      | Instruction::IfLowerThanJumpZero { location }
      | Instruction::IfGreaterThanJumpZero { location }
      | Instruction::IfLowerOrEqualJumpZero { location }
      | Instruction::IfGreaterOrEqualJumpZero { location }
      | Instruction::FunctionCall { location } => {
        result.insert(*location as usize);
      }
      Instruction::Switch { cases } => {
        for SwitchCase { location, .. } in cases {
          result.insert(*location as usize);
        }
      }
      _ => {}
    }
  }

  result
}
