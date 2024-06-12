use crate::clients::captcha::BaseCaptchaClient;
use crate::identity::IdentityManager;
use crate::sync::Mutex;
use gf_auth_http::{BaseHttpClient, Headers, HttpClient, HttpError};
use gf_auth_model::api::gameforge::{GameforgeAuthRequest, GameforgeAuthResponse};
use maybe_async::maybe_async;
use rand::Rng;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub mod r#impl;

pub type ClientResult<T, E = Box<dyn std::error::Error>> = Result<T, E>;

#[maybe_async]
pub trait BaseGameforgeClient
where
  Self: Send + Sync + Clone + Debug,
{
  fn get_http(&self) -> &HttpClient;

  fn get_captcha_client(&self) -> impl BaseCaptchaClient;

  fn api_url(&self, url: &str) -> String;

  fn get_identity_manager(&self) -> Arc<Mutex<IdentityManager>>;

  async fn authenticate(
    &self,
    email: &str,
    password: &str,
    locale: Option<&str>,
  ) -> ClientResult<Uuid> {
    self
      .authenticate_with_captcha(email, password, locale, None)
      .await
  }

  async fn authenticate_with_captcha(
    &self,
    email: &str,
    password: &str,
    locale: Option<&str>,
    captcha: Option<(Uuid, usize)>,
  ) -> ClientResult<Uuid> {
    let client = self.get_captcha_client();
    if let Some((_, attempt)) = &captcha {
      if attempt >= &client.get_config().max_attempts_overall {
        todo!("Throw an error on max attempts")
      }
    }

    let url = self.api_url("auth/sessions");
    let locale = locale.unwrap_or("pl-PL");
    let blackbox = self
      .get_identity_manager()
      .lock()
      .await
      .unwrap()
      .generate_blackbox();

    let mut headers = Headers::new();
    if let Some((challenge_id, _)) = &captcha {
      headers.insert("gf-challenge-id".to_owned(), challenge_id.to_string());
    }

    let request = GameforgeAuthRequest {
      blackbox,
      email,
      password,
      locale,
    };

    match self.get_http().post(&url, Some(&headers), &request).await {
      Ok(response) => {
        let response = serde_json::from_slice::<GameforgeAuthResponse>(&response.bytes())?;
        Ok(response.token)
      }
      Err(HttpError::Status { status, headers }) if status == 409 => {
        let new_challenge_id = {
          let raw = headers.get("gf-challenge-id").unwrap(); // FIXME
          let raw = raw.split(";").next().unwrap().to_owned(); // FIXME

          Uuid::from_str(&raw)?
        };

        println!("Captcha is required, challenge id: {}", &new_challenge_id);
        if client.solve(&new_challenge_id).await? {
          println!("Solved");
        }

        let (_, attempt) = captcha.unwrap_or((Default::default(), 0));
        return Ok(
          self
            .authenticate_with_captcha(
              email,
              password,
              Some(locale),
              Some((new_challenge_id, attempt + 1)),
            )
            .await?,
        );
      }
      Err(err) => return Err(Box::try_from(err).unwrap()), // TODO,
    }
  }
}
