use std::{
  collections::{HashMap, HashSet, LinkedList},
  fmt::Debug
};

use petgraph::{
  algo::dominators::{simple_fast, Dominators},
  graph::NodeIndex,
  prelude::DiGraph,
  visit::{EdgeRef, IntoNodeIdentifiers, IntoNodeReferences},
  Direction
};

use crate::{
  disassembler::{Instruction, InstructionInfo, SwitchCase},
  formatters::AssemblyFormatter
};

use super::{
  cfg_reducer::{CfgReducer, NodeReductionError},
  function::FunctionInfo,
  ControlFlow
};

#[derive(Clone, Copy, Debug)]
pub enum EdgeType {
  Jump,
  ConditionalJump,
  ConditionalFlow,
  Case(i64),
  Flow
}

#[derive(Debug, Clone, Copy)]
pub struct FunctionGraphNode<'input, 'bytes> {
  pub instructions: &'input [InstructionInfo<'bytes>]
}

#[derive(Debug, Clone)]
pub struct FunctionGraph<'input, 'bytes> {
  graph:      DiGraph<FunctionGraphNode<'input, 'bytes>, EdgeType>,
  dominators: Dominators<NodeIndex>,
  frontiers:  HashMap<NodeIndex, HashSet<NodeIndex>>
}

impl<'input: 'bytes, 'bytes> FunctionGraph<'input, 'bytes> {
  pub fn generate(function: &FunctionInfo<'input, 'bytes>) -> Self {
    let mut graph: DiGraph<FunctionGraphNode<'input, 'bytes>, EdgeType> = Default::default();
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
          let index = graph.add_node(FunctionGraphNode {
            instructions: &function.instructions[cindex..=index]
          });
          node_indices.insert(cnode, index);
          current_index = None;
          current_node = None;
        }
        _ if destinations.contains(&next) => {
          let index = graph.add_node(FunctionGraphNode {
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
      let index = graph.add_node(FunctionGraphNode {
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
              graph.add_edge(node_index, *to, EdgeType::Jump);
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

    graph = remove_unreachable(graph, 0.into());
    let dominators: Dominators<NodeIndex> = simple_fast(&graph, 0.into());
    let frontiers = domination_frontiers(&graph, &dominators);

    Self {
      graph,
      dominators,
      frontiers
    }
  }

  pub fn to_dot_string(&self, formatter: &AssemblyFormatter) -> String {
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
        "node_{node}[margin=0.0,label=<<table border=\"0\"><tr><td bgcolor=\"#AAAAAA\">Node {node}</td></tr><tr><td align=\"text\">{assembly}<br align=\"left\" /></td></tr></table>>,shape=rectangle,color={color}]",
        node = index.index(),
        assembly = assembly
          .trim_start_matches('\n')
          .replace('\n', "<br align=\"left\" />")
          .replace('\t', "    "),
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

    for edge in self.graph.edge_references() {
      diagram.push_back(format!(
        "node_{origin}->node_{dest}[color={color},{extra}]",
        origin = edge.source().index(),
        dest = edge.target().index(),
        color = {
          match edge.weight() {
            EdgeType::ConditionalJump | EdgeType::Case(..) => "darkgreen",
            EdgeType::ConditionalFlow => "red4",
            EdgeType::Flow | EdgeType::Jump => "black"
          }
        },
        extra = {
          match edge.weight() {
            EdgeType::Case(value) => format!("label=\"{value}\""),
            _ => "".to_owned()
          }
        }
      ));
    }

    diagram.push_back("}".to_owned());

    diagram.into_iter().collect::<Vec<_>>().join("")
  }

  pub fn get_node(&self, node: NodeIndex) -> Option<&FunctionGraphNode> {
    self.graph.node_weight(node)
  }

  pub fn reduce_control_flow(&self) -> Result<HashMap<NodeIndex, ControlFlow>, NodeReductionError> {
    let reducer = CfgReducer {
      graph:      &self.graph,
      dominators: &self.dominators,
      frontiers:  &self.frontiers
    };

    reducer.reduce(0.into())
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

// https://github.com/m4b/petgraph/blob/9a6af51bf9803414d68e27f5e8d08600ce2a6212/src/algo/dominators.rs#L90
fn domination_frontiers<N: Debug, E>(
  graph: &DiGraph<N, E>,
  dominators: &Dominators<NodeIndex>
) -> HashMap<NodeIndex, HashSet<NodeIndex>> {
  let mut frontiers = HashMap::from_iter(graph.node_identifiers().map(|v| (v, HashSet::default())));

  for node in graph.node_identifiers() {
    let (predecessors, predecessors_len) = {
      let ret = graph.neighbors_directed(node, Direction::Incoming);
      let count = ret.clone().count();
      (ret, count)
    };

    if predecessors_len >= 2 {
      for p in predecessors {
        let mut runner = p;

        if let Some(dominator) = dominators.immediate_dominator(node) {
          while runner != dominator {
            if runner != node {
              frontiers
                .entry(runner)
                .or_insert(HashSet::default())
                .insert(node);
            }

            runner = dominators.immediate_dominator(runner).unwrap();
          }
        }
      }
    }
  }

  frontiers
}

fn remove_unreachable<'i: 'b, 'b>(
  graph: DiGraph<FunctionGraphNode<'i, 'b>, EdgeType>,
  root: NodeIndex
) -> DiGraph<FunctionGraphNode<'i, 'b>, EdgeType> {
  let mut connected: HashSet<NodeIndex> = Default::default();
  let mut stack = vec![root];

  while let Some(head) = stack.pop() {
    if !connected.contains(&head) {
      connected.insert(head);
      for edge in graph.edges_directed(head, Direction::Outgoing) {
        stack.push(edge.target());
      }
    }
  }

  graph.filter_map(
    |node, n| connected.contains(&node).then_some(*n),
    |_, e| Some(*e)
  )
}
