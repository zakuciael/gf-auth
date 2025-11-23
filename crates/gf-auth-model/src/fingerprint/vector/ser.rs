use crate::fingerprint::vector::Vector;
use base64::Engine;
use serde::{Serialize, Serializer};

impl Serialize for Vector {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let vector = self.to_string();
    let base64_encoded = base64::engine::general_purpose::STANDARD.encode(&vector);

    serializer.serialize_str(&base64_encoded)
  }
}
