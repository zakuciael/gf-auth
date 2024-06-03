use gf_auth_model::Fingerprint;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TimingRange {
  min: u32,
  max: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Identity {
  #[serde(default)]
  timing: TimingRange,
  fingerprint: Fingerprint,

  #[serde(default = "uuid::Uuid::new_v4")]
  installation_id: Uuid,
}

impl Default for TimingRange {
  fn default() -> Self {
    TimingRange { min: 150, max: 300 }
  }
}

#[cfg(test)]
mod tests {
  use crate::identity::model::{Identity, TimingRange};
  use std::fs;
  use uuid::uuid;

  #[test]
  fn parse_identity_with_only_fingerprint() {
    let file = fs::read_to_string("../../resources/identity/identity_only_fingerprint.json")
      .expect("Failed to read fingerprint file.");

    assert!(serde_json::from_str::<Identity>(&file).is_ok());
  }

  #[test]
  fn parse_identity_file() {
    let file = fs::read_to_string("../../resources/identity/identity_full.json")
      .expect("Failed to read fingerprint file.");

    let identity = serde_json::from_str::<Identity>(&file);
    assert!(identity.is_ok());

    let identity = identity.unwrap();
    assert_eq!(
      identity.installation_id,
      uuid!("fe2495d2-0d0c-4dde-8525-076ff5570a59")
    );
    assert_eq!(identity.timing, TimingRange::default());
  }
}
