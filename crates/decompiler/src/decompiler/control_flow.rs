use std::collections::HashSet;

use petgraph::graph::NodeIndex;

#[derive(Debug, Clone)]
pub enum ControlFlow {
  If {
    node:  NodeIndex,
    then:  Box<ControlFlow>,
    after: Option<Box<ControlFlow>>
  },
  IfElse {
    node:  NodeIndex,
    then:  Box<ControlFlow>,
    els:   Box<ControlFlow>,
    after: Option<Box<ControlFlow>>
  },
  Leaf {
    node: NodeIndex
  },
  AndOr {
    node:  NodeIndex,
    with:  Box<ControlFlow>,
    after: Box<ControlFlow>
  },
  WhileLoop {
    node:  NodeIndex,
    body:  Box<ControlFlow>,
    after: Option<Box<ControlFlow>>
  },
  Flow {
    node:  NodeIndex,
    after: Box<ControlFlow>
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
    cases: Vec<(ControlFlow, Vec<CaseValue>)>,
    after: Option<Box<ControlFlow>>
  }
}

impl ControlFlow {
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

  pub fn after(&self) -> Option<&ControlFlow> {
    match self {
      ControlFlow::If { after, .. }
      | ControlFlow::IfElse { after, .. }
      | ControlFlow::WhileLoop { after, .. }
      | ControlFlow::Switch { after, .. } => after.as_ref().map(|bx| bx.as_ref()),
      ControlFlow::AndOr { after, .. } | ControlFlow::Flow { after, .. } => Some(after),
      ControlFlow::Continue { .. } | ControlFlow::Break { .. } | ControlFlow::Leaf { .. } => None
    }
  }

  pub fn dfs_in_order<E>(
    &self,
    mut cb: impl FnMut(&ControlFlow) -> Result<(), E>
  ) -> Result<(), E> {
    let mut stack = vec![self];

    while let Some(flow) = stack.pop() {
      match flow {
        ControlFlow::If { then, after, .. } => {
          if let Some(after) = after {
            stack.push(after);
          }
          stack.push(then);
        }
        ControlFlow::IfElse {
          then, els, after, ..
        } => {
          if let Some(after) = after {
            stack.push(after);
          }
          stack.push(els);
          stack.push(then);
        }
        ControlFlow::AndOr { with, after, .. } => {
          stack.push(after);
          stack.push(with);
        }
        ControlFlow::WhileLoop { body, after, .. } => {
          if let Some(after) = after {
            stack.push(after);
          }
          stack.push(body);
        }
        ControlFlow::Flow { after, .. } => {
          stack.push(after);
        }
        ControlFlow::Switch { cases, after, .. } => {
          if let Some(after) = after {
            stack.push(after);
          }
          stack.extend(cases.iter().map(|(cf, _)| cf));
        }
        ControlFlow::Leaf { .. } | ControlFlow::Break { .. } | ControlFlow::Continue { .. } => {}
      }

      cb(flow)?;
    }

    Ok(())
  }

  pub fn dfs_post_order<E>(
    &self,
    mut cb: impl FnMut(&ControlFlow) -> Result<(), E>
  ) -> Result<(), E> {
    let mut stack = vec![self];
    let mut visited: HashSet<NodeIndex> = Default::default();

    while let Some(flow) = stack.pop() {
      if visited.contains(&flow.node()) {
        cb(flow)?;
        continue;
      }

      visited.insert(flow.node());
      stack.push(flow);
      match flow {
        ControlFlow::If { then, after, .. } => {
          if let Some(after) = after {
            stack.push(after);
          }
          stack.push(then);
        }
        ControlFlow::IfElse {
          then, els, after, ..
        } => {
          if let Some(after) = after {
            stack.push(after);
          }
          stack.push(els);
          stack.push(then);
        }
        ControlFlow::AndOr { with, after, .. } => {
          stack.push(after);
          stack.push(with);
        }
        ControlFlow::WhileLoop { body, after, .. } => {
          if let Some(after) = after {
            stack.push(after);
          }
          stack.push(body);
        }
        ControlFlow::Flow { after, .. } => {
          stack.push(after);
        }
        ControlFlow::Switch { cases, after, .. } => {
          if let Some(after) = after {
            stack.push(after);
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
