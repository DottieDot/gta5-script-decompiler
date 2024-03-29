use std::{backtrace::Backtrace, collections::VecDeque};

use thiserror::Error;

use crate::script::Script;

use super::{
  stack_entry::{BinaryOperator, StackEntry, UnaryOperator},
  Confidence, Function, LinkedValueType, Primitives, ScriptGlobals, ScriptStatics, StackEntryInfo,
  ValueType, ValueTypeInfo
};

#[derive(Default, Debug, Clone)]
pub struct Stack<'i> {
  stack: VecDeque<StackEntryInfo<'i>>
}

impl<'i> Stack<'i> {
  pub fn push_int(&mut self, val: i64) {
    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::Int(val),
      ty:    {
        let mut ty = LinkedValueType::new_primitive(Primitives::Int);
        ty.confidence(Confidence::Low);
        ty.make_shared()
      }
    })
  }

  pub fn push_float(&mut self, val: f32) {
    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::Float(val),
      ty:    {
        let mut ty = LinkedValueType::new_primitive(Primitives::Int);
        ty.confidence(Confidence::High);
        ty.make_shared()
      }
    })
  }

  pub fn push_string(&mut self, script: &'i Script) -> Result<(), InvalidStackError> {
    static UNKNOWN_STRING: &str = "<UNKNOWN>";

    let index = self.pop()?;

    let StackEntryInfo { entry: StackEntry::Int(n), .. } = index else {
      return Err(InvalidStackError {
      backtrace: Backtrace::capture()
    })
    };

    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::String(script.get_string(n as usize).unwrap_or(UNKNOWN_STRING)),
      ty:    {
        let mut ty = LinkedValueType::new_primitive(Primitives::String);
        ty.confidence(Confidence::High);
        ty.make_shared()
      }
    });

    Ok(())
  }

  pub fn set_last_as_field(&mut self) -> Result<(), InvalidStackError> {
    let popped = self.pop()?;
    self.stack.push_back(StackEntryInfo {
      ty:    LinkedValueType::struct_field(&popped.ty, 0),
      entry: StackEntry::StructField {
        source: Box::new(popped),
        field:  0
      }
    });
    Ok(())
  }

  pub fn push_offset(&mut self) -> Result<(), InvalidStackError> {
    let offset = Box::new(self.pop()?);
    let source = Box::new(self.pop()?);

    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::Offset { source, offset },
      // TODO: CAN OFFSETS BE DYNAMIC?
      ty:    LinkedValueType::new_primitive(Primitives::Unknown).make_shared()
    });

    Ok(())
  }

  pub fn push_const_offset(&mut self, offset: i64) -> Result<(), InvalidStackError> {
    let source = Box::new(self.pop()?);

    let source_type = source.ty.borrow_mut().ref_type();
    let field = LinkedValueType::struct_field(&source_type, offset as usize);

    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::Offset {
        source,
        offset: Box::new(StackEntryInfo {
          entry: StackEntry::Int(offset),
          ty:    {
            let mut ty = LinkedValueType::new_primitive(Primitives::Int);
            ty.confidence(Confidence::High);
            ty.make_shared()
          }
        })
      },
      ty:    field
    });

    Ok(())
  }

  pub fn push_array_item(&mut self, item_size: usize) -> Result<(), InvalidStackError> {
    let source = Box::new(self.pop()?);
    let index = Box::new(self.pop()?);

    let array_item_type = source.ty.borrow_mut().array_item_type();

    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::ArrayItem {
        source,
        item_size,
        index
      },
      ty:    array_item_type
    });

    Ok(())
  }

  pub fn push_local(&mut self, local_index: usize, fun: &Function) {
    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::Local(local_index),
      ty:    fun
        .local_index_type(local_index)
        .cloned()
        .unwrap_or_else(|| LinkedValueType::new_primitive(Primitives::Unknown).make_shared())
    })
  }

  pub fn push_static(&mut self, static_index: usize, statics: &ScriptStatics) {
    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::Static(static_index),
      ty:    statics
        .get_static(static_index)
        .cloned()
        .unwrap_or_else(|| LinkedValueType::new_primitive(Primitives::Unknown).make_shared())
    })
  }

  pub fn push_global(&mut self, global_index: usize, globals: &ScriptGlobals) {
    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::Global(global_index),
      ty:    globals.get_global(global_index).clone()
    })
  }

  pub fn push_deref(&mut self) -> Result<(), InvalidStackError> {
    let source = Box::new(self.pop()?);

    let ref_type = source.ty.borrow_mut().ref_type();

    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::Deref(source),
      ty:    ref_type
    });

    Ok(())
  }

  pub fn push_reference(&mut self) -> Result<(), InvalidStackError> {
    let source = Box::new(self.pop()?);

    let cloned = source.ty.clone();
    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::Ref(source),
      ty:    {
        let mut ty = LinkedValueType::new_ref(cloned);
        ty.confidence(Confidence::High);
        ty.make_shared()
      }
    });

    Ok(())
  }

  pub fn push_catch(&mut self) {
    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::CatchValue,
      ty:    {
        let mut ty = LinkedValueType::new_primitive(Primitives::Int);
        ty.confidence(Confidence::Medium);
        ty.make_shared()
      }
    })
  }

  pub fn push_binary_operator(
    &mut self,
    ty: Primitives,
    lhs_ty: ValueTypeInfo,
    rhs_ty: ValueTypeInfo,
    op: BinaryOperator
  ) -> Result<(), InvalidStackError> {
    let rhs = Box::new(self.pop()?);
    let lhs = Box::new(self.pop()?);

    lhs.ty.borrow_mut().hint(lhs_ty);
    rhs.ty.borrow_mut().hint(rhs_ty);

    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::BinaryOperator { lhs, rhs, op },
      ty:    {
        let mut ty = LinkedValueType::new_primitive(ty);
        ty.confidence(Confidence::High);
        ty.make_shared()
      }
    });
    Ok(())
  }

  pub fn push_unary_operator(
    &mut self,
    ty: Primitives,
    operand_type: ValueTypeInfo,
    op: UnaryOperator
  ) -> Result<(), InvalidStackError> {
    let lhs = Box::new(self.pop()?);

    lhs.ty.borrow_mut().hint(operand_type);

    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::UnaryOperator { lhs, op },
      ty:    LinkedValueType::new_primitive(ty).make_shared()
    });
    Ok(())
  }

  pub fn push_vector_binary_operator(
    &mut self,
    op: BinaryOperator
  ) -> Result<(), InvalidStackError> {
    let ty = LinkedValueType::new_vector3().make_shared();

    let a = self.pop_n(3)?;
    let b = self.pop_n(3)?;

    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::Struct {
        origin: Box::new(StackEntryInfo {
          entry: StackEntry::BinaryOperator {
            lhs: Box::new(StackEntryInfo {
              entry: StackEntry::ResultStruct { values: a },
              ty:    ty.clone()
            }),
            rhs: Box::new(StackEntryInfo {
              entry: StackEntry::ResultStruct { values: b },
              ty:    ty.clone()
            }),
            op
          },
          ty:    ty.clone()
        }),
        size:   3
      },
      ty
    });

    Ok(())
  }

  pub fn push_vector_unary_operator(&mut self, op: UnaryOperator) -> Result<(), InvalidStackError> {
    let ty = LinkedValueType::new_vector3().make_shared();

    let a = self.pop_n(3)?;

    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::Struct {
        origin: Box::new(StackEntryInfo {
          entry: StackEntry::UnaryOperator {
            lhs: Box::new(StackEntryInfo {
              entry: StackEntry::ResultStruct { values: a },
              ty:    ty.clone()
            }),
            op
          },
          ty:    ty.clone()
        }),
        size:   3
      },
      ty
    });

    Ok(())
  }

  pub fn push_float_to_vector(&mut self) -> Result<(), InvalidStackError> {
    let float = self.pop()?;
    let ty = LinkedValueType::new_vector3().make_shared();
    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::Struct {
        origin: Box::new(StackEntryInfo {
          entry: StackEntry::FloatToVector(Box::new(float)),
          ty:    ty.clone()
        }),
        size:   3
      },
      ty
    });

    Ok(())
  }

  pub fn push_cast(
    &mut self,
    ty: Primitives,
    from_type: ValueTypeInfo
  ) -> Result<(), InvalidStackError> {
    let source = Box::new(self.pop()?);

    source.ty.borrow_mut().hint(from_type);

    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::Cast { source },
      ty:    {
        let mut ty = LinkedValueType::new_primitive(ty);
        ty.confidence(Confidence::Medium);
        ty.make_shared()
      }
    });
    Ok(())
  }

  pub fn push_string_hash(&mut self) -> Result<(), InvalidStackError> {
    let source = Box::new(self.pop()?);

    source.ty.borrow_mut().hint(ValueTypeInfo {
      ty:         ValueType::Primitive(Primitives::String),
      confidence: Confidence::High
    });

    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::StringHash(source),
      ty:    {
        let mut ty = LinkedValueType::new_primitive(Primitives::Int);
        ty.confidence(Confidence::Medium);
        ty.make_shared()
      }
    });
    Ok(())
  }

  pub fn push_dup(&mut self) -> Result<(), InvalidStackError> {
    let back = self.stack.back().ok_or(InvalidStackError {
      backtrace: Backtrace::capture()
    })?;

    if back.entry.size() > 1 {
      let (last, _) = back.clone().split_off();

      self.stack.push_back(last);
    } else {
      self.stack.push_back(back.clone());
    }

    Ok(())
  }

  pub fn push_load_n(&mut self) -> Result<(), InvalidStackError> {
    let addr = self.pop()?;
    let count = self.pop()?;

    let StackEntry::Int(n) = count.entry else {
      return Err(InvalidStackError {
      backtrace: Backtrace::capture()
    })
    };

    let addr = match addr {
      StackEntryInfo {
        entry: StackEntry::Ref(rf),
        ..
      } => *rf,
      _ => addr
    };

    addr.ty.borrow_mut().struct_size(n as usize);

    let cloned = addr.ty.clone();
    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::Struct {
        origin: Box::new(addr),
        size:   n as usize
      },
      ty:    cloned
    });

    Ok(())
  }

  pub fn push_const_int_binary_operator(
    &mut self,
    op: BinaryOperator,
    value: i64
  ) -> Result<(), InvalidStackError> {
    let lhs = Box::new(self.pop()?);
    let rhs = Box::new(StackEntryInfo {
      entry: StackEntry::Int(value),
      ty:    {
        let mut ty = LinkedValueType::new_primitive(Primitives::Int);
        ty.confidence(Confidence::Medium);
        ty.make_shared()
      }
    });

    lhs.ty.borrow_mut().hint(ValueTypeInfo {
      ty:         ValueType::Primitive(Primitives::Int),
      confidence: Confidence::Medium
    });

    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::BinaryOperator { lhs, rhs, op },
      ty:    {
        let mut ty = LinkedValueType::new_primitive(Primitives::Int);
        ty.confidence(Confidence::Medium);
        ty.make_shared()
      }
    });

    Ok(())
  }

  pub fn push_function_call(&mut self, function: &Function) -> Result<(), InvalidStackError> {
    let mut args: Vec<StackEntryInfo> = self.pop_n(function.parameter_count)?;
    args.reverse();

    let mut param_iter = function.parameters.iter();
    for arg in &args {
      if let Some(param) = param_iter.next() {
        LinkedValueType::link(&arg.ty, param);
      }
      let _ = param_iter.advance_by(arg.entry.size());
    }

    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::FunctionCallResult {
        args,
        function_address: function.location,
        return_values: function.return_count
      },
      ty:    function
        .returns
        .clone()
        .expect("void function pushed to stack")
    });

    Ok(())
  }

  pub fn push_native_call(
    &mut self,
    arg_count: usize,
    return_count: usize,
    native_hash: u64
  ) -> Result<(), InvalidStackError> {
    let mut args = self.pop_n(arg_count)?;
    args.reverse();
    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::NativeCallResult {
        return_values: return_count,
        native_hash,
        args
      },
      ty:    {
        let mut ty = LinkedValueType::new_primitive(Primitives::Unknown);
        if return_count > 1 {
          ty.confidence(Confidence::Medium);
        }
        ty.struct_size(return_count);
        ty.make_shared()
      }
    });

    Ok(())
  }

  pub fn pop(&mut self) -> Result<StackEntryInfo<'i>, InvalidStackError> {
    let back = self.stack.pop_back().ok_or(InvalidStackError {
      backtrace: Backtrace::capture()
    })?;

    if back.entry.size() > 1 {
      let (last, rest) = back.split_off();

      if let Some(rest) = rest {
        self.stack.push_back(rest);
      }

      Ok(last)
    } else {
      Ok(back)
    }
  }

  pub fn pop_n(&mut self, mut n: usize) -> Result<Vec<StackEntryInfo<'i>>, InvalidStackError> {
    let mut result = Vec::with_capacity(n);
    while n > 0 {
      let back = self.get_back()?;

      if back.entry.size() > n {
        result.push(self.pop()?);
        n -= 1;
      } else {
        let popped = self.stack.pop_back().ok_or(InvalidStackError {
          backtrace: Backtrace::capture()
        })?;
        n -= popped.entry.size();
        result.push(popped);
      }
    }

    Ok(result)
  }

  pub fn nth_back(&mut self, n: usize) -> Result<StackEntryInfo<'i>, InvalidStackError> {
    let back = self
      .stack
      .iter()
      .rev()
      .nth(n)
      .ok_or(InvalidStackError {
        backtrace: Backtrace::capture()
      })?
      .clone();

    if back.entry.size() > 1 {
      let (last, rest) = back.split_off();

      if let Some(rest) = rest {
        self.stack.push_back(rest);
      }

      Ok(last)
    } else {
      Ok(back)
    }
  }

  pub fn try_make_bitwise_logical(&mut self) -> Result<(), InvalidStackError> {
    let last = self.pop()?;
    match last.entry {
      StackEntry::BinaryOperator {
        lhs,
        rhs,
        op: BinaryOperator::BitwiseAnd
      } => {
        self.stack.push_back(StackEntryInfo {
          entry: StackEntry::BinaryOperator {
            lhs,
            rhs,
            op: BinaryOperator::LogicalAnd
          },
          ty:    last.ty
        });
        Ok(())
      }
      StackEntry::BinaryOperator {
        lhs,
        rhs,
        op: BinaryOperator::BitwiseOr
      } => {
        self.stack.push_back(StackEntryInfo {
          entry: StackEntry::BinaryOperator {
            lhs,
            rhs,
            op: BinaryOperator::LogicalOr
          },
          ty:    last.ty
        });
        Ok(())
      }
      _ => {
        self.stack.push_back(last);
        Ok(())
      }
    }
  }

  fn get_back(&self) -> Result<&StackEntryInfo, InvalidStackError> {
    self.stack.back().ok_or(InvalidStackError {
      backtrace: Backtrace::capture()
    })
  }
}

#[derive(Debug, Error)]
#[error("Stack is in an invalid state:\n${backtrace:#?}")]
pub struct InvalidStackError {
  pub backtrace: Backtrace
}
