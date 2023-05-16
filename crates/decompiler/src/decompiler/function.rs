use petgraph::graph::NodeIndex;
use std::{cell::RefCell, collections::HashMap, println, rc::Rc};

use crate::{
  decompiler::{
    decompiled::Statement,
    stack_entry::{BinaryOperator, UnaryOperator}
  },
  disassembler::{Instruction, InstructionInfo},
  formatters::AssemblyFormatter,
  script::Script
};

use super::{
  decompiled::{DecompiledFunction, StatementInfo},
  function_graph::FunctionGraph,
  stack::{InvalidStackError, Stack},
  Confidence, ControlFlow, LinkedValueType, Primitives, ScriptGlobals, ScriptStatics, StackEntry,
  StackEntryInfo, ValueType, ValueTypeInfo
};

pub struct FunctionInfo<'input, 'bytes> {
  pub name:         String,
  pub location:     usize,
  pub parameters:   u32,
  pub returns:      u32,
  pub locals:       u32,
  pub instructions: &'input [InstructionInfo<'bytes>]
}

#[derive(Clone, Debug)]
pub struct Function<'input, 'bytes> {
  pub name:         String,
  pub location:     usize,
  pub parameters:   Vec<Rc<RefCell<LinkedValueType>>>,
  pub locals:       Vec<Rc<RefCell<LinkedValueType>>>,
  pub returns:      Option<Rc<RefCell<LinkedValueType>>>,
  pub instructions: &'input [InstructionInfo<'bytes>],
  pub graph:        FunctionGraph<'input, 'bytes>
}

impl<'input: 'bytes, 'bytes> Function<'input, 'bytes> {
  pub fn new(info: FunctionInfo<'input, 'bytes>) -> Self {
    let graph = FunctionGraph::generate(&info);
    Self {
      name: info.name,
      location: info.location,
      parameters: (0..info.parameters)
        .map(|_| {
          LinkedValueType::Type(ValueTypeInfo {
            ty:         ValueType::Primitive(Primitives::Unknown),
            confidence: Confidence::None
          })
          .make_shared()
        })
        .collect(),
      locals: (0..info.locals)
        .map(|_| {
          LinkedValueType::Type(ValueTypeInfo {
            ty:         ValueType::Primitive(Primitives::Unknown),
            confidence: Confidence::None
          })
          .make_shared()
        })
        .collect(),
      returns: match info.returns {
        0 => None,
        1 => {
          Some(
            LinkedValueType::Type(ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Unknown),
              confidence: Confidence::None
            })
            .make_shared()
          )
        }
        _ => {
          Some(
            LinkedValueType::Type(ValueTypeInfo {
              ty:         ValueType::Struct {
                fields: (0..info.returns)
                  .map(|_| {
                    LinkedValueType::Type(ValueTypeInfo {
                      ty:         ValueType::Primitive(Primitives::Unknown),
                      confidence: Confidence::None
                    })
                    .make_shared()
                  })
                  .collect()
              },
              confidence: Confidence::None
            })
            .make_shared()
          )
        }
      },
      instructions: info.instructions,
      graph
    }
  }

  // Temporary
  pub fn dot_string(&self, formatter: AssemblyFormatter) -> String {
    self.graph.to_dot_string(formatter)
  }

  pub fn decompile(
    &self,
    script: &Script,
    functions: &HashMap<usize, Function>,
    statics: &ScriptStatics,
    globals: &mut ScriptGlobals
  ) -> Result<DecompiledFunction<'input, 'bytes>, InvalidStackError> {
    let flow = self.graph.reconstruct_control_flow();

    let statements = self.decompile_iteratively(&flow, script, functions, statics, globals)?;

    self.add_statement_types(&statements);

    Ok(DecompiledFunction {
      name: self.name.clone(),
      params: self.parameters.clone(),
      returns: self.returns.clone(),
      locals: self.locals.clone(),
      statements
    })
  }

  pub fn decompile_iteratively(
    &self,
    root: &ControlFlow,
    script: &Script,
    functions: &HashMap<usize, Function>,
    statics: &ScriptStatics,
    globals: &mut ScriptGlobals
  ) -> Result<Vec<StatementInfo<'input, 'bytes>>, InvalidStackError> {
    let mut statements: HashMap<
      NodeIndex,
      (
        Vec<StatementInfo>,
        Option<StackEntryInfo>,
        &[InstructionInfo]
      )
    > = Default::default();
    let mut stack = Stack::default();

    root.dfs_in_order(|flow| {
      let (node_statements, conditional, _) = statements.entry(flow.node()).or_insert_with(|| {
        (
          Default::default(),
          Default::default(),
          &self.instructions[0..0]
        )
      });
      *conditional = self.decompile_node(
        node_statements,
        &mut stack,
        script,
        functions,
        flow,
        statics,
        globals
      )?;
      Ok(())
    })?;

    root.dfs_post_order::<InvalidStackError>(|flow| {
      Self::combine_control_flow(flow, &mut statements);
      Ok(())
    })?;

    Ok(statements.remove(&root.node()).expect("no root").0)
  }

  fn combine_control_flow<'i, 'b>(
    flow: &ControlFlow,
    statements: &mut HashMap<
      NodeIndex,
      (
        Vec<StatementInfo<'i, 'b>>,
        Option<StackEntryInfo>,
        &'i [InstructionInfo<'b>]
      )
    >
  ) {
    match flow {
      ControlFlow::If { then, .. } => {
        let then = statements
          .remove(&then.node())
          .expect("flow statement already consumed")
          .0;

        let (node_statements, conditional, trailing_instructions) = statements
          .get_mut(&flow.node())
          .expect("flow not visited in order");

        node_statements.push(StatementInfo {
          instructions: trailing_instructions,
          statement:    Statement::If {
            condition: conditional.take().unwrap(),
            then
          }
        });
      }
      ControlFlow::IfElse { then, els, .. } => {
        let then = statements
          .remove(&then.node())
          .expect("flow statement already consumed")
          .0;
        let els = statements
          .remove(&els.node())
          .expect("flow statement already consumed")
          .0;

        let (node_statements, conditional, trailing_instructions) = statements
          .get_mut(&flow.node())
          .expect("flow not visited in order");

        node_statements.push(StatementInfo {
          instructions: trailing_instructions,
          statement:    Statement::IfElse {
            condition: conditional.take().unwrap(),
            then,
            els
          }
        });
      }
      ControlFlow::WhileLoop { body, .. } => {
        let body = statements
          .remove(&body.node())
          .expect("flow statement already consumed")
          .0;

        let (node_statements, conditional, trailing_instructions) = statements
          .get_mut(&flow.node())
          .expect("flow not visited in order");

        node_statements.push(StatementInfo {
          instructions: trailing_instructions,
          statement:    Statement::WhileLoop {
            condition: conditional.take().unwrap(),
            body
          }
        });
      }
      ControlFlow::Switch { cases, .. } => {
        let cases = cases
          .iter()
          .map(|(case, values)| {
            (
              statements
                .remove(&case.node())
                .expect("flow statement already consumed")
                .0,
              values.clone()
            )
          })
          .collect();

        let (node_statements, conditional, trailing_instructions) = statements
          .get_mut(&flow.node())
          .expect("flow not visited in order");

        node_statements.push(StatementInfo {
          instructions: trailing_instructions,
          statement:    Statement::Switch {
            condition: conditional.take().unwrap(),
            cases
          }
        });
      }
      ControlFlow::AndOr { with, .. } => {
        let with = statements
          .remove(&with.node())
          .expect("flow statement already consumed");

        // TODO: Trailing instructions
        let (node_statements, conditional, _) = statements
          .get_mut(&flow.node())
          .expect("flow not visited in order");

        node_statements.extend(with.0);
        *conditional = with.1;
      }
      ControlFlow::Flow { .. }
      | ControlFlow::Break { .. }
      | ControlFlow::Continue { .. }
      | ControlFlow::Leaf { .. } => {}
    }

    if let Some(after) = flow.after() {
      let after = statements
        .remove(&after.node())
        .expect("flow statement already consumed");

      // TODO: Trailing instructions
      let (node_statements, conditional, _) = statements
        .get_mut(&flow.node())
        .expect("flow not visited in order");

      node_statements.extend(after.0);
      *conditional = after.1;
    }
  }

  fn decompile_node(
    &self,
    statements: &mut Vec<StatementInfo<'input, 'bytes>>,
    stack: &mut Stack,
    script: &Script,
    functions: &HashMap<usize, Function>,
    flow: &ControlFlow,
    statics: &ScriptStatics,
    globals: &mut ScriptGlobals
  ) -> Result<Option<StackEntryInfo>, InvalidStackError> {
    let node = self.graph.get_node(flow.node()).unwrap();

    for (index, info) in node.instructions.iter().enumerate() {
      match &info.instruction {
        Instruction::Nop => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Nop
          })
        }
        Instruction::IntegerAdd => {
          stack.push_binary_operator(
            Primitives::Int,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            BinaryOperator::Add
          )?
        }
        Instruction::IntegerSubtract => {
          stack.push_binary_operator(
            Primitives::Int,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            BinaryOperator::Subtract
          )?
        }
        Instruction::IntegerMultiply => {
          stack.push_binary_operator(
            Primitives::Int,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            BinaryOperator::Multiply
          )?
        }
        Instruction::IntegerDivide => {
          stack.push_binary_operator(
            Primitives::Int,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            BinaryOperator::Divide
          )?
        }
        Instruction::IntegerModulo => {
          stack.push_binary_operator(
            Primitives::Int,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            BinaryOperator::Modulo
          )?
        }
        Instruction::IntegerNot => {
          stack.push_unary_operator(
            Primitives::Bool,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            UnaryOperator::Not
          )?
        }
        Instruction::IntegerNegate => {
          stack.push_unary_operator(
            Primitives::Int,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            UnaryOperator::Negate
          )?
        }
        Instruction::IntegerEquals => {
          stack.push_binary_operator(
            Primitives::Bool,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            BinaryOperator::Equal
          )?
        }
        Instruction::IntegerNotEquals => {
          stack.push_binary_operator(
            Primitives::Bool,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            BinaryOperator::NotEqual
          )?
        }
        Instruction::IntegerGreaterThan => {
          stack.push_binary_operator(
            Primitives::Bool,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            BinaryOperator::GreaterThan
          )?
        }
        Instruction::IntegerGreaterOrEqual => {
          stack.push_binary_operator(
            Primitives::Bool,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            BinaryOperator::GreaterOrEqual
          )?
        }
        Instruction::IntegerLowerThan => {
          stack.push_binary_operator(
            Primitives::Bool,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            BinaryOperator::LowerThan
          )?
        }
        Instruction::IntegerLowerOrEqual => {
          stack.push_binary_operator(
            Primitives::Bool,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            BinaryOperator::LowerOrEqual
          )?
        }
        Instruction::FloatAdd => {
          stack.push_binary_operator(
            Primitives::Float,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            BinaryOperator::Add
          )?
        }
        Instruction::FloatSubtract => {
          stack.push_binary_operator(
            Primitives::Float,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            BinaryOperator::Subtract
          )?
        }
        Instruction::FloatMultiply => {
          stack.push_binary_operator(
            Primitives::Float,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            BinaryOperator::Multiply
          )?
        }
        Instruction::FloatDivide => {
          stack.push_binary_operator(
            Primitives::Float,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            BinaryOperator::Divide
          )?
        }
        Instruction::FloatModule => {
          stack.push_binary_operator(
            Primitives::Float,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            BinaryOperator::Modulo
          )?
        }
        Instruction::FloatNegate => {
          stack.push_unary_operator(
            Primitives::Bool,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            UnaryOperator::Negate
          )?
        }
        Instruction::FloatEquals => {
          stack.push_binary_operator(
            Primitives::Bool,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            BinaryOperator::Equal
          )?
        }
        Instruction::FloatNotEquals => {
          stack.push_binary_operator(
            Primitives::Bool,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            BinaryOperator::NotEqual
          )?
        }
        Instruction::FloatGreaterThan => {
          stack.push_binary_operator(
            Primitives::Bool,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            BinaryOperator::GreaterThan
          )?
        }
        Instruction::FloatGreaterOrEqual => {
          stack.push_binary_operator(
            Primitives::Bool,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            BinaryOperator::GreaterOrEqual
          )?
        }
        Instruction::FloatLowerThan => {
          stack.push_binary_operator(
            Primitives::Bool,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            BinaryOperator::LowerThan
          )?
        }
        Instruction::FloatLowerOrEqual => {
          stack.push_binary_operator(
            Primitives::Bool,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            },
            BinaryOperator::LowerOrEqual
          )?
        }
        Instruction::VectorAdd => stack.push_vector_binary_operator(BinaryOperator::Add)?,
        Instruction::VectorSubtract => {
          stack.push_vector_binary_operator(BinaryOperator::Subtract)?
        }
        Instruction::VectorMultiply => {
          stack.push_vector_binary_operator(BinaryOperator::Multiply)?
        }
        Instruction::VectorDivide => stack.push_vector_binary_operator(BinaryOperator::Divide)?,
        Instruction::VectorNegate => todo!(), // TODO
        Instruction::BitwiseAnd => {
          stack.push_binary_operator(
            Primitives::Int,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            BinaryOperator::BitwiseAnd
          )?
        }
        Instruction::BitwiseOr => {
          stack.push_binary_operator(
            Primitives::Int,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            BinaryOperator::BitwiseOr
          )?
        }
        Instruction::BitwiseXor => {
          stack.push_binary_operator(
            Primitives::Int,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            BinaryOperator::BitwiseXor
          )?
        }
        Instruction::IntegerToFloat => {
          stack.push_cast(
            Primitives::Float,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            }
          )?
        }
        Instruction::FloatToInteger => {
          stack.push_cast(
            Primitives::Int,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Float),
              confidence: Confidence::High
            }
          )?
        }
        Instruction::FloatToVector => todo!(), // TODO
        Instruction::PushConstU8 { c1 } => stack.push_int(*c1 as i64),
        Instruction::PushConstU8U8 { c1, c2 } => {
          stack.push_int(*c1 as i64);
          stack.push_int(*c2 as i64)
        }
        Instruction::PushConstU8U8U8 { c1, c2, c3 } => {
          stack.push_int(*c1 as i64);
          stack.push_int(*c2 as i64);
          stack.push_int(*c3 as i64)
        }
        Instruction::PushConstU32 { c1 } => stack.push_int(*c1 as i64),
        Instruction::PushConstFloat { c1 } => stack.push_float(*c1),
        Instruction::Dup => stack.push_dup()?,
        Instruction::Drop => {
          stack.pop()?;
        }
        Instruction::NativeCall {
          arg_count,
          return_count,
          native_index
        } => {
          if *return_count == 0 {
            statements.push(StatementInfo {
              instructions: &self.instructions[index..=index],
              statement:    Statement::NativeCall {
                args:        {
                  let mut args = stack.pop_n(*arg_count as usize)?;
                  args.reverse();
                  args
                },
                native_hash: script.natives[*native_index as usize]
              }
            })
          } else {
            stack.push_native_call(
              *arg_count as usize,
              *return_count as usize,
              script.natives[*native_index as usize]
            )?
          }
        }
        Instruction::Enter { .. } => { /* SKIP */ }
        Instruction::Leave { return_count, .. } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Return {
              values: stack.pop_n(*return_count as usize)?
            }
          })
        }
        Instruction::Load => stack.push_deref()?,
        Instruction::Store => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              source:      stack.pop()?,
              destination: stack.pop()?
            }
          })
        }
        Instruction::StoreRev => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              source:      stack.nth_back(0)?,
              destination: {
                let dest = stack.nth_back(1)?;
                let ty = dest.ty.borrow_mut().ref_type();
                StackEntryInfo {
                  entry: StackEntry::Deref(Box::new(dest)),
                  ty
                }
              }
            }
          })
        }
        Instruction::LoadN => stack.push_load_n()?,
        Instruction::StoreN => {
          stack.push_deref()?;
          let dest = stack.pop()?;
          let n = stack.pop()?;
          let StackEntry::Int(n) = n.entry else {
            Err(InvalidStackError)?
          };

          let mut popped = stack.pop_n(n as usize)?;
          let value = if popped.len() > 1 {
            StackEntryInfo {
              ty:    LinkedValueType::Type(ValueTypeInfo {
                ty:         ValueType::Struct {
                  fields: popped.iter().map(|v| v.ty.clone()).collect()
                },
                confidence: Confidence::High
              })
              .make_shared(),
              entry: StackEntry::ResultStruct { values: popped }
            }
          } else {
            popped.swap_remove(0)
          };

          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              source:      value,
              destination: dest
            }
          })
        }
        Instruction::ArrayU8 { item_size } => {
          stack.push_array_item(*item_size as usize)?;
          stack.push_reference()?
        }
        Instruction::ArrayU8Load { item_size } => {
          stack.push_array_item(*item_size as usize)?;
        }
        Instruction::ArrayU8Store { item_size } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_array_item(*item_size as usize)?;
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::LocalU8 { offset } => {
          stack.push_local(*offset as usize, self);
          stack.push_reference()?
        }
        Instruction::LocalU8Load { offset } => stack.push_local(*offset as usize, self),
        Instruction::LocalU8Store { offset } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_local(*offset as usize, self);
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::StaticU8 { static_index } => {
          stack.push_static(*static_index as usize, statics);
          stack.push_reference()?;
        }
        Instruction::StaticU8Load { static_index } => {
          stack.push_static(*static_index as usize, statics)
        }
        Instruction::StaticU8Store { static_index } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_static(*static_index as usize, statics);
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::AddU8 { value } => {
          stack.push_const_int_binary_operator(BinaryOperator::Add, *value as i64)?
        }
        Instruction::MultiplyU8 { value } => {
          stack.push_const_int_binary_operator(BinaryOperator::Multiply, *value as i64)?
        }
        Instruction::Offset => {
          stack.push_offset()?;
          stack.push_reference()?
        }
        Instruction::OffsetU8 { offset } => {
          stack.push_const_offset(*offset as i64)?;
          stack.push_reference()?
        }
        Instruction::OffsetU8Load { offset } => stack.push_const_offset(*offset as i64)?,
        Instruction::OffsetU8Store { offset } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_const_offset(*offset as i64)?;
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::PushConstS16 { c1 } => stack.push_int(*c1 as i64),
        Instruction::AddS16 { value } => {
          stack.push_const_int_binary_operator(BinaryOperator::Add, *value as i64)?
        }
        Instruction::MultiplyS16 { value } => {
          stack.push_const_int_binary_operator(BinaryOperator::Multiply, *value as i64)?
        }
        Instruction::OffsetS16 { offset } => {
          stack.push_const_offset(*offset as i64)?;
          stack.push_reference()?
        }
        Instruction::OffsetS16Load { offset } => {
          stack.push_const_offset(*offset as i64)?;
        }
        Instruction::OffsetS16Store { offset } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_const_offset(*offset as i64)?;
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::ArrayU16 { item_size } => {
          stack.push_array_item(*item_size as usize)?;
          stack.push_reference()?
        }
        Instruction::ArrayU16Load { item_size } => {
          stack.push_array_item(*item_size as usize)?;
        }
        Instruction::ArrayU16Store { item_size } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_array_item(*item_size as usize)?;
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::LocalU16 { local_index } => {
          stack.push_local(*local_index as usize, self);
          stack.push_reference()?
        }
        Instruction::LocalU16Load { local_index } => stack.push_local(*local_index as usize, self),
        Instruction::LocalU16Store { local_index } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_local(*local_index as usize, self);
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::StaticU16 { static_index } => {
          stack.push_static(*static_index as usize, statics);
          stack.push_reference()?
        }
        Instruction::StaticU16Load { static_index } => {
          stack.push_static(*static_index as usize, statics)
        }
        Instruction::StaticU16Store { static_index } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_static(*static_index as usize, statics);
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::GlobalU16 { global_index } => {
          stack.push_global(*global_index as usize, globals);
          stack.push_reference()?
        }
        Instruction::GlobalU16Load { global_index } => {
          stack.push_global(*global_index as usize, globals)
        }
        Instruction::GlobalU16Store { global_index } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_global(*global_index as usize, globals);
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::Jump { .. }
        | Instruction::JumpZero { .. }
        | Instruction::IfEqualJumpZero { .. }
        | Instruction::IfNotEqualJumpZero { .. }
        | Instruction::IfGreaterThanJumpZero { .. }
        | Instruction::IfGreaterOrEqualJumpZero { .. }
        | Instruction::IfLowerThanJumpZero { .. }
        | Instruction::IfLowerOrEqualJumpZero { .. }
        | Instruction::Switch { .. } => {
          match &info.instruction {
            Instruction::IfEqualJumpZero { .. } => {
              stack.push_binary_operator(
                Primitives::Bool,
                ValueTypeInfo {
                  ty:         ValueType::Primitive(Primitives::Int),
                  confidence: Confidence::Medium
                },
                ValueTypeInfo {
                  ty:         ValueType::Primitive(Primitives::Int),
                  confidence: Confidence::Medium
                },
                BinaryOperator::Equal
              )?
            }
            Instruction::IfNotEqualJumpZero { .. } => {
              stack.push_binary_operator(
                Primitives::Bool,
                ValueTypeInfo {
                  ty:         ValueType::Primitive(Primitives::Int),
                  confidence: Confidence::Medium
                },
                ValueTypeInfo {
                  ty:         ValueType::Primitive(Primitives::Int),
                  confidence: Confidence::Medium
                },
                BinaryOperator::NotEqual
              )?
            }
            Instruction::IfGreaterThanJumpZero { .. } => {
              stack.push_binary_operator(
                Primitives::Bool,
                ValueTypeInfo {
                  ty:         ValueType::Primitive(Primitives::Int),
                  confidence: Confidence::Medium
                },
                ValueTypeInfo {
                  ty:         ValueType::Primitive(Primitives::Int),
                  confidence: Confidence::Medium
                },
                BinaryOperator::GreaterThan
              )?
            }
            Instruction::IfGreaterOrEqualJumpZero { .. } => {
              stack.push_binary_operator(
                Primitives::Bool,
                ValueTypeInfo {
                  ty:         ValueType::Primitive(Primitives::Int),
                  confidence: Confidence::Medium
                },
                ValueTypeInfo {
                  ty:         ValueType::Primitive(Primitives::Int),
                  confidence: Confidence::Medium
                },
                BinaryOperator::GreaterOrEqual
              )?
            }
            Instruction::IfLowerThanJumpZero { .. } => {
              stack.push_binary_operator(
                Primitives::Bool,
                ValueTypeInfo {
                  ty:         ValueType::Primitive(Primitives::Int),
                  confidence: Confidence::Medium
                },
                ValueTypeInfo {
                  ty:         ValueType::Primitive(Primitives::Int),
                  confidence: Confidence::Medium
                },
                BinaryOperator::LowerThan
              )?
            }
            Instruction::IfLowerOrEqualJumpZero { .. } => {
              stack.push_binary_operator(
                Primitives::Bool,
                ValueTypeInfo {
                  ty:         ValueType::Primitive(Primitives::Int),
                  confidence: Confidence::Medium
                },
                ValueTypeInfo {
                  ty:         ValueType::Primitive(Primitives::Int),
                  confidence: Confidence::Medium
                },
                BinaryOperator::LowerOrEqual
              )?
            }
            _ => {}
          }

          match &flow {
            ControlFlow::If { .. }
            | ControlFlow::IfElse { .. }
            | ControlFlow::WhileLoop { .. }
            | ControlFlow::Switch { .. } => {
              return Ok(Some(stack.pop()?));
            }
            ControlFlow::AndOr { .. } => {
              stack.pop()?;
              return Ok(None);
            }
            ControlFlow::Break { .. } => {
              statements.push(StatementInfo {
                instructions: &self.instructions[index..=index],
                statement:    Statement::Break
              })
            }
            ControlFlow::Continue { .. } => {
              statements.push(StatementInfo {
                instructions: &self.instructions[index..=index],
                statement:    Statement::Continue
              })
            }
            ControlFlow::Leaf { .. } | ControlFlow::Flow { .. } => {}
          };
        }
        Instruction::FunctionCall { location } => {
          let location = *location as usize;
          let target = functions.get(&location).expect("TODO HANDLE THIS");
          if target.returns.is_some() {
            stack.push_function_call(target)?
          } else {
            statements.push(StatementInfo {
              instructions: &self.instructions[index..=index],
              statement:    Statement::FunctionCall {
                args:             {
                  let mut args = stack.pop_n(target.parameters.len())?;
                  args.reverse();
                  args
                },
                function_address: target.location
              }
            })
          }
        }
        Instruction::StaticU24 { static_index } => {
          stack.push_static(*static_index as usize, statics);
          stack.push_reference()?
        }
        Instruction::StaticU24Load { static_index } => {
          stack.push_static(*static_index as usize, statics)
        }
        Instruction::StaticU24Store { static_index } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_static(*static_index as usize, statics);
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::GlobalU24 { global_index } => {
          stack.push_global(*global_index as usize, globals);
          stack.push_reference()?;
        }
        Instruction::GlobalU24Load { global_index } => {
          stack.push_global(*global_index as usize, globals)
        }
        Instruction::GlobalU24Store { global_index } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_global(*global_index as usize, globals);
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::PushConstU24 { c1 } => stack.push_int(*c1 as i64),
        Instruction::String => stack.push_string()?,
        Instruction::StringHash => stack.push_string_hash()?,
        Instruction::TextLabelAssignString { buffer_size } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::StringCopy {
              destination: stack.pop()?,
              string:      stack.pop()?,
              max_length:  *buffer_size as usize
            }
          })
        }
        Instruction::TextLabelAssignInt { buffer_size } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::IntToString {
              destination: stack.pop()?,
              int:         stack.pop()?,
              max_length:  *buffer_size as usize
            }
          })
        }
        Instruction::TextLabelAppendString { buffer_size } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::StringConcat {
              destination: stack.pop()?,
              string:      stack.pop()?,
              max_length:  *buffer_size as usize
            }
          })
        }
        Instruction::TextLabelAppendInt { buffer_size } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::StringIntConcat {
              destination: stack.pop()?,
              int:         stack.pop()?,
              max_length:  *buffer_size as usize
            }
          })
        }
        Instruction::TextLabelCopy => {
          let destination = stack.pop()?;
          let buffer_size = stack.pop()?;
          let StackEntry::Int(count) = stack.pop()?.entry else {
            Err(InvalidStackError)?
          };
          let source = stack.pop_n(count as usize)?;

          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::MemCopy {
              destination,
              source,
              buffer_size,
              count: count as usize
            }
          })
        }
        Instruction::Catch => stack.push_catch(),
        Instruction::Throw => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Throw {
              value: stack.pop()?
            }
          })
        }
        Instruction::CallIndirect => todo!(),
        Instruction::PushConstM1 => stack.push_int(-1),
        Instruction::PushConst0 => stack.push_int(0),
        Instruction::PushConst1 => stack.push_int(1),
        Instruction::PushConst2 => stack.push_int(2),
        Instruction::PushConst3 => stack.push_int(3),
        Instruction::PushConst4 => stack.push_int(4),
        Instruction::PushConst5 => stack.push_int(5),
        Instruction::PushConst6 => stack.push_int(6),
        Instruction::PushConst7 => stack.push_int(7),
        Instruction::PushConstFm1 => stack.push_float(-1f32),
        Instruction::PushConstF0 => stack.push_float(0f32),
        Instruction::PushConstF1 => stack.push_float(1f32),
        Instruction::PushConstF2 => stack.push_float(2f32),
        Instruction::PushConstF3 => stack.push_float(3f32),
        Instruction::PushConstF4 => stack.push_float(4f32),
        Instruction::PushConstF5 => stack.push_float(5f32),
        Instruction::PushConstF6 => stack.push_float(6f32),
        Instruction::PushConstF7 => stack.push_float(7f32),
        Instruction::BitTest => {
          stack.push_binary_operator(
            Primitives::Bool,
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            ValueTypeInfo {
              ty:         ValueType::Primitive(Primitives::Int),
              confidence: Confidence::Medium
            },
            BinaryOperator::BitTest
          )?
        }
      };
    }

    Ok(None)
  }

  pub fn local_index_type(&self, index: usize) -> Option<&Rc<RefCell<LinkedValueType>>> {
    if index < self.parameters.len() {
      Some(&self.parameters[index])
    } else if index < self.parameters.len() + 2 {
      None
    } else if index < self.parameters.len() + 2 + self.locals.len() {
      Some(&self.locals[index - self.parameters.len() - 2])
    } else {
      None
    }
  }

  fn add_statement_types(&self, statements: &[StatementInfo]) {
    for info in statements {
      match &info.statement {
        Statement::Nop => {}
        Statement::Assign {
          destination,
          source
        } => {
          LinkedValueType::link(&destination.ty, &source.ty);
        }
        Statement::Return { values } => {
          match &values[..] {
            [value] => {
              LinkedValueType::link(self.returns.as_ref().unwrap(), &value.ty);
            }
            [] => {}
            values => {
              self
                .returns
                .as_ref()
                .unwrap()
                .borrow_mut()
                .hint(ValueTypeInfo {
                  ty:         ValueType::Struct {
                    fields: values.iter().map(|v| v.ty.clone()).collect()
                  },
                  confidence: Confidence::High
                })
            }
          }
        }
        Statement::Throw { .. } => {}
        Statement::FunctionCall { .. } => {}
        Statement::NativeCall { .. } => {}
        Statement::If { condition, then } => {
          condition.ty.borrow_mut().hint(ValueTypeInfo {
            ty:         ValueType::Primitive(Primitives::Bool),
            confidence: Confidence::Medium
          });
          self.add_statement_types(then);
        }
        Statement::IfElse {
          condition,
          then,
          els
        } => {
          condition.ty.borrow_mut().hint(ValueTypeInfo {
            ty:         ValueType::Primitive(Primitives::Bool),
            confidence: Confidence::Medium
          });
          self.add_statement_types(then);
          self.add_statement_types(els);
        }
        Statement::WhileLoop { condition, body } => {
          condition.ty.borrow_mut().hint(ValueTypeInfo {
            ty:         ValueType::Primitive(Primitives::Bool),
            confidence: Confidence::Medium
          });
          self.add_statement_types(body);
        }
        Statement::Switch { condition, cases } => {
          condition.ty.borrow_mut().hint(ValueTypeInfo {
            ty:         ValueType::Primitive(Primitives::Int),
            confidence: Confidence::Medium
          });
          for (body, _) in cases {
            self.add_statement_types(body);
          }
        }
        Statement::Break => {}
        Statement::Continue => {}
        Statement::StringCopy {
          destination,
          string,
          ..
        } => {
          destination.ty.borrow_mut().hint(ValueTypeInfo {
            ty:         ValueType::Ref(
              LinkedValueType::Type(ValueTypeInfo {
                ty:         ValueType::Primitive(Primitives::String),
                confidence: Confidence::High
              })
              .make_shared()
            ),
            confidence: Confidence::High
          });
          string.ty.borrow_mut().hint(ValueTypeInfo {
            ty:         ValueType::Primitive(Primitives::String),
            confidence: Confidence::High
          });
        }
        Statement::IntToString {
          destination, int, ..
        } => {
          destination.ty.borrow_mut().hint(ValueTypeInfo {
            ty:         ValueType::Ref(
              LinkedValueType::Type(ValueTypeInfo {
                ty:         ValueType::Primitive(Primitives::String),
                confidence: Confidence::High
              })
              .make_shared()
            ),
            confidence: Confidence::High
          });
          int.ty.borrow_mut().hint(ValueTypeInfo {
            ty:         ValueType::Primitive(Primitives::Int),
            confidence: Confidence::High
          });
        }
        Statement::StringConcat {
          destination,
          string,
          ..
        } => {
          destination.ty.borrow_mut().hint(ValueTypeInfo {
            ty:         ValueType::Ref(
              LinkedValueType::Type(ValueTypeInfo {
                ty:         ValueType::Primitive(Primitives::String),
                confidence: Confidence::High
              })
              .make_shared()
            ),
            confidence: Confidence::High
          });
          string.ty.borrow_mut().hint(ValueTypeInfo {
            ty:         ValueType::Primitive(Primitives::String),
            confidence: Confidence::High
          });
        }
        Statement::StringIntConcat {
          destination, int, ..
        } => {
          destination.ty.borrow_mut().hint(ValueTypeInfo {
            ty:         ValueType::Ref(
              LinkedValueType::Type(ValueTypeInfo {
                ty:         ValueType::Primitive(Primitives::String),
                confidence: Confidence::High
              })
              .make_shared()
            ),
            confidence: Confidence::High
          });
          int.ty.borrow_mut().hint(ValueTypeInfo {
            ty:         ValueType::Primitive(Primitives::Int),
            confidence: Confidence::High
          });
        }
        Statement::MemCopy { buffer_size, .. } => {
          buffer_size.ty.borrow_mut().hint(ValueTypeInfo {
            ty:         ValueType::Primitive(Primitives::Int),
            confidence: Confidence::High
          });
        }
      }
    }
  }
}
