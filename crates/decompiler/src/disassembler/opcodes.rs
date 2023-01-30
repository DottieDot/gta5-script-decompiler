use num_enum::TryFromPrimitive;

#[repr(u8)]
#[derive(TryFromPrimitive, PartialEq, Eq, Clone, Copy)]
pub enum Opcode {
  Nop,
  IntegerAdd,
  IntegerSubtract,
  IntegerMultiply,
  IntegerDivide,
  IntegerModulo,
  IntegerNot,
  IntegerNegate,
  IntegerEquals,
  IntegerNotEquals,
  IntegerGreaterThan,
  IntegerGreaterOrEqual,
  IntegerLowerThan,
  IntegerLowerOrEqual,
  FloatAdd,
  FloatSubtract,
  FloatMultiply,
  FloatDivide,
  FloatModule,
  FloatNegate,
  FloatEquals,
  FloatNotEquals,
  FloatGreaterThan,
  FloatGreaterOrEqual,
  FloatLowerThan,
  FloatLowerOrEqual,
  VectorAdd,
  VectorSubtract,
  VectorMultiply,
  VectorDivide,
  VectorNegate,
  BitwiseAnd,
  BitwiseOr,
  BitwiseXor,
  IntegerToFloat,
  FloatToInteger,
  FloatToVector,
  PushConstU8,
  PushConstU8U8,
  PushConstU8U8U8,
  PushConstU32,
  PushConstFloat,
  Dup,
  Drop,
  NativeCall,
  Enter,
  Leave,
  Load,
  Store,
  StoreRev,
  LoadN,
  StoreN,
  ArrayU8,
  ArrayU8Load,
  ArrayU8Store,
  LocalU8,
  LocalU8Load,
  LocalU8Store,
  StaticU8,
  StaticU8Load,
  StaticU8Store,
  AddU8,
  MultiplyU8,
  Offset,
  OffsetU8,
  OffsetU8Load,
  OffsetU8Store,
  PushConstS16,
  AddS16,
  MultiplyS16,
  OffsetS16,
  OffsetS16Load,
  OffsetS16Store,
  ArrayU16,
  ArrayU16Load,
  ArrayU16Store,
  LocalU16,
  LocalU16Load,
  LocalU16Store,
  StaticU16,
  StaticU16Load,
  StaticU16Store,
  GlobalU16,
  GlobalU16Load,
  GlobalU16Store,
  Jump,
  JumpZero,
  IfEqualJump,
  IfNotEqualJump,
  IfGreaterThanJump,
  IfGreaterOrEqualJump,
  IfLowerThanJump,
  IfLowerOrEqualJump,
  FunctionCall,
  StaticU24,
  StaticU24Load,
  StaticU24Store,
  GlobalU24,
  GlobalU24Load,
  GlobalU24Store,
  PushConstU24,
  Switch,
  String,
  StringHash,
  TextLabelAssignString,
  TextLabelAssignInt,
  TextLabelAppendString,
  TextLabelAppendInt,
  TextLabelCopy,
  Catch,
  Throw,
  CallIndirect,
  PushConstM1,
  PushConst0,
  PushConst1,
  PushConst2,
  PushConst3,
  PushConst4,
  PushConst5,
  PushConst6,
  PushConst7,
  PushConstFm1,
  PushConstF0,
  PushConstF1,
  PushConstF2,
  PushConstF3,
  PushConstF4,
  PushConstF5,
  PushConstF6,
  PushConstF7,
  BitTest,
}
