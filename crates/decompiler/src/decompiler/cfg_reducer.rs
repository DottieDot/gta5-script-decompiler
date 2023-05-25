use std::collections::{HashMap, HashSet};

use petgraph::{
  algo::dominators::Dominators, graph::NodeIndex, prelude::DiGraph, visit::EdgeRef, Direction
};
use thiserror::Error;

use crate::disassembler::{Instruction, InstructionInfo};

use super::function_graph::{EdgeType, FunctionGraphNode};

#[derive(Debug, Clone)]
pub enum ReducedNode {
  If {
    node:  NodeIndex,
    then:  NodeIndex,
    after: Option<NodeIndex>
  },
  IfElse {
    node:  NodeIndex,
    then:  NodeIndex,
    els:   NodeIndex,
    after: Option<NodeIndex>
  },
  Leaf {
    node: NodeIndex
  },
  AndOr {
    node:  NodeIndex,
    with:  NodeIndex,
    after: NodeIndex
  },
  WhileLoop {
    node:  NodeIndex,
    body:  NodeIndex,
    after: Option<NodeIndex>
  },
  Flow {
    node:  NodeIndex,
    after: NodeIndex
  },
  Break {
    node:   NodeIndex,
    breaks: NodeIndex
  },
  Continue {
    node:      NodeIndex,
    continues: NodeIndex
  },
  Switch {
    node:  NodeIndex,
    cases: Vec<(NodeIndex, Vec<CaseValue>)>,
    after: Option<NodeIndex>
  }
}

#[derive(Debug, Clone, Copy)]
pub enum CaseValue {
  Default,
  Value(i64)
}

pub struct CfgReducer<'g, 'i, 'b> {
  graph:      &'g DiGraph<FunctionGraphNode<'i, 'b>, EdgeType>,
  dominators: &'g Dominators<NodeIndex>,
  frontiers:  &'g HashMap<NodeIndex, HashSet<NodeIndex>>
}

impl<'g, 'i, 'b> CfgReducer<'g, 'i, 'b> {
  fn initial_reduce(&self) -> Vec<ReducedNode> {
    todo!()
  }

  fn reduce_node(&self, node: NodeIndex) -> Result<ReducedNode, NodeReductionError> {
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
          .inner_or_else(|| self.try_reduce_while_loop(node, *cond_flow, Some(*cond_jmp)))
          .inner_or_else(|| self.try_reduce_if(node, *cond_flow, Some(*cond_jmp)))
          .inner_or_else(|| self.try_reduce_if_else(node, *cond_jmp, *cond_flow))?
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
          .try_reduce_while_loop(node, *then, None)
          .inner_or_else(|| self.try_reduce_if(node, *then, None))?
          .map_or(
            Err(NodeReductionError {
              node,
              message: "Unrecognized control flow"
            }),
            |reduced| Ok(reduced)
          )
      }
      ([(after, EdgeType::Flow) | (after, EdgeType::Jump)], []) => {
        Ok(ReducedNode::Flow {
          node,
          after: *after
        })
      }
      // TODO: continue, break, and switch
      _ => {
        Err(NodeReductionError {
          node,
          message: "Unrecognized control flow"
        })
      }
    }
  }

  fn try_reduce_bi_flow(
    &self,
    node: NodeIndex,
    cond_flow: NodeIndex,
    cond_jmp: NodeIndex
  ) -> Result<Option<ReducedNode>, NodeReductionError> {
    if cond_flow == cond_jmp {
      Ok(Some(ReducedNode::Flow {
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
  ) -> Result<Option<ReducedNode>, NodeReductionError> {
    if !self.is_and_or_node(node) {
      Ok(None)
    } else if self.frontiers[&cond_jmp].contains(&cond_flow) {
      Err(NodeReductionError {
        node,
        message: "inverse and/or statements are not supported"
      })
    } else if self.frontiers[&cond_flow].contains(&cond_jmp) {
      Ok(Some(ReducedNode::AndOr {
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
    cond_jmp: Option<NodeIndex>
  ) -> Result<Option<ReducedNode>, NodeReductionError> {
    if let Some(cond_jmp) = cond_jmp && self.frontiers[&cond_jmp].contains(&node) {
      Err(NodeReductionError {
        node,
        message: "inverse while loops are not supported"
      })
    } else if self.frontiers[&cond_flow].contains(&node) {
      let after = cond_jmp
        .and_then(|cond_jmp| 
          self.is_valid_after_node(node, cond_jmp)
            .then_some(cond_jmp)
        );
      Ok(Some(ReducedNode::WhileLoop {
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
    cond_jmp: Option<NodeIndex>
  ) -> Result<Option<ReducedNode>, NodeReductionError> {
    if let Some(cond_jmp) = cond_jmp && self.frontiers[&cond_jmp].contains(&cond_flow) {
      Err(NodeReductionError {
        node,
        message: "inverse if statements are not supported"
      })
    } else if cond_jmp.map(|cond_jmp| self.frontiers[&cond_flow].contains(&cond_jmp)).unwrap_or(true) {
      let after = cond_jmp
        .and_then(|cond_jmp| 
          self.is_valid_after_node(node, cond_jmp)
            .then_some(cond_jmp)
        );
      Ok(Some(ReducedNode::If {
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
    cond_flow: NodeIndex
  ) -> Result<Option<ReducedNode>, NodeReductionError> {
    if self.frontiers[&cond_jmp].contains(&cond_flow)
      || self.frontiers[&cond_flow].contains(&cond_jmp)
    {
      return Ok(None);
    }

    let mut intersection = self.frontiers[&cond_jmp]
      .intersection(&self.frontiers[&cond_flow])
      .filter(|after| self.is_valid_after_node(node, **after));

    let (after, None) = (intersection.next(), intersection.next()) else {
      panic!("this should not be possible");
    };

    Ok(Some(ReducedNode::IfElse {
      node,
      then: cond_jmp,
      els: cond_flow,
      after: after.copied()
    }))
  }

  fn is_valid_after_node(&self, for_node: NodeIndex, candidate: NodeIndex) -> bool {
    !self.frontiers[&for_node].contains(&candidate)
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
