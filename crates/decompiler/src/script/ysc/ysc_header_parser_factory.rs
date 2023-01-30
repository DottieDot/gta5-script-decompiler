use thiserror::Error;

use super::{header_parsers::PcYscHeaderParser, YscHeaderParser};

pub struct YscHeaderParserFactory;

impl YscHeaderParserFactory {
  pub fn create(bytes: &[u8]) -> Result<Box<dyn YscHeaderParser>, UnknownMagicError> {
    let is_rsc7 = u32::from_le_bytes(bytes[..4].try_into().unwrap()) == 0x37435352;
    let offset = if is_rsc7 { 0x10 } else { 0 };

    let magic = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());

    let parser = match (magic, magic & 0xFFFF) {
      (0x52ACB3A8 | 0x9204B3A8, _) | (_, 0xA588 | 0xB0B8) => PcYscHeaderParser,
      _ => return Err(UnknownMagicError { magic })
    };

    Ok(Box::new(parser))
  }
}

#[derive(Error, Debug)]
#[error("Invalid magic 0x{magic:08X}")]
pub struct UnknownMagicError {
  pub magic: u32
}
