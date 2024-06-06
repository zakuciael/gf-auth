use crate::{Fingerprint, TimingRange};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Identity {
  #[serde(default)]
  pub timing: TimingRange,
  pub fingerprint: Fingerprint,

  #[serde(default = "uuid::Uuid::new_v4")]
  pub installation_id: Uuid,
}

#[cfg(test)]
mod tests {
  use crate::identity::Identity;
  use crate::TimingRange;
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
