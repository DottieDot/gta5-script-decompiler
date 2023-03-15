use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum StackEntry {
  Int(i64),
  Float(f32),
  String(usize),
  LocalPointer(usize),
  StaticPointer(usize),
  GlobalPointer(usize),
  Offset {
    source: Box<StackEntry>,
    offset: usize
  },
  ArrayItem {
    source:    Box<StackEntry>,
    item_size: usize,
    index:     usize
  },
  ArrayItemPointer {
    source:    Box<StackEntry>,
    item_size: usize,
    index:     usize
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
    ty:  Type
  },
  UnaryOperator {
    lhs:   Box<StackEntry>,
    value: i64
  },
  Cast {
    value: Box<StackEntry>,
    ty:    Type
  },
  StringHash(Box<StackEntry>),
  Throw(Box<StackEntry>),
  CallIndirect(Box<StackEntry>),

  // TODO: MULTIPLE RETURN VALUES!
  NativeCallResult {
    args: Rc<Vec<StackEntry>>,
    ty:   Type
  },
  FunctionCallResult {
    args: Rc<Vec<StackEntry>>,
    ty:   Type
  }
}

impl StackEntry {
  pub fn ty(&self) -> Type {
    match self {
      StackEntry::Int(_) => Type::Int,
      StackEntry::Float(_) => Type::Float,
      StackEntry::String(_) => Type::String,
      StackEntry::LocalPointer(_) => Type::Pointer(Box::new(Type::Unknown)),
      StackEntry::StaticPointer(_) => Type::Pointer(Box::new(Type::Unknown)),
      StackEntry::GlobalPointer(_) => Type::Pointer(Box::new(Type::Unknown)),
      StackEntry::Offset { .. } => Type::Pointer(Box::new(Type::Unknown)),
      StackEntry::ArrayItem { source, .. } => {
        match source.ty() {
          Type::Array(_, ty) | Type::Pointer(ty) => *ty,
          _ => Type::Unknown
        }
      }
      StackEntry::ArrayItemPointer { source, .. } => {
        match source.ty() {
          Type::Array(_, ty) | Type::Pointer(ty) => Type::Pointer(ty),
          _ => Type::Pointer(Box::new(Type::Unknown))
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
      StackEntry::StringHash(_) => Type::Int,
      StackEntry::Throw(_) => Type::Int,
      StackEntry::CallIndirect(_) => Type::Function,
      StackEntry::NativeCallResult { ty, .. } => ty.clone(),
      StackEntry::FunctionCallResult { ty, .. } => ty.clone()
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
  Function,
  Unknown
}
