#[derive(Clone, Debug)]
pub enum StackEntry {
  Int(i64),
  Float(f32),
  String(usize),
  Offset {
    source: Box<StackEntry>,
    offset: Box<StackEntry>
  },
  ArrayItem {
    source:    Box<StackEntry>,
    index:     Box<StackEntry>,
    item_size: usize
  },
  Local(usize),
  Static(usize),
  Global(usize),
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
  StringHash(Box<StackEntry>) // CallIndirectResult(Box<StackEntry>),

                              // // TODO: MULTIPLE RETURN VALUES!
                              // NativeCallResult {
                              //   args: Rc<Vec<StackEntry>>,
                              //   ty:   Type
                              // },
                              // FunctionCallResult {
                              //   args: Rc<Vec<StackEntry>>,
                              //   ty:   Type
                              // }
}

impl StackEntry {
  pub fn ty(&self) -> Type {
    match self {
      StackEntry::Int(_) => Type::Int,
      StackEntry::Float(_) => Type::Float,
      StackEntry::String(_) => Type::String,
      StackEntry::Offset { .. } => Type::Pointer(Box::new(Type::Unknown)),
      StackEntry::ArrayItem { source, .. } => {
        match source.ty() {
          Type::Array(_, ty) | Type::Pointer(ty) => *ty,
          _ => Type::Unknown
        }
      }
      StackEntry::Local(_) => Type::Unknown,
      StackEntry::Static(_) => Type::Unknown,
      StackEntry::Global(_) => Type::Unknown,
      StackEntry::Deref(entry) => {
        match entry.ty() {
          Type::Pointer(ty) | Type::Array(_, ty) => *ty,
          _ => Type::Unknown
        }
      }
      StackEntry::Ref(entry) => Type::Pointer(Box::new(entry.ty())),
      StackEntry::CatchValue => Type::Int,
      StackEntry::BinaryOperator { ty, .. } => ty.clone(),
      StackEntry::UnaryOperator { .. } => Type::Int,
      StackEntry::Cast { ty, .. } => ty.clone(),
      StackEntry::StringHash(_) => Type::Int
    }
  }
}

#[derive(Clone, Debug)]
pub enum Type {
  Int,
  Float,
  String,
  Pointer(Box<Type>),
  Array(usize, Box<Type>),
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
