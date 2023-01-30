mod ysc;

pub use ysc::*;

#[derive(Debug)]
pub struct ScriptInfo {
  pub name:            String,
  pub name_hash:       u32,
  pub globals_version: u32,
  pub parameter_count: u32
}

#[derive(Debug)]
pub struct Script {
  pub header:  ScriptInfo,
  pub code:    Vec<u8>,
  pub strings: Vec<u8>
}
