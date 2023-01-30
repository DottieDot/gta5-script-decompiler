use super::YscScriptHeader;

pub trait YscHeaderParser {
  fn parse(&self, bytes: &[u8]) -> anyhow::Result<YscScriptHeader>;
}
