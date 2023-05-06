use std::{
  collections::{HashMap, HashSet, LinkedList},
  hash::Hash
};

use petgraph::{
  algo::dominators::{simple_fast, Dominators},
  graph::NodeIndex,
  prelude::DiGraph,
  visit::{
    EdgeCount, EdgeIndexable, EdgeRef, FilterNode, GraphBase, IntoEdgesDirected,
    IntoNodeIdentifiers, IntoNodeReferences, NodeIndexable
  },
  Direction
};

use crate::{
  disassembler::{Instruction, InstructionInfo, SwitchCase},
  formatters::AssemblyFormatter
};

use super::function::FunctionInfo;

#[derive(Debug)]
pub enum ControlFlow {
  If {
    node: NodeIndex,
    then: Box<ControlFlow>
  },
  IfAfter {
    node:  NodeIndex,
    then:  Box<ControlFlow>,
    after: Box<ControlFlow>
  },
  IfElse {
    node: NodeIndex,
    then: Box<ControlFlow>,
    els:  Box<ControlFlow>
  },
  IfElseAfter {
    node:  NodeIndex,
    then:  Box<ControlFlow>,
    els:   Box<ControlFlow>,
    after: Box<ControlFlow>
  },
  Leaf {
    node: NodeIndex
  },
  AndOr {
    node:  NodeIndex,
    with:  Box<ControlFlow>,
    after: Box<ControlFlow>
  } // Loop {
    //   body: Box<ControlFlow>
    // },
    // WhileLoop {
    //   body: Box<ControlFlow>,
    //   els:  Box<ControlFlow>
    // },
    // DoWhile {
    //   body: Box<ControlFlow>,
    //   els:  Box<ControlFlow>
    // }
}

#[derive(Clone, Copy, Debug)]
pub enum EdgeType {
  Jump,
  ConditionalJump,
  ConditionalFlow,
  Case(i64),
  Flow
}

#[derive(Debug, Clone)]
pub struct FunctionGraphNode<'input, 'bytes> {
  instructions: &'input [InstructionInfo<'bytes>]
}

#[derive(Debug, Clone)]
pub struct FunctionGraph<'input, 'bytes> {
  graph:      DiGraph<FunctionGraphNode<'input, 'bytes>, EdgeType>,
  dominators: Dominators<NodeIndex>,
  frontiers:  HashMap<NodeIndex, HashSet<NodeIndex>>
}

impl<'input, 'bytes> FunctionGraph<'input, 'bytes> {
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

    let dominators: Dominators<NodeIndex> = simple_fast(&graph, 0.into());
    let frontiers = domination_frontiers(&graph, &dominators);

    Self {
      graph,
      dominators,
      frontiers
    }
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

  pub fn reconstruct_control_flow(&self) -> ControlFlow {
    self.node_control_flow(self.dominators.root())
  }

  fn node_control_flow(&self, node: NodeIndex) -> ControlFlow {
    let edges = self
      .graph
      .edges_directed(node, Direction::Outgoing)
      .filter(|edge| !self.frontiers[&node].contains(&edge.target()))
      .map(|edge| (edge.target(), edge.weight()))
      .collect::<Vec<_>>();

    match &edges[..] {
      [(a, EdgeType::ConditionalJump), (b, EdgeType::ConditionalFlow)]
      | [(a, EdgeType::ConditionalFlow), (b, EdgeType::ConditionalJump)] => {
        if self.is_and_or_node(*a) && self.frontiers[a].contains(b) {
          ControlFlow::AndOr {
            node,
            with: Box::new(self.node_control_flow(*a)),
            after: Box::new(self.node_control_flow(*b))
          }
        } else if self.is_and_or_node(*b) && self.frontiers[b].contains(a) {
          ControlFlow::AndOr {
            node,
            with: Box::new(self.node_control_flow(*b)),
            after: Box::new(self.node_control_flow(*a))
          }
        } else if self.frontiers[a].contains(b) {
          ControlFlow::IfAfter {
            node,
            then: Box::new(self.node_control_flow(*a)),
            after: Box::new(self.node_control_flow(*b))
          }
        } else if self.frontiers[b].contains(a) {
          ControlFlow::IfAfter {
            node,
            then: Box::new(self.node_control_flow(*b)),
            after: Box::new(self.node_control_flow(*a))
          }
        } else {
          let intersect = self.frontiers[a]
            .intersection(&self.frontiers[b])
            .copied()
            .collect::<Vec<_>>();

          match intersect[..] {
            [after] if !self.frontiers[a].contains(&after) => {
              ControlFlow::IfElseAfter {
                node,
                then: Box::new(self.node_control_flow(*a)),
                els: Box::new(self.node_control_flow(*b)),
                after: Box::new(self.node_control_flow(after))
              }
            }
            [] | [_] => {
              ControlFlow::IfElse {
                node,
                then: Box::new(self.node_control_flow(*a)),
                els: Box::new(self.node_control_flow(*b))
              }
            }
            _ => todo!()
          }
        }
      }
      [(then, EdgeType::ConditionalFlow) | (then, EdgeType::ConditionalJump)] => {
        ControlFlow::If {
          node,
          then: Box::new(self.node_control_flow(*then))
        }
      }
      [] => ControlFlow::Leaf { node },
      _ => todo!()
    }
  }

  fn is_and_or_node(&self, node: NodeIndex) -> bool {
    let Some(last) = self.last_singular_dominated_node(node) else {
      return false;
    };

    matches!(
      self.graph.node_weight(last).unwrap().instructions.last(),
      Some(InstructionInfo {
        instruction: Instruction::BitwiseAnd | Instruction::BitwiseOr,
        ..
      })
    )
  }

  fn last_singular_dominated_node(&self, node: NodeIndex) -> Option<NodeIndex> {
    let frontiers = &self.frontiers[&node];

    let mut iter = frontiers.iter();
    let (Some(frontier), None) = (iter.next(), iter.next()) else {
      return None;
    };

    let mut edges = self
      .graph
      .edges_directed(*frontier, Direction::Incoming)
      .filter(|edge| {
        self
          .dominators
          .dominators(edge.source())
          .map(|mut doms| doms.any(|dom| dom == node))
          .unwrap_or_default()
      });

    match (edges.next(), edges.next()) {
      (Some(last), None) => {
        println!("{:?}", last.source());
        Some(last.source())
      }
      _ => None
    }
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

fn domination_frontiers<N, E>(
  graph: &DiGraph<N, E>,
  dominators: &Dominators<<DiGraph<N, E> as GraphBase>::NodeId>
) -> HashMap<<DiGraph<N, E> as GraphBase>::NodeId, HashSet<<DiGraph<N, E> as GraphBase>::NodeId>> {
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

        match dominators.immediate_dominator(node) {
          Some(dominator) => {
            while runner != dominator {
              frontiers
                .entry(runner)
                .or_insert(HashSet::default())
                .insert(node);

              if let Some(dom) = dominators.immediate_dominator(runner) {
                runner = dom;
              } else {
                break;
              }
            }
          }
          None => ()
        }
      }
    }
  }

  frontiers
}
