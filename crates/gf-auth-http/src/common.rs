use std::collections::HashMap;
use std::error::Error;
use std::{fmt, io};

use maybe_async::maybe_async;
use serde::Serialize;

pub type Headers = HashMap<String, String>;
pub type Query<'a> = HashMap<&'a str, &'a str>;
pub type Form<'a> = HashMap<&'a str, &'a str>;

#[derive(thiserror::Error, Debug)]
pub enum HttpError<T: Error> {
  #[error("status code: {status}")]
  Status { status: u16, headers: Headers },
  #[error("request: {0}")]
  Client(T),
  #[error("I/O: {0}")]
  IO(#[from] io::Error),
}

impl<T: Error> HttpError<T> {
  pub fn from_status(status: u16, headers: Headers) -> Self {
    HttpError::Status { status, headers }
  }

  pub fn from_client(err: T) -> Self {
    HttpError::Client(err)
  }
}

pub struct HttpResponse {
  status: u16,
  headers: Headers,
  body: String,
}

impl HttpResponse {
  pub fn new(status: u16, headers: Headers, body: String) -> Self {
    HttpResponse {
      status,
      headers,
      body,
    }
  }

  pub fn status(&self) -> u16 {
    self.status
  }

  pub fn headers(&self) -> &Headers {
    &self.headers
  }
  pub fn body(&self) -> &str {
    &self.body
  }
}

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
  ) -> Result<HttpResponse, Self::Error>;

  async fn post<T>(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &T,
  ) -> Result<HttpResponse, Self::Error>
  where
    T: Serialize + Send + ?Sized + Sync;

  async fn post_form(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &Form<'_>,
  ) -> Result<HttpResponse, Self::Error>;

  async fn put<T>(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &T,
  ) -> Result<HttpResponse, Self::Error>
  where
    T: Serialize + Send + ?Sized + Sync;

  async fn delete<T>(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &T,
  ) -> Result<HttpResponse, Self::Error>
  where
    T: Serialize + Send + ?Sized + Sync;

  async fn options(
    &self,
    url: &str,
    headers: Option<&Headers>,
  ) -> Result<HttpResponse, Self::Error>;
}

pub trait CustomCertHttpClient {
  fn with_custom_cert<CA, CLIENT, KEY>(ca: &CA, client: &CLIENT, key: &KEY) -> Self
  where
    CA: AsRef<[u8]>,
    CLIENT: AsRef<[u8]>,
    KEY: AsRef<[u8]>;
}
