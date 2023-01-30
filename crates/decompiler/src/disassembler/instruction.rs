#[derive(Clone, Debug)]
pub enum Instruction {
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
  PushConstU8 {
    c1: u8
  },
  PushConstU8U8 {
    c1: u8,
    c2: u8
  },
  PushConstU8U8U8 {
    c1: u8,
    c2: u8,
    c3: u8
  },
  PushConstU32 {
    c1: u32
  },
  PushConstFloat {
    c1: f32
  },
  Dup,
  Drop,
  NativeCall {
    arg_count:    u8,
    return_count: u8,
    native_index: u16
  },
  Enter {
    parameter_count: u8,
    var_count:       u16,
    name:            Option<String>
  },
  Leave {
    parameter_count: u8,
    return_count:    u8
  },
  Load,
  Store,
  StoreRev,
  LoadN,
  StoreN,
  ArrayU8 {
    item_size: u8
  },
  ArrayU8Load {
    item_size: u8
  },
  ArrayU8Store {
    item_size: u8
  },
  LocalU8 {
    local_index: u8
  },
  LocalU8Load {
    local_index: u8
  },
  LocalU8Store {
    local_index: u8
  },
  StaticU8 {
    static_index: u8
  },
  StaticU8Load {
    static_index: u8
  },
  StaticU8Store {
    static_index: u8
  },
  AddU8 {
    value: u8
  },
  MultiplyU8 {
    value: u8
  },
  Offset,

  /// Reference to the offset
  OffsetU8 {
    offset: u8
  },

  /// Reads the value at the offset
  OffsetU8Load {
    offset: u8
  },

  /// Sets the value at the offset
  OffsetU8Store {
    offset: u8
  },

  PushConstS16 {
    c1: i16
  },
  AddS16 {
    value: i16
  },
  MultiplyS16 {
    value: i16
  },
  OffsetS16 {
    offset: i16
  },
  OffsetS16Load {
    offset: i16
  },
  OffsetS16Store {
    offset: i16
  },
  ArrayU16 {
    item_size: u16
  },
  ArrayU16Load {
    item_size: u16
  },
  ArrayU16Store {
    item_size: u16
  },
  LocalU16 {
    local_index: u16
  },
  LocalU16Load {
    local_index: u16
  },
  LocalU16Store {
    local_index: u16
  },
  StaticU16 {
    static_index: u16
  },
  StaticU16Load {
    static_index: u16
  },
  StaticU16Store {
    static_index: u16
  },
  GlobalU16 {
    global_index: u16
  },
  GlobalU16Load {
    global_index: u16
  },
  GlobalU16Store {
    global_index: u16
  },
  Jump {
    location: u32
  },
  JumpZero {
    location: u32
  },
  IfEqualJump {
    location: u32
  },
  IfNotEqualJump {
    location: u32
  },
  IfGreaterThanJump {
    location: u32
  },
  IfGreaterOrEqualJump {
    location: u32
  },
  IfLowerThanJump {
    location: u32
  },
  IfLowerOrEqualJump {
    location: u32
  },
  FunctionCall {
    location: u32
  },
  StaticU24 {
    static_index: u32
  },
  StaticU24Load {
    static_index: u32
  },
  StaticU24Store {
    static_index: u32
  },
  GlobalU24 {
    global_index: u32
  },
  GlobalU24Load {
    global_index: u32
  },
  GlobalU24Store {
    global_index: u32
  },
  PushConstU24 {
    c1: u32
  },
  Switch {
    cases: Vec<(u32, u32)>
  },
  String,
  StringHash,
  TextLabelAssignString {
    buffer_size: u8
  },
  TextLabelAssignInt {
    buffer_size: u8
  },
  TextLabelAppendString {
    buffer_size: u8
  },
  TextLabelAppendInt {
    buffer_size: u8
  },
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
  BitTest
}
