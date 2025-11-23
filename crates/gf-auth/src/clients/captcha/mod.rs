use futures::FutureExt;
use gf_auth_http::{BaseHttpClient, Headers, HttpClient, Query};
use gf_auth_model::api::captcha::{CaptchaAnswer, CaptchaChallenge, CaptchaChallengeStatus};
use http::header::ORIGIN;
use maybe_async::maybe_async;
use rand::distributions::Uniform;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

pub mod r#impl;

pub type CaptchaClientResult<T, E = Box<dyn std::error::Error>> = Result<T, E>;

// Captcha request flow:
// - [x] Get config
// - [ ] Load resources
// - [ ] Send answer (random)
// - If max attempts for a single challenge id is exceeded, then we need to re-login to get a new challenge id.

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CaptchaClientConfig {
  pub api_base: String,
  pub user_agent: String,
  pub locale: String,
  pub max_attempts_per_challenge: usize,
  pub max_attempts_overall: usize,
}

#[maybe_async]
pub trait BaseCaptchaClient
where
  Self: Send + Sync + Clone + Debug,
{
  fn get_http(&self) -> &HttpClient;

  fn get_config(&self) -> &CaptchaClientConfig;

  fn api_url(&self, url: &str, challenge_id: &Uuid) -> String;

  fn get_headers(&self) -> Headers;

  async fn solve(&self, challenge_id: &Uuid) -> CaptchaClientResult<bool> {
    for attempt in 0..self.get_config().max_attempts_per_challenge {
      println!("Captcha solve attempt: {attempt}");
      let config = self.get_challenge(challenge_id).await?;
      let _resources = self
        .get_resource("text", &config)
        .then(|_| self.get_resource("drag-icons", &config))
        .then(|_| self.get_resource("drop-target", &config))
        .await?;

      let answer = rand::thread_rng().sample(Uniform::from(0..4));
      println!("Trying answer: {answer}");
      if self.answer(answer, challenge_id).await? {
        return Ok(true);
      }
    }

    Ok(false)
  }

  async fn get_challenge(&self, challenge_id: &Uuid) -> CaptchaClientResult<CaptchaChallenge> {
    let url = self.api_url("/", challenge_id);

    let res = self
      .get_http()
      .get(&url, Some(&self.get_headers()), &Query::new())
      .await?;

    Ok(serde_json::from_slice(&res.bytes())?)
  }

  async fn get_resource(
    &self,
    url: &str,
    challenge_config: &CaptchaChallenge,
  ) -> CaptchaClientResult<Vec<u8>> {
    let url = self.api_url(&url, &challenge_config.id);
    let last_updated = challenge_config.last_updated.timestamp_millis().to_string();
    let query = Query::from([("", last_updated.as_str())]);

    Ok(
      self
        .get_http()
        .get(&url, Some(&self.get_headers()), &query)
        .await
        .map(|res| res.bytes().into())?,
    )
  }

  async fn answer(&self, answer: u8, challenge_id: &Uuid) -> CaptchaClientResult<bool> {
    let url = self.api_url("/", challenge_id);

    let res: CaptchaChallenge = self
      .get_http()
      .post(&url, Some(&self.get_headers()), &CaptchaAnswer::new(answer))
      .await
      .map(|res| serde_json::from_slice(&res.bytes()))??;

    Ok(res.status == CaptchaChallengeStatus::Solved)
  }
}
