use crate::{Blackbox, BlackboxError};
use serde::ser::Error;
use serde::{Serialize, Serializer};

impl Serialize for Blackbox {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let blackbox = self.encode().map_err(|err| match err {
      BlackboxError::Json(err) => Error::custom(format!("invalid json: {}", err)),
      BlackboxError::Encode => Error::custom("unable to encode blackbox"),
    })?;

    serializer.serialize_str(&blackbox)
  }
}
