use std::collections::HashMap;

use crate::{
  decompiler::{
    decompiled::Statement,
    stack_entry::{BinaryOperator, Type, UnaryOperator}
  },
  disassembler::{Instruction, InstructionInfo},
  formatters::AssemblyFormatter,
  script::Script
};

use super::{
  decompiled::{DecompiledFunction, StatementInfo},
  function_graph::{ControlFlow, FunctionGraph},
  stack::{InvalidStackError, Stack}
};

pub struct FunctionInfo<'input, 'bytes> {
  pub name:         String,
  pub location:     usize,
  pub parameters:   u32,
  pub return_count: u32,
  pub instructions: &'input [InstructionInfo<'bytes>]
}

#[derive(Clone, Debug)]
pub struct Function<'input, 'bytes> {
  pub name:         String,
  pub location:     usize,
  pub parameters:   u32,
  pub return_count: u32,
  pub instructions: &'input [InstructionInfo<'bytes>],
  pub graph:        FunctionGraph<'input, 'bytes>
}

impl<'input: 'bytes, 'bytes> Function<'input, 'bytes> {
  pub fn new(info: FunctionInfo<'input, 'bytes>) -> Self {
    let graph = FunctionGraph::generate(&info);
    Self {
      name:         info.name,
      location:     info.location,
      parameters:   info.parameters,
      return_count: info.return_count,
      instructions: info.instructions,
      graph:        graph
    }
  }

  // Temporary
  pub fn dot_string(&self, formatter: AssemblyFormatter) -> String {
    self.graph.to_dot_string(formatter)
  }

  pub fn decompile(
    &self,
    script: &Script,
    functions: &HashMap<usize, Function>
  ) -> Result<DecompiledFunction<'input, 'bytes>, InvalidStackError> {
    let flow = self.graph.reconstruct_control_flow();

    let statements = self.decompile_node(script, functions, &flow, Default::default())?;

    Ok(DecompiledFunction {
      name: self.name.clone(),
      params: self.parameters as usize,
      statements
    })
  }

  fn decompile_node(
    &self,
    script: &Script,
    functions: &HashMap<usize, Function>,
    flow: &ControlFlow,
    mut stack: Stack
  ) -> Result<Vec<StatementInfo<'input, 'bytes>>, InvalidStackError> {
    let mut statements: Vec<StatementInfo> = Default::default();
    let node = self.graph.get_node(flow.node()).unwrap();

    for (index, info) in node.instructions.iter().enumerate() {
      match &info.instruction {
        Instruction::Nop => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Nop
          })
        }
        Instruction::IntegerAdd => stack.push_binary_operator(Type::Int, BinaryOperator::Add)?,
        Instruction::IntegerSubtract => {
          stack.push_binary_operator(Type::Int, BinaryOperator::Subtract)?
        }
        Instruction::IntegerMultiply => {
          stack.push_binary_operator(Type::Int, BinaryOperator::Multiply)?
        }
        Instruction::IntegerDivide => {
          stack.push_binary_operator(Type::Int, BinaryOperator::Divide)?
        }
        Instruction::IntegerModulo => {
          stack.push_binary_operator(Type::Int, BinaryOperator::Modulo)?
        }
        Instruction::IntegerNot => stack.push_unary_operator(Type::Bool, UnaryOperator::Not)?,
        Instruction::IntegerNegate => {
          stack.push_unary_operator(Type::Int, UnaryOperator::Negate)?
        }
        Instruction::IntegerEquals => {
          stack.push_binary_operator(Type::Bool, BinaryOperator::Equal)?
        }
        Instruction::IntegerNotEquals => {
          stack.push_binary_operator(Type::Bool, BinaryOperator::NotEqual)?
        }
        Instruction::IntegerGreaterThan => {
          stack.push_binary_operator(Type::Bool, BinaryOperator::GreaterThan)?
        }
        Instruction::IntegerGreaterOrEqual => {
          stack.push_binary_operator(Type::Bool, BinaryOperator::GreaterOrEqual)?
        }
        Instruction::IntegerLowerThan => {
          stack.push_binary_operator(Type::Bool, BinaryOperator::LowerThan)?
        }
        Instruction::IntegerLowerOrEqual => {
          stack.push_binary_operator(Type::Bool, BinaryOperator::LowerOrEqual)?
        }
        Instruction::FloatAdd => stack.push_binary_operator(Type::Float, BinaryOperator::Add)?,
        Instruction::FloatSubtract => {
          stack.push_binary_operator(Type::Float, BinaryOperator::Subtract)?
        }
        Instruction::FloatMultiply => {
          stack.push_binary_operator(Type::Float, BinaryOperator::Multiply)?
        }
        Instruction::FloatDivide => {
          stack.push_binary_operator(Type::Float, BinaryOperator::Divide)?
        }
        Instruction::FloatModule => {
          stack.push_binary_operator(Type::Float, BinaryOperator::Modulo)?
        }
        Instruction::FloatNegate => stack.push_unary_operator(Type::Bool, UnaryOperator::Negate)?,
        Instruction::FloatEquals => {
          stack.push_binary_operator(Type::Bool, BinaryOperator::Equal)?
        }
        Instruction::FloatNotEquals => {
          stack.push_binary_operator(Type::Bool, BinaryOperator::NotEqual)?
        }
        Instruction::FloatGreaterThan => {
          stack.push_binary_operator(Type::Bool, BinaryOperator::GreaterThan)?
        }
        Instruction::FloatGreaterOrEqual => {
          stack.push_binary_operator(Type::Bool, BinaryOperator::GreaterOrEqual)?
        }
        Instruction::FloatLowerThan => {
          stack.push_binary_operator(Type::Bool, BinaryOperator::LowerThan)?
        }
        Instruction::FloatLowerOrEqual => {
          stack.push_binary_operator(Type::Bool, BinaryOperator::LowerOrEqual)?
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
          stack.push_binary_operator(Type::Int, BinaryOperator::BitwiseAnd)?
        }
        Instruction::BitwiseOr => {
          stack.push_binary_operator(Type::Int, BinaryOperator::BitwiseOr)?
        }
        Instruction::BitwiseXor => {
          stack.push_binary_operator(Type::Int, BinaryOperator::BitwiseXor)?
        }
        Instruction::IntegerToFloat => stack.push_cast(Type::Float)?,
        Instruction::FloatToInteger => stack.push_cast(Type::Int)?,
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
        Instruction::StoreRev => todo!(),
        Instruction::LoadN => stack.push_load_n()?,
        Instruction::StoreN => todo!(),
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
          stack.push_local(*offset as usize);
          stack.push_reference()?
        }
        Instruction::LocalU8Load { offset } => stack.push_local(*offset as usize),
        Instruction::LocalU8Store { offset } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_local(*offset as usize);
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::StaticU8 { static_index } => {
          stack.push_static(*static_index as usize);
          stack.push_reference()?;
        }
        Instruction::StaticU8Load { static_index } => stack.push_static(*static_index as usize),
        Instruction::StaticU8Store { static_index } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_static(*static_index as usize);
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
        Instruction::Offset => stack.push_offset()?,
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
          stack.push_local(*local_index as usize);
          stack.push_reference()?
        }
        Instruction::LocalU16Load { local_index } => stack.push_local(*local_index as usize),
        Instruction::LocalU16Store { local_index } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_local(*local_index as usize);
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::StaticU16 { static_index } => {
          stack.push_static(*static_index as usize);
          stack.push_reference()?
        }
        Instruction::StaticU16Load { static_index } => stack.push_static(*static_index as usize),
        Instruction::StaticU16Store { static_index } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_static(*static_index as usize);
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::GlobalU16 { global_index } => {
          stack.push_global(*global_index as usize);
          stack.push_reference()?
        }
        Instruction::GlobalU16Load { global_index } => stack.push_global(*global_index as usize),
        Instruction::GlobalU16Store { global_index } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_global(*global_index as usize);
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
              stack.push_binary_operator(Type::Bool, BinaryOperator::Equal)?
            }
            Instruction::IfNotEqualJumpZero { .. } => {
              stack.push_binary_operator(Type::Bool, BinaryOperator::NotEqual)?
            }
            Instruction::IfGreaterThanJumpZero { .. } => {
              stack.push_binary_operator(Type::Bool, BinaryOperator::GreaterThan)?
            }
            Instruction::IfGreaterOrEqualJumpZero { .. } => {
              stack.push_binary_operator(Type::Bool, BinaryOperator::GreaterOrEqual)?
            }
            Instruction::IfLowerThanJumpZero { .. } => {
              stack.push_binary_operator(Type::Bool, BinaryOperator::LowerThan)?
            }
            Instruction::IfLowerOrEqualJumpZero { .. } => {
              stack.push_binary_operator(Type::Bool, BinaryOperator::LowerOrEqual)?
            }
            _ => {}
          }

          match &flow {
            ControlFlow::If { then, .. } => {
              statements.push(StatementInfo {
                instructions: &self.instructions[index..=index],
                statement:    Statement::If {
                  condition: stack.pop()?,
                  then:      self.decompile_node(script, functions, then, stack.clone())?
                }
              })
            }
            ControlFlow::IfElse { then, els, .. } => {
              statements.push(StatementInfo {
                instructions: &self.instructions[index..=index],
                statement:    Statement::IfElse {
                  condition: stack.pop()?,
                  then:      self.decompile_node(script, functions, then, stack.clone())?,
                  els:       self.decompile_node(script, functions, els, stack.clone())?
                }
              })
            }
            ControlFlow::Leaf { .. } | ControlFlow::Flow { .. } => {}
            ControlFlow::AndOr { .. } => todo!(),
            ControlFlow::WhileLoop { body, .. } => {
              statements.push(StatementInfo {
                instructions: &self.instructions[index..=index],
                statement:    Statement::WhileLoop {
                  condition: stack.pop()?,
                  body:      self.decompile_node(script, functions, body, stack.clone())?
                }
              })
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
            ControlFlow::Switch { cases, .. } => {
              statements.push(StatementInfo {
                instructions: &self.instructions[index..=index],
                statement:    Statement::Switch {
                  condition: stack.pop()?,
                  cases:     cases
                    .iter()
                    .map(|(body, cases)| {
                      Ok((
                        self.decompile_node(script, functions, body, stack.clone())?,
                        cases.clone()
                      ))
                    })
                    .collect::<Result<_, _>>()?
                }
              })
            }
          };
        }
        Instruction::FunctionCall { location } => {
          let location = *location as usize;
          let target = functions.get(&location).expect("TODO HANDLE THIS");
          if target.return_count > 0 {
            stack.push_function_call(
              target.parameters as usize,
              target.return_count as usize,
              target.location
            )?
          } else {
            statements.push(StatementInfo {
              instructions: &self.instructions[index..=index],
              statement:    Statement::FunctionCall {
                args:             {
                  let mut args = stack.pop_n(target.parameters as usize)?;
                  args.reverse();
                  args
                },
                function_address: target.location
              }
            })
          }
        }
        Instruction::StaticU24 { static_index } => {
          stack.push_static(*static_index as usize);
          stack.push_reference()?
        }
        Instruction::StaticU24Load { static_index } => stack.push_static(*static_index as usize),
        Instruction::StaticU24Store { static_index } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_static(*static_index as usize);
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::GlobalU24 { global_index } => {
          stack.push_global(*global_index as usize);
          stack.push_reference()?;
        }
        Instruction::GlobalU24Load { global_index } => stack.push_global(*global_index as usize),
        Instruction::GlobalU24Store { global_index } => {
          statements.push(StatementInfo {
            instructions: &self.instructions[index..=index],
            statement:    Statement::Assign {
              destination: {
                stack.push_global(*global_index as usize);
                stack.pop()?
              },
              source:      stack.pop()?
            }
          })
        }
        Instruction::PushConstU24 { c1 } => stack.push_int(*c1 as i64),
        Instruction::String => stack.push_string()?,
        Instruction::StringHash => stack.push_string_hash()?,
        Instruction::TextLabelAssignString { buffer_size } => todo!(),
        Instruction::TextLabelAssignInt { buffer_size } => todo!(),
        Instruction::TextLabelAppendString { buffer_size } => todo!(),
        Instruction::TextLabelAppendInt { buffer_size } => todo!(),
        Instruction::TextLabelCopy => todo!(),
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
        Instruction::BitTest => stack.push_binary_operator(Type::Bool, BinaryOperator::BitTest)?
      };
    }

    if let Some(after) = flow.after() {
      statements.extend(self.decompile_node(script, functions, after, stack)?);
    }

    Ok(statements)
  }
}
