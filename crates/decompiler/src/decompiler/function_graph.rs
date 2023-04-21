use std::collections::{HashMap, HashSet, LinkedList};

use crate::{
  disassembler::{Instruction, InstructionInfo, SwitchCase},
  formatters::AssemblyFormatter
};

use super::function::FunctionInfo;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EdgeDest {
  Success(usize),
  Fallback(usize),
  Case(usize, i64),
  Flow(usize)
}

impl EdgeDest {
  pub fn dest(self) -> usize {
    match self {
      EdgeDest::Success(dest)
      | EdgeDest::Fallback(dest)
      | EdgeDest::Flow(dest)
      | EdgeDest::Case(dest, ..) => dest
    }
  }
}

#[derive(Default, Debug, Clone)]

pub struct FunctionGraphNode<'input, 'bytes> {
  pub id:                  usize,
  pub instructions:        &'input [InstructionInfo<'bytes>],
  pub children:            Vec<EdgeDest>,
  pub parents:             Vec<usize>,
  pub dominates:           HashSet<usize>,
  pub dominance_frontiers: HashSet<usize>
}

impl<'input, 'bytes> FunctionGraphNode<'input, 'bytes> {
  pub fn get_success_node(&self) -> Option<EdgeDest> {
    self
      .children
      .iter()
      .find(|dest| matches!(dest, EdgeDest::Success(..)))
      .copied()
  }

  pub fn get_fallback_node(&self) -> Option<EdgeDest> {
    self
      .children
      .iter()
      .find(|dest| matches!(dest, EdgeDest::Fallback(..)))
      .copied()
  }

  pub fn get_flow_node(&self) -> Option<EdgeDest> {
    self
      .children
      .iter()
      .find(|dest| matches!(dest, EdgeDest::Success(..)))
      .copied()
  }

  fn set_dominates(&mut self, node: usize) {
    self.dominates.insert(node);
  }

  fn set_domination_frontier(&mut self, node: usize) {
    self.dominance_frontiers.insert(node);
  }
}

#[derive(Default, Debug, Clone)]
pub struct FunctionGraph<'input, 'bytes> {
  nodes:      HashMap<usize, FunctionGraphNode<'input, 'bytes>>,
  entrypoint: usize
}

impl<'input, 'bytes> FunctionGraph<'input, 'bytes> {
  pub fn generate(function: &FunctionInfo<'input, 'bytes>) -> Self {
    let mut nodes: HashMap<usize, FunctionGraphNode<'input, 'bytes>> = Default::default();
    let mut nodes_and_edges = NodesAndEdgesGenerator::generate(function);

    for (id, instructions) in nodes_and_edges.nodes {
      nodes.insert(
        id,
        FunctionGraphNode {
          id,
          instructions,
          children: nodes_and_edges
            .parent_to_child
            .remove(&id)
            .expect("invalid graph"),
          parents: nodes_and_edges
            .child_to_parent
            .remove(&id)
            .expect("invalid graph"),
          ..Default::default()
        }
      );
    }

    Self {
      nodes,
      entrypoint: function.location
    }
  }

  fn get_dominating_recursive(&mut self, node: &FunctionGraphNode, visited: &mut HashSet<usize>) {
    // for children in node.children {}

    todo!()
  }

  pub fn to_dot_string(&self, formatter: AssemblyFormatter) -> String {
    let mut first = true;
    let mut diagram: LinkedList<String> = Default::default();
    diagram.push_back(r#"digraph{graph[splines=ortho,rankdir=TB,concentrate=true]node[fontname="Consolas",fontcolor=black]"#.to_owned());
    for (_, node) in &self.nodes {
      let assembly = formatter.format(node.instructions, false);
      diagram.push_back(format!(
        "node_{node}[label=\"{assembly}\\l\",shape=rectangle,color={color}]",
        node = node.id,
        assembly = assembly
          .trim_start_matches('\n')
          .replace('\n', "\\l")
          .replace('\t', "    ")
          .replace('"', "\\\""),
        color = {
          if first {
            first = false;
            "darkgreen"
          } else if node.children.is_empty() {
            "red4"
          } else {
            "black"
          }
        }
      ));
    }

    for (origin, destinations) in self
      .nodes
      .iter()
      .map(|(origin, node)| (*origin, &node.children))
    {
      for destination in destinations {
        diagram.push_back(format!(
          "node_{origin}->node_{dest}[color={color}]",
          dest = destination.dest(),
          color = {
            match destination {
              EdgeDest::Success(..) => "darkgreen",
              EdgeDest::Case(..) => "darkgreen",
              EdgeDest::Fallback(..) => "red4",
              EdgeDest::Flow(..) => "black"
            }
          }
        ));
      }
    }

    diagram.push_back("}".to_owned());

    diagram.into_iter().collect::<Vec<_>>().join("")
  }

  pub fn entrypoint(&self) -> usize {
    self.entrypoint
  }

  pub fn node(&self, node: usize) -> Option<&FunctionGraphNode> {
    self.nodes.get(&node)
  }
}

#[derive(Default, Debug, Clone)]
struct NodesAndEdgesGenerator<'input, 'bytes> {
  nodes:           HashMap<usize, &'input [InstructionInfo<'bytes>]>,
  parent_to_child: HashMap<usize, Vec<EdgeDest>>,
  child_to_parent: HashMap<usize, Vec<usize>>
}

impl<'input, 'bytes> NodesAndEdgesGenerator<'input, 'bytes> {
  pub fn generate(function: &FunctionInfo<'input, 'bytes>) -> Self {
    let mut graph = Self::default();
    let destinations = get_destinations(function.instructions);

    let mut current_index: Option<usize> = None;
    let mut current_node: Option<usize> = None;

    for (index, instruction) in function.instructions.iter().enumerate() {
      let cindex = *current_index.get_or_insert(index);
      let cnode = *current_node.get_or_insert(instruction.pos);
      let next = instruction.pos + instruction.bytes.len();

      match &instruction.instruction {
        Instruction::Leave { .. } => {
          graph
            .nodes
            .insert(cnode, &function.instructions[cindex..=index]);
          current_index = None;
          current_node = None;
        }
        Instruction::Jump { location } => {
          graph.add_join(cnode, *location as usize);
          graph
            .nodes
            .insert(cnode, &function.instructions[cindex..=index]);
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
          graph
            .nodes
            .insert(cnode, &function.instructions[cindex..=index]);
          current_index = None;
          current_node = None;
        }
        Instruction::Switch { cases } => {
          graph.add_switch(cnode, cases, next);
          graph
            .nodes
            .insert(cnode, &function.instructions[cindex..=index]);
          current_index = None;
          current_node = None;
        }
        _ => {
          if destinations.contains(&next) {
            graph.add_jmp(cnode, next);
            graph
              .nodes
              .insert(cnode, &function.instructions[cindex..=index]);
            current_index = None;
            current_node = None;
          }
        }
      }
    }

    if let Some(index) = current_index {
      graph.nodes.insert(
        function.instructions[index].pos,
        &function.instructions[index..]
      );
    }

    for (node, _) in &graph.nodes {
      let _ = graph.parent_to_child.try_insert(*node, Default::default());
      let _ = graph.child_to_parent.try_insert(*node, Default::default());
    }

    graph
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
    let edges = self.parent_to_child.entry(origin).or_default();

    if !edges.contains(&destination) {
      edges.push(destination);
      self
        .child_to_parent
        .entry(destination.dest())
        .or_default()
        .push(origin);
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
