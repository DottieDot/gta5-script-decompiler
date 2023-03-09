use std::{io, string::FromUtf8Error};

use binary_reader::{BinaryReader, Endian};
use thiserror::Error;

use self::opcodes::Opcode;

mod instruction;
mod instruction_info;
pub(self) mod opcodes;

pub use instruction::*;
pub use instruction_info::*;

pub fn disassemble(code: &[u8]) -> Result<Vec<InstructionInfo>, DisassembleError> {
  let mut result: Vec<InstructionInfo> = Default::default();

  let mut reader = BinaryReader::from_u8(code);
  reader.set_endian(Endian::Little);

  while reader.pos != reader.length {
    let start_pos = reader.pos;
    let raw_opcode = reader.read_u8()?;
    let instruction = match Opcode::try_from(raw_opcode).map_err(|e| {
      DisassembleError::ReadInstructionError {
        input:  raw_opcode,
        source: e
      }
    })? {
      Opcode::Nop => Instruction::Nop,
      Opcode::IntegerAdd => Instruction::IntegerAdd,
      Opcode::IntegerSubtract => Instruction::IntegerSubtract,
      Opcode::IntegerMultiply => Instruction::IntegerMultiply,
      Opcode::IntegerDivide => Instruction::IntegerDivide,
      Opcode::IntegerModulo => Instruction::IntegerModulo,
      Opcode::IntegerNot => Instruction::IntegerNot,
      Opcode::IntegerNegate => Instruction::IntegerNegate,
      Opcode::IntegerEquals => Instruction::IntegerEquals,
      Opcode::IntegerNotEquals => Instruction::IntegerNotEquals,
      Opcode::IntegerGreaterThan => Instruction::IntegerGreaterThan,
      Opcode::IntegerGreaterOrEqual => Instruction::IntegerGreaterOrEqual,
      Opcode::IntegerLowerThan => Instruction::IntegerLowerThan,
      Opcode::IntegerLowerOrEqual => Instruction::IntegerLowerOrEqual,
      Opcode::FloatAdd => Instruction::FloatAdd,
      Opcode::FloatSubtract => Instruction::FloatSubtract,
      Opcode::FloatMultiply => Instruction::FloatMultiply,
      Opcode::FloatDivide => Instruction::FloatDivide,
      Opcode::FloatModule => Instruction::FloatModule,
      Opcode::FloatNegate => Instruction::FloatNegate,
      Opcode::FloatEquals => Instruction::FloatEquals,
      Opcode::FloatNotEquals => Instruction::FloatNotEquals,
      Opcode::FloatGreaterThan => Instruction::FloatGreaterThan,
      Opcode::FloatGreaterOrEqual => Instruction::FloatGreaterOrEqual,
      Opcode::FloatLowerThan => Instruction::FloatLowerThan,
      Opcode::FloatLowerOrEqual => Instruction::FloatLowerOrEqual,
      Opcode::VectorAdd => Instruction::VectorAdd,
      Opcode::VectorSubtract => Instruction::VectorSubtract,
      Opcode::VectorMultiply => Instruction::VectorMultiply,
      Opcode::VectorDivide => Instruction::VectorDivide,
      Opcode::VectorNegate => Instruction::VectorNegate,
      Opcode::BitwiseAnd => Instruction::BitwiseAnd,
      Opcode::BitwiseOr => Instruction::BitwiseOr,
      Opcode::BitwiseXor => Instruction::BitwiseXor,
      Opcode::IntegerToFloat => Instruction::IntegerToFloat,
      Opcode::FloatToInteger => Instruction::FloatToInteger,
      Opcode::FloatToVector => Instruction::FloatToVector,
      Opcode::PushConstU8 => {
        Instruction::PushConstU8 {
          c1: reader.read_u8()?
        }
      }
      Opcode::PushConstU8U8 => {
        Instruction::PushConstU8U8 {
          c1: reader.read_u8()?,
          c2: reader.read_u8()?
        }
      }
      Opcode::PushConstU8U8U8 => {
        Instruction::PushConstU8U8U8 {
          c1: reader.read_u8()?,
          c2: reader.read_u8()?,
          c3: reader.read_u8()?
        }
      }
      Opcode::PushConstU32 => {
        Instruction::PushConstU32 {
          c1: reader.read_u32()?
        }
      }
      Opcode::PushConstFloat => {
        Instruction::PushConstFloat {
          c1: reader.read_f32()?
        }
      }
      Opcode::Dup => Instruction::Dup,
      Opcode::Drop => Instruction::Drop,
      Opcode::NativeCall => {
        let val = reader.read_u8()?;
        let return_count = val & 0b00000011;
        let arg_count = val & 0b11111100;
        Instruction::NativeCall {
          arg_count,
          return_count,
          native_index: reader.read_u16()?
        }
      }
      Opcode::Enter => {
        Instruction::Enter {
          arg_count:  reader.read_u8()?,
          frame_size: reader.read_u16()?,
          name:       {
            let length = reader.read_u8()?;
            if length == 0 {
              None
            } else {
              Some(
                String::from_utf8(reader.read_bytes(length as usize)?.to_vec()).map_err(|e| {
                  DisassembleError::InvalidFunctionNameError {
                    pos:    reader.pos,
                    source: e
                  }
                })?
              )
            }
          }
        }
      }
      Opcode::Leave => {
        Instruction::Leave {
          parameter_count: reader.read_u8()?,
          return_count:    reader.read_u8()?
        }
      }
      Opcode::Load => Instruction::Load,
      Opcode::Store => Instruction::Store,
      Opcode::StoreRev => Instruction::StoreRev,
      Opcode::LoadN => Instruction::LoadN,
      Opcode::StoreN => Instruction::StoreN,
      Opcode::ArrayU8 => {
        Instruction::ArrayU8 {
          item_size: reader.read_u8()?
        }
      }
      Opcode::ArrayU8Load => {
        Instruction::ArrayU8Load {
          item_size: reader.read_u8()?
        }
      }
      Opcode::ArrayU8Store => {
        Instruction::ArrayU8Store {
          item_size: reader.read_u8()?
        }
      }
      Opcode::LocalU8 => {
        Instruction::LocalU8 {
          offset: reader.read_u8()?
        }
      }
      Opcode::LocalU8Load => {
        Instruction::LocalU8Load {
          offset: reader.read_u8()?
        }
      }
      Opcode::LocalU8Store => {
        Instruction::LocalU8Store {
          offset: reader.read_u8()?
        }
      }
      Opcode::StaticU8 => {
        Instruction::StaticU8 {
          static_index: reader.read_u8()?
        }
      }
      Opcode::StaticU8Load => {
        Instruction::StaticU8Load {
          static_index: reader.read_u8()?
        }
      }
      Opcode::StaticU8Store => {
        Instruction::StaticU8Store {
          static_index: reader.read_u8()?
        }
      }
      Opcode::AddU8 => {
        Instruction::AddU8 {
          value: reader.read_u8()?
        }
      }
      Opcode::MultiplyU8 => {
        Instruction::MultiplyU8 {
          value: reader.read_u8()?
        }
      }
      Opcode::Offset => Instruction::Offset,
      Opcode::OffsetU8 => {
        Instruction::OffsetU8 {
          offset: reader.read_u8()?
        }
      }
      Opcode::OffsetU8Load => {
        Instruction::OffsetU8Load {
          offset: reader.read_u8()?
        }
      }
      Opcode::OffsetU8Store => {
        Instruction::OffsetU8Store {
          offset: reader.read_u8()?
        }
      }
      Opcode::PushConstS16 => {
        Instruction::PushConstS16 {
          c1: reader.read_i16()?
        }
      }
      Opcode::AddS16 => {
        Instruction::AddS16 {
          value: reader.read_i16()?
        }
      }
      Opcode::MultiplyS16 => {
        Instruction::MultiplyS16 {
          value: reader.read_i16()?
        }
      }
      Opcode::OffsetS16 => {
        Instruction::OffsetS16 {
          offset: reader.read_i16()?
        }
      }
      Opcode::OffsetS16Load => {
        Instruction::OffsetS16Load {
          offset: reader.read_i16()?
        }
      }
      Opcode::OffsetS16Store => {
        Instruction::OffsetS16Store {
          offset: reader.read_i16()?
        }
      }
      Opcode::ArrayU16 => {
        Instruction::ArrayU16 {
          item_size: reader.read_u16()?
        }
      }
      Opcode::ArrayU16Load => {
        Instruction::ArrayU16Load {
          item_size: reader.read_u16()?
        }
      }
      Opcode::ArrayU16Store => {
        Instruction::ArrayU16Store {
          item_size: reader.read_u16()?
        }
      }
      Opcode::LocalU16 => {
        Instruction::LocalU16 {
          local_index: reader.read_u16()?
        }
      }
      Opcode::LocalU16Load => {
        Instruction::LocalU16Load {
          local_index: reader.read_u16()?
        }
      }
      Opcode::LocalU16Store => {
        Instruction::LocalU16Store {
          local_index: reader.read_u16()?
        }
      }
      Opcode::StaticU16 => {
        Instruction::StaticU16 {
          static_index: reader.read_u16()?
        }
      }
      Opcode::StaticU16Load => {
        Instruction::StaticU16Load {
          static_index: reader.read_u16()?
        }
      }
      Opcode::StaticU16Store => {
        Instruction::StaticU16Store {
          static_index: reader.read_u16()?
        }
      }
      Opcode::GlobalU16 => {
        Instruction::GlobalU16 {
          global_index: reader.read_u16()?
        }
      }
      Opcode::GlobalU16Load => {
        Instruction::GlobalU16Load {
          global_index: reader.read_u16()?
        }
      }
      Opcode::GlobalU16Store => {
        Instruction::GlobalU16Store {
          global_index: reader.read_u16()?
        }
      }
      Opcode::Jump => {
        Instruction::Jump {
          location: get_jump_address(&mut reader)?
        }
      }
      Opcode::JumpZero => {
        Instruction::JumpZero {
          location: get_jump_address(&mut reader)?
        }
      }
      Opcode::IfEqualJumpZero => {
        Instruction::IfEqualJumpZero {
          location: get_jump_address(&mut reader)?
        }
      }
      Opcode::IfNotEqualJumpZero => {
        Instruction::IfNotEqualJumpZero {
          location: get_jump_address(&mut reader)?
        }
      }
      Opcode::IfGreaterThanJumpZero => {
        Instruction::IfGreaterThanJumpZero {
          location: get_jump_address(&mut reader)?
        }
      }
      Opcode::IfGreaterOrEqualJumpZero => {
        Instruction::IfGreaterOrEqualJumpZero {
          location: get_jump_address(&mut reader)?
        }
      }
      Opcode::IfLowerThanJumpZero => {
        Instruction::IfLowerThanJumpZero {
          location: get_jump_address(&mut reader)?
        }
      }
      Opcode::IfLowerOrEqualJumpZero => {
        Instruction::IfLowerOrEqualJumpZero {
          location: get_jump_address(&mut reader)?
        }
      }
      Opcode::FunctionCall => {
        Instruction::FunctionCall {
          location: reader.read_u24()?
        }
      }
      Opcode::StaticU24 => {
        Instruction::StaticU24 {
          static_index: reader.read_u24()?
        }
      }
      Opcode::StaticU24Load => {
        Instruction::StaticU24Load {
          static_index: reader.read_u24()?
        }
      }
      Opcode::StaticU24Store => {
        Instruction::StaticU24Store {
          static_index: reader.read_u24()?
        }
      }
      Opcode::GlobalU24 => {
        Instruction::GlobalU24 {
          global_index: reader.read_u24()?
        }
      }
      Opcode::GlobalU24Load => {
        Instruction::GlobalU24Load {
          global_index: reader.read_u24()?
        }
      }
      Opcode::GlobalU24Store => {
        Instruction::GlobalU24Store {
          global_index: reader.read_u24()?
        }
      }
      Opcode::PushConstU24 => {
        Instruction::PushConstU24 {
          c1: reader.read_u24()?
        }
      }
      Opcode::Switch => {
        Instruction::Switch {
          cases: {
            let count = reader.read_u8()?;
            (0..count)
              .map(|_| {
                reader
                  .read_u32()
                  .map_err(DisassembleError::from)
                  .and_then(|v| {
                    get_jump_address(&mut reader).map(|v2| {
                      SwitchCase {
                        value:    v,
                        location: v2
                      }
                    })
                  })
              })
              .collect::<Result<_, _>>()?
          }
        }
      }
      Opcode::String => Instruction::String,
      Opcode::StringHash => Instruction::StringHash,
      Opcode::TextLabelAssignString => {
        Instruction::TextLabelAssignString {
          buffer_size: reader.read_u8()?
        }
      }
      Opcode::TextLabelAssignInt => {
        Instruction::TextLabelAssignInt {
          buffer_size: reader.read_u8()?
        }
      }
      Opcode::TextLabelAppendString => {
        Instruction::TextLabelAppendString {
          buffer_size: reader.read_u8()?
        }
      }
      Opcode::TextLabelAppendInt => {
        Instruction::TextLabelAppendInt {
          buffer_size: reader.read_u8()?
        }
      }
      Opcode::TextLabelCopy => Instruction::TextLabelCopy,
      Opcode::Catch => Instruction::Catch,
      Opcode::Throw => Instruction::Throw,
      Opcode::CallIndirect => Instruction::CallIndirect,
      Opcode::PushConstM1 => Instruction::PushConstM1,
      Opcode::PushConst0 => Instruction::PushConst0,
      Opcode::PushConst1 => Instruction::PushConst1,
      Opcode::PushConst2 => Instruction::PushConst2,
      Opcode::PushConst3 => Instruction::PushConst3,
      Opcode::PushConst4 => Instruction::PushConst4,
      Opcode::PushConst5 => Instruction::PushConst5,
      Opcode::PushConst6 => Instruction::PushConst6,
      Opcode::PushConst7 => Instruction::PushConst7,
      Opcode::PushConstFm1 => Instruction::PushConstFm1,
      Opcode::PushConstF0 => Instruction::PushConstF0,
      Opcode::PushConstF1 => Instruction::PushConstF1,
      Opcode::PushConstF2 => Instruction::PushConstF2,
      Opcode::PushConstF3 => Instruction::PushConstF3,
      Opcode::PushConstF4 => Instruction::PushConstF4,
      Opcode::PushConstF5 => Instruction::PushConstF5,
      Opcode::PushConstF6 => Instruction::PushConstF6,
      Opcode::PushConstF7 => Instruction::PushConstF7,
      Opcode::BitTest => Instruction::BitTest
    };
    result.push(InstructionInfo {
      instruction,
      pos: start_pos,
      size: reader.pos - start_pos
    });
  }

  Ok(result)
}

fn get_jump_address(reader: &mut BinaryReader) -> Result<u32, DisassembleError> {
  let offset = reader.read_i16()?;
  Ok(
    add_i16_to_usize(reader.pos, offset).ok_or(DisassembleError::InvalidJump {
      pos: reader.pos,
      offset
    })? as u32
  )
}

fn add_i16_to_usize(usize: usize, i16: i16) -> Option<usize> {
  if i16 < 0 {
    usize.checked_sub(-i16 as usize)
  } else {
    usize.checked_add(i16 as usize)
  }
}

#[derive(Debug, Error)]
pub enum DisassembleError {
  #[error("{} is not a recognized instruction", input)]
  ReadInstructionError {
    input:  u8,
    #[source]
    source: <Opcode as TryFrom<u8>>::Error
  },

  #[error("Read error: {}", source)]
  ReadError {
    #[source]
    #[from]
    source: io::Error
  },

  #[error("Invalid jump offset at: {}, with offset: {}", pos, offset)]
  InvalidJump { pos: usize, offset: i16 },

  #[error("Failed to parse function name at: {}", pos)]
  InvalidFunctionNameError {
    pos:    usize,
    #[source]
    source: FromUtf8Error
  }
}
