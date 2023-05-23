use std::{cell::RefCell, collections::HashMap, fs, path::Path};

use itertools::Itertools;
use serde::Deserialize;

use super::FromJsonFileError;

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'i"))]
struct Json<'i>(Vec<Vec<&'i str>>);

pub struct CrossMap {
  hashes:         Vec<Vec<u64>>,
  original_cache: RefCell<HashMap<u64, u64>>
}

impl CrossMap {
  pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
    let json = serde_json::from_str::<Json>(json)?;

    Ok(Self {
      hashes:         json
        .0
        .into_iter()
        .map(|history| {
          history
            .into_iter()
            .map(|hash| u64::from_str_radix(hash.trim_start_matches("0x"), 16).unwrap())
            .collect_vec()
        })
        .collect_vec(),
      original_cache: Default::default()
    })
  }

  pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self, FromJsonFileError> {
    let contents = fs::read_to_string(path)?;

    Ok(Self::from_json(&contents)?)
  }

  pub fn get_original_hash(&self, current: u64) -> u64 {
    *self
      .original_cache
      .borrow_mut()
      .entry(current)
      .or_insert_with(|| {
        let history = self
          .hashes
          .iter()
          .find(|history| history.contains(&current));

        if let Some(history) = history {
          *history.iter().find(|h| **h != 0).unwrap_or(&current)
        } else {
          current
        }
      })
  }
}
