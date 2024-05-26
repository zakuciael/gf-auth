use super::{BaseHttpClient, Form, Headers, Query};

use std::io::Cursor;
use std::{io, time::Duration};

use crate::common::CustomCertHttpClient;
use maybe_async::sync_impl;
use serde_json::Value;
use ureq::{request, Request};

#[cfg(all(
  any(feature = "ureq-rustls-tls", feature = "ureq-rustls-tls-native-certs"),
  feature = "ureq-native-tls"
))]
compile_error!(
  "`ureq-rustls-tls` / `ureq-rustls-tls-native-certs` and `ureq-native-tls` \
  features cannot be enabled at the same time."
);

#[cfg(all(feature = "ureq-rustls-tls", feature = "ureq-rustls-tls-native-certs"))]
compile_error!(
  "`ureq-rustls-tls` and `ureq-rustls-tls-native-certs` \
  features cannot be enabled at the same time."
);

#[derive(thiserror::Error, Debug)]
pub enum UreqError {
  /// The request couldn't be completed because there was an error when trying to do so
  #[error("transport: {0}")]
  Transport(#[from] ureq::Transport),

  /// There was an error when trying to decode the response
  #[error("I/O: {0}")]
  Io(#[from] io::Error),

  /// The request was made, but the server returned an unsuccessful status
  /// code, such as 404 or 503.
  #[error("status code {}", ureq::Response::status(.0))]
  StatusCode(ureq::Response),
}

#[derive(Debug, Clone)]
pub struct UreqClient {
  agent: ureq::Agent,
}

impl Default for UreqClient {
  fn default() -> Self {
    let agent = ureq::AgentBuilder::new()
      .try_proxy_from_env(true)
      .timeout(Duration::from_secs(10));

    #[cfg(feature = "ureq-native-tls")]
    let agent = agent.tls_connector(std::sync::Arc::new(
      native_tls::TlsConnector::builder()
        // rust-native-tls defaults to a minimum of TLS 1.0, which is insecure
        .min_protocol_version(Some(native_tls::Protocol::Tlsv12))
        .build()
        .expect("Failed to initialize TLS"),
    ));

    Self {
      agent: agent.build(),
    }
  }
}

impl UreqClient {
  fn request<D>(
    &self,
    mut request: Request,
    headers: Option<&Headers>,
    send_request: D,
  ) -> Result<ureq::Response, UreqError>
  where
    D: Fn(Request) -> Result<ureq::Response, ureq::Error>,
  {
    if let Some(headers) = headers {
      for (key, val) in headers.iter() {
        request = request.set(key, val);
      }
    }

    log::info!("Making request {:?}", request);
    match send_request(request) {
      Ok(response) => Ok(response),
      Err(err) => match err {
        ureq::Error::Status(_, response) => Err(UreqError::StatusCode(response)),
        ureq::Error::Transport(transport) => Err(UreqError::Transport(transport)),
      },
    }
  }
}

#[sync_impl]
impl BaseHttpClient for UreqClient {
  type Error = UreqError;

  #[inline]
  fn get(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &Query,
  ) -> Result<ureq::Response, Self::Error> {
    let request = self.agent.get(url);
    let sender = |mut req: Request| {
      for (key, val) in payload.iter() {
        req = req.query(key, val);
      }
      req.call()
    };
    self.request(request, headers, sender)
  }

  #[inline]
  fn post(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &Value,
  ) -> Result<ureq::Response, Self::Error> {
    let request = self.agent.post(url);
    let sender = |req: Request| req.send_json(payload.clone());
    self.request(request, headers, sender)
  }

  #[inline]
  fn post_form(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &Form<'_>,
  ) -> Result<ureq::Response, Self::Error> {
    let request = self.agent.post(url);
    let sender = |req: Request| {
      let payload = payload
        .iter()
        .map(|(key, val)| (*key, *val))
        .collect::<Vec<_>>();

      req.send_form(&payload)
    };

    self.request(request, headers, sender)
  }

  #[inline]
  fn put(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &Value,
  ) -> Result<ureq::Response, Self::Error> {
    let request = self.agent.put(url);
    let sender = |req: Request| req.send_json(payload.clone());
    self.request(request, headers, sender)
  }

  #[inline]
  fn delete(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &Value,
  ) -> Result<ureq::Response, Self::Error> {
    let request = self.agent.delete(url);
    let sender = |req: Request| req.send_json(payload.clone());
    self.request(request, headers, sender)
  }

  fn options(&self, url: &str, headers: Option<&Headers>) -> Result<ureq::Response, Self::Error> {
    let request = self.agent.request("OPTIONS", url);
    let sender = |req: Request| req.call();
    self.request(request, headers, sender)
  }
}

impl CustomCertHttpClient for UreqClient {
  #[cfg(any(feature = "ureq-rustls-tls", feature = "ureq-rustls-tls-native-certs"))]
  fn with_custom_cert<CA, CLIENT, KEY>(ca: &CA, client: &CLIENT, key: &KEY) -> Self
  where
    CA: AsRef<[u8]>,
    CLIENT: AsRef<[u8]>,
    KEY: AsRef<[u8]>,
  {
    #[cfg(feature = "ureq-rustls-tls")]
    let mut root_store = rustls::RootCertStore::empty();

    // Copied from ureq source
    #[cfg(feature = "ureq-rustls-tls-native-certs")]
    let mut root_store = {
      let mut root_store = rustls::RootCertStore::empty();
      let native_certs = rustls_native_certs::load_native_certs().unwrap_or_else(|e| {
        log::error!("loading native certificates: {}", e);
        vec![]
      });
      let (valid_count, invalid_count) =
        root_store.add_parsable_certificates(native_certs.into_iter().map(|c| c.into()));
      if valid_count == 0 && invalid_count > 0 {
        log::error!(
          "no valid certificates loaded by rustls-native-certs. all HTTPS requests will fail."
        );
      }
      root_store
    };

    root_store
      .add_parsable_certificates(rustls_pemfile::certs(&mut Cursor::new(ca.as_ref())).flatten());

    let private_key = rustls_pemfile::private_key(&mut Cursor::new(key.as_ref()))
      .and_then(|item| item.ok_or(io::Error::from(io::ErrorKind::UnexpectedEof)))
      .expect("Failed to read private key");
    let client_certs = rustls_pemfile::certs(&mut Cursor::new(client.as_ref()))
      .collect::<Result<Vec<_>, _>>()
      .expect("Failed to read client certificate");

    let agent = ureq::AgentBuilder::new()
      .try_proxy_from_env(true)
      .timeout(Duration::from_secs(10))
      .tls_config(std::sync::Arc::new(
        rustls::ClientConfig::builder()
          .with_root_certificates(root_store)
          .with_client_auth_cert(client_certs, private_key)
          .expect("Failed to initialize TLS"),
      ))
      .build();

    Self { agent }
  }

  #[cfg(feature = "ureq-native-tls")]
  fn with_custom_cert<CA, CLIENT, KEY>(ca: &CA, client: &CLIENT, key: &KEY) -> Self
  where
    CA: AsRef<[u8]>,
    CLIENT: AsRef<[u8]>,
    KEY: AsRef<[u8]>,
  {
    let identity = native_tls::Identity::from_pkcs8(client.as_ref(), key.as_ref())
      .expect("Failed to read client certificate");

    let root =
      native_tls::Certificate::from_pem(ca.as_ref()).expect("Failed to read CA root certificate");

    let agent = ureq::AgentBuilder::new()
      .try_proxy_from_env(true)
      .timeout(Duration::from_secs(10))
      .tls_connector(std::sync::Arc::new(
        native_tls::TlsConnector::builder()
          .identity(identity)
          .add_root_certificate(root)
          .build()
          .expect("Failed to initialize TLS"),
      ))
      .build();

    Self { agent }
  }
}

#[cfg(test)]
mod tests {
  use crate::common::{BaseHttpClient, CustomCertHttpClient};
  use crate::ureq::{UreqClient, UreqError};

  fn create_response_with_custom_cert() -> Result<(), UreqError> {
    let client = UreqClient::with_custom_cert(
      include_bytes!("../../../resources/ca.pem"),
      include_bytes!("../../../resources/client.pem"),
      include_bytes!("../../../resources/key.pem"),
    );

    let response = client.get(
      "https://events2.gameforge.com/",
      Default::default(),
      &Default::default(),
    )?;

    assert_eq!(response.status(), 200);
    Ok(())
  }

  #[test]
  #[cfg(feature = "ureq-native-tls")]
  fn create_client_with_custom_cert_native_tls() -> Result<(), UreqError> {
    println!("Testing ureq with native-tls");
    create_response_with_custom_cert()
  }

  #[test]
  #[cfg(any(feature = "ureq-rustls-tls", feature = "ureq-rustls-tls-native-certs"))]
  fn create_client_with_custom_cert_rustls_tls() -> Result<(), UreqError> {
    println!("Testing ureq with rustls-tls / ureq-rustls-tls-native-certs");
    create_response_with_custom_cert()
  }
}
