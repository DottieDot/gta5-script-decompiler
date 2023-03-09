/// Represents a ysc instruction.
///
/// Details based on <https://github.com/alexguirre/gtav-sc-tools/blob/master/docs/InstructionSet.md>.
#[derive(Clone, Debug)]
pub enum Instruction {
  /// # Description
  /// No operation
  Nop,

  /// # Mnemonic
  /// IADD
  ///
  ///  # Description
  /// Adds `i1` and `i2`.
  ///
  /// # Stack
  /// `i1 i2 -> i3`
  IntegerAdd,

  /// # Mnemonic
  /// ISUB
  ///
  /// # Description
  /// Subtract `i2` from `i1`.
  ///
  /// # Stack
  /// `i1 i2 -> i3`
  IntegerSubtract,

  /// # Mnemonic
  /// IMUL
  ///
  /// # Description
  /// Multiplies `i1` with `i2.
  ///
  /// # Stack
  /// `i1 i2 -> i3`
  IntegerMultiply,

  /// # Mnemonic
  /// IDIV
  ///
  /// # Description
  /// Divide `i1` by `i2`.
  ///
  /// # Stack
  /// `i1 i2 -> i3`
  IntegerDivide,

  /// # Mnemonic
  /// IMOD
  ///
  /// # Description
  /// Divide `i1` by `i2` and push the remainder on the stack.
  ///
  /// # Stack
  /// `i1 i2 -> i3`
  IntegerModulo,

  /// # Mnemonic
  /// INOT
  ///
  /// # Description
  /// Logical operation of `i1`: if `i1 == 0`, `1` is pushed on the stack; otherwise `0` is pushed.
  ///
  /// # Stack
  /// `i1 -> i2`
  IntegerNot,

  /// # Mnemonic
  /// INEG
  ///
  /// # Description
  /// Negates i1.
  ///
  /// # Stack
  /// `i1 -> i2`
  IntegerNegate,

  /// # Mnemonic
  /// IEQ
  ///
  /// # Description
  /// Checks if `i1` is equal to `i2`.
  ///
  /// # Stack
  /// `i1 i2 -> flag`
  IntegerEquals,

  /// # Mnemonic
  /// INE
  ///
  /// # Description
  /// Checks if `i1` is equal to `i2`.
  ///
  /// # Stack
  /// `i1 i2 -> flag`
  IntegerNotEquals,

  /// # Mnemonic
  /// IGT
  ///
  /// # Description
  /// Checks if `i1` is greater than `i2`.
  ///
  /// # Stack
  /// `i1 i2 -> flag`
  IntegerGreaterThan,

  /// # Mnemonic
  /// IGE
  ///
  /// # Description
  /// Checks if `i1` is greater or equal to `i2`.
  ///
  /// # Stack
  /// `i1 i2 -> flag`
  IntegerGreaterOrEqual,

  /// # Mnemonic
  /// ILT
  ///
  /// # Description
  /// Checks if `i1` is lower than `i2`.
  ///
  /// # Stack
  /// `i1 i2 -> flag`
  IntegerLowerThan,

  /// # Mnemonic
  /// ILE
  ///
  /// # Description
  /// Checks if `i1` is lower or equal to `i2`.
  ///
  /// # Stack
  /// `i1 i2 -> flag`
  IntegerLowerOrEqual,

  /// # Mnemonic
  /// FADD
  ///
  /// # Description
  /// Adds `f1` to `f2`.
  ///
  /// # Stack
  /// `f1 f2 -> f3`
  FloatAdd,

  /// # Mnemonic
  /// FSUB
  ///
  /// # Description
  /// Subtracts `f2` from `f1`.
  ///
  /// # Stack
  /// `f1 f2 -> f3`
  FloatSubtract,

  /// # Mnemonic
  /// FMUL
  ///
  /// # Description
  /// Multiplies `f1` by `f2`.
  ///
  /// # Stack
  /// `f1 f2 -> f3`
  FloatMultiply,

  /// # Mnemonic
  /// FDIV
  ///
  /// # Description
  /// Divides `f1` by `f2`.
  ///
  /// # Stack
  /// `f1 f2 -> f3`
  FloatDivide,

  /// # Mnemonic
  /// FMOD
  ///
  /// # Description
  /// Divides `f1` by `f2` and pushes the remainder on the stack.
  ///
  /// # Stack
  /// `f1 f2 -> f3`
  FloatModule,

  /// # Mnemonic
  /// FNEG
  ///
  /// # Description
  /// Negates `f1`.
  ///
  /// # Stack
  /// `f1 -> f2`
  FloatNegate,

  /// # Mnemonic
  /// FEQ
  ///
  /// # Description
  /// Checks if `f1` is equal to `f2`.
  ///
  /// # Stack
  /// `f1 f2 -> flag`
  FloatEquals,

  /// # Mnemonic
  /// FNE
  ///
  /// # Description
  /// Checks if `f1` is not equal to `f2`.
  ///
  /// # Stack
  /// `f1 f2 -> flag`
  FloatNotEquals,

  /// # Mnemonic
  /// FGT
  ///
  /// # Description
  /// Checks if `f1` is greater than `f2`.
  ///
  /// # Stack
  /// `f1 f2 -> flag`
  FloatGreaterThan,

  /// # Mnemonic
  /// FGE
  ///
  /// # Description
  /// Checks if `f1` is greater or equal to `f2`.
  ///
  /// # Stack
  /// `f1 f2 -> flag`
  FloatGreaterOrEqual,

  /// # Mnemonic
  /// FLT
  ///
  /// # Description
  /// Checks if `f1` is lower than `f2`.
  ///
  /// # Stack
  /// `f1 f2 -> flag`
  FloatLowerThan,

  /// # Mnemonic
  /// FLE
  ///
  /// # Description
  /// Checks if `f1` is lower or equal to `f2`.
  ///
  /// # Stack
  /// `f1 f2 -> flag`
  FloatLowerOrEqual,

  /// # Mnemonic
  /// VADD
  ///
  /// # Description
  /// Adds `v1` and `v2`.
  ///
  /// # Stack
  /// `v1 v2 -> v3`
  VectorAdd,

  /// # Mnemonic
  /// VSUB
  ///
  /// # Description
  /// Subtracts `v2` from `v1`.
  ///
  /// # Stack
  /// `v1 v2 -> v3`
  VectorSubtract,

  /// # Mnemonic
  /// VMUL
  ///
  /// # Description
  /// Multiplies `v1` by `v2`.
  ///
  /// # Stack
  /// `v1 v2 -> v3`
  VectorMultiply,

  /// # Mnemonic
  /// FDIV
  ///
  /// # Description
  /// Divides `v1` by `v2`.
  ///
  /// # Stack
  /// `v1 v2 -> v3`
  VectorDivide,

  /// # Mnemonic
  /// VNEG
  ///
  /// # Description
  /// Negates `v1`.
  ///
  /// # Stack
  /// `v1 -> v2`
  VectorNegate,

  /// # Mnemonic
  /// IAND
  ///
  /// # Description
  /// Bitwise and on `i1` and `i2`.
  ///
  /// # Stack
  /// `i1 i2 -> i3`
  BitwiseAnd,

  /// # Mnemonic
  /// IOR
  ///
  /// # Description
  /// Bitwise or on `i1` and `i2`.
  ///
  /// # Stack
  /// `i1 i2 -> i3`
  BitwiseOr,

  /// # Mnemonic
  /// IXOR
  ///
  /// # Description
  /// Bitwise xor on `i1` and `i2`.
  ///
  /// # Stack
  /// `i1 i2 -> i3`
  BitwiseXor,

  /// # Mnemonic
  /// I2F
  ///
  /// # Description
  /// Converts `i1` to a float.
  ///
  /// # Stack
  /// `i1 -> f1`
  IntegerToFloat,

  /// # Mnemonic
  /// F2I
  ///
  /// # Description
  /// Converts `f1` to an integer.
  ///
  /// # Stack
  /// `f1 -> i1`
  FloatToInteger,

  /// # Mnemonic
  /// F2V
  ///
  /// # Description
  /// Converts `f1` to a vector by duplicating it twice.
  ///
  /// # Stack
  /// `f1 -> f1 f1 f1`
  FloatToVector,

  /// # Mnemonic
  /// PUSH_CONST_U8
  ///
  /// # Description
  /// Pushes it's operand on the stack.
  ///
  /// # Stack
  /// `-> c1`
  PushConstU8 {
    /// The value to push on the stack.
    c1: u8
  },

  /// # Mnemonic
  /// PUSH_CONST_U8_U8
  ///
  /// # Description
  /// Pushes it's operands on the stack.
  ///
  /// # Stack
  /// `-> c1 c2`
  PushConstU8U8 {
    /// The 1st value to push on the stack.
    c1: u8,

    /// The 2nd value to push on the stack.
    c2: u8
  },

  /// # Mnemonic
  /// PUSH_CONST_U8_U8_U8
  ///
  /// # Description
  /// Pushes it's operand on the stack.
  ///
  /// # Stack
  /// `-> c1 c2 c3`
  PushConstU8U8U8 {
    /// The 1st value to push on the stack.
    c1: u8,

    /// The 2nd value to push on the stack.
    c2: u8,

    /// The 3rd value to push on the stack.
    c3: u8
  },

  /// # Mnemonic
  /// PUSH_CONST_U32
  ///
  /// # Description
  /// Pushes it's operand on the stack.
  ///
  /// # Stack
  /// `-> c1`
  PushConstU32 {
    /// The value to push on the stack.
    c1: u32
  },

  /// # Mnemonic
  /// PUSH_CONST_F
  ///
  /// # Description
  /// Pushes it's operand on the stack.
  ///
  /// # Stack
  /// `-> c1`
  PushConstFloat {
    /// The value to push on the stack.
    c1: f32
  },

  /// # Mnemonic
  /// DUP
  ///
  /// # Description
  /// Duplicates the top value on the stack.
  ///
  /// # Stack
  /// `v1 -> v1 v1`
  Dup,

  /// # Mnemonic
  /// DROP
  ///
  /// # Description
  /// Pops the stack.
  ///
  /// # Stack
  /// `v1 ->`
  Drop,

  /// # Mnemonic
  /// NATIVE
  ///
  /// # Description
  /// Calls the native command at the `native_index` with specified number of arguments and pushes the result on the stack.
  ///
  /// # Stack
  /// `arg1...argN -> return1...returnN`
  NativeCall {
    /// The number of arguments the native accepts.
    arg_count: u8,

    /// The number of values the native returns.
    return_count: u8,

    /// The index of the native to call.
    native_index: u16
  },

  /// # Mnemonic
  /// ENTER
  ///
  /// # Description
  /// Pushes the offset of the `callerFrame` on the stack and advances the stack by `frame_size - arg_count + 1`.
  /// This creates space for locals (initialized to `0`) and sets the new frame to start at `arg1`.
  ///
  /// # Stack
  /// `arg1...argN returnAddr -> [arg1...argN returnAddr callerFrame local1...localN empty]`
  Enter {
    /// The number of parameters the function accepts.
    arg_count: u8,

    /// The size of the frame required for the function.
    ///
    /// For correct behavior this should be `parameter_count` + 2 (for the `returnAddr` and `callerFrame`) + the number of locals, so `frame_size = arg_count + 2 + num_locals`.
    frame_size: u16,

    /// The name of the function. This is removed from production builds.
    name: Option<String>
  },

  Leave {
    parameter_count: u8,
    return_count:    u8
  },

  /// # Mnemonic
  /// LOAD
  ///
  /// # Description
  /// Dereferences `addr1` and pushes the value on the stack.
  ///
  /// # Stack
  /// `addr1 -> v1`
  Load,

  /// # Mnemonic
  /// STORE
  ///
  /// # Description
  /// Dereferences `addr1` and sets the value at the address to `v1`.
  ///
  /// # Stack
  /// `v1 addr1 ->`
  Store,

  /// # Mnemonic
  /// STORE_REV
  ///
  /// # Description
  /// Dereferences `addr1` and sets the value at the address to `v1`, without removing `addr1` from the stack.
  ///
  /// # Stack
  /// `v1 addr1 -> addr1`
  StoreRev,

  /// # Mnemonic
  /// LOAD_N
  ///
  /// # Description
  /// Reads `N` values from pointer `addr1` and pushes the mon the stack.
  ///
  /// # Stack
  /// `N addr1 -> v1..vN`
  LoadN,

  /// # Mnemonic
  /// STORE_N
  ///
  /// # Description
  /// Writes `N` variables to pointer `addr1`
  ///
  /// # Stack
  /// `v1..vN N addr1 ->`
  StoreN,

  /// # Mnemonic
  /// ARRAY_U8
  ///
  /// # Description
  /// Pushes the address of the item at `index` in `arrayAddr` on the stack.
  ///
  /// Effectively does `arrayAddr + (index * item_size)`.
  ///
  /// # Stack
  /// `index arrayAddr -> addr1`
  ArrayU8 {
    /// The size of the items in the array.
    item_size: u8
  },

  /// # Mnemonic
  /// ARRAY_U8_LOAD
  ///
  /// # Description
  /// Pushes the value of the item at `index` in `arrayAddr` on the stack.
  ///
  /// Effectively does `arrayAddr + (index * item_size)` and dereferences it.
  ///
  /// # Stack
  /// `index arrayAddr -> v1`
  ArrayU8Load { item_size: u8 },

  /// # Mnemonic
  /// ARRAY_U8_STORES
  ///
  /// # Description
  /// Sets item at `index` in `arrayAddr` to `v1`.
  ///
  /// Effectively does `arrayAddr + (index * item_size)`, dereferences it, and sets it to `v1`.
  ///
  /// # Stack
  /// `v1 index arrayAddr ->`
  ArrayU8Store { item_size: u8 },

  /// # Mnemonic
  /// LOCAL_U8
  ///
  /// # Description
  /// Pushes the address of the frame offset `offset` on the stack.
  ///
  /// # Stack
  /// `-> addr1`
  LocalU8 { offset: u8 },

  /// # Mnemonic
  /// LOCAL_U8_LOAD
  ///
  /// # Description
  /// Pushes the value at the frame offset `offset` on the stack.
  ///
  /// # Stack
  /// `-> v1`
  LocalU8Load { offset: u8 },

  /// # Mnemonic
  /// LOCAL_U8_STORE
  ///
  /// # Description
  /// Sets the value at the frame offset `offset` to `v1`.
  ///
  /// # Stack
  /// `v1 ->`
  LocalU8Store { offset: u8 },

  /// # Mnemonic
  /// STATIC_U8
  ///
  /// # Description
  /// Pushes the address of the static offset `static_index` on the stack.
  ///
  /// # Stack
  /// `-> addr1`
  StaticU8 { static_index: u8 },

  /// # Mnemonic
  /// STATIC_U8_LOAD
  ///
  /// # Description
  /// Pushes the value at the static offset `static_index` on the stack.
  ///
  /// # Stack
  /// `-> v1`
  StaticU8Load { static_index: u8 },

  /// # Mnemonic
  /// STATIC_U8_STORE
  ///
  /// # Description
  /// Sets the value at the static offset `static_index`.
  ///
  /// # Stack
  /// `v1 ->`
  StaticU8Store { static_index: u8 },

  /// # Mnemonic
  /// IADD_U8
  ///
  /// # Description
  /// Adds `value` to the value on top of the stack.
  ///
  /// # Stack
  /// `n1 -> n2`
  AddU8 { value: u8 },

  /// # Mnemonic
  /// IMUL_U8
  ///
  /// # Description
  /// Multiplies the value on top of the stack with `value`.
  ///
  /// # Stack
  /// `n1 -> n2`
  MultiplyU8 { value: u8 },

  /// # Mnemonic
  /// IOFFSET
  ///
  /// # Description
  /// Offsets `addr` by `n1` (signed).
  ///
  /// # Stack
  /// `addr1 n1 -> addr2`
  Offset,

  /// # Mnemonic
  /// IOFFSET_U8
  ///
  /// # Description
  /// Offsets `addr` by `offset`.
  ///
  /// # Stack
  /// `addr1 -> addr2`
  OffsetU8 { offset: u8 },

  /// # Mnemonic
  /// IOFFSET_U8_LOAD
  ///
  /// # Description
  /// Offsets `addr` by `offset` and dereferences it.
  ///
  /// # Stack
  /// `addr1 -> v1`
  OffsetU8Load { offset: u8 },

  /// # Mnemonic
  /// IOFFSET_U8_LOAD
  ///
  /// # Description
  /// Offsets `addr` by `offset` and sets the value to `v1`.
  ///
  /// # Stack
  /// `v1 addr1 ->`
  OffsetU8Store { offset: u8 },

  /// # Mnemonic
  /// PUSH_CONST_S16
  ///
  /// # Description
  /// Pushes `c1` on the stack.
  ///
  /// # Stack
  /// `-> c1`
  PushConstS16 { c1: i16 },

  /// # Mnemonic
  /// IADD_S16
  ///
  /// # Description
  /// Adds `value` to the value on top of the stack.
  ///
  /// # Stack
  /// `n1 -> n2`
  AddS16 { value: i16 },

  /// # Mnemonic
  /// IMUL_S16
  ///
  /// # Description
  /// Multiplies the value on top of the stack with `value`.
  ///
  /// # Stack
  /// `n1 -> n2`
  MultiplyS16 { value: i16 },

  /// # Mnemonic
  /// IOFFSET_S16
  ///
  /// # Description
  /// Offsets `addr` by `offset`.
  ///
  /// # Stack
  /// `addr1 -> addr2`
  OffsetS16 { offset: i16 },

  /// # Mnemonic
  /// IOFFSET_S16_LOAD
  ///
  /// # Description
  /// Offsets `addr` by `offset` and dereferences it.
  ///
  /// # Stack
  /// `addr1 -> v1`
  OffsetS16Load { offset: i16 },

  /// # Mnemonic
  /// IOFFSET_S16_STORE
  ///
  /// # Description
  /// Offsets `addr` by `offset` and sets its value to `v1`.
  ///
  /// # Stack
  /// `v1 addr1 -> addr2`
  OffsetS16Store { offset: i16 },

  /// # Mnemonic
  /// ARRAY_U16
  ///
  /// # Description
  /// Pushes the address of the item at `index` in `arrayAddr` on the stack.
  ///
  /// Effectively does `arrayAddr + (index * item_size)`.
  ///
  /// # Stack
  /// `index arrayAddr -> addr1`
  ArrayU16 {
    /// The size of the items in the array.
    item_size: u16
  },

  /// # Mnemonic
  /// ARRAY_U16_LOAD
  ///
  /// # Description
  /// Pushes the value of the item at `index` in `arrayAddr` on the stack.
  ///
  /// Effectively does `arrayAddr + (index * item_size)` and dereferences it.
  ///
  /// # Stack
  /// `index arrayAddr -> v1`
  ArrayU16Load {
    /// The size of the items in the array.
    item_size: u16
  },

  /// # Mnemonic
  /// ARRAY_U8_STORE
  ///
  /// # Description
  /// Pushes the value of the item at `index` in `arrayAddr` on the stack.
  ///
  /// Effectively does `arrayAddr + (index * item_size)` and dereferences it.
  ///
  /// # Stack
  /// `index arrayAddr -> v1`
  ArrayU16Store {
    /// The size of the items in the array.
    item_size: u16
  },

  /// # Mnemonic
  /// LOCAL_U16
  ///
  /// # Description
  /// Pushes the address of the frame offset `offset` on the stack.
  ///
  /// # Stack
  /// `-> addr1`
  LocalU16 { local_index: u16 },

  /// # Mnemonic
  /// LOCAL_U16_LOAD
  ///
  /// # Description
  /// Pushes the value at the frame offset `offset` on the stack.
  ///
  /// # Stack
  /// `-> v1`
  LocalU16Load { local_index: u16 },

  /// # Mnemonic
  /// LOCAL_U16_STORE
  ///
  /// # Description
  /// Sets the value at the frame offset `offset` to `v1`.
  ///
  /// # Stack
  /// `v1 ->`
  LocalU16Store { local_index: u16 },

  /// # Mnemonic
  /// STATIC_U16
  ///
  /// # Description
  /// Pushes the address of the static offset `static_index` on the stack.
  ///
  /// # Stack
  /// `-> addr1`
  StaticU16 { static_index: u16 },

  /// # Mnemonic
  /// STATIC_U16_LOAD
  ///
  /// # Description
  /// Pushes the value at the static offset `static_index` on the stack.
  ///
  /// # Stack
  /// `-> v1`
  StaticU16Load { static_index: u16 },

  /// # Mnemonic
  /// STATIC_U16_STORE
  ///
  /// # Description
  /// Sets the value at the static offset `static_index`.
  ///
  /// # Stack
  /// `v1 ->`
  StaticU16Store { static_index: u16 },

  /// # Mnemonic
  /// GLOBAL_U16
  ///
  /// # Description
  /// Pushes the address of the global `global_index` on the stack.
  ///
  /// # Stack
  /// `-> addr1`
  GlobalU16 { global_index: u16 },

  /// # Mnemonic
  /// GLOBAL_U16_LOAD
  ///
  /// # Description
  /// Pushes the value of the global `global_index` on the stack.
  ///
  /// # Stack
  /// `-> v1`
  GlobalU16Load { global_index: u16 },

  /// # Mnemonic
  /// GLOBAL_U16_STORE
  ///
  /// # Description
  /// Sets the value of global `global_index` to `v1`.
  ///
  /// # Stack
  /// `v1 ->`
  GlobalU16Store { global_index: u16 },

  /// # Mnemonic
  /// J
  ///
  /// # Description
  /// Sets the program counter to `location`.
  ///
  /// # Stack
  /// `->`
  Jump { location: u32 },

  /// # Mnemonic
  /// JZ
  ///
  /// # Description
  /// Sets te program counter to `location` if `v1 == 0`.
  ///
  /// # Stack
  /// `v1 ->`
  JumpZero { location: u32 },

  /// # Mnemonic
  /// IEQ_JZ
  ///
  /// # Description
  /// Sets the program counter to `location` if `i1 != i2`.
  ///
  /// # Stack
  /// `i1 i2 ->`
  IfEqualJumpZero { location: u32 },

  /// # Mnemonic
  /// INE_JZ
  ///
  /// # Description
  /// Sets the program counter to `location` if `i1 == i2`.
  ///
  /// # Stack
  /// `i1 i2 ->`
  IfNotEqualJumpZero { location: u32 },

  /// # Mnemonic
  /// IGT_JZ
  ///
  /// # Description
  /// Sets the program counter to `location` if `i1 <= i2`.
  ///
  /// # Stack
  /// `i1 i2 ->`
  IfGreaterThanJumpZero { location: u32 },

  /// # Mnemonic
  /// IGE_JZ
  ///
  /// # Description
  /// Sets the program counter to `location` if `i1 < i2`.
  ///
  /// # Stack
  /// `i1 i2 ->`
  IfGreaterOrEqualJumpZero { location: u32 },

  /// # Mnemonic
  /// IGT_JZ
  ///
  /// # Description
  /// Sets the program counter to `location` if `i1 >= i2`.
  ///
  /// # Stack
  /// `i1 i2 ->`
  IfLowerThanJumpZero { location: u32 },

  /// # Mnemonic
  /// IGT_JZ
  ///
  /// # Description
  /// Sets the program counter to `location` if `i1 > i2`.
  ///
  /// # Stack
  /// `i1 i2 ->`
  IfLowerOrEqualJumpZero { location: u32 },

  /// # Mnemonic
  /// CALL
  ///
  /// # Description
  /// Pushes the address of the next instruction on the stack and sets the program counter to `location`.
  ///
  /// # Stack
  /// `-> returnAddr`
  FunctionCall { location: u32 },

  /// # Mnemonic
  /// STATIC_U24
  ///
  /// # Description
  /// Pushes the address of the static offset `static_index` on the stack.
  ///
  /// # Stack
  /// `-> addr1`
  StaticU24 { static_index: u32 },

  /// # Mnemonic
  /// STATIC_U24_LOAD
  ///
  /// # Description
  /// Pushes the value at the static offset `static_index` on the stack.
  ///
  /// # Stack
  /// `-> v1`
  StaticU24Load { static_index: u32 },

  /// # Mnemonic
  /// STATIC_U24_STORE
  ///
  /// # Description
  /// Sets the value at the static offset `static_index`.
  ///
  /// # Stack
  /// `v1 ->`
  StaticU24Store { static_index: u32 },

  /// # Mnemonic
  /// GLOBAL_U24
  ///
  /// # Description
  /// Pushes the address of the global `global_index` on the stack.
  ///
  /// # Stack
  /// `-> addr1`
  GlobalU24 { global_index: u32 },

  /// # Mnemonic
  /// GLOBAL_U24_LOAD
  ///
  /// # Description
  /// Pushes the value of the global `global_index` on the stack.
  ///
  /// # Stack
  /// `-> v1`
  GlobalU24Load { global_index: u32 },

  /// # Mnemonic
  /// GLOBAL_U24_STORE
  ///
  /// # Description
  /// Sets the value of global `global_index` to `v1`.
  ///
  /// # Stack
  /// `v1 ->`
  GlobalU24Store { global_index: u32 },

  /// # Mnemonic
  /// PUSH_CONST_U24
  ///
  /// # Description
  /// Pushes `c1` on the stack.
  ///
  /// # Stack
  /// `-> c1`
  PushConstU24 { c1: u32 },

  /// # Mnemonic
  /// SWITCH
  ///
  /// # Description
  /// If `i1` matches any of the `cases` the program counter is set to the location of the case.
  /// If `i1` doesn't match with any case, it simply continues.
  ///
  /// # Stack
  /// `i1 ->`
  Switch { cases: Vec<SwitchCase> },

  /// # Mnemonic
  /// STRING
  ///
  /// # Description
  /// Pushes the address of the string at `string_index` on the stack.
  ///
  /// # Stack
  /// `string_index -> str1`
  String,

  /// # Mnemonic
  /// STRINGHASH
  ///
  /// # Description
  /// Calculates a JOAAT (Jenkins One-At-A-Time) hash for the string on top of the stack.
  ///
  /// # Stack
  /// `str1 -> i1`
  StringHash,

  /// # Mnemonic
  /// TEXT_LABEL_ASSIGN_STRING
  ///
  /// # Description
  /// Copies `str1` into `addr1`, with a maximum of length of `buffer_size` (including the null terminator).
  ///
  /// # Stack
  /// `str1 addr1 ->`
  TextLabelAssignString { buffer_size: u8 },

  /// # Mnemonic
  /// TEXT_LABEL_ASSIGN_INT
  ///
  /// # Description
  /// Converts `i1` into a string and copies it into `ddr1`, with a maximum of length of `buffer_size` (including the null terminator).
  ///
  /// # Stack
  /// `i1 addr1 ->`
  TextLabelAssignInt { buffer_size: u8 },

  /// # Mnemonic
  /// TEXT_LABEL_APPEND_STRING
  ///
  /// # Description
  /// Appends `str1` to `str2`, with a total maximum of length of `buffer_size` (including the null terminator).
  ///
  /// # Stack
  /// `str1 str2 ->`
  TextLabelAppendString { buffer_size: u8 },

  /// # Mnemonic
  /// TEXT_LABEL_APPEND_INT
  ///
  /// # Description
  /// Converts `i1` to a string and appends it to `str2`, with a total maximum of length of `buffer_size` (including the null terminator).
  ///
  /// # Stack
  /// `i1 str2 ->`
  TextLabelAppendInt { buffer_size: u8 },

  /// # Mnemonic
  /// TEXT_LABEL_COPY
  ///
  /// # Description
  /// Copies `src`to `dest`.
  ///
  /// # Stack
  /// `src1...srcN N destCount destAddr ->`
  TextLabelCopy,

  /// # Mnemonic
  /// CATCH
  ///
  /// # Description
  /// Sets the catch handler for the script to the next instruction, frame, and stack position.
  /// Pushes `-1` on the stack to indicate that no error has occurred.
  /// Only one catch handler can be set for a script, subsequent `CATCH`es will override the stored handler.
  ///
  /// # Stack
  /// `-> -1`
  Catch,

  /// # Mnemonic
  /// THROW
  ///
  /// # Description
  /// Restores the frame and stack position to that of the stack handler, pushes `errorCode` on the stack, and jumps to the catch handler.
  /// If no catch handler is registered the script will terminate.
  /// The `errorCode` should be different from `-1` to indicate an error has ocurred.
  ///
  /// # Stack
  /// `errorCode -> errorCode`
  Throw,

  /// # Mnemonic
  /// CALLINDIRECT
  ///
  /// # Description
  /// Calls the function addressed stored on top of the stack.
  ///
  /// # Stack
  /// `funcAddr -> returnAddr`
  CallIndirect,

  /// # Mnemonic
  /// PUSH_CONST_M1
  ///
  /// # Description
  /// Pushes `-1` on the stack.
  ///
  /// # Stack
  /// `-> -1`
  PushConstM1,

  /// # Mnemonic
  /// PUSH_CONST_0
  ///
  /// # Description
  /// Pushes `0` on the stack.
  ///
  /// # Stack
  /// `-> 0`
  PushConst0,

  /// # Mnemonic
  /// PUSH_CONST_1
  ///
  /// # Description
  /// Pushes `1` on the stack.
  ///
  /// # Stack
  /// `-> 1`
  PushConst1,

  /// # Mnemonic
  /// PUSH_CONST_2
  ///
  /// # Description
  /// Pushes `2` on the stack.
  ///
  /// # Stack
  /// `-> 2`
  PushConst2,

  /// # Mnemonic
  /// PUSH_CONST_3
  ///
  /// # Description
  /// Pushes `3` on the stack.
  ///
  /// # Stack
  /// `-> 3`
  PushConst3,

  /// # Mnemonic
  /// PUSH_CONST_4
  ///
  /// # Description
  /// Pushes `4` on the stack.
  ///
  /// # Stack
  /// `-> 4`
  PushConst4,

  /// # Mnemonic
  /// PUSH_CONST_5
  ///
  /// # Description
  /// Pushes `5` on the stack.
  ///
  /// # Stack
  /// `-> 5`
  PushConst5,

  /// # Mnemonic
  /// PUSH_CONST_6
  ///
  /// # Description
  /// Pushes `6` on the stack.
  ///
  /// # Stack
  /// `-> 6`
  PushConst6,

  /// # Mnemonic
  /// PUSH_CONST_7
  ///
  /// # Description
  /// Pushes `7` on the stack.
  ///
  /// # Stack
  /// `-> 7`
  PushConst7,

  /// # Mnemonic
  /// PUSH_CONST_FM1
  ///
  /// # Description
  /// Pushes `-1.0` on the stack.
  ///
  /// # Stack
  /// `-> -1.0`
  PushConstFm1,

  /// # Mnemonic
  /// PUSH_CONST_F0
  ///
  /// # Description
  /// Pushes `0.0` on the stack.
  ///
  /// # Stack
  /// `-> 0.0`
  PushConstF0,

  /// # Mnemonic
  /// PUSH_CONST_F1
  ///
  /// # Description
  /// Pushes `1.0` on the stack.
  ///
  /// # Stack
  /// `-> 1.0`
  PushConstF1,

  /// # Mnemonic
  /// PUSH_CONST_F2
  ///
  /// # Description
  /// Pushes `2.0` on the stack.
  ///
  /// # Stack
  /// `-> 2.0`
  PushConstF2,

  /// # Mnemonic
  /// PUSH_CONST_F3
  ///
  /// # Description
  /// Pushes `3.0` on the stack.
  ///
  /// # Stack
  /// `-> 3.0`
  PushConstF3,

  /// # Mnemonic
  /// PUSH_CONST_F4
  ///
  /// # Description
  /// Pushes `4.0` on the stack.
  ///
  /// # Stack
  /// `-> 4.0`
  PushConstF4,

  /// # Mnemonic
  /// PUSH_CONST_F5
  ///
  /// # Description
  /// Pushes `5.0` on the stack.
  ///
  /// # Stack
  /// `-> 5.0`
  PushConstF5,

  /// # Mnemonic
  /// PUSH_CONST_F6
  ///
  /// # Description
  /// Pushes `6.0` on the stack.
  ///
  /// # Stack
  /// `-> 6.0`
  PushConstF6,

  /// # Mnemonic
  /// PUSH_CONST_F7
  ///
  /// # Description
  /// Pushes `7.0` on the stack.
  ///
  /// # Stack
  /// `-> 7.0`
  PushConstF7,

  /// # Mnemonic
  /// BITTEST
  ///
  /// # Description
  /// Checks if bit `i2` is set on `i1`.
  /// Sets `flag` to `1` if the check succeeds, `0` if the check fails.
  ///
  /// # Stack
  /// `i1 i2 -> flag`
  BitTest
}

#[derive(Debug, Clone, Copy)]
pub struct SwitchCase {
  pub value:    u32,
  pub location: u32
}
