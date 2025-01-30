use std::collections::{HashMap, HashSet};

use petgraph::graph::NodeIndex;

#[derive(Debug, Clone)]
pub enum ControlFlow {
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

impl ControlFlow {
  pub fn flow_type(&self) -> FlowType {
    match self {
      ControlFlow::If { node, after, .. } | ControlFlow::IfElse { node, after, .. } => {
        FlowType::NonBreakable {
          node:  *node,
          after: *after
        }
      }
      ControlFlow::AndOr { node, after, .. } => {
        FlowType::NonBreakable {
          node:  *node,
          after: Some(*after)
        }
      }
      ControlFlow::WhileLoop { node, after, .. } => {
        FlowType::Loop {
          node:  *node,
          after: *after
        }
      }
      ControlFlow::Switch { node, after, .. } => {
        FlowType::Switch {
          node:  *node,
          after: *after
        }
      }
      ControlFlow::Flow { node, .. }
      | ControlFlow::Break { node, .. }
      | ControlFlow::Continue { node, .. }
      | ControlFlow::Leaf { node } => {
        FlowType::NonBreakable {
          node:  *node,
          after: None
        }
      }
    }
  }

  pub fn node(&self) -> NodeIndex {
    match self {
      ControlFlow::If { node, .. }
      | ControlFlow::IfElse { node, .. }
      | ControlFlow::Leaf { node }
      | ControlFlow::AndOr { node, .. }
      | ControlFlow::WhileLoop { node, .. }
      | ControlFlow::Flow { node, .. }
      | ControlFlow::Break { node, .. }
      | ControlFlow::Continue { node, .. }
      | ControlFlow::Switch { node, .. } => *node
    }
  }

  pub fn after(&self) -> Option<NodeIndex> {
    match self {
      ControlFlow::If { after, .. }
      | ControlFlow::IfElse { after, .. }
      | ControlFlow::WhileLoop { after, .. }
      | ControlFlow::Switch { after, .. } => *after,
      ControlFlow::AndOr { after, .. } | ControlFlow::Flow { after, .. } => Some(*after),
      ControlFlow::Continue { .. } | ControlFlow::Break { .. } | ControlFlow::Leaf { .. } => None
    }
  }

  pub fn dfs_in_order<E>(
    &self,
    nodes: &HashMap<NodeIndex, ControlFlow>,
    mut cb: impl FnMut(&ControlFlow, Option<&ControlFlow>) -> Result<(), E>
  ) -> Result<(), E> {
    let mut stack: Vec<(NodeIndex, Option<NodeIndex>)> = vec![(self.node(), None)];

    while let Some((node, parent)) = stack.pop() {
      let flow = nodes.get(&node).unwrap();
      match flow {
        ControlFlow::If { then, after, .. } => {
          if let Some(after) = after {
            stack.push((*after, Some(node)));
          }
          stack.push((*then, Some(node)));
        }
        ControlFlow::IfElse {
          then, els, after, ..
        } => {
          if let Some(after) = after {
            stack.push((*after, Some(node)));
          }
          stack.push((*els, Some(node)));
          stack.push((*then, Some(node)));
        }
        ControlFlow::AndOr { with, after, .. } => {
          stack.push((*after, Some(node)));
          stack.push((*with, Some(node)));
        }
        ControlFlow::WhileLoop { body, after, .. } => {
          if let Some(after) = after {
            stack.push((*after, Some(node)));
          }
          stack.push((*body, Some(node)));
        }
        ControlFlow::Flow { after, .. } => {
          stack.push((*after, Some(node)));
        }
        ControlFlow::Switch { cases, after, .. } => {
          if let Some(after) = after {
            stack.push((*after, Some(node)));
          }
          stack.extend(cases.iter().map(|(cf, _)| (*cf, Some(node))));
        }
        ControlFlow::Leaf { .. } | ControlFlow::Break { .. } | ControlFlow::Continue { .. } => {}
      }

      let parent_flow = parent.map(|p| nodes.get(&p).unwrap());

      cb(flow, parent_flow)?;
    }

    Ok(())
  }

  pub fn dfs_post_order<E>(
    &self,
    nodes: &HashMap<NodeIndex, ControlFlow>,
    mut cb: impl FnMut(&ControlFlow) -> Result<(), E>
  ) -> Result<(), E> {
    let mut stack = vec![self.node()];
    let mut visited: HashSet<NodeIndex> = Default::default();

    while let Some(node) = stack.pop() {
      let flow = nodes.get(&node).unwrap();

      if visited.contains(&flow.node()) {
        cb(flow)?;
        continue;
      }

      visited.insert(flow.node());
      stack.push(node);
      match flow {
        ControlFlow::If { then, after, .. } => {
          if let Some(after) = after {
            stack.push(*after);
          }
          stack.push(*then);
        }
        ControlFlow::IfElse {
          then, els, after, ..
        } => {
          if let Some(after) = after {
            stack.push(*after);
          }
          stack.push(*els);
          stack.push(*then);
        }
        ControlFlow::AndOr { with, after, .. } => {
          stack.push(*after);
          stack.push(*with);
        }
        ControlFlow::WhileLoop { body, after, .. } => {
          if let Some(after) = after {
            stack.push(*after);
          }
          stack.push(*body);
        }
        ControlFlow::Flow { after, .. } => {
          stack.push(*after);
        }
        ControlFlow::Switch { cases, after, .. } => {
          if let Some(after) = after {
            stack.push(*after);
          }
          stack.extend(cases.iter().map(|(cf, _)| cf));
        }
        ControlFlow::Leaf { .. } | ControlFlow::Break { .. } | ControlFlow::Continue { .. } => {}
      }
    }

    Ok(())
  }
}

#[derive(Debug, Clone, Copy)]
pub enum CaseValue {
  Default,
  Value(i64)
}

#[derive(Debug, Clone, Copy)]
pub enum FlowType {
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
