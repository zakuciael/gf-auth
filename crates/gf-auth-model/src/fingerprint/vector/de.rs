use crate::fingerprint::vector::Vector;
use base64::Engine;
use chrono::DateTime;
use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;

struct VectorVisitor;

impl<'de> Visitor<'de> for VectorVisitor {
  type Value = Vector;

  fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
    write!(formatter, "base64 encoded vector string")
  }

  fn visit_str<E>(self, vector: &str) -> Result<Self::Value, E>
  where
    E: Error,
  {
    let base64_decoded = {
      let buf = base64::engine::general_purpose::STANDARD
        .decode(vector)
        .map_err(|_| Error::invalid_value(Unexpected::Str(vector), &self))?;

      String::from_utf8(buf).map_err(|err| {
        Error::invalid_value(Unexpected::Bytes(err.as_bytes()), &"valid utf8 string")
      })?
    };

    let divider_index = base64_decoded
      .rfind(' ')
      .ok_or(Error::custom("no divider found"))?;

    let content = base64_decoded[0..divider_index].to_owned();
    let time = {
      let raw = &base64_decoded[divider_index + 1..];
      let value = raw
        .parse::<i64>()
        .map_err(|_| Error::invalid_type(Unexpected::Str(raw), &"signed 64-bit integer"))?;
      DateTime::from_timestamp_millis(value).ok_or(Error::invalid_value(
        Unexpected::Signed(value),
        &"valid timestamp",
      ))?
    };

    Ok(Vector::new(content, time))
  }
}

impl<'de> Deserialize<'de> for Vector {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_str(VectorVisitor)
  }
}
