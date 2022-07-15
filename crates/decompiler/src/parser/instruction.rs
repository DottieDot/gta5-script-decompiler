use strum::EnumString;

#[derive(EnumString, Clone, Debug)]
pub enum Instruction {
  #[strum(serialize = "NOP")]
  Nop,

  #[strum(serialize = "IADD")]
  IntegerAdd,

  #[strum(serialize = "ISUB")]
  IntegerSubtract,

  #[strum(serialize = "IMUL")]
  IntegerMultiply,

  #[strum(serialize = "IDIV")]
  IntegerDivide,

  #[strum(serialize = "IMOD")]
  IntigerModulo,

  #[strum(serialize = "INOT")]
  IntegerNot,

  #[strum(serialize = "INEG")]
  IntegerNegate,

  #[strum(serialize = "IEQ")]
  IntegerEquals,

  #[strum(serialize = "INE")]
  IntegerNotEquals,

  #[strum(serialize = "IGT")]
  IntegerGreaterThan,

  #[strum(serialize = "IGE")]
  IntegerGreaterOrEqual,

  #[strum(serialize = "ILT")]
  IntegerLowerThan,

  #[strum(serialize = "ILE")]
  IntegerLowerOrEqual,

  #[strum(serialize = "FADD")]
  FloatAdd,

  #[strum(serialize = "FSub")]
  FloatSubtract,

  #[strum(serialize = "FMUL")]
  FloatMultiply,

  #[strum(serialize = "FDIV")]
  FloatDivide,

  #[strum(serialize = "FMOD")]
  FloatModule,

  #[strum(serialize = "FNEG")]
  FloatNegate,

  #[strum(serialize = "FEQ")]
  FloatEquals,

  #[strum(serialize = "FNE")]
  FloatNotEquals,

  #[strum(serialize = "FGT")]
  FloatGreaterThan,

  #[strum(serialize = "FGE")]
  FloatGreaterOrEqual,

  #[strum(serialize = "FLT")]
  FloatLowerThan,

  #[strum(serialize = "FLE")]
  FloatLowerOrEqual,

  #[strum(serialize = "VADD")]
  VectorAdd,

  #[strum(serialize = "VSUB")]
  VectorSubtract,

  #[strum(serialize = "VMUL")]
  VectorMultiply,

  #[strum(serialize = "VDIV")]
  VectorDivide,

  #[strum(serialize = "VNEG")]
  VectorNegate,

  #[strum(serialize = "IAND")]
  BitwiseAnd,

  #[strum(serialize = "IOR")]
  BitwiseOr,

  #[strum(serialize = "IXOR")]
  BitwiseXor,

  #[strum(serialize = "I2F")]
  IntegerToFloat,

  #[strum(serialize = "F2I")]
  FloatToInteger,

  #[strum(serialize = "F2V")]
  FloatToVector,

  #[strum(serialize = "PUSH_CONST_U8")]
  PushConstU8(u8),

  #[strum(serialize = "PUSH_CONST_U8_U8")]
  PushConstU8U8(u8, u8),

  #[strum(serialize = "PUSH_CONST_U8_U8_U8")]
  PushConstU8U8U8(u8, u8, u8),

  #[strum(serialize = "PUSH_CONST_U32")]
  PushConstU32(u32),

  #[strum(serialize = "PUSH_CONST_F")]
  PushConstFloat(f32),

  #[strum(serialize = "DUP")]
  Dup,

  #[strum(serialize = "DROP")]
  Drop,

  #[strum(serialize = "NATIVE")]
  NativeCall {
    arg_count:    u8,
    return_count: u8,
    native_index: u16
  },

  #[strum(serialize = "ENTER")]
  Enter {
    paramter_count: u8,
    var_count:      u16,
    name:           Option<String>
  },

  #[strum(serialize = "LEAVE")]
  Leave(u8, u8),

  #[strum(serialize = "LOAD")]
  Load,

  #[strum(serialize = "STORE")]
  Store,

  #[strum(serialize = "STORE_REV")]
  StoreRev,

  #[strum(serialize = "LOAD_N")]
  LoadN,

  #[strum(serialize = "STORE_N")]
  StoreN,

  #[strum(serialize = "ARRAY_U8")]
  ArrayU8(u8),

  #[strum(serialize = "ARRAY_U8_LOAD")]
  ArrayU8Load(u8),

  #[strum(serialize = "ARRAY_U8_STORE")]
  ArrayU8Store(u8),

  #[strum(serialize = "LOCAL_U8")]
  LocalU8(u8),

  #[strum(serialize = "LOCAL_U8_LOAD")]
  LocalU8Load(u8),

  #[strum(serialize = "LOCAL_U8_STORE")]
  LocalU8Store(u8),

  #[strum(serialize = "STATIC_U8")]
  StaticU8(u8),

  #[strum(serialize = "STATIC_U8_LOAD")]
  StaticU8Load(u8),

  #[strum(serialize = "STATIC_U8_STORE")]
  StaticU8Store(u8),

  #[strum(serialize = "IADDU8")]
  AddU8(u8),

  #[strum(serialize = "IMULU8")]
  MultiplyU8(u8),

  #[strum(serialize = "IOFFSET")]
  Offset,

  #[strum(serialize = "IOFFSET_U8")]
  OffsetU8(u8),

  #[strum(serialize = "IOFFSET_U8_LOAD")]
  OffsetU8Load(u8),

  #[strum(serialize = "IOFFSET_U8_STORE")]
  OffsetU8Store(u8),

  #[strum(serialize = "PUSH_CONST_S16")]
  PushConstS16(i16),

  #[strum(serialize = "ADD_S16")]
  AddS16(i16),

  #[strum(serialize = "IMULT_S16")]
  MultiplyS16(i16),

  #[strum(serialize = "IOFFSET_S16")]
  OffsetS16(i16),

  #[strum(serialize = "IOFFSET_S16_LOAD")]
  OffsetS16Load(i16),

  #[strum(serialize = "IOFFSET_S16_STORE")]
  OffsetS16Store(i16),

  #[strum(serialize = "ARRAY_U16")]
  ArrayU16(u16),

  #[strum(serialize = "ARRAY_U16_LOAD")]
  ArrayU16Load(u16),

  #[strum(serialize = "ARRAY_U16_STORE")]
  ArrayU16Store(u16),

  #[strum(serialize = "LOCAL_U16")]
  LocalU16(u16),

  #[strum(serialize = "LOCAL_U16_LOAD")]
  LocalU16Load(u16),

  #[strum(serialize = "LOCAL_U16_STORE")]
  LocalU16Store(u16),

  #[strum(serialize = "STATIC_U16")]
  StaticU16(u16),

  #[strum(serialize = "STATIC_U16_LOAD")]
  StaticU16Load(u16),

  #[strum(serialize = "STATIC_U16_STORE")]
  StaticU16Store(u16),

  #[strum(serialize = "GLOBAL_U16")]
  GlobalU16(u16),

  #[strum(serialize = "GLOBAL_U16_LOAD")]
  GlobalU16Load(u16),

  #[strum(serialize = "GLOBAL_U16_STORE")]
  GlobalU16Store(u16),

  #[strum(serialize = "J")]
  Jump(u32),

  #[strum(serialize = "JZ")]
  JumpZero(u32),

  #[strum(serialize = "IEQ_JZ")]
  IfEqualJump(u32),

  #[strum(serialize = "INE_JZ")]
  IfNotEqualJump(u32),

  #[strum(serialize = "IGT_JZ")]
  IfGreaterThanJump(u32),

  #[strum(serialize = "IGE_JZ")]
  IfGreaterOrEqualJump(u32),

  #[strum(serialize = "ILT_JZ")]
  IfLowerThanJump(u32),

  #[strum(serialize = "ILE_JZ")]
  IfLowerOrEqualJump(u32),

  #[strum(serialize = "CALL")]
  FunctionCall(u32),

  #[strum(serialize = "GLOBAL_U24")]
  GlobalU24(u32),

  #[strum(serialize = "GLOBAL_U24_LOAD")]
  GlobalU24Load(u32),

  #[strum(serialize = "GLOBAL_U24_STORE")]
  GlobalU24Store(u32),

  #[strum(serialize = "PUSH_CONST_U24")]
  PushConstU24(u32),

  #[strum(serialize = "SWITCH")]
  Switch(Vec<(u32, u32)>),

  #[strum(serialize = "STRING")]
  String,

  #[strum(serialize = "STRINGHASH")]
  Stringhash,

  #[strum(serialize = "TEXT_LABEL_ASSIGN_STRING")]
  TextLabelAssignString(u8),

  #[strum(serialize = "TEXT_LABEL_ASSIGN_INT")]
  TextLabelAssignInt(u8),

  #[strum(serialize = "TEXT_LABEL_APPEND_STRING")]
  TextLabelAppendString(u8),

  #[strum(serialize = "TEXT_LABEL_APPEND_INT")]
  TextLabelAppendInt(u8),

  #[strum(serialize = "TEXT_LABEL_COPY")]
  TextLabelCopy,

  #[strum(serialize = "CATCH")]
  Catch,

  #[strum(serialize = "THROW")]
  Throw,

  #[strum(serialize = "CALLINDIRECT")]
  CallIndirect,

  #[strum(serialize = "PUSH_CONST_M1")]
  PushConstM1,

  #[strum(serialize = "PUSH_CONST_0")]
  PushConst0,

  #[strum(serialize = "PUSH_CONST_1")]
  PushConst1,

  #[strum(serialize = "PUSH_CONST_2")]
  PushConst2,

  #[strum(serialize = "PUSH_CONST_3")]
  PushConst3,

  #[strum(serialize = "PUSH_CONST_4")]
  PushConst4,

  #[strum(serialize = "PUSH_CONST_5")]
  PushConst5,

  #[strum(serialize = "PUSH_CONST_6")]
  PushConst6,

  #[strum(serialize = "PUSH_CONST_7")]
  PushConst7,

  #[strum(serialize = "PUSH_CONST_FM1")]
  PushConstFm1,

  #[strum(serialize = "PUSH_CONST_F0")]
  PushConstF0,

  #[strum(serialize = "PUSH_CONST_F1")]
  PushConstF1,

  #[strum(serialize = "PUSH_CONST_F2")]
  PushConstF2,

  #[strum(serialize = "PUSH_CONST_F3")]
  PushConstF3,

  #[strum(serialize = "PUSH_CONST_F4")]
  PushConstF4,

  #[strum(serialize = "PUSH_CONST_F5")]
  PushConstF5,

  #[strum(serialize = "PUSH_CONST_F6")]
  PushConstF6,

  #[strum(serialize = "PUSH_CONST_F7")]
  PushConstF7,

  #[strum(serialize = "BITTEST")]
  BitTest
}
