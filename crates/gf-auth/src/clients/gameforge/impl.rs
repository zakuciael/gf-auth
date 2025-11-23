use crate::clients::captcha::r#impl::CaptchaClient;
use crate::clients::captcha::BaseCaptchaClient;
use crate::clients::gameforge::BaseGameforgeClient;
use crate::identity::IdentityManager;
use crate::sync::Mutex;
use gf_auth_http::HttpClient;
use gf_auth_model::Identity;
use std::sync::Arc;

static BASE_API_URL: &str = "https://spark.gameforge.com/api/v1/";

#[derive(Debug, Clone)]
pub struct GameforgeClient {
  identity_manager: Arc<Mutex<IdentityManager>>,
  http_client: HttpClient,
}

impl GameforgeClient {
  pub fn new(identity: Identity) -> Self {
    GameforgeClient {
      identity_manager: Arc::new(Mutex::new(IdentityManager::new(identity))),
      http_client: HttpClient::default(),
    }
  }
}

impl BaseGameforgeClient for GameforgeClient {
  fn get_http(&self) -> &HttpClient {
    &self.http_client
  }

  fn get_captcha_client(&self) -> impl BaseCaptchaClient {
    CaptchaClient::new(None)
  }

  fn api_url(&self, url: &str) -> String {
    let mut base = BASE_API_URL.to_owned();
    if !base.ends_with('/') {
      base.push('/');
    }

    base + url
  }

  fn get_identity_manager(&self) -> Arc<Mutex<IdentityManager>> {
    Arc::clone(&self.identity_manager)
  }
}

#[cfg(test)]
mod tests {
  use crate::clients::gameforge::r#impl::GameforgeClient;
  use crate::clients::gameforge::BaseGameforgeClient;
  use gf_auth_model::Identity;
  use std::fs;

  fn get_client() -> GameforgeClient {
    let file = fs::read_to_string("../../resources/identity/identity_only_fingerprint.json")
      .expect("Failed to read identity file.");
    let identity = serde_json::from_str::<Identity>(&file).expect("Failed to parse identity file");

    GameforgeClient::new(identity)
  }

  #[maybe_async::test(feature = "__sync", async(feature = "__async", tokio::test))]
  async fn auth() -> Result<(), Box<dyn std::error::Error>> {
    let client = get_client();

    let res = client
      .authenticate("team.banzar@gmail.com", "g7PxXKc$e$hh7!?n", None)
      .await?;

    println!("{res}");

    Ok(())
  }
}
