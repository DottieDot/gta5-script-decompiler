use std::collections::VecDeque;

use thiserror::Error;

use super::stack_entry::{BinaryOperator, StackEntry, Type, UnaryOperator};

#[derive(Default, Debug)]
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
    let index = self.stack.pop_back().ok_or(InvalidStackError)?;

    let StackEntry::Int(n) = index else {
      return Err(InvalidStackError)
    };

    self.stack.push_back(StackEntry::String(n as usize));

    Ok(())
  }

  pub fn push_offset(&mut self) -> Result<(), InvalidStackError> {
    let source = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
    let offset = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);

    self.stack.push_back(StackEntry::Offset { source, offset });

    Ok(())
  }

  pub fn push_const_offset(&mut self, offset: i64) -> Result<(), InvalidStackError> {
    let source = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);

    self.stack.push_back(StackEntry::Offset {
      source,
      offset: Box::new(StackEntry::Int(offset))
    });

    Ok(())
  }

  pub fn push_array_item(&mut self, item_size: usize) -> Result<(), InvalidStackError> {
    let source = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
    let index = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
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
    let source = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
    self.stack.push_back(StackEntry::Deref(source));
    Ok(())
  }

  pub fn push_reference(&mut self) -> Result<(), InvalidStackError> {
    let source = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
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
    let lhs = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
    let rhs = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
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
    let lhs = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
    self
      .stack
      .push_back(StackEntry::UnaryOperator { lhs, ty, op });
    Ok(())
  }

  pub fn push_vector_binary_operator(
    &mut self,
    op: BinaryOperator
  ) -> Result<(), InvalidStackError> {
    let x1 = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
    let y1 = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
    let z1 = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
    let x2 = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
    let y2 = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
    let z2 = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);

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
    let source = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
    self.stack.push_back(StackEntry::Cast { source, ty });
    Ok(())
  }

  pub fn push_string_hash(&mut self) -> Result<(), InvalidStackError> {
    let source = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
    self.stack.push_back(StackEntry::StringHash(source));
    Ok(())
  }

  pub fn push_dup(&mut self) -> Result<(), InvalidStackError> {
    let top = self.stack.back().ok_or(InvalidStackError)?.clone();
    self.stack.push_back(top);
    Ok(())
  }

  pub fn push_load_n(&mut self) -> Result<(), InvalidStackError> {
    let addr = self.stack.pop_back().ok_or(InvalidStackError)?;
    let count = self.stack.pop_back().ok_or(InvalidStackError)?;

    let StackEntry::Int(n) = count else {
      return Err(InvalidStackError)
    };

    for i in 0..n {
      self
        .stack
        .push_back(StackEntry::Deref(Box::new(StackEntry::Offset {
          source: Box::new(addr.clone()),
          offset: Box::new(StackEntry::Int(i))
        })));
    }

    Ok(())
  }

  pub fn push_const_int_binary_operator(
    &mut self,
    op: BinaryOperator,
    value: i64
  ) -> Result<(), InvalidStackError> {
    let lhs = Box::new(self.stack.pop_back().ok_or(InvalidStackError)?);
    let rhs = Box::new(StackEntry::Int(value));

    self.stack.push_back(StackEntry::BinaryOperator {
      lhs,
      rhs,
      ty: Type::Int,
      op
    });

    Ok(())
  }
}

#[derive(Debug, Error)]
#[error("Stack is in an invalid state")]
pub struct InvalidStackError;
