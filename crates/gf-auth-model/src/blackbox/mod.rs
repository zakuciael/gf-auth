mod constants;
mod ser;

use crate::fingerprint::Fingerprint;

#[derive(Debug)]
pub struct Blackbox(pub Fingerprint);

impl Blackbox {
  pub fn new(fingerprint: Fingerprint) -> Self {
    Self(fingerprint)
  }
}

#[cfg(test)]
mod tests {
  use crate::blackbox::Blackbox;
  use crate::fingerprint::Fingerprint;
  use std::fs;

  fn load_data() -> (Fingerprint, String) {
    let blackbox_str = fs::read_to_string("../../resources/blackbox/encoded_blackbox.txt")
      .expect("Failed to read blackbox file");
    let fingerprint = {
      let file = fs::read_to_string("../../resources/blackbox/fingerprint_no_request.json")
        .expect("Failed to read fingerprint file");

      serde_json::from_str::<Fingerprint>(&file).expect("Failed to parse fingerprint")
    };

    (fingerprint, blackbox_str)
  }

  #[test]
  fn serialize() -> Result<(), Box<dyn std::error::Error>> {
    let (fingerprint, blackbox_str) = load_data();

    let serialized = serde_plain::to_string(&Blackbox::new(fingerprint))?;
    assert_eq!(serialized, blackbox_str);

    Ok(())
  }
}
