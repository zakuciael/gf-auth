mod timings;
mod utils;
mod vector;

pub use crate::fingerprint::timings::TimingRange;

use crate::fingerprint::vector::Vector;
use chrono::{DateTime, Utc};
use gf_auth_macros::{DeserializeTuple, SerializeTuple};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Request {
  features: Vec<u64>,
  installation: String,
  session: String,
}

#[derive(Serialize, SerializeTuple, Deserialize, DeserializeTuple, Debug, Clone, PartialEq)]
pub struct Fingerprint {
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
  os_version: Option<String>,
  vector: Vector,
  #[serde(alias = "userAgent")]
  user_agent: String,
  #[serde(alias = "serverTimeInMS")]
  server_time: DateTime<Utc>,
  #[serde(alias = "request")]
  #[serde(default)]
  request: Option<Request>,
}

impl Fingerprint {
  pub fn update_vector(&mut self) {
    self.vector.update();
  }

  pub fn update_server_time(&mut self) {
    // TODO: Actually fetch game1.js file and extract date from the header
    self.server_time = Utc::now();
  }

  pub fn update_delta(&mut self, range: &TimingRange) {
    self.delta = range.generate();
  }

  pub fn update_creation(&mut self) {
    self.creation = Utc::now();
  }

  pub fn set_request(&mut self, request: Request) {
    self.request = Some(request);
  }
}

#[cfg(test)]
mod tests {
  use std::fs;

  use crate::fingerprint::Fingerprint;

  #[test]
  fn parse_fingerprint() {
    let file = fs::read_to_string("../../resources/blackbox/fingerprint_no_request.json")
      .expect("Failed to read fingerprint file");

    assert!(serde_json::from_str::<Fingerprint>(&file).is_ok())
  }

  #[test]
  fn parse_fingerprint_with_request() {
    let file = fs::read_to_string("../../resources/blackbox/fingerprint_with_request.json")
      .expect("Failed to read fingerprint file.");

    assert!(serde_json::from_str::<Fingerprint>(&file).is_ok())
  }
}
