use thiserror::Error;

#[derive(Error, Debug)]
pub enum FromJsonFileError {
  #[error("{source}")]
  JsonError {
    #[from]
    source: serde_json::Error
  },
  #[error("{source}")]
  FileError {
    #[from]
    source: std::io::Error
  }
}
