use std::collections::{HashMap, HashSet, LinkedList};

use petgraph::{
  graph::NodeIndex,
  prelude::DiGraph,
  visit::{
    EdgeCount, EdgeIndexable, EdgeRef, IntoEdgesDirected, IntoNodeReferences, NodeIndexable
  },
  Direction
};

use crate::{
  disassembler::{Instruction, InstructionInfo, SwitchCase},
  formatters::AssemblyFormatter
};

use super::function::FunctionInfo;

#[derive(Clone, Copy, Debug)]
pub enum EdgeType {
  Jump,
  ConditionalJump,
  ConditionalFlow,
  Case(i64),
  Flow
}

#[derive(Debug, Clone)]
pub struct TmpFunctionGraphNode<'input, 'bytes> {
  instructions: &'input [InstructionInfo<'bytes>]
}

#[derive(Debug, Clone)]
pub struct TmpFunctionGraph<'input, 'bytes> {
  graph: DiGraph<TmpFunctionGraphNode<'input, 'bytes>, EdgeType>
}

impl<'input, 'bytes> TmpFunctionGraph<'input, 'bytes> {
  pub fn generate(function: &FunctionInfo<'input, 'bytes>) -> Self {
    let mut graph: DiGraph<TmpFunctionGraphNode<'input, 'bytes>, EdgeType> = Default::default();
    let destinations = get_destinations(function.instructions);
    let mut node_indices: HashMap<usize, NodeIndex> = Default::default();

    let mut current_index: Option<usize> = None;
    let mut current_node: Option<usize> = None;

    for (index, instruction) in function.instructions.iter().enumerate() {
      let cindex = *current_index.get_or_insert(index);
      let cnode = *current_node.get_or_insert(instruction.pos);
      let next: usize = instruction.pos + instruction.bytes.len();

      match &instruction.instruction {
        Instruction::Leave { .. }
        | Instruction::Jump { .. }
        | Instruction::JumpZero { .. }
        | Instruction::IfEqualJumpZero { .. }
        | Instruction::IfNotEqualJumpZero { .. }
        | Instruction::IfLowerThanJumpZero { .. }
        | Instruction::IfGreaterThanJumpZero { .. }
        | Instruction::IfLowerOrEqualJumpZero { .. }
        | Instruction::IfGreaterOrEqualJumpZero { .. }
        | Instruction::Switch { .. } => {
          let index = graph.add_node(TmpFunctionGraphNode {
            instructions: &function.instructions[cindex..=index]
          });
          node_indices.insert(cnode, index);
          current_index = None;
          current_node = None;
        }
        _ if destinations.contains(&next) => {
          let index = graph.add_node(TmpFunctionGraphNode {
            instructions: &function.instructions[cindex..=index]
          });
          node_indices.insert(cnode, index);
          current_index = None;
          current_node = None;
        }
        _ => {}
      }
    }

    if let Some(cindex) = current_index {
      let index = graph.add_node(TmpFunctionGraphNode {
        instructions: &function.instructions[cindex..]
      });
      node_indices.insert(function.instructions[cindex].pos, index);
    }

    for node_index in graph.node_indices() {
      let node = graph.node_weight(node_index).unwrap();
      for instr in node.instructions {
        let next = instr.pos + instr.bytes.len();
        match &instr.instruction {
          Instruction::Leave { .. } => {}
          Instruction::JumpZero { location }
          | Instruction::IfEqualJumpZero { location }
          | Instruction::IfNotEqualJumpZero { location }
          | Instruction::IfLowerThanJumpZero { location }
          | Instruction::IfGreaterThanJumpZero { location }
          | Instruction::IfLowerOrEqualJumpZero { location }
          | Instruction::IfGreaterOrEqualJumpZero { location } => {
            if let Some(to) = node_indices.get(&(*location as usize)) {
              graph.add_edge(node_index, *to, EdgeType::ConditionalJump);
            }
            if let Some(flow) = node_indices.get(&next) {
              graph.add_edge(node_index, *flow, EdgeType::ConditionalFlow);
            }
          }
          Instruction::Jump { location } => {
            if let Some(to) = node_indices.get(&(*location as usize)) {
              graph.add_edge(node_index, *to, EdgeType::ConditionalJump);
            }
          }
          Instruction::Switch { cases } => {
            for case in cases {
              if let Some(to) = node_indices.get(&(case.location as usize)) {
                graph.add_edge(node_index, *to, EdgeType::Case(case.value as i64));
              }
            }
            if let Some(flow) = node_indices.get(&next) {
              graph.add_edge(node_index, *flow, EdgeType::ConditionalFlow);
            }
          }
          _ if destinations.contains(&next) => {
            if let Some(flow) = node_indices.get(&next) {
              graph.add_edge(node_index, *flow, EdgeType::Flow);
            }
          }
          _ => {}
        }
      }
    }

    Self { graph }
  }

  pub fn to_dot_string(&self, formatter: AssemblyFormatter) -> String {
    let mut first = true;
    let mut diagram: LinkedList<String> = Default::default();
    diagram.push_back(r#"digraph{graph[splines=ortho,rankdir=TB,concentrate=true]node[fontname="Consolas",fontcolor=black]"#.to_owned());
    for (index, node) in self.graph.node_references() {
      let has_edges = self
        .graph
        .edges_directed(index, Direction::Outgoing)
        .any(|_| true);
      let assembly = formatter.format(node.instructions, false);
      diagram.push_back(format!(
        "node_{node}[label=\"{assembly}\\l\",shape=rectangle,color={color}]",
        node = index.index(),
        assembly = assembly
          .trim_start_matches('\n')
          .replace('\n', "\\l")
          .replace('\t', "    ")
          .replace('"', "\\\""),
        color = {
          if first {
            first = false;
            "darkgreen"
          } else if !has_edges {
            "red4"
          } else {
            "black"
          }
        }
      ));
    }

    for (i, edge) in self.graph.edge_references().enumerate() {
      diagram.push_back(format!(
        "node_{origin}->node_{dest}[color={color}]",
        origin = edge.source().index(),
        dest = edge.target().index(),
        color = {
          match edge.weight() {
            EdgeType::ConditionalJump | EdgeType::Case(..) => "darkgreen",
            EdgeType::ConditionalFlow => "red4",
            EdgeType::Flow | EdgeType::Jump => "black"
          }
        }
      ));
    }

    diagram.push_back("}".to_owned());

    diagram.into_iter().collect::<Vec<_>>().join("")
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
      | Instruction::IfGreaterOrEqualJumpZero { location } => {
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
