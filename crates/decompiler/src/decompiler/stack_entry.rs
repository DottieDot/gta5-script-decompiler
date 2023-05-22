use std::{cell::RefCell, rc::Rc};

use thiserror::Error;

use super::LinkedValueType;

#[derive(Clone, Debug)]
pub enum StackEntry<'i> {
  Int(i64),
  Float(f32),
  String(&'i str),
  Struct {
    origin: Box<StackEntryInfo<'i>>,
    size:   usize
  },
  ResultStruct {
    values: Vec<StackEntryInfo<'i>>
  },
  StructField {
    source: Box<StackEntryInfo<'i>>,
    field:  usize
  },
  Offset {
    source: Box<StackEntryInfo<'i>>,
    offset: Box<StackEntryInfo<'i>>
  },
  ArrayItem {
    source:    Box<StackEntryInfo<'i>>,
    index:     Box<StackEntryInfo<'i>>,
    item_size: usize
  },
  Local(usize),
  Static(usize),
  Global(usize),
  Deref(Box<StackEntryInfo<'i>>),
  Ref(Box<StackEntryInfo<'i>>),
  FloatToVector(Box<StackEntryInfo<'i>>),
  CatchValue,
  BinaryOperator {
    lhs: Box<StackEntryInfo<'i>>,
    rhs: Box<StackEntryInfo<'i>>,
    op:  BinaryOperator
  },
  UnaryOperator {
    lhs: Box<StackEntryInfo<'i>>,
    op:  UnaryOperator
  },
  Cast {
    source: Box<StackEntryInfo<'i>>
  },
  StringHash(Box<StackEntryInfo<'i>>),
  FunctionCallResult {
    args:             Vec<StackEntryInfo<'i>>,
    function_address: usize,
    return_values:    usize
  },
  NativeCallResult {
    args:          Vec<StackEntryInfo<'i>>,
    return_values: usize,
    native_hash:   u64
  }
}

#[derive(Debug, Clone)]
pub struct StackEntryInfo<'i> {
  pub entry: StackEntry<'i>,
  pub ty:    Rc<RefCell<LinkedValueType>>
}

impl<'i> StackEntryInfo<'i> {
  pub fn split_off(mut self) -> (Self, Option<Self>) {
    // Avoid unnecessary clone
    if self.entry.size() == 1 {
      return (self, None);
    }

    let cloned = self.clone();
    match &mut self.entry {
      StackEntry::Struct { origin, size } => {
        size.checked_sub(1).expect("corrupted stack entry");
        let field = StackEntryInfo {
          entry: StackEntry::StructField {
            source: origin.clone(),
            field:  *size
          },
          ty:    LinkedValueType::struct_field(&self.ty, *size)
        };
        (field, Some(self))
      }
      StackEntry::ResultStruct { values } => {
        values.pop().expect("corrupted stack entry");
        let field = StackEntryInfo {
          entry: StackEntry::StructField {
            source: Box::new(cloned),
            field:  values.len()
          },
          ty:    LinkedValueType::struct_field(&self.ty, values.len())
        };
        (field, Some(self))
      }
      StackEntry::FunctionCallResult { return_values, .. }
      | StackEntry::NativeCallResult { return_values, .. } => {
        let field_index = return_values.checked_sub(1).expect("corrupted stack entry");
        let field = StackEntryInfo {
          entry: StackEntry::StructField {
            source: Box::new(cloned),
            field:  field_index
          },
          ty:    LinkedValueType::struct_field(&self.ty, field_index)
        };

        *return_values -= 1;

        (field, if *return_values > 0 { Some(self) } else { None })
      }
      _ => panic!("StackEntry::size(&self) is not implemented correctly")
    }
  }
}

impl<'i> StackEntry<'i> {
  pub fn size(&self) -> usize {
    match self {
      Self::Struct { size, .. } => *size,
      Self::ResultStruct { values } => values.len(),
      Self::FunctionCallResult { return_values, .. } => *return_values,
      Self::NativeCallResult { return_values, .. } => *return_values,
      _ => 1
    }
  }
}

#[derive(Debug, Error)]
#[error("Value cannot be split off")]
pub struct SplitOffError;

#[derive(Copy, Clone, Debug)]
pub enum BinaryOperator {
  Add,
  Subtract,
  Multiply,
  Divide,
  BitwiseAnd,
  BitwiseOr,
  BitwiseXor,
  Modulo,
  Equal,
  NotEqual,
  GreaterThan,
  GreaterOrEqual,
  LowerThan,
  LowerOrEqual,
  BitTest,
  LogicalAnd,
  LogicalOr
}

#[derive(Copy, Clone, Debug)]
pub enum UnaryOperator {
  Not,
  Negate
}
