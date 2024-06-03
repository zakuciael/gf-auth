#[derive(thiserror::Error, Debug)]
pub enum BlackboxError {
  #[error("failed to serialize/deserialize fingerprint: {0}")]
  Json(#[from] serde_json::Error),

  #[error("failed to encode blackbox")]
  Encode,
}

pub type BlackboxResult<T, E = BlackboxError> = Result<T, E>;
