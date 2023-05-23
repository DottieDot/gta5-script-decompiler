use std::{collections::HashMap, fs, path::Path};

use nativedocgen_model::{DocumentRoot, Native};

use super::FromJsonFileError;

pub struct Natives {
  _document: DocumentRoot,
  natives:   HashMap<u64, Native>
}

impl Natives {
  pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
    let document = serde_json::from_str::<DocumentRoot>(json)?;

    Ok(Self {
      natives:   document
        .natives
        .iter()
        .filter_map(|(key, value)| {
          u64::from_str_radix(key.trim_start_matches("0x"), 16)
            .map(|hash| (hash, (*value).clone()))
            .ok()
        })
        .collect(),
      _document: document
    })
  }

  pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self, FromJsonFileError> {
    let contents = fs::read_to_string(path)?;

    Ok(Self::from_json(&contents)?)
  }

  pub fn get_native(&self, hash: u64) -> Option<&Native> {
    self.natives.get(&hash)
  }
}
