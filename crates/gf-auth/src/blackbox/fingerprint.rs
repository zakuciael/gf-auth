use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
  use crate::blackbox::fingerprint::Fingerprint;
  use std::error::Error;
  use std::fs;

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
}
