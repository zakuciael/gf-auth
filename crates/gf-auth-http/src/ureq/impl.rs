use crate::common::HttpError;
use crate::ureq::utils::convert_headers;
use std::error::Error as StdError;
use ureq::{Error, Response};

impl From<Error> for HttpError<Error> {
  fn from(error: Error) -> Self {
    match error {
      Error::Status(_, response) => response.into(),
      Error::Transport(_) => HttpError::Client(error),
    }
  }
}

impl<T: StdError> From<Response> for HttpError<T> {
  fn from(response: Response) -> Self {
    HttpError::Status {
      status: response.status(),
      headers: convert_headers(&response),
    }
  }
}
