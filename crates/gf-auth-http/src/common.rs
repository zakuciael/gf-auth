use std::collections::HashMap;
use std::fmt;

use maybe_async::maybe_async;
use serde_json::Value;

pub type Headers = HashMap<String, String>;
pub type Query<'a> = HashMap<&'a str, &'a str>;
pub type Form<'a> = HashMap<&'a str, &'a str>;

#[cfg(feature = "client-ureq")]
#[cfg(not(all(feature = "client-reqwest", feature = "client-ureq")))]
pub type Response = ureq::Response;

#[cfg(feature = "client-reqwest")]
#[cfg(not(all(feature = "client-reqwest", feature = "client-ureq")))]
pub type Response = reqwest::Response;

/// This trait represents the interface to be implemented for an HTTP client,
/// which is kept separate from the gf-auth client for cleaner code.
///
/// When a request doesn't need to pass parameters, the empty or default value
/// of the payload type should be passed, like `json!({})` or `Query::new()`.
/// This avoids using `Option<T>` because `Value` itself may be null in other
/// different ways (`Value::Null`, an empty `Value::Object`...), so this removes
/// redundancy and edge cases (a `Some(Value::Null), for example, doesn't make
/// much sense).
#[maybe_async]
pub trait BaseHttpClient: Send + Default + Clone + fmt::Debug {
  type Error;

  async fn get(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &Query,
  ) -> Result<Response, Self::Error>;

  async fn post(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &Value,
  ) -> Result<Response, Self::Error>;

  async fn post_form(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &Form<'_>,
  ) -> Result<Response, Self::Error>;

  async fn put(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &Value,
  ) -> Result<Response, Self::Error>;

  async fn delete(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &Value,
  ) -> Result<Response, Self::Error>;

  async fn options(&self, url: &str, headers: Option<&Headers>) -> Result<Response, Self::Error>;
}

pub trait CustomCertHttpClient {
  fn with_custom_cert<CA, CLIENT, KEY>(ca: &CA, client: &CLIENT, key: &KEY) -> Self
  where
    CA: AsRef<[u8]>,
    CLIENT: AsRef<[u8]>,
    KEY: AsRef<[u8]>;
}
