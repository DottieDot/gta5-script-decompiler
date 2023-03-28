use super::YscScriptHeader;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpcodeVersion {
  B2628,
  B2699,
  B2802
}

pub trait YscHeaderParser {
  fn opcode_version(&self) -> OpcodeVersion;

  fn parse(&self, bytes: &[u8]) -> anyhow::Result<YscScriptHeader>;
}
