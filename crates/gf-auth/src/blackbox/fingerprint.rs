use std::error::Error;
use std::ops::Add;

use chrono::{DateTime, TimeDelta, Utc};
use rand::distributions::uniform::UniformSampler;
use rand::distributions::Uniform;
use rand::Rng;
use serde::{Deserialize, Serialize};

const VECTOR_CONTENT_LENGTH: usize = 100;
const GAME_LENGTH: usize = 27;

pub type FingerprintResult<T, E = FingerprintError> = Result<T, E>;
pub type FingerprintVector<'a> = (&'a str, DateTime<Utc>);

#[derive(thiserror::Error, Debug)]
pub enum FingerprintError {
  #[error("json parse error: {0}")]
  ParseJson(#[from] serde_json::Error),

  #[error("input/output error: {0}")]
  IO(#[from] std::io::Error),

  #[error("conversion error: {0}")]
  Convert(String),
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BlackboxRequest {
  features: Vec<u64>,
  installation: String,
  session: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Fingerprint {
  #[serde(alias = "v")]
  version: u32,
  #[serde(alias = "tz")]
  timezone: String,
  #[serde(alias = "dnt")]
  do_not_track: bool,
  #[serde(alias = "product")]
  browser_engine: String,
  #[serde(alias = "osType")]
  os_name: String,
  #[serde(alias = "app")]
  browser_name: String,
  #[serde(alias = "vendor")]
  vendor: String,
  #[serde(alias = "mem")]
  memory: u32,
  #[serde(alias = "con")]
  concurrency: u32,
  #[serde(alias = "lang")]
  languages: String,
  #[serde(alias = "plugins")]
  plugins: String,
  gpu: String,
  fonts: String,
  #[serde(alias = "audioC")]
  audio_context: String,
  width: u32,
  height: u32,
  #[serde(alias = "depth")]
  color_depth: u32,
  #[serde(alias = "video")]
  video_codecs: String,
  #[serde(alias = "audio")]
  audio_codecs: String,
  #[serde(alias = "media")]
  media_devices: String,
  #[serde(alias = "permissions")]
  navigator_permissions: String,
  #[serde(alias = "audioFP")]
  audio_fingerprint: f64,
  #[serde(alias = "webglFP")]
  webgl_fingerprint: String,
  #[serde(alias = "canvasFP")]
  canvas_fingerprint: f64,
  creation: DateTime<Utc>,
  #[serde(alias = "uuid")]
  game: String,
  #[serde(alias = "d")]
  delta: u32,
  #[serde(alias = "osVersion")]
  os_version: String,
  vector: String,
  #[serde(alias = "userAgent")]
  user_agent: String,
  #[serde(alias = "serverTimeInMS")]
  server_time: DateTime<Utc>,
  #[serde(alias = "request", skip_serializing_if = "Option::is_none")]
  request: Option<BlackboxRequest>,
}

impl Fingerprint {
  fn generate_vector() -> String {
    let rng = rand::thread_rng();
    let content: String = rng
      .sample_iter(Uniform::new(32, 126))
      .take(VECTOR_CONTENT_LENGTH)
      .map(char::from)
      .collect();
    let time = Utc::now();
    format!("{} {}", content, time.timestamp_millis())
  }

  fn set_request(&mut self, request: BlackboxRequest) {
    self.request = Some(request);
  }

  fn update_vector(&mut self) -> FingerprintResult<()> {
    let current_time = Utc::now();
    let (content, old_time) = self.unpack_vector()?;

    let content = if old_time.add(TimeDelta::milliseconds(1000)) < current_time {
      let mid = (&content[1..]).to_owned();
      let rand_char = rand::thread_rng().sample(Uniform::new(32, 126)).to_string();
      mid + &rand_char
    } else {
      content.to_string()
    };

    self.pack_vector((&content, current_time));
    Ok(())
  }

  fn pack_vector(&mut self, vector: FingerprintVector) {
    self.vector = format!("{} {}", vector.0, vector.1.timestamp_millis())
  }

  fn unpack_vector(&self) -> FingerprintResult<FingerprintVector> {
    let index = self
      .vector
      .find(' ')
      .ok_or_else(|| FingerprintError::Convert("invalid vector".to_owned()))?;
    let time: i64 = (&self.vector[index + 1..])
      .parse()
      .map_err(|_| FingerprintError::Convert("vector time is not a valid integer".to_owned()))?;
    let time = DateTime::from_timestamp_millis(time)
      .ok_or_else(|| FingerprintError::Convert("vector time is not a valid date".to_owned()))?;

    Ok((&self.vector[0..index], time))
  }

  fn get_server_time() -> DateTime<Utc> {
    Utc::now()
  }
}

#[cfg(test)]
mod tests {
  use std::error::Error;
  use std::fs;

  use crate::blackbox::fingerprint::Fingerprint;

  #[test]
  fn parse_fingerprint() -> Result<(), Box<dyn Error>> {
    let file =
      fs::read_to_string("./resources/fingerprint1.json").expect("Failed to read fingerprint file");

    serde_json::from_str::<Fingerprint>(&file)?;
    Ok(())
  }

  #[test]
  fn parse_fingerprint_with_request() -> Result<(), Box<dyn Error>> {
    let file = fs::read_to_string("./resources/fingerprint2.json")
      .expect("Failed to read fingerprint file.");

    serde_json::from_str::<Fingerprint>(&file)?;
    Ok(())
  }

  #[test]
  fn unpack_vector() -> Result<(), Box<dyn Error>> {
    let file =
      fs::read_to_string("./resources/fingerprint1.json").expect("Failed to read fingerprint file");

    println!(
      "{:?}",
      serde_json::from_str::<Fingerprint>(&file)?.unpack_vector()
    );
    Ok(())
  }

  #[test]
  fn test() -> Result<(), Box<dyn Error>> {
    println!("{}", Fingerprint::generate_vector());

    Ok(())
  }
}
