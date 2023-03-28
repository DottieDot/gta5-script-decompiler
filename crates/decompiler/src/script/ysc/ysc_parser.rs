use std::{cmp, fs, io, path::Path};

use thiserror::Error;

use crate::{
  disassembler::opcodes::Opcode,
  script::{Script, ScriptInfo}
};

use super::{OpcodeVersion, UnknownMagicError, YscHeaderParserFactory};

pub fn parse_ysc(bytes: &[u8]) -> Result<Script, ParseYscError> {
  let header_parser = YscHeaderParserFactory::create(bytes)?;
  let header = header_parser.parse(bytes)?;

  let mut code = flatten_table(
    bytes,
    header.code_size as usize,
    &header
      .code_table_offsets
      .iter()
      .map(|i| *i as usize)
      .collect::<Vec<_>>(),
    0x4000
  );
  patch_opcodes(header_parser.opcode_version(), &mut code)?;

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

  Ok(Script {
    header: ScriptInfo {
      name:            header.script_name,
      name_hash:       header.name_hash,
      globals_version: header.globals_version,
      parameter_count: header.parameter_count
    },
    code,
    strings
  })
}

pub fn parse_ysc_file(path: impl AsRef<Path> + ToString) -> Result<Script, ParseYscFileError> {
  let path_ref = path.as_ref();

  let contents = fs::read(path_ref).map_err(|e| {
    ParseYscFileError::ReadFileError {
      path:   path_ref.to_str().map(str::to_owned),
      source: e
    }
  })?;

  parse_ysc(&contents).map_err(|e| {
    ParseYscFileError::ParseError {
      path:   path_ref.to_str().map(str::to_owned),
      source: e
    }
  })
}

fn flatten_table(
  bytes: &[u8],
  total_size: usize,
  block_offsets: &[usize],
  block_size: usize
) -> Vec<u8> {
  block_offsets
    .iter()
    .enumerate()
    .flat_map(|(index, offset)| {
      let to_take = cmp::min(total_size - index * block_size, block_size);
      let end_offset = *offset + to_take;
      bytes[(*offset)..end_offset].to_vec()
    })
    .collect::<Vec<_>>()
}

#[derive(Error, Debug)]
pub enum ParseYscError {
  #[error("{source}")]
  InvalidMagic {
    #[from]
    source: UnknownMagicError
  },

  #[error("Failed to parse ysc header: {source}")]
  FailedToParseHeader {
    #[from]
    source: anyhow::Error
  },

  #[error("Invalid opcode {opcode} at {position}")]
  InvalidOpcode { opcode: u8, position: usize }
}

#[derive(Error, Debug)]
pub enum ParseYscFileError {
  #[error("Failed to parse ysc file {path:?}: {source}")]
  ParseError {
    path:   Option<String>,
    source: ParseYscError
  },

  #[error("Failed to open ysc file {path:?}: {source}")]
  ReadFileError {
    path:   Option<String>,
    source: io::Error
  }
}

fn patch_opcodes(version: OpcodeVersion, bytes: &mut [u8]) -> Result<(), ParseYscError> {
  let mut i = 0;
  while i < bytes.len() {
    let byte = &mut bytes[i];
    if version <= OpcodeVersion::B2802 {
      if *byte >= Opcode::StaticU24.into() {
        *byte += 3; // StaticU24, StaticU24Load, StaticU24Store
      }
    }

    let opcode = Opcode::try_from(*byte)
      .map_err(|_| {
        ParseYscError::InvalidOpcode {
          opcode:   *byte,
          position: i
        }
      })
      .unwrap();
    i += opcode.size(&bytes[i..]) as usize;
  }

  Ok(())
}
