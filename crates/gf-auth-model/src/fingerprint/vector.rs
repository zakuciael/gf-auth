use crate::fingerprint::utils::{generate_vector, random_ascii_char};
use base64::Engine;
use chrono::{DateTime, TimeDelta, Utc};
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::str::FromStr;

#[derive(thiserror::Error, Debug)]
pub(crate) enum VectorError {
  #[error("invalid vector")]
  Invalid,
  #[error("invalid vector timestamp")]
  Timestamp,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Vector {
  pub content: String,
  pub time: DateTime<Utc>,
}

impl Vector {
  pub fn update(&mut self) {
    let current_time = Utc::now();

    if self.time.add(TimeDelta::milliseconds(1000)) < current_time {
      let mid = self.content[1..].to_owned();
      let rand_char = random_ascii_char().to_string();

      self.content = mid + &rand_char;
    }

    self.time = current_time;
  }
}

impl Default for Vector {
  fn default() -> Self {
    Vector {
      content: generate_vector(),
      time: Utc::now(),
    }
  }
}

impl Display for Vector {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {}", self.content, self.time.timestamp_millis())
  }
}

impl FromStr for Vector {
  type Err = VectorError;

  fn from_str(vector: &str) -> Result<Self, Self::Err> {
    let index = vector.rfind(' ').ok_or(VectorError::Invalid)?;

    let content = vector[0..index].to_owned();
    let time = DateTime::from_timestamp_millis(
      vector[index + 1..]
        .parse::<i64>()
        .map_err(|_| VectorError::Timestamp)?,
    )
    .ok_or(VectorError::Timestamp)?;

    Ok(Vector { content, time })
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

impl Serialize for Vector {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&base64::engine::general_purpose::STANDARD.encode(self.to_string()))
  }
}

struct VectorVisitor;

impl<'de> Visitor<'de> for VectorVisitor {
  type Value = Vector;

  fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
    write!(formatter, "an vector string")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: Error,
  {
    let vector = String::from_utf8(
      base64::engine::general_purpose::STANDARD
        .decode(value)
        .map_err(|_| Error::custom("failed to decode base64 vector"))?,
    )
    .map_err(|_| Error::custom("invalid utf8 string"))?;

    Vector::from_str(&vector)
      .map_err(|err| Error::custom(format!("invalid vector string: {:?}", err)))
  }

  fn visit_none<E>(self) -> Result<Self::Value, E>
  where
    E: Error,
  {
    Ok(Vector::default())
  }

  fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_str(VectorVisitor)
  }
}
