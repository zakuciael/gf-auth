use crate::{Blackbox, Fingerprint};
use base64::Engine;
use gf_auth_traits::DeserializeTuple;
use num_traits::FromPrimitive;
use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;

struct BlackboxVisitor;

struct FingerprintWrapper {
  value: Fingerprint,
}

impl<'de> Deserialize<'de> for FingerprintWrapper {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(FingerprintWrapper {
      value: Fingerprint::deserialize_tuple(deserializer)?,
    })
  }
}

impl<'de> Visitor<'de> for BlackboxVisitor {
  type Value = Blackbox;

  fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
    write!(formatter, "blackbox string starting with prefix `tra:`")
  }

  fn visit_str<E>(self, blackbox: &str) -> Result<Self::Value, E>
  where
    E: Error,
  {
    if !blackbox.starts_with("tra:") {
      return Err(Error::invalid_value(Unexpected::Str(blackbox), &self));
    }

    let blackbox = blackbox.replace("tra:", "");
    let base64_decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
      .decode(&blackbox)
      .map_err(|_| Error::invalid_value(Unexpected::Str(&blackbox), &"valid base64 string"))?;

    let mut gf_decoded = vec![base64_decoded[0]];
    for i in 1..base64_decoded.len() {
      let mut a = base64_decoded[i] as u32;
      let b = base64_decoded[i - 1] as u32;

      if a < b {
        a += 0x100;
      }

      let c = a - b;
      let c =
        u8::from_u32(c).ok_or(Error::custom(format!("{} is not a valid 8-bit integer", c)))?;
      gf_decoded.push(c);
    }

    let url_decoded = percent_encoding::percent_decode(&gf_decoded).collect::<Vec<_>>();
    let fingerprint = serde_json::from_slice::<FingerprintWrapper>(&url_decoded)
      .map_err(|err| Error::custom(format!("failed to deserialize fingerprint: {}", err)))?;

    Ok(Blackbox(fingerprint.value))
  }
}

impl<'de> Deserialize<'de> for Blackbox {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_str(BlackboxVisitor)
  }
}
