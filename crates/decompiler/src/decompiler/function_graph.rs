use std::{
  cmp::Ordering,
  collections::{HashMap, HashSet, LinkedList},
  fmt::Debug,
  ops::Sub
};

use itertools::Itertools;
use petgraph::{
  algo::dominators::{simple_fast, Dominators},
  graph::NodeIndex,
  prelude::DiGraph,
  visit::{EdgeRef, IntoNodeIdentifiers, IntoNodeReferences},
  Direction
};

use crate::{
  common::{bubble_sort_by, ParentedList},
  disassembler::{Instruction, InstructionInfo, SwitchCase},
  formatters::AssemblyFormatter
};

use super::{function::FunctionInfo, control_flow::ControlFlow, CaseValue};



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

  pub fn reconstruct_control_flow(&self) -> ControlFlow {
    self.node_control_flow(self.dominators.root(), Default::default())
  }

  fn node_control_flow(
    &self,
    node: NodeIndex,
    parents: ParentedList<'_, FlowParentType>
  ) -> ControlFlow {
    let dominated_edges = self
      .graph
      .edges_directed(node, Direction::Outgoing)
      .filter(|edge| !self.frontiers[&node].contains(&edge.target()))
      .map(|edge| (edge.target(), edge.weight()))
      .collect::<Vec<_>>();

    let frontier_edges = self
      .graph
      .edges_directed(node, Direction::Outgoing)
      .filter(|edge| self.frontiers[&node].contains(&edge.target()))
      .map(|edge| (edge.target(), edge.weight()))
      .collect::<Vec<_>>();

    match (&dominated_edges[..], &frontier_edges[..]) {
      (
        [(cond_jmp, EdgeType::ConditionalJump), (cond_flow, EdgeType::ConditionalFlow)]
        | [(cond_flow, EdgeType::ConditionalFlow), (cond_jmp, EdgeType::ConditionalJump)],
        _
      ) => {
        if self.frontiers[cond_jmp].contains(&node) {
          panic!("inverse if statements are not supported");
        } else if self.frontiers[cond_flow].contains(&node) {
          ControlFlow::WhileLoop {
            node,
            body: Box::new(
              self.node_control_flow(*cond_flow, parents.with_appended(FlowParentType::Loop{ node, after: Some(*cond_flow)}))
            ),
            after: Some(Box::new(self.node_control_flow(*cond_jmp, parents)))
          }
        } else if self.is_and_or_node(*cond_jmp) && self.frontiers[cond_jmp].contains(cond_flow) {
          panic!("inverse if statements are not supported");
        } else if self.is_and_or_node(*cond_flow) && self.frontiers[cond_flow].contains(cond_jmp) {
          ControlFlow::AndOr {
            node,
            with: Box::new(self.node_control_flow(*cond_flow, parents)),
            after: Box::new(self.node_control_flow(*cond_jmp, parents))
          }
        } else if self.frontiers[cond_jmp].contains(cond_flow) {
          panic!("inverse if statements are not supported");
        } else if self.frontiers[cond_flow].is_empty() ||self.frontiers[cond_flow].contains(cond_jmp)   {
          ControlFlow::If {
            node,
            then: Box::new(self.node_control_flow(*cond_flow, parents.with_appended(FlowParentType::NonBreakable { node, after: Some(*cond_jmp) }))),
            after: Some(Box::new(self.node_control_flow(*cond_jmp, parents)))
          }
        } else {
          let intersect = self.frontiers[cond_jmp]
            .intersection(&self.frontiers[cond_flow])
            .copied()
            .collect::<Vec<_>>();

          match intersect[..] {
            [after] if self.frontiers[cond_jmp].contains(&after) && !self.frontiers[&node].contains(&after) => {
              ControlFlow::IfElse {
                node,
                then: Box::new(self.node_control_flow(*cond_flow, parents.with_appended(FlowParentType::NonBreakable { node, after: Some(after) }))),
                els: Box::new(self.node_control_flow(*cond_jmp, parents.with_appended(FlowParentType::NonBreakable { node, after: Some(after) }))),
                after: Some(Box::new(self.node_control_flow(after, parents)))
              }
            }
            [] | [_] => {
              ControlFlow::IfElse {
                node,
                then: Box::new(self.node_control_flow(*cond_flow, parents)),
                els: Box::new(self.node_control_flow(*cond_jmp, parents)),
                after: None
              }
            }
            _ => todo!()
          }
        }
      }
      ([(then, EdgeType::ConditionalFlow) | (then, EdgeType::ConditionalJump)], _) => {
        if self.frontiers[then].contains(&node) {
          ControlFlow::WhileLoop {
            node,
            body: Box::new(self.node_control_flow(
              *then,
              parents.with_appended(FlowParentType::Loop{ node, after: None })
            )),
            after: None
          }
        } else {
          ControlFlow::If {
            node,
            then: Box::new(self.node_control_flow(*then, parents)),
            after: None
          }
        }
      }
      ([.., (_, EdgeType::Case(..))] | [(_, EdgeType::Case(..)), ..], []) => {
        let grouped = dominated_edges.iter().rev().group_by(|(dest, _)| *dest);

        let mut cases = grouped
          .into_iter()
          .map(|(key, group)| {
            (key, group.map(|(_, e)| match e {
              EdgeType::ConditionalFlow => CaseValue::Default,
              EdgeType::Case(value) => CaseValue::Value(*value),
              _ => panic!("unexpected switch flow")
            }).collect_vec())
          })
          .collect_vec();

        bubble_sort_by(&mut cases, |(a, _), (b, _)| {
          let a_frontiers_b = self.frontiers
            .get(a)
            .map(|front| front.contains(b))
            .unwrap_or_default();
          let b_frontiers_a = self.frontiers
            .get(b)
            .map(|front| front.contains(a))
            .unwrap_or_default();
          match (a_frontiers_b, b_frontiers_a) {
            (true, true) => panic!("circular switch case"),
            (false, false) => Ordering::Equal,
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater
          }
        });
        
        let case_set = cases.iter().map(|(index, _)| *index).collect::<HashSet<_>>();

        let mut case_frontiers = cases
          .iter()
          .flat_map(|(node, _)| self.frontiers[node].sub(&case_set))
          .dedup();

        let mut after_node = match (case_frontiers.next(), case_frontiers.next()) {
          (None, _) => None,
          (Some(frontier), None) => Some(frontier),
          (Some(_), Some(_)) => panic!("multiple frontiers for switch cases.")
        };

        if let Some(parent_after) = self.get_first_after(parents) {
          if after_node.map(|n| n == parent_after).unwrap_or_default() {
            after_node = None;
          }
        }

        ControlFlow::Switch {
          node,
          cases: cases
            .into_iter()
            .map(|(case_node, cases)| {
              (self.node_control_flow(case_node, parents.with_appended(FlowParentType::Switch { node, after: after_node })), cases)
            })
            .collect(),
          after: after_node.map(|after| Box::new(self.node_control_flow(after, parents)))
        }
      }
      ([(after, EdgeType::Flow) | (after, EdgeType::Jump)], []) => {
        ControlFlow::Flow {
          node,
          after: Box::new(self.node_control_flow(*after, parents))
        }
      }
      ([], [(target, EdgeType::Jump)]) if let Some(breaks) = self.get_break(*target, parents) => {
        ControlFlow::Break { node, breaks }
      }
      ([], [(target, EdgeType::Jump)]) if let Some(continues) = self.get_continue(*target, parents) => {
        ControlFlow::Continue{ node, continues }
      }
      ([], _) => ControlFlow::Leaf { node },
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

  fn get_break(
    &self,
    target: NodeIndex,
    parents: ParentedList<'_, FlowParentType>
  ) -> Option<NodeIndex> {
    let mut iter = parents.iter().peekable();
    while let Some(FlowParentType::Loop { node, after } | FlowParentType::Switch { node, after }) =
      iter.find(|flow| {
        matches!(
          flow,
          FlowParentType::Loop { .. } | FlowParentType::Switch { .. }
        )
      })
    {
      let mut after = *after;

      while let Some(next) = iter.peek() {
        // https://github.com/rust-lang/rust/issues/53667
        if after.is_some() {
          break;
        }

        match next {
          FlowParentType::Loop { node, .. } => {
            after = Some(*node);
            break;
          }
          FlowParentType::NonBreakable {
            after: after_node, ..
          }
          | FlowParentType::Switch {
            after: after_node, ..
          } => {
            after = *after_node;
            iter.next();
          }
        }
      }

      if after.is_some() && after.unwrap() == target {
        return Some(*node);
      }
    }

    None
  }

  fn get_continue(
    &self,
    target: NodeIndex,
    parents: ParentedList<'_, FlowParentType>
  ) -> Option<NodeIndex> {
    let mut loop_node = None;
    let mut after_node = None;

    for parent in parents.iter() {
      match parent {
        FlowParentType::Loop { node, .. } if *node == target => {
          loop_node = Some(*node);
          break;
        }
        FlowParentType::Loop { node, .. } if *node != target => {
          after_node.get_or_insert(*node);
        }
        FlowParentType::Switch {
          after: Some(after), ..
        }
        | FlowParentType::NonBreakable {
          after: Some(after), ..
        } => {
          after_node.get_or_insert(*after);
        }
        FlowParentType::Switch { .. }
        | FlowParentType::NonBreakable { .. }
        | FlowParentType::Loop { .. } => {}
      }
    }

    after_node.and(loop_node)
  }

  fn get_first_after(&self, parents: ParentedList<'_, FlowParentType>) -> Option<NodeIndex> {
    for parent in parents.iter() {
      match parent {
        FlowParentType::Loop { after, .. }
        | FlowParentType::Switch { after , ..} 
        | FlowParentType::NonBreakable { after, .. } if after.is_some() => { 
          return *after;
        }
        _ => {}
    }
    }
    None
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
      (Some(last), None) => Some(last.source()),
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

#[derive(Debug, Clone, Copy)]
enum FlowParentType {
  Loop {
    node:  NodeIndex,
    after: Option<NodeIndex>
  },
  Switch {
    node:  NodeIndex,
    after: Option<NodeIndex>
  },
  NonBreakable {
    #[allow(dead_code)]
    node:  NodeIndex,
    after: Option<NodeIndex>
  }
}
