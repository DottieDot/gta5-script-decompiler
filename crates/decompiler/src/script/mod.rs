mod ysc;

use std::ffi::CStr;

pub use ysc::*;

#[derive(Debug)]
pub struct ScriptInfo {
  pub name:            String,
  pub name_hash:       u32,
  pub globals_version: u32,
  pub parameter_count: u32,
  pub static_count:    u32
}

#[derive(Debug)]
pub struct Script {
  pub header:  ScriptInfo,
  pub code:    Vec<u8>,
  pub strings: Vec<u8>,
  pub natives: Vec<u64>
}

impl Script {
  pub fn get_string(&self, index: usize) -> Option<&str> {
    CStr::from_bytes_until_nul(&self.strings[index..])
      .ok()
      .and_then(|cstr| cstr.to_str().ok())
  }
}
