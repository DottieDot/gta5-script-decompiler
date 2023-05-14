use std::collections::VecDeque;

use itertools::Itertools;
use thiserror::Error;

use super::stack_entry::{BinaryOperator, StackEntry, Type, UnaryOperator};

#[derive(Default, Debug, Clone)]
pub struct Stack {
  stack: VecDeque<StackEntry>
}

impl Stack {
  pub fn push_int(&mut self, val: i64) {
    self.stack.push_back(StackEntry::Int(val))
  }

  pub fn push_float(&mut self, val: f32) {
    self.stack.push_back(StackEntry::Float(val))
  }

  pub fn push_string(&mut self) -> Result<(), InvalidStackError> {
    let index = self.pop()?;

    let StackEntry::Int(n) = index else {
      return Err(InvalidStackError)
    };

    self.stack.push_back(StackEntry::String(n as usize));

    Ok(())
  }

  pub fn push_offset(&mut self) -> Result<(), InvalidStackError> {
    let source = Box::new(self.pop()?);
    let offset = Box::new(self.pop()?);

    self.stack.push_back(StackEntry::Offset { source, offset });

    Ok(())
  }

  pub fn push_const_offset(&mut self, offset: i64) -> Result<(), InvalidStackError> {
    let source = Box::new(self.pop()?);

    self.stack.push_back(StackEntry::Offset {
      source,
      offset: Box::new(StackEntry::Int(offset))
    });

    Ok(())
  }

  pub fn push_array_item(&mut self, item_size: usize) -> Result<(), InvalidStackError> {
    let source = Box::new(self.pop()?);
    let index = Box::new(self.pop()?);
    self.stack.push_back(StackEntry::ArrayItem {
      source,
      item_size,
      index
    });
    Ok(())
  }

  pub fn push_local(&mut self, local_index: usize) {
    self.stack.push_back(StackEntry::Local(local_index))
  }

  pub fn push_static(&mut self, static_index: usize) {
    self.stack.push_back(StackEntry::Static(static_index))
  }

  pub fn push_global(&mut self, global_index: usize) {
    self.stack.push_back(StackEntry::Global(global_index))
  }

  pub fn push_deref(&mut self) -> Result<(), InvalidStackError> {
    let source = Box::new(self.pop()?);
    self.stack.push_back(StackEntry::Deref(source));
    Ok(())
  }

  pub fn push_reference(&mut self) -> Result<(), InvalidStackError> {
    let source = Box::new(self.pop()?);
    self.stack.push_back(StackEntry::Ref(source));
    Ok(())
  }

  pub fn push_catch(&mut self) {
    self.stack.push_back(StackEntry::CatchValue)
  }

  pub fn push_binary_operator(
    &mut self,
    ty: Type,
    op: BinaryOperator
  ) -> Result<(), InvalidStackError> {
    let rhs = Box::new(self.pop()?);
    let lhs = Box::new(self.pop()?);
    self
      .stack
      .push_back(StackEntry::BinaryOperator { lhs, rhs, ty, op });
    Ok(())
  }

  pub fn push_unary_operator(
    &mut self,
    ty: Type,
    op: UnaryOperator
  ) -> Result<(), InvalidStackError> {
    let lhs = Box::new(self.pop()?);
    self
      .stack
      .push_back(StackEntry::UnaryOperator { lhs, ty, op });
    Ok(())
  }

  pub fn push_vector_binary_operator(
    &mut self,
    op: BinaryOperator
  ) -> Result<(), InvalidStackError> {
    let x1 = Box::new(self.pop()?);
    let y1 = Box::new(self.pop()?);
    let z1 = Box::new(self.pop()?);
    let x2 = Box::new(self.pop()?);
    let y2 = Box::new(self.pop()?);
    let z2 = Box::new(self.pop()?);

    self.stack.push_back(StackEntry::BinaryOperator {
      lhs: x1,
      rhs: x2,
      ty: Type::Float,
      op
    });
    self.stack.push_back(StackEntry::BinaryOperator {
      lhs: y1,
      rhs: y2,
      ty: Type::Float,
      op
    });
    self.stack.push_back(StackEntry::BinaryOperator {
      lhs: z1,
      rhs: z2,
      ty: Type::Float,
      op
    });

    Ok(())
  }

  pub fn push_cast(&mut self, ty: Type) -> Result<(), InvalidStackError> {
    let source = Box::new(self.pop()?);
    self.stack.push_back(StackEntry::Cast { source, ty });
    Ok(())
  }

  pub fn push_string_hash(&mut self) -> Result<(), InvalidStackError> {
    let source = Box::new(self.pop()?);
    self.stack.push_back(StackEntry::StringHash(source));
    Ok(())
  }

  pub fn push_dup(&mut self) -> Result<(), InvalidStackError> {
    let back = self.stack.back().ok_or(InvalidStackError)?;

    if back.size() > 1 {
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

    let StackEntry::Int(n) = count else {
      return Err(InvalidStackError)
    };

    let addr = match addr {
      StackEntry::Ref(rf) => *rf,
      _ => addr
    };

    self.stack.push_back(StackEntry::Struct {
      origin: Box::new(addr),
      size:   n as usize
    });

    Ok(())
  }

  pub fn push_const_int_binary_operator(
    &mut self,
    op: BinaryOperator,
    value: i64
  ) -> Result<(), InvalidStackError> {
    let lhs = Box::new(self.pop()?);
    let rhs = Box::new(StackEntry::Int(value));

    self.stack.push_back(StackEntry::BinaryOperator {
      lhs,
      rhs,
      ty: Type::Int,
      op
    });

    Ok(())
  }

  pub fn push_function_call(
    &mut self,
    arg_count: usize,
    return_count: usize,
    function_address: usize
  ) -> Result<(), InvalidStackError> {
    let mut args = self.pop_n(arg_count)?;
    args.reverse();
    self.stack.push_back(StackEntry::FunctionCallResult {
      args,
      function_address,
      return_values: return_count
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
    self.stack.push_back(StackEntry::NativeCallResult {
      return_values: return_count,
      native_hash,
      args
    });

    Ok(())
  }

  pub fn pop(&mut self) -> Result<StackEntry, InvalidStackError> {
    let back = self.stack.pop_back().ok_or(InvalidStackError)?;

    if back.size() > 1 {
      let (last, rest) = back.split_off();

      if let Some(rest) = rest {
        self.stack.push_back(rest);
      }

      Ok(last)
    } else {
      Ok(back)
    }
  }

  pub fn pop_n(&mut self, mut n: usize) -> Result<Vec<StackEntry>, InvalidStackError> {
    let mut result = Vec::with_capacity(n);
    while n > 0 {
      let back = self.back()?;

      if back.size() > n {
        result.push(self.pop()?);
        n -= 1;
      } else {
        let popped = self.stack.pop_back().ok_or(InvalidStackError)?;
        n -= popped.size();
        result.push(popped);
      }
    }

    Ok(result)
  }

  pub fn try_make_bitwise_logical(&mut self) -> Result<(), InvalidStackError> {
    let last = self.pop()?;
    match last {
      StackEntry::BinaryOperator {
        lhs,
        rhs,
        ty,
        op: BinaryOperator::BitwiseAnd
      } => {
        self.stack.push_back(StackEntry::BinaryOperator {
          lhs,
          rhs,
          ty,
          op: BinaryOperator::LogicalAnd
        });
        Ok(())
      }
      StackEntry::BinaryOperator {
        lhs,
        rhs,
        ty,
        op: BinaryOperator::BitwiseOr
      } => {
        self.stack.push_back(StackEntry::BinaryOperator {
          lhs,
          rhs,
          ty,
          op: BinaryOperator::LogicalOr
        });
        Ok(())
      }
      _ => {
        self.stack.push_back(last);
        Ok(())
      }
    }
  }

  fn back(&self) -> Result<&StackEntry, InvalidStackError> {
    self.stack.back().ok_or(InvalidStackError)
  }
}

#[derive(Debug, Error)]
#[error("Stack is in an invalid state")]
pub struct InvalidStackError;
