use crate::disassembler::{Instruction, InstructionInfo};

pub struct AssemblyFormatter {
  pub include_offset:    bool,
  pub max_bytes_to_show: usize
}

impl AssemblyFormatter {
  pub fn format(&self, code: &[u8], instructions: &[InstructionInfo]) -> String {
    let mut lines: Vec<String> = Default::default();
    let mut fn_counter = 0;

    for info in instructions {
      let bytes = create_byte_string(code, self.max_bytes_to_show, info.pos, info.size);
      let bytes_len = bytes.len();

      let prefix = if self.include_offset {
        format!("{:08X} {bytes}", info.pos)
      } else {
        bytes
      };

      let prefix_without_bytes = if self.include_offset {
        format!("{:08X} {}", info.pos, " ".repeat(bytes_len))
      } else {
        " ".repeat(bytes_len)
      };

      match &info.instruction {
        Instruction::Nop => lines.push(format!("{prefix}\tNOP")),
        Instruction::IntegerAdd => lines.push(format!("{prefix}\tIADD")),
        Instruction::IntegerSubtract => lines.push(format!("{prefix}\tISUB")),
        Instruction::IntegerMultiply => lines.push(format!("{prefix}\tIMUL")),
        Instruction::IntegerDivide => lines.push(format!("{prefix}\tIDIV")),
        Instruction::IntegerModulo => lines.push(format!("{prefix}\tIMOD")),
        Instruction::IntegerNot => lines.push(format!("{prefix}\tINOT")),
        Instruction::IntegerNegate => lines.push(format!("{prefix}\tINEG")),
        Instruction::IntegerEquals => lines.push(format!("{prefix}\tIEQ")),
        Instruction::IntegerNotEquals => lines.push(format!("{prefix}\tINE")),
        Instruction::IntegerGreaterThan => lines.push(format!("{prefix}\tIGT")),
        Instruction::IntegerGreaterOrEqual => lines.push(format!("{prefix}\tIGE")),
        Instruction::IntegerLowerThan => lines.push(format!("{prefix}\tILT")),
        Instruction::IntegerLowerOrEqual => lines.push(format!("{prefix}\tILE")),
        Instruction::FloatAdd => lines.push(format!("{prefix}\tFADD")),
        Instruction::FloatSubtract => lines.push(format!("{prefix}\tFSUB")),
        Instruction::FloatMultiply => lines.push(format!("{prefix}\tFMUL")),
        Instruction::FloatDivide => lines.push(format!("{prefix}\tFDIV")),
        Instruction::FloatModule => lines.push(format!("{prefix}\tFMOD")),
        Instruction::FloatNegate => lines.push(format!("{prefix}\tFNEG")),
        Instruction::FloatEquals => lines.push(format!("{prefix}\tFEQ")),
        Instruction::FloatNotEquals => lines.push(format!("{prefix}\tFNE")),
        Instruction::FloatGreaterThan => lines.push(format!("{prefix}\tFGT")),
        Instruction::FloatGreaterOrEqual => lines.push(format!("{prefix}\tFGE")),
        Instruction::FloatLowerThan => lines.push(format!("{prefix}\tFLT")),
        Instruction::FloatLowerOrEqual => lines.push(format!("{prefix}\tFLE")),
        Instruction::VectorAdd => lines.push(format!("{prefix}\tVADD")),
        Instruction::VectorSubtract => lines.push(format!("{prefix}\tVSUB")),
        Instruction::VectorMultiply => lines.push(format!("{prefix}\tVMUL")),
        Instruction::VectorDivide => lines.push(format!("{prefix}\tVDIV")),
        Instruction::VectorNegate => lines.push(format!("{prefix}\tVNEG")),
        Instruction::BitwiseAnd => lines.push(format!("{prefix}\tIAND")),
        Instruction::BitwiseOr => lines.push(format!("{prefix}\tIOR")),
        Instruction::BitwiseXor => lines.push(format!("{prefix}\tIXOR")),
        Instruction::IntegerToFloat => lines.push(format!("{prefix}\tI2F")),
        Instruction::FloatToInteger => lines.push(format!("{prefix}\tF2I")),
        Instruction::FloatToVector => lines.push(format!("{prefix}\tF2V")),
        Instruction::PushConstU8 { c1 } => lines.push(format!("{prefix}\tPUSH_CONST_U8 {c1}")),
        Instruction::PushConstU8U8 { c1, c2 } => {
          lines.push(format!("{prefix}\tPUSH_CONST_U8_U8 {c1} {c2}"))
        }
        Instruction::PushConstU8U8U8 { c1, c2, c3 } => {
          lines.push(format!("{prefix}\tPUSH_CONST_U8_U8_U8 {c1} {c2} {c3}"))
        }
        Instruction::PushConstU32 { c1 } => lines.push(format!("{prefix}\tPUSH_CONST_U32 {c1}")),
        Instruction::PushConstFloat { c1 } => lines.push(format!("{prefix}\tPUSH_CONST_F {c1}")),
        Instruction::Dup => lines.push(format!("{prefix}\tDUP")),
        Instruction::Drop => lines.push(format!("{prefix}\tDROP")),
        Instruction::NativeCall {
          arg_count,
          return_count,
          native_index
        } => {
          lines.push(format!(
            "{prefix}\tNATIVE {arg_count} {return_count} {native_index}"
          ))
        }
        Instruction::Enter {
          parameter_count,
          var_count,
          name
        } => {
          fn_counter += 1;
          let display_name = name.clone().unwrap_or_else(|| format!("func_{fn_counter}"));

          lines.extend_from_slice(&[
            prefix_without_bytes.clone(),
            format!("{prefix_without_bytes}; ========== S U B R O U T I N E =========="),
            prefix_without_bytes.clone(),
            format!("{prefix_without_bytes}; {display_name}"),
            if let Some(name) = name {
              format!("{prefix}\tENTER {parameter_count} {var_count} \"{name}\"")
            } else {
              format!("{prefix}\tENTER {parameter_count} {var_count}")
            }
          ])
        }
        Instruction::Leave {
          parameter_count,
          return_count
        } => lines.push(format!("{prefix}\tLEAVE {parameter_count} {return_count}")),

        Instruction::Load => lines.push(format!("{prefix}\tLOAD")),
        Instruction::Store => lines.push(format!("{prefix}\tSTORE")),
        Instruction::StoreRev => lines.push(format!("{prefix}\tSTORE_REV")),
        Instruction::LoadN => lines.push(format!("{prefix}\tLOAD_N")),
        Instruction::StoreN => lines.push(format!("{prefix}\tSTORE_N")),
        Instruction::ArrayU8 { item_size } => lines.push(format!("{prefix}\tARRAY_U8 {item_size}")),
        Instruction::ArrayU8Load { item_size } => {
          lines.push(format!("{prefix}\tARRAY_U8_LOAD {item_size}"))
        }
        Instruction::ArrayU8Store { item_size } => {
          lines.push(format!("{prefix}\tARRAY_U8_STORE {item_size}"))
        }
        Instruction::LocalU8 { local_index } => {
          lines.push(format!("{prefix}\tLOCAL_U8 {local_index}"))
        }
        Instruction::LocalU8Load { local_index } => {
          lines.push(format!("{prefix}\tLOCAL_U8_LOAD {local_index}"))
        }
        Instruction::LocalU8Store { local_index } => {
          lines.push(format!("{prefix}\tLOCAL_U8_STORE {local_index}"))
        }
        Instruction::StaticU8 { static_index } => {
          lines.push(format!("{prefix}\tSTATIC_U8 {static_index}"))
        }
        Instruction::StaticU8Load { static_index } => {
          lines.push(format!("{prefix}\tSTATIC_U8_LOAD {static_index}"))
        }
        Instruction::StaticU8Store { static_index } => {
          lines.push(format!("{prefix}\tSTATIC_U8_STORE {static_index}"))
        }
        Instruction::AddU8 { value } => lines.push(format!("{prefix}\tIADD_U8 {value}")),
        Instruction::MultiplyU8 { value } => lines.push(format!("{prefix}\tIMUL_U8 {value}")),
        Instruction::Offset => lines.push(format!("{prefix}\tIOFFSET")),
        Instruction::OffsetU8 { offset } => lines.push(format!("{prefix}\tIOFFSET_U8 {offset}")),
        Instruction::OffsetU8Load { offset } => {
          lines.push(format!("{prefix}\tIOFFSET_U8_LOAD {offset}"))
        }
        Instruction::OffsetU8Store { offset } => {
          lines.push(format!("{prefix}\tIOFFSET_U8_STORE {offset}"))
        }
        Instruction::PushConstS16 { c1 } => lines.push(format!("{prefix}\tPUSH_CONST_S16 {c1}")),
        Instruction::AddS16 { value } => lines.push(format!("{prefix}\tIADD_S16 {value}")),
        Instruction::MultiplyS16 { value } => lines.push(format!("{prefix}\tIMUL_S16 {value}")),
        Instruction::OffsetS16 { offset } => lines.push(format!("{prefix}\tIOFFSET_S16 {offset}")),
        Instruction::OffsetS16Load { offset } => {
          lines.push(format!("{prefix}\tIOFFSET_S16_LOAD {offset}"))
        }
        Instruction::OffsetS16Store { offset } => {
          lines.push(format!("{prefix}\tIOFFSET_S16_STORE {offset}"))
        }
        Instruction::ArrayU16 { item_size } => {
          lines.push(format!("{prefix}\tARRAY_U16 {item_size}"))
        }
        Instruction::ArrayU16Load { item_size } => {
          lines.push(format!("{prefix}\tARRAY_U16_LOAD {item_size}"))
        }
        Instruction::ArrayU16Store { item_size } => {
          lines.push(format!("{prefix}\tARRAY_U16_STORE {item_size}"))
        }
        Instruction::LocalU16 { local_index } => {
          lines.push(format!("{prefix}\tLOCAL_U16 {local_index}"))
        }
        Instruction::LocalU16Load { local_index } => {
          lines.push(format!("{prefix}\tLOCAL_U16_LOAD {local_index}"))
        }
        Instruction::LocalU16Store { local_index } => {
          lines.push(format!("{prefix}\tLOCAL_U16_STORE {local_index}"))
        }
        Instruction::StaticU16 { static_index } => {
          lines.push(format!("{prefix}\tSTATIC_U16 {static_index}"))
        }
        Instruction::StaticU16Load { static_index } => {
          lines.push(format!("{prefix}\tSTATIC_U16_LOAD {static_index}"))
        }
        Instruction::StaticU16Store { static_index } => {
          lines.push(format!("{prefix}\tSTATIC_U16_STORE {static_index}"))
        }
        Instruction::GlobalU16 { global_index } => {
          lines.push(format!("{prefix}\tGLOBAL_U16 {global_index}"))
        }
        Instruction::GlobalU16Load { global_index } => {
          lines.push(format!("{prefix}\tGLOBAL_U16_LOAD {global_index}"))
        }
        Instruction::GlobalU16Store { global_index } => {
          lines.push(format!("{prefix}\tGLOBAL_U16_STORE {global_index}"))
        }
        Instruction::Jump { location } => lines.push(format!("{prefix}\tJ loc_{location}")),
        Instruction::JumpZero { location } => lines.push(format!("{prefix}\tJZ loc_{location}")),
        Instruction::IfEqualJump { location } => {
          lines.push(format!("{prefix}\tIEQ_JZ loc_{location}"))
        }
        Instruction::IfNotEqualJump { location } => {
          lines.push(format!("{prefix}\tINE_JZ loc_{location}"))
        }
        Instruction::IfGreaterThanJump { location } => {
          lines.push(format!("{prefix}\tIGT_JZ loc_{location}"))
        }
        Instruction::IfGreaterOrEqualJump { location } => {
          lines.push(format!("{prefix}\tIGE_JZ loc_{location}"))
        }
        Instruction::IfLowerThanJump { location } => {
          lines.push(format!("{prefix}\tILT_JZ loc_{location}"))
        }
        Instruction::IfLowerOrEqualJump { location } => {
          lines.push(format!("{prefix}\tILE_JZ loc_{location}"))
        }
        Instruction::FunctionCall { location } => {
          lines.push(format!("{prefix}\tCALL loc_{location}"))
        }
        Instruction::StaticU24 { static_index } => {
          lines.push(format!("{prefix}\tSTATIC_U24 {static_index}"))
        }
        Instruction::StaticU24Load { static_index } => {
          lines.push(format!("{prefix}\tSTATIC_U24_LOAD {static_index}"))
        }
        Instruction::StaticU24Store { static_index } => {
          lines.push(format!("{prefix}\tSTATIC_U24_STORE {static_index}"))
        }
        Instruction::GlobalU24 { global_index } => {
          lines.push(format!("{prefix}\tGLOBAL_U24 {global_index}"))
        }
        Instruction::GlobalU24Load { global_index } => {
          lines.push(format!("{prefix}\tGLOBAL_U24_LOAD {global_index}"))
        }
        Instruction::GlobalU24Store { global_index } => {
          lines.push(format!("{prefix}\tGLOBAL_U24_STORE {global_index}"))
        }
        Instruction::PushConstU24 { c1 } => lines.push(format!("{prefix}\tPUSH_CONST_U24 {c1}")),
        Instruction::Switch { cases } => {
          lines.push(format!("{prefix}\tSWITCH"));
          lines.extend(cases.iter().map(|(value, location)| {
            format!("{prefix_without_bytes}\t\tCASE 0x{value:08X}:loc_{location} ; {value}")
          }))
        }
        Instruction::String => lines.push(format!("{prefix}\tSTRING")),
        Instruction::StringHash => lines.push(format!("{prefix}\tSTRING_HASH")),
        Instruction::TextLabelAssignString { buffer_size } => {
          lines.push(format!("{prefix}\tTEXT_LABEL_ASSIGN_STRING {buffer_size}"))
        }
        Instruction::TextLabelAssignInt { buffer_size } => {
          lines.push(format!("{prefix}\tTEXT_LABEL_ASSIGN_INT {buffer_size}"))
        }
        Instruction::TextLabelAppendString { buffer_size } => {
          lines.push(format!("{prefix}\tTEXT_LABEL_APPEND_STRING {buffer_size}"))
        }
        Instruction::TextLabelAppendInt { buffer_size } => {
          lines.push(format!("{prefix}\tTEXT_LABEL_APPEND_INT {buffer_size}"))
        }
        Instruction::TextLabelCopy => lines.push(format!("{prefix}\tTEXT_LABEL_COPY")),
        Instruction::Catch => lines.push(format!("{prefix}\tCATCH")),
        Instruction::Throw => lines.push(format!("{prefix}\tTHROW")),
        Instruction::CallIndirect => lines.push(format!("{prefix}\tCALLINDIRECT")),
        Instruction::PushConstM1 => lines.push(format!("{prefix}\tPUSH_CONST_M1")),
        Instruction::PushConst0 => lines.push(format!("{prefix}\tPUSH_CONST_0")),
        Instruction::PushConst1 => lines.push(format!("{prefix}\tPUSH_CONST_1")),
        Instruction::PushConst2 => lines.push(format!("{prefix}\tPUSH_CONST_2")),
        Instruction::PushConst3 => lines.push(format!("{prefix}\tPUSH_CONST_3")),
        Instruction::PushConst4 => lines.push(format!("{prefix}\tPUSH_CONST_4")),
        Instruction::PushConst5 => lines.push(format!("{prefix}\tPUSH_CONST_5")),
        Instruction::PushConst6 => lines.push(format!("{prefix}\tPUSH_CONST_6")),
        Instruction::PushConst7 => lines.push(format!("{prefix}\tPUSH_CONST_7")),
        Instruction::PushConstFm1 => lines.push(format!("{prefix}\tPUSH_CONST_FM1")),
        Instruction::PushConstF0 => lines.push(format!("{prefix}\tPUSH_CONST_F0")),
        Instruction::PushConstF1 => lines.push(format!("{prefix}\tPUSH_CONST_F1")),
        Instruction::PushConstF2 => lines.push(format!("{prefix}\tPUSH_CONST_F2")),
        Instruction::PushConstF3 => lines.push(format!("{prefix}\tPUSH_CONST_F3")),
        Instruction::PushConstF4 => lines.push(format!("{prefix}\tPUSH_CONST_F4")),
        Instruction::PushConstF5 => lines.push(format!("{prefix}\tPUSH_CONST_F5")),
        Instruction::PushConstF6 => lines.push(format!("{prefix}\tPUSH_CONST_F6")),
        Instruction::PushConstF7 => lines.push(format!("{prefix}\tPUSH_CONST_F7")),
        Instruction::BitTest => lines.push(format!("{prefix}\tBITTEST"))
      }
    }

    lines.join("\n")
  }
}

// Terrible code, please refactor :)
fn create_byte_string(code: &[u8], max_bytes: usize, offset: usize, count: usize) -> String {
  if max_bytes == 0 {
    return "".to_owned();
  }

  let bytes_too_many = count.saturating_sub(max_bytes);
  let marker_len = bytes_too_many
    .checked_ilog10()
    .map(|n| n + 3)
    .unwrap_or_default();
  let additional_bytes_to_remove = marker_len.div_ceil(3) as usize;

  let real_bytes_too_many = bytes_too_many + additional_bytes_to_remove;
  let real_marker_len = bytes_too_many
    .checked_ilog10()
    .map(|n| n + 3)
    .unwrap_or_default();
  let bytes_to_hide = real_marker_len.div_ceil(3) as usize;

  let num_bytes = usize::min(count, max_bytes).saturating_sub(bytes_to_hide);

  let mut bytes = code[offset..offset + num_bytes]
    .iter()
    .map(|byte| format!("{byte:02X} "))
    .collect::<String>();
  bytes.truncate(bytes.len().saturating_sub(1));

  let mut result = if real_bytes_too_many > 0 {
    format!("{bytes} +{real_bytes_too_many}")
  } else {
    bytes
  };

  let max_len = max_bytes * 3 - 1;
  if result.len() > max_len {
    result.truncate(max_len - 1);
    result += "…";
  } else {
    result += &" ".repeat(max_len - result.len());
  }
  result += " ";

  result
}
