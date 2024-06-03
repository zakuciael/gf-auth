mod error;

pub use crate::blackbox::error::{BlackboxError, BlackboxResult};
use crate::fingerprint::Fingerprint;
use base64::Engine;
use gf_auth_traits::{DeserializeTuple, SerializeTuple};
use num_traits::cast::FromPrimitive;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug)]
pub struct Blackbox(
  #[serde(serialize_with = "crate::fingerprint::Fingerprint::serialize_tuple")]
  #[serde(deserialize_with = "crate::fingerprint::Fingerprint::deserialize_tuple")]
  pub Fingerprint,
);

impl Blackbox {
  pub fn new(fingerprint: Fingerprint) -> Blackbox {
    Blackbox(fingerprint)
  }

  pub fn encode(&self) -> BlackboxResult<String> {
    let json = serde_json::to_string(&self)?;
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

  #[test]
  fn encode() {
    let fingerprint_file =
      fs::read_to_string("../../resources/blackbox/fingerprint_no_request.json")
        .expect("Failed to read fingerprint file");
    let blackbox_file = fs::read_to_string("../../resources/blackbox/encoded_blackbox.txt")
      .expect("Failed to read blackbox file");

    let fingerprint =
      serde_json::from_str::<Fingerprint>(&fingerprint_file).expect("Failed to parse fingerprint");
    let blackbox = Blackbox::new(fingerprint);

    let encoded = blackbox.encode();
    assert!(encoded.is_ok());
    assert_eq!(encoded.unwrap(), blackbox_file);
  }
}
