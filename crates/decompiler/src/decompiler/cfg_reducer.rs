use std::{
  cmp::Ordering,
  collections::{HashMap, HashSet, VecDeque},
  ops::Sub
};

use itertools::Itertools;
use petgraph::{
  algo::dominators::Dominators, graph::NodeIndex, prelude::DiGraph, visit::EdgeRef, Direction
};
use thiserror::Error;

use crate::{
  common::try_bubble_sort_by,
  disassembler::{Instruction, InstructionInfo}
};

use super::{
  function_graph::{EdgeType, FunctionGraphNode},
  CaseValue, ControlFlow, FlowType
};

pub struct CfgReducer<'g, 'i, 'b> {
  pub graph:      &'g DiGraph<FunctionGraphNode<'i, 'b>, EdgeType>,
  pub dominators: &'g Dominators<NodeIndex>,
  pub frontiers:  &'g HashMap<NodeIndex, HashSet<NodeIndex>>
}

impl<'g, 'i, 'b> CfgReducer<'g, 'i, 'b> {
  pub fn reduce(
    &self,
    root: NodeIndex
  ) -> Result<HashMap<NodeIndex, ControlFlow>, NodeReductionError> {
    let mut result: HashMap<NodeIndex, ControlFlow> = HashMap::new();
    let mut parents: Vec<FlowType> = vec![];
    let mut stack: Vec<(NodeIndex, usize)> = vec![(root, 0)];
    let mut claimed_nodes: HashSet<NodeIndex> = Default::default();

    // DFS
    while let Some((node, depth)) = stack.pop() {
      if depth < parents.len() {
        parents.drain(depth + 1..);
      }

      let reduced = self.reduce_node(node, &parents, &claimed_nodes)?;
      match &reduced {
        ControlFlow::If { then, after, .. } => {
          if let Some(after) = after {
            stack.push((*after, depth));
            claimed_nodes.insert(*after);
          }
          stack.push((*then, depth + 1));
          claimed_nodes.insert(*then);
        }
        ControlFlow::IfElse {
          then, els, after, ..
        } => {
          if let Some(after) = after {
            stack.push((*after, depth));
            claimed_nodes.insert(*after);
          }
          stack.push((*els, depth + 1));
          claimed_nodes.insert(*els);
          stack.push((*then, depth + 1));
          claimed_nodes.insert(*then);
        }
        ControlFlow::AndOr { with, after, .. } => {
          stack.push((*after, depth));
          claimed_nodes.insert(*after);
          stack.push((*with, depth + 1));
          claimed_nodes.insert(*with);
        }
        ControlFlow::WhileLoop { body, after, .. } => {
          if let Some(after) = after {
            stack.push((*after, depth));
            claimed_nodes.insert(*after);
          }
          stack.push((*body, depth + 1));
          claimed_nodes.insert(*body);
        }
        ControlFlow::Flow { after, .. } => {
          stack.push((*after, depth));
          claimed_nodes.insert(*after);
        }
        ControlFlow::Switch { cases, after, .. } => {
          if let Some(after) = after {
            stack.push((*after, depth));
            claimed_nodes.insert(*after);
          }

          for (node, _) in cases {
            stack.push((*node, depth + 1));
            claimed_nodes.insert(*node);
          }
        }
        ControlFlow::Leaf { .. } | ControlFlow::Break { .. } | ControlFlow::Continue { .. } => {}
      }

      parents.push(reduced.flow_type());
      result.insert(node, reduced);
    }

    Ok(result)
  }

  fn reduce_node(
    &self,
    node: NodeIndex,
    parents: &[FlowType],
    claimed_nodes: &HashSet<NodeIndex>
  ) -> Result<ControlFlow, NodeReductionError> {
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
        self
          .try_reduce_bi_flow(node, *cond_flow, *cond_jmp)
          .inner_or_else(|| self.try_reduce_and_or(node, *cond_flow, *cond_jmp))
          .inner_or_else(|| {
            self.try_reduce_while_loop(node, *cond_flow, Some(*cond_jmp), parents, claimed_nodes)
          })
          .inner_or_else(|| {
            self.try_reduce_if(node, *cond_flow, Some(*cond_jmp), parents, claimed_nodes)
          })
          .inner_or_else(|| {
            self.try_reduce_if_else(node, *cond_jmp, *cond_flow, parents, claimed_nodes)
          })?
          .map_or(
            Err(NodeReductionError {
              node,
              message: "Unrecognized control flow"
            }),
            |reduced| Ok(reduced)
          )
      }
      ([(then, EdgeType::ConditionalFlow)], _) => {
        self
          .try_reduce_while_loop(node, *then, None, parents, claimed_nodes)
          .inner_or_else(|| self.try_reduce_if(node, *then, None, parents, claimed_nodes))?
          .map_or(
            Err(NodeReductionError {
              node,
              message: "Unrecognized control flow"
            }),
            |reduced| Ok(reduced)
          )
      }
      (cases @ [.., (_, EdgeType::Case(..))] | cases @ [(_, EdgeType::Case(..)), ..], []) => {
        self.reduce_switch(node, cases, parents, claimed_nodes)
      }
      ([(after, EdgeType::Flow) | (after, EdgeType::Jump)], []) => {
        Ok(ControlFlow::Flow {
          node,
          after: *after
        })
      }
      ([], [(target, EdgeType::Jump)]) => {
        Ok(
          self
            .try_reduce_break(node, *target, parents)
            .or_else(|| self.try_reduce_continue(node, *target, parents))
            .unwrap_or(ControlFlow::Leaf { node })
        )
      }
      (
        [],
        [(a, EdgeType::ConditionalFlow), (b, EdgeType::ConditionalJump)]
        | [(a, EdgeType::ConditionalJump), (b, EdgeType::ConditionalFlow)]
      ) if *a == *b => Ok(ControlFlow::Leaf { node }),
      ([], [(after, EdgeType::Flow)])
        if self.is_valid_after_node(*after, parents, claimed_nodes) =>
      {
        Ok(ControlFlow::Flow {
          node,
          after: *after
        })
      }
      ([], [(_, EdgeType::Flow)] | []) => Ok(ControlFlow::Leaf { node }),
      _ => {
        Err(NodeReductionError {
          node,
          message: "Unrecognized control flow"
        })
        .unwrap()
      }
    }
  }

  fn reduce_switch(
    &self,
    switch_node: NodeIndex,
    cases: &[(NodeIndex, &EdgeType)],
    parents: &[FlowType],
    claimed_nodes: &HashSet<NodeIndex>
  ) -> Result<ControlFlow, NodeReductionError> {
    let grouped = cases.iter().rev().group_by(|(dest, _)| *dest);

    let mut cases = grouped
      .into_iter()
      .map(|(key, group)| {
        (
          key,
          group
            .map(|(_, e)| {
              match e {
                EdgeType::ConditionalFlow => CaseValue::Default,
                EdgeType::Case(value) => CaseValue::Value(*value),
                _ => panic!("unexpected switch flow")
              }
            })
            .collect_vec()
        )
      })
      .collect_vec();

    try_bubble_sort_by(&mut cases, |(a, _), (b, _)| {
      let a_frontiers_b = self
        .frontiers
        .get(a)
        .map(|front| front.contains(b))
        .unwrap_or_default();
      let b_frontiers_a = self
        .frontiers
        .get(b)
        .map(|front| front.contains(a))
        .unwrap_or_default();
      match (a_frontiers_b, b_frontiers_a) {
        (true, true) => {
          Err(NodeReductionError {
            node:    switch_node,
            message: "switch has case nodes that frontier at each other"
          })
        }
        (false, false) => Ok(Ordering::Equal),
        (true, false) => Ok(Ordering::Less),
        (false, true) => Ok(Ordering::Greater)
      }
    })?;

    let after_node = self.get_switch_after_node(switch_node, &cases, parents, claimed_nodes)?;

    Ok(ControlFlow::Switch {
      node: switch_node,
      cases,
      after: after_node
    })
  }

  fn get_switch_after_node(
    &self,
    switch_node: NodeIndex,
    cases: &[(NodeIndex, Vec<CaseValue>)],
    parents: &[FlowType],
    claimed_nodes: &HashSet<NodeIndex>
  ) -> Result<Option<NodeIndex>, NodeReductionError> {
    let case_set = cases
      .iter()
      .map(|(index, _)| *index)
      .collect::<HashSet<_>>();

    let case_frontiers = cases
      .iter()
      .flat_map(|(n, _)| self.frontiers[n].sub(&case_set))
      .filter(|n| self.is_valid_after_node(*n, parents, claimed_nodes))
      .dedup()
      .collect_vec();

    let mut iter = case_frontiers.iter();
    let (after_node, None) = (iter.next(), iter.next()) else {
      return self.get_after_node_expensive(
        switch_node,
        &cases.iter().map(|(node, _)| *node).collect_vec(),
        case_frontiers
      );
    };

    Ok(after_node.copied())
  }

  fn try_reduce_bi_flow(
    &self,
    node: NodeIndex,
    cond_flow: NodeIndex,
    cond_jmp: NodeIndex
  ) -> Result<Option<ControlFlow>, NodeReductionError> {
    if cond_flow == cond_jmp {
      Ok(Some(ControlFlow::Flow {
        node,
        after: cond_flow
      }))
    } else {
      Ok(None)
    }
  }

  fn try_reduce_and_or(
    &self,
    node: NodeIndex,
    cond_flow: NodeIndex,
    cond_jmp: NodeIndex
  ) -> Result<Option<ControlFlow>, NodeReductionError> {
    if self.frontiers[&cond_jmp].contains(&cond_flow) && self.is_and_or_node(cond_jmp) {
      Err(NodeReductionError {
        node,
        message: "inverse and/or statements are not supported"
      })
    } else if self.frontiers[&cond_flow].contains(&cond_jmp) && self.is_and_or_node(cond_flow) {
      Ok(Some(ControlFlow::AndOr {
        node,
        with: cond_flow,
        after: cond_jmp
      }))
    } else {
      Ok(None)
    }
  }

  fn try_reduce_while_loop(
    &self,
    node: NodeIndex,
    cond_flow: NodeIndex,
    cond_jmp: Option<NodeIndex>,
    parents: &[FlowType],
    claimed_nodes: &HashSet<NodeIndex>
  ) -> Result<Option<ControlFlow>, NodeReductionError> {
    if let Some(cond_jmp) = cond_jmp
      && self.frontiers[&cond_jmp].contains(&node)
    {
      Err(NodeReductionError {
        node,
        message: "inverse while loops are not supported"
      })
    } else if self.frontiers[&cond_flow].contains(&node) {
      let after = cond_jmp.and_then(|cond_jmp| {
        self
          .is_valid_after_node(cond_jmp, parents, claimed_nodes)
          .then_some(cond_jmp)
      });
      Ok(Some(ControlFlow::WhileLoop {
        node,
        body: cond_flow,
        after
      }))
    } else {
      Ok(None)
    }
  }

  fn try_reduce_if(
    &self,
    node: NodeIndex,
    cond_flow: NodeIndex,
    cond_jmp: Option<NodeIndex>,
    parents: &[FlowType],
    claimed_nodes: &HashSet<NodeIndex>
  ) -> Result<Option<ControlFlow>, NodeReductionError> {
    if let Some(cond_jmp) = cond_jmp
      && self.frontiers[&cond_jmp].contains(&cond_flow)
    {
      Err(NodeReductionError {
        node,
        message: "inverse if statements are not supported"
      })
    } else if cond_jmp
      .map(|cond_jmp| self.frontiers[&cond_flow].contains(&cond_jmp))
      .unwrap_or(true)
    {
      let after = cond_jmp.and_then(|cond_jmp| {
        self
          .is_valid_after_node(cond_jmp, parents, claimed_nodes)
          .then_some(cond_jmp)
      });
      Ok(Some(ControlFlow::If {
        node,
        then: cond_flow,
        after
      }))
    } else {
      Ok(None)
    }
  }

  fn try_reduce_if_else(
    &self,
    node: NodeIndex,
    cond_jmp: NodeIndex,
    cond_flow: NodeIndex,
    parents: &[FlowType],
    claimed_nodes: &HashSet<NodeIndex>
  ) -> Result<Option<ControlFlow>, NodeReductionError> {
    if self.frontiers[&cond_jmp].contains(&cond_flow)
      || self.frontiers[&cond_flow].contains(&cond_jmp)
    {
      return Ok(None);
    }

    let after = self.get_if_else_after_node(node, cond_jmp, cond_flow, parents, claimed_nodes)?;

    Ok(Some(ControlFlow::IfElse {
      node,
      then: cond_flow,
      els: cond_jmp,
      after: after
    }))
  }

  fn get_if_else_after_node(
    &self,
    node: NodeIndex,
    cond_jmp: NodeIndex,
    cond_flow: NodeIndex,
    parents: &[FlowType],
    claimed_nodes: &HashSet<NodeIndex>
  ) -> Result<Option<NodeIndex>, NodeReductionError> {
    let intersection = self.frontiers[&cond_jmp]
      .intersection(&self.frontiers[&cond_flow])
      .filter(|after| self.is_valid_after_node(**after, parents, claimed_nodes))
      .copied();

    let mut iter = intersection.clone();
    let (after, None) = (iter.next(), iter.next()) else {
      return self.get_after_node_expensive(
        node,
        &[cond_jmp, cond_flow],
        intersection.collect_vec()
      );
    };

    Ok(after)
  }

  fn get_after_node_expensive(
    &self,
    node: NodeIndex,
    initial: &[NodeIndex],
    candidates: Vec<NodeIndex>
  ) -> Result<Option<NodeIndex>, NodeReductionError> {
    let mut result = vec![];
    let mut queue = VecDeque::from_iter(initial.iter().copied());

    while let Some(item) = queue.pop_front() {
      if candidates.contains(&item) {
        result.push(item);
      } else {
        let outgoing = self.graph.edges_directed(item, Direction::Outgoing);

        for edge in outgoing {
          match edge.weight() {
            EdgeType::Jump | EdgeType::ConditionalJump | EdgeType::Flow => {
              queue.push_back(edge.target())
            }
            EdgeType::ConditionalFlow | EdgeType::Case(_) => {}
          }
        }
      }
    }

    result.dedup();
    match result[..] {
      [] => Ok(None),
      [single] => Ok(Some(single)),
      _ => {
        Err(NodeReductionError {
          node,
          message: "multiple valid after node frontiers"
        })
      }
    }
  }

  fn is_valid_after_node(
    &self,
    candidate: NodeIndex,
    parents: &[FlowType],
    claimed_nodes: &HashSet<NodeIndex>
  ) -> bool {
    if claimed_nodes.contains(&candidate) {
      return false;
    }

    for parent in parents {
      match parent {
        FlowType::Loop { after, node }
        | FlowType::Switch { after, node }
        | FlowType::NonBreakable { after, node } => {
          if after.map(|node| node == candidate).unwrap_or_default() || *node == candidate {
            return false;
          }
        }
      }
    }

    true
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
      (Some(last), None) => Some(last.source()),
      _ => None
    }
  }

  fn try_reduce_break(
    &self,
    node: NodeIndex,
    target: NodeIndex,
    parents: &[FlowType]
  ) -> Option<ControlFlow> {
    let mut iter = parents.iter().rev().peekable();
    while let Some(FlowType::Loop { after, .. } | FlowType::Switch { after, .. }) =
      iter.find(|flow| matches!(flow, FlowType::Loop { .. } | FlowType::Switch { .. }))
    {
      let mut after = *after;

      while let Some(next) = iter.peek() {
        match next {
          FlowType::Loop { node, .. } => {
            after = Some(*node);
            break;
          }
          FlowType::NonBreakable {
            after: after_node, ..
          }
          | FlowType::Switch {
            after: after_node, ..
          } => {
            after = *after_node;
            iter.next();
            break;
          }
        }
      }

      if after.is_some() && after.unwrap() == target {
        return Some(ControlFlow::Break {
          node,
          breaks: target
        });
      }
    }

    None
  }

  fn try_reduce_continue(
    &self,
    node: NodeIndex,
    target: NodeIndex,
    parents: &[FlowType]
  ) -> Option<ControlFlow> {
    if self.get_first_after(parents) == Some(target) {
      return None;
    }

    let mut loop_node = None;
    let mut after_node = None;

    for parent in parents.iter().rev() {
      match parent {
        FlowType::Loop {
          node: parent_node, ..
        } => {
          if *parent_node == target
            || self
              .graph
              .edges_directed(*parent_node, Direction::Incoming)
              .any(|edge| edge.source() == target)
          {
            loop_node = Some(*parent_node);
            break;
          } else {
            after_node.get_or_insert(*parent_node);
          }
        }
        FlowType::Switch {
          after: Some(after), ..
        }
        | FlowType::NonBreakable {
          after: Some(after), ..
        } => {
          after_node.get_or_insert(*after);
        }
        FlowType::Switch { .. } | FlowType::NonBreakable { .. } => {}
      }
    }

    after_node.and(loop_node.map(|loop_node| {
      ControlFlow::Continue {
        node,
        continues: loop_node
      }
    }))
  }

  fn get_first_after(&self, parents: &[FlowType]) -> Option<NodeIndex> {
    parents.iter().rev().find_map(|parent| {
      match parent {
        FlowType::Loop { after, .. }
        | FlowType::Switch { after, .. }
        | FlowType::NonBreakable { after, .. } => *after
      }
    })
  }
}

#[derive(Debug, Error)]
#[error("Failed to reduce node {node:?} in function ...: {message}")]
pub struct NodeReductionError {
  node:    NodeIndex,
  message: &'static str
}

pub trait InnerOrElse {
  fn inner_or_else(self, fun: impl FnOnce() -> Self) -> Self;
}

impl<T, E> InnerOrElse for Result<Option<T>, E> {
  fn inner_or_else(self, fun: impl FnOnce() -> Self) -> Self {
    if let Ok(None) = self {
      fun()
    } else {
      self
    }
  }
}
