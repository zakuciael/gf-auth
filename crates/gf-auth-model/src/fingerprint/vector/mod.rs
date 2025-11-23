use crate::fingerprint::utils::random_ascii_char;
use chrono::{DateTime, TimeDelta, Utc};
use std::fmt::{Display, Formatter};
use std::ops::Add;

mod de;
mod ser;

#[derive(Debug, Clone)]
pub struct Vector {
  pub content: String,
  pub time: DateTime<Utc>,
}

impl Vector {
  pub fn new(content: String, time: DateTime<Utc>) -> Self {
    Vector { content, time }
  }

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

impl Display for Vector {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {}", self.content, self.time.timestamp_millis())
  }
}

impl PartialEq for Vector {
  fn eq(&self, other: &Self) -> bool {
    self.content.eq(&other.content)
      && self
        .time
        .timestamp_millis()
        .eq(&other.time.timestamp_millis())
  }
}

#[cfg(test)]
mod tests {
  use crate::fingerprint::utils::generate_vector;
  use crate::fingerprint::vector::Vector;
  use base64::Engine;
  use chrono::Utc;

  fn get_test_data() -> (Vector, String) {
    let content = generate_vector();
    let time = Utc::now();

    let vector = Vector::new(content.clone(), time);

    let vector_str = {
      let value = format!("{} {}", &content, time.timestamp_millis());
      base64::engine::general_purpose::STANDARD.encode(value)
    };

    (vector, vector_str)
  }

  #[test]
  fn serialize() -> Result<(), Box<dyn std::error::Error>> {
    let (vector, vector_str) = get_test_data();

    assert_eq!(serde_plain::to_string(&vector)?, vector_str);
    Ok(())
  }

  #[test]
  fn deserialize() -> Result<(), Box<dyn std::error::Error>> {
    let (vector, vector_str) = get_test_data();

    assert_eq!(serde_plain::from_str::<Vector>(&vector_str)?, vector);
    Ok(())
  }
}
