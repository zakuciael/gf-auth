mod common;
#[cfg(feature = "client-reqwest")]
#[cfg(not(all(feature = "client-reqwest", feature = "client-ureq")))]
mod reqwest;

#[cfg(feature = "client-ureq")]
#[cfg(not(all(feature = "client-reqwest", feature = "client-ureq")))]
mod ureq;

#[cfg(any(feature = "client-reqwest", feature = "client-ureq"))]
#[cfg(not(all(feature = "client-reqwest", feature = "client-ureq")))]
pub use crate::common::{BaseHttpClient, CustomCertHttpClient, Form, Headers, HttpError, Query};

#[cfg(feature = "client-reqwest")]
#[cfg(not(all(feature = "client-reqwest", feature = "client-ureq")))]
pub use crate::reqwest::{ReqwestClient as HttpClient, ReqwestClientError as ClientError};

#[cfg(feature = "client-ureq")]
#[cfg(not(all(feature = "client-reqwest", feature = "client-ureq")))]
pub use self::ureq::{UreqClient as HttpClient, UreqClientError as ClientError};

#[cfg(all(feature = "client-reqwest", feature = "client-ureq"))]
compile_error!(
  "`client-reqwest` and `client-ureq` features cannot both be enabled at \
    the same time, if you want to use `client-ureq` you need to set \
    `default-features = false`"
);

#[cfg(not(any(feature = "client-reqwest", feature = "client-ureq")))]
compile_error!(
  "You have to enable at least one of the available clients with the \
    `client-reqwest` or `client-ureq` features."
);

#[cfg(all(feature = "__async", feature = "__sync"))]
compile_error!("Async and Sync versions of HttpClient cannot both be enabled at the same time.");
