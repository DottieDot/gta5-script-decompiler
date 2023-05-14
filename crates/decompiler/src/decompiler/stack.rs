use std::collections::VecDeque;

use thiserror::Error;

use super::{
  stack_entry::{BinaryOperator, StackEntry, UnaryOperator},
  Confidence, Function, LinkedValueType, Primitives, ScriptGlobals, ScriptStatics, StackEntryInfo,
  ValueType, ValueTypeInfo
};

#[derive(Default, Debug, Clone)]
pub struct Stack {
  stack: VecDeque<StackEntryInfo>
}

impl Stack {
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

  pub fn push_string(&mut self) -> Result<(), InvalidStackError> {
    let index = self.pop()?;

    let StackEntryInfo { entry: StackEntry::Int(n), .. } = index else {
      return Err(InvalidStackError)
    };

    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::String(n as usize),
      ty:    {
        let mut ty = LinkedValueType::new_primitive(Primitives::String);
        ty.confidence(Confidence::High);
        ty.make_shared()
      }
    });

    Ok(())
  }

  pub fn push_offset(&mut self) -> Result<(), InvalidStackError> {
    let source = Box::new(self.pop()?);
    let offset = Box::new(self.pop()?);

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
    let field = source_type.borrow_mut().struct_field(offset as usize);

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

  pub fn push_global(&mut self, global_index: usize, globals: &mut ScriptGlobals) {
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
    todo!();

    // let x1 = Box::new(self.pop()?);
    // let y1 = Box::new(self.pop()?);
    // let z1 = Box::new(self.pop()?);
    // let x2 = Box::new(self.pop()?);
    // let y2 = Box::new(self.pop()?);
    // let z2 = Box::new(self.pop()?);

    // self.stack.push_back(StackEntry::BinaryOperator {
    //   lhs: x1,
    //   rhs: x2,
    //   ty: Type::Float,
    //   op
    // });
    // self.stack.push_back(StackEntry::BinaryOperator {
    //   lhs: y1,
    //   rhs: y2,
    //   ty: Type::Float,
    //   op
    // });
    // self.stack.push_back(StackEntry::BinaryOperator {
    //   lhs: z1,
    //   rhs: z2,
    //   ty: Type::Float,
    //   op
    // });

    // Ok(())
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
    let back = self.stack.back().ok_or(InvalidStackError)?;

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
      return Err(InvalidStackError)
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
    let mut args = self.pop_n(function.parameters.len())?;
    args.reverse();
    self.stack.push_back(StackEntryInfo {
      entry: StackEntry::FunctionCallResult {
        args,
        function_address: function.location,
        return_values: function
          .returns
          .as_ref()
          .map(|r| (**r).borrow().size())
          .unwrap_or_default()
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
        ty.confidence(Confidence::Medium);
        ty.struct_size(return_count);
        ty.make_shared()
      }
    });

    Ok(())
  }

  pub fn pop(&mut self) -> Result<StackEntryInfo, InvalidStackError> {
    let back = self.stack.pop_back().ok_or(InvalidStackError)?;

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

  pub fn pop_n(&mut self, mut n: usize) -> Result<Vec<StackEntryInfo>, InvalidStackError> {
    let mut result = Vec::with_capacity(n);
    while n > 0 {
      let back = self.back()?;

      if back.entry.size() > n {
        result.push(self.pop()?);
        n -= 1;
      } else {
        let popped = self.stack.pop_back().ok_or(InvalidStackError)?;
        n -= popped.entry.size();
        result.push(popped);
      }
    }

    Ok(result)
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

  fn back(&self) -> Result<&StackEntryInfo, InvalidStackError> {
    self.stack.back().ok_or(InvalidStackError)
  }
}

#[derive(Debug, Error)]
#[error("Stack is in an invalid state")]
pub struct InvalidStackError;
