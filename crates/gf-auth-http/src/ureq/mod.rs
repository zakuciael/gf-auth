mod r#impl;
mod utils;

use std::io::Read;
use std::time::Duration;

use crate::common::{
  BaseHttpClient, CustomCertHttpClient, Headers, HttpError, HttpResponse, Query,
};
use crate::ureq::utils::convert_headers;
use maybe_async::sync_impl;
use serde::Serialize;
use ureq::{Agent, AgentBuilder, Error, Request, Response};

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

pub type UreqClientError = Error;

#[derive(Debug, Clone)]
pub struct UreqClient {
  agent: Agent,
}

impl Default for UreqClient {
  fn default() -> Self {
    let agent = AgentBuilder::new()
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
  ) -> Result<HttpResponse, HttpError<Error>>
  where
    D: Fn(Request) -> Result<Response, Error>,
  {
    if let Some(headers) = headers {
      for (key, val) in headers.iter() {
        request = request.set(key, val);
      }
    }

    log::info!("Making request {:?}", request);
    match send_request(request) {
      Ok(response) => {
        let mut buf = vec![];
        let headers = convert_headers(&response);
        let status = response.status();
        response.into_reader().read_to_end(&mut buf);

        Ok(HttpResponse::new(status, headers, buf))
      }
      Err(err) => Err(err.into()),
    }
  }
}

#[sync_impl]
impl BaseHttpClient for UreqClient {
  type Error = HttpError<Error>;

  #[inline]
  fn get(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &Query,
  ) -> Result<HttpResponse, Self::Error> {
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
  fn post<T>(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &T,
  ) -> Result<HttpResponse, Self::Error>
  where
    T: Serialize + Send + ?Sized + Sync,
  {
    let request = self.agent.post(url);
    let sender = |req: Request| req.send_json(payload);
    self.request(request, headers, sender)
  }

  #[inline]
  fn put<T>(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &T,
  ) -> Result<HttpResponse, Self::Error>
  where
    T: Serialize + Send + ?Sized + Sync,
  {
    let request = self.agent.put(url);
    let sender = |req: Request| req.send_json(payload);
    self.request(request, headers, sender)
  }

  #[inline]
  fn delete<T>(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &T,
  ) -> Result<HttpResponse, Self::Error>
  where
    T: Serialize + Send + ?Sized + Sync,
  {
    let request = self.agent.delete(url);
    let sender = |req: Request| req.send_json(payload);
    self.request(request, headers, sender)
  }

  fn options(&self, url: &str, headers: &Headers) -> Result<HttpResponse, Self::Error> {
    let request = self.agent.request("OPTIONS", url);
    let sender = |req: Request| req.call();
    self.request(request, Some(headers), sender)
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

    root_store.add_parsable_certificates(
      rustls_pemfile::certs(&mut std::io::Cursor::new(ca.as_ref())).flatten(),
    );

    let private_key = rustls_pemfile::private_key(&mut std::io::Cursor::new(key.as_ref()))
      .and_then(|item| item.ok_or(std::io::Error::from(std::io::ErrorKind::UnexpectedEof)))
      .expect("Failed to read private key");
    let client_certs = rustls_pemfile::certs(&mut std::io::Cursor::new(client.as_ref()))
      .collect::<Result<Vec<_>, _>>()
      .expect("Failed to read client certificate");

    let agent = AgentBuilder::new()
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

    let agent = AgentBuilder::new()
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
  use crate::ureq::UreqClient;
  use crate::HttpError;
  use ureq::Error;

  fn create_response_with_custom_cert() -> Result<(), HttpError<Error>> {
    let client = UreqClient::with_custom_cert(
      include_bytes!("../../../../resources/ca.pem"),
      include_bytes!("../../../../resources/client.pem"),
      include_bytes!("../../../../resources/key.pem"),
    );

    let response = client.get(
      "https://events2.gameforge.com/",
      Default::default(),
      &Default::default(),
    )?;

    assert_eq!(response.status, 200);
    Ok(())
  }

  #[test]
  #[cfg(feature = "ureq-native-tls")]
  fn create_client_with_custom_cert_native_tls() -> Result<(), HttpError<Error>> {
    println!("Testing ureq with native-tls");
    create_response_with_custom_cert()
  }

  #[test]
  #[cfg(any(feature = "ureq-rustls-tls", feature = "ureq-rustls-tls-native-certs"))]
  fn create_client_with_custom_cert_rustls_tls() -> Result<(), HttpError<Error>> {
    println!("Testing ureq with rustls-tls / ureq-rustls-tls-native-certs");
    create_response_with_custom_cert()
  }
}
