use std::ops::Range;

use thiserror::Error;

use super::{
  parse_assembly, util::flatten_table, Instruction, ParseAssemblyError, ParseScriptHeaderError,
  ScriptHeader
};

#[derive(Debug)]
pub struct Script {
  pub header:       ScriptHeader,
  pub code:         Vec<u8>,
  pub strings:      Vec<u8>,
  pub instructions: Vec<(Range<usize>, Instruction)>
}

impl Script {
  pub fn from_pc_script(bytes: &[u8]) -> Result<Self, ParseScriptError> {
    let header = ScriptHeader::from_pc_header(bytes)?;

    let code = flatten_table(
      bytes,
      header.code_size as usize,
      &header
        .code_table_offsets
        .iter()
        .map(|i| *i as usize)
        .collect::<Vec<_>>(),
      0x4000
    );

    let strings = flatten_table(
      bytes,
      header.strings_size as usize,
      &header
        .string_table_offsets
        .iter()
        .map(|i| *i as usize)
        .collect::<Vec<_>>(),
      0x4000
    );

    let instructions = parse_assembly(&code)?;

    Ok(Script {
      header,
      code,
      strings,
      instructions
    })
  }
}

#[derive(Debug, Error)]
pub enum ParseScriptError {
  #[error("Failed to parse header: {}", source)]
  ParseHeaderError {
    #[from]
    #[source]
    source: ParseScriptHeaderError
  },

  #[error("Failed to parse assembly: {}", source)]
  ParseAssemblyError {
    #[from]
    #[source]
    source: ParseAssemblyError
  }
}
