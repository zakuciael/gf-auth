use crate::blackbox::constants::URI_COMPONENT_SET;
use crate::{Blackbox, Fingerprint};
use base64::Engine;
use gf_auth_traits::SerializeTuple;
use num_traits::FromPrimitive;
use serde::ser::Error;
use serde::{Serialize, Serializer};

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

impl Serialize for Blackbox {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let json = {
      let wrapper = FingerprintWrapper { value: &self.0 };
      serde_json::to_string(&wrapper)
        .map_err(|err| Error::custom(format!("failed to serialize fingerprint: {}", err)))?
    };

    let url_encoded = percent_encoding::utf8_percent_encode(&json, URI_COMPONENT_SET)
      .collect::<String>()
      .into_bytes();

    let mut gf_encoded = vec![url_encoded[0]];
    for i in 1..url_encoded.len() {
      let a = gf_encoded[i - 1];
      let b = url_encoded[i];

      let c = (a as u32 + b as u32) % 0x100;
      let c =
        u8::from_u32(c).ok_or(Error::custom(format!("{} is not a valid 8-bit integer", c)))?;

      gf_encoded.push(c);
    }

    let base64_encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(gf_encoded);
    let blackbox = "tra:".to_owned() + &base64_encoded;
    serializer.serialize_str(&blackbox)
  }
}
