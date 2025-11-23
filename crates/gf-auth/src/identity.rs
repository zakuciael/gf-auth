use gf_auth_model::{Blackbox, Identity};

#[derive(Debug, Clone)]
pub struct IdentityManager {
  pub identity: Identity,
}

impl IdentityManager {
  pub fn new(identity: Identity) -> IdentityManager {
    IdentityManager { identity }
  }

  pub fn generate_blackbox(&mut self) -> Blackbox {
    let fingerprint = &mut self.identity.fingerprint;
    fingerprint.update_vector();
    fingerprint.update_server_time();
    fingerprint.update_delta(&self.identity.timing);
    fingerprint.update_creation();

    Blackbox::new(fingerprint.clone())
  }
}

#[cfg(test)]
mod tests {
  use crate::identity::IdentityManager;
  use gf_auth_model::Identity;
  use std::fs;

  #[test]
  fn generate_blackbox() {
    let file = fs::read_to_string("../../resources/identity/identity_only_fingerprint.json")
      .expect("Failed to read fingerprint file.");

    let identity = serde_json::from_str::<Identity>(&file).expect("Failed to parse identity file");
    let mut manager = IdentityManager::new(identity);

    let a =
      serde_plain::to_string(&manager.generate_blackbox()).expect("Failed to serialize blackbox A");
    let b =
      serde_plain::to_string(&manager.generate_blackbox()).expect("Failed to serialize blackbox B");

    assert_ne!(&a, &b)
  }
}
