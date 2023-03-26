use std::{cmp, fs, io, path::Path, string::FromUtf8Error};

use thiserror::Error;

use crate::script::{Script, ScriptInfo};

use super::{UnknownMagicError, YscHeaderParserFactory};

pub fn parse_ysc(bytes: &[u8]) -> Result<Script, ParseYscError> {
  let header_parser = YscHeaderParserFactory::create(bytes)?;
  let header = header_parser.parse(bytes)?;

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
  }
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
