use thiserror::Error;

#[derive(Clone, Debug)]
pub enum StackEntry {
  Int(i64),
  Float(f32),
  String(usize),
  Struct {
    values: Vec<StackEntry>
  },
  StructField {
    source: Box<StackEntry>,
    field:  usize
  },
  Offset {
    source: Box<StackEntry>,
    offset: Box<StackEntry>
  },
  ArrayItem {
    source:    Box<StackEntry>,
    index:     Box<StackEntry>,
    item_size: usize
  },
  LocalRef(usize),
  StaticRef(usize),
  GlobalRef(usize),
  Deref(Box<StackEntry>),
  Ref(Box<StackEntry>),
  CatchValue,
  BinaryOperator {
    lhs: Box<StackEntry>,
    rhs: Box<StackEntry>,
    ty:  Type,
    op:  BinaryOperator
  },
  UnaryOperator {
    lhs: Box<StackEntry>,
    ty:  Type,
    op:  UnaryOperator
  },
  Cast {
    source: Box<StackEntry>,
    ty:     Type
  },
  StringHash(Box<StackEntry>),
  FunctionCallResult {
    args:             Vec<StackEntry>,
    function_address: usize,
    return_values:    usize
  },
  NativeCallResult {
    args:          Vec<StackEntry>,
    return_values: usize,
    native_hash:   u64
  }
}

impl StackEntry {
  pub fn ty(&self) -> Type {
    match self {
      Self::Int(_) => Type::Int,
      Self::Float(_) => Type::Float,
      Self::String(_) => Type::String,
      Self::Offset { .. } => Type::Pointer(Box::new(Type::Unknown)),
      Self::ArrayItem { source, .. } => {
        match source.ty() {
          Type::Array(_, ty) | Type::Pointer(ty) => *ty,
          _ => Type::Unknown
        }
      }
      Self::LocalRef(_) => Type::Unknown,
      Self::StaticRef(_) => Type::Unknown,
      Self::GlobalRef(_) => Type::Unknown,
      Self::Deref(entry) => {
        match entry.ty() {
          Type::Pointer(ty) | Type::Array(_, ty) => *ty,
          _ => Type::Unknown
        }
      }
      Self::Ref(entry) => Type::Pointer(Box::new(entry.ty())),
      Self::CatchValue => Type::Int,
      Self::BinaryOperator { ty, .. } => ty.clone(),
      Self::UnaryOperator { .. } => Type::Int,
      Self::Cast { ty, .. } => ty.clone(),
      Self::StringHash(_) => Type::Int,
      Self::FunctionCallResult { .. } => Type::Struct,
      Self::NativeCallResult { .. } => Type::Struct,
      Self::Struct { .. } => Type::Struct,
      Self::StructField { .. } => Type::Unknown
    }
  }

  pub fn size(&self) -> usize {
    match self {
      Self::Struct { values } => values.len(),
      Self::FunctionCallResult { return_values, .. } => *return_values,
      Self::NativeCallResult { return_values, .. } => *return_values,
      _ => 1
    }
  }

  pub fn split_off(mut self) -> (Self, Option<Self>) {
    // Avoid unnecessary clone
    if self.size() == 1 {
      return (self, None);
    }

    let cloned = self.clone();
    match &mut self {
      Self::Struct { values } => {
        values.pop().expect("corrupted stack entry");
        let field = Self::StructField {
          source: Box::new(cloned),
          field:  values.len()
        };
        (field, Some(self))
      }
      Self::FunctionCallResult { return_values, .. }
      | Self::NativeCallResult { return_values, .. } => {
        let field = Self::StructField {
          source: Box::new(cloned),
          field:  return_values.checked_sub(1).expect("corrupted stack entry")
        };

        *return_values -= 1;

        (field, if *return_values > 0 { Some(self) } else { None })
      }
      _ => panic!("StackEntry::size(&self) is not implemented correctly")
    }
  }
}

#[derive(Debug, Error)]
#[error("Value cannot be split off")]
pub struct SplitOffError;

#[derive(Clone, Debug)]
pub enum Type {
  Int,
  Float,
  String,
  Pointer(Box<Type>),
  Array(usize, Box<Type>),
  Struct,
  Bool,
  Unknown
}

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
  BitTest
}

#[derive(Copy, Clone, Debug)]
pub enum UnaryOperator {
  Not,
  Negate
}
