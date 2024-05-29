mod utils;
mod vector;

use crate::fingerprint::vector::Vector;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Request {
  features: Vec<u64>,
  installation: String,
  session: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
  game: Option<String>,
  #[serde(alias = "d")]
  delta: u32,
  #[serde(alias = "osVersion")]
  os_version: String,
  vector: Option<Vector>,
  #[serde(alias = "userAgent")]
  user_agent: String,
  #[serde(alias = "serverTimeInMS")]
  server_time: DateTime<Utc>,
  #[serde(alias = "request", skip_serializing_if = "Option::is_none")]
  request: Option<Request>,
}

impl Fingerprint {
  pub fn update_vector(&mut self) {
    if let Some(ref mut vector) = &mut self.vector {
      vector.update();
    } else {
      self.vector = Some(Vector::default());
    }
  }

  pub fn update_server_time(&mut self) {
    // TODO: Actually fetch game1.js file and extract date from the header
    self.server_time = Utc::now();
  }

  pub fn set_request(&mut self, request: Request) {
    self.request = Some(request);
  }
}

#[cfg(test)]
mod tests {
  use std::error::Error;
  use std::fs;

  use crate::fingerprint::Fingerprint;

  #[test]
  fn parse_fingerprint() -> Result<(), Box<dyn Error>> {
    let file =
      fs::read_to_string("./resources/fingerprint1.json").expect("Failed to read fingerprint file");

    println!("{:?}", serde_json::from_str::<Fingerprint>(&file)?);
    Ok(())
  }

  #[test]
  fn parse_fingerprint_with_request() -> Result<(), Box<dyn Error>> {
    let file = fs::read_to_string("./resources/fingerprint2.json")
      .expect("Failed to read fingerprint file.");

    println!("{:?}", serde_json::from_str::<Fingerprint>(&file)?);
    Ok(())
  }

  #[test]
  fn update_vector() -> Result<(), Box<dyn Error>> {
    let file =
      fs::read_to_string("./resources/fingerprint1.json").expect("Failed to read fingerprint file");

    let mut fingerprint = serde_json::from_str::<Fingerprint>(&file)?;
    println!("{:?}", fingerprint.vector);

    fingerprint.update_vector();
    println!("{:?}", fingerprint.vector);
    Ok(())
  }
}
