use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CaptchaChallengeStatus {
  Presented,
  Solved,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CaptchaChallenge {
  pub id: Uuid,
  #[serde(with = "chrono::serde::ts_milliseconds")]
  pub last_updated: DateTime<Utc>,
  pub status: CaptchaChallengeStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CaptchaAnswer {
  pub answer: u8,
}

impl CaptchaAnswer {
  pub fn new(answer: u8) -> Self {
    Self { answer }
  }
}
