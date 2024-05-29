use std::error::Error as StdError;

use crate::common::{Headers, HttpError};
use crate::reqwest::utils::convert_headers;
use reqwest::{Error, Response};

impl From<Error> for HttpError<Error> {
  fn from(error: Error) -> Self {
    match error.status() {
      Some(status) => HttpError::Status {
        status: status.as_u16(),
        headers: Headers::new(),
      },
      None => HttpError::Client(error),
    }
  }
}

impl<T: StdError> From<Response> for HttpError<T> {
  fn from(response: Response) -> Self {
    HttpError::Status {
      status: response.status().as_u16(),
      headers: convert_headers(response.headers()),
    }
  }
}
