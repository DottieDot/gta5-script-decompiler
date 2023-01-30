use thiserror::Error;

use super::{header_parsers::PcYscHeaderParser, YscHeaderParser};

pub struct YscHeaderParserFactory;

impl YscHeaderParserFactory {
  pub fn create(bytes: &[u8]) -> Result<Box<dyn YscHeaderParser>, UnknownMagicError> {
    let magic = u32::from_le_bytes(bytes[..4].try_into().unwrap());

    let parser = match (magic, magic & 0xFFFF) {
      (0x4005BE20 | 0x405A9ED0, _) | (_, 0xA588 | 0xB0B8) => PcYscHeaderParser,
      _ => return Err(UnknownMagicError { magic }),
    };

    Ok(Box::new(parser))
  }
}

#[derive(Error, Debug)]
#[error("Invalid magic {magic}")]
pub struct UnknownMagicError {
  pub magic: u32,
}
