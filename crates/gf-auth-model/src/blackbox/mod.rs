mod error;
mod ser;

pub use crate::blackbox::error::{BlackboxError, BlackboxResult};
use crate::fingerprint::Fingerprint;
use base64::Engine;
use gf_auth_traits::SerializeTuple;
use num_traits::cast::FromPrimitive;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize, Serializer};

const URI_COMPONENT_SET: &AsciiSet = &NON_ALPHANUMERIC
  .remove(b'-')
  .remove(b'_')
  .remove(b'.')
  .remove(b'!')
  .remove(b'~')
  .remove(b'*')
  .remove(b'\'')
  .remove(b'(')
  .remove(b')');

#[derive(Deserialize, Debug)]
pub struct Blackbox(pub Fingerprint);

impl Blackbox {
  pub fn new(fingerprint: Fingerprint) -> Blackbox {
    Blackbox(fingerprint)
  }

  pub fn encode(&self) -> BlackboxResult<String> {
    let json = {
      struct FingerprintWrapper<'a> {
        value: &'a Fingerprint,
      }

      impl<'a> Serialize for FingerprintWrapper<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
          S: Serializer,
        {
          self.value.serialize_tuple(serializer)
        }
      }

      let wrapper = FingerprintWrapper { value: &self.0 };
      serde_json::to_string(&wrapper)?
    };
    let url_encoded = percent_encoding::utf8_percent_encode(&json, URI_COMPONENT_SET)
      .collect::<String>()
      .into_bytes();

    let mut result = vec![url_encoded[0]];
    for i in 1..url_encoded.len() {
      let a = result[i - 1];
      let b = url_encoded[i];

      let c = u8::from_u32((a as u32 + b as u32) % 0x100).ok_or(BlackboxError::Encode)?;
      result.push(c);
    }

    let blackbox = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(result);
    Ok("tra:".to_owned() + &blackbox)
  }
}

#[cfg(test)]
mod tests {
  use crate::blackbox::Blackbox;
  use crate::fingerprint::Fingerprint;
  use std::fs;

  fn load_blackbox() -> (String, Blackbox) {
    let fingerprint_file =
      fs::read_to_string("../../resources/blackbox/fingerprint_no_request.json")
        .expect("Failed to read fingerprint file");
    let blackbox_file = fs::read_to_string("../../resources/blackbox/encoded_blackbox.txt")
      .expect("Failed to read blackbox file");

    let fingerprint =
      serde_json::from_str::<Fingerprint>(&fingerprint_file).expect("Failed to parse fingerprint");

    (blackbox_file, Blackbox::new(fingerprint))
  }

  #[test]
  fn encode() {
    let (blackbox_file, blackbox) = load_blackbox();

    let encoded = blackbox.encode();
    assert!(encoded.is_ok());
    assert_eq!(encoded.unwrap(), blackbox_file);
  }

  #[test]
  fn serialize() {
    let (blackbox_file, blackbox) = load_blackbox();

    let encoded = serde_json::to_string(&blackbox);
    assert!(encoded.is_ok());
    assert_eq!(encoded.unwrap(), format!("\"{}\"", blackbox_file));
  }
}
