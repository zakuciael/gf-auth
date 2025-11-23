use crate::clients::captcha::{BaseCaptchaClient, CaptchaClientConfig};
use gf_auth_http::{Headers, HttpClient};
use http::header::{ORIGIN, USER_AGENT as USER_AGENT_HEADER};
use uuid::Uuid;

static API_BASE_URL: &str = "https://image-drop-challenge.gameforge.com/challenge";
static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.104 Safari/537.36";
static LOCALE: &str = "en-US";
static MAX_ATTEMPTS_PER_CHALLENGE_ID: usize = 3;
static MAX_ATTEMPTS_OVERALL: usize = 5;

#[derive(Debug, Clone)]
pub struct CaptchaClient {
  http_client: HttpClient,
  config: CaptchaClientConfig,
}

impl CaptchaClient {
  pub fn new(config: Option<CaptchaClientConfig>) -> Self {
    Self {
      config: config.unwrap_or_else(|| CaptchaClientConfig {
        api_base: API_BASE_URL.to_owned(),
        user_agent: USER_AGENT.to_owned(),
        locale: LOCALE.to_owned(),
        max_attempts_per_challenge: MAX_ATTEMPTS_PER_CHALLENGE_ID,
        max_attempts_overall: MAX_ATTEMPTS_OVERALL,
      }),
      http_client: HttpClient::default(),
    }
  }
}

impl BaseCaptchaClient for CaptchaClient {
  fn get_http(&self) -> &HttpClient {
    &self.http_client
  }

  fn get_config(&self) -> &CaptchaClientConfig {
    &self.config
  }

  fn api_url(&self, url: &str, challenge_id: &Uuid) -> String {
    let config = self.get_config();
    let base = {
      if config.api_base.ends_with('/') {
        let mut base = config.api_base.to_owned();
        base.remove(url.len() - 1);
        base
      } else {
        config.api_base.to_owned()
      }
    };
    let url = {
      if url.starts_with('/') {
        let mut url = url.to_owned();
        url.remove(0);
        url
      } else {
        url.to_owned()
      }
    };

    format!("{base}/{}/{}/{url}", challenge_id, &config.locale)
  }

  fn get_headers(&self) -> Headers {
    let mut headers = Headers::new();
    headers.insert(
      USER_AGENT_HEADER.to_string(),
      self.get_config().user_agent.to_owned(),
    );
    headers.insert(ORIGIN.to_string(), "spark://www.gameforge.com".to_owned());

    headers
  }
}
