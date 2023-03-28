use thiserror::Error;

use super::{header_parsers::PcYscHeaderParser, OpcodeVersion, YscHeaderParser};

pub struct YscHeaderParserFactory;

impl YscHeaderParserFactory {
  pub fn create(bytes: &[u8]) -> Result<Box<dyn YscHeaderParser>, UnknownMagicError> {
    let is_rsc7 = u32::from_le_bytes(bytes[..4].try_into().unwrap()) == 0x37435352;
    let offset = if is_rsc7 { 0x10 } else { 0 };

    let magic = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());

    let parser = match magic & 0xFFFF {
      0xB0B8 => PcYscHeaderParser::new(OpcodeVersion::B2628), // GTA V b2628
      0x2699 => PcYscHeaderParser::new(OpcodeVersion::B2699), // GTA V b2699
      0xB3A8 => PcYscHeaderParser::new(OpcodeVersion::B2802), // GTA V b2802
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
