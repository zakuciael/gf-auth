mod r#impl;
mod utils;

use crate::reqwest::utils::convert_headers;
use maybe_async::async_impl;
use reqwest::{Client, ClientBuilder, Error, Method, RequestBuilder};
use serde::Serialize;

use crate::common::{CustomCertHttpClient, Headers, HttpError, HttpResponse, Query};

#[cfg(all(
  any(
    feature = "reqwest-default-tls",
    feature = "reqwest-native-tls",
    feature = "reqwest-native-tls-vendored"
  ),
  feature = "reqwest-rustls-tls"
))]
compile_error!(
  "`reqwest-default-tls` / `reqwest-native-tls` / `reqwest-native-tls-vendored` \
  and `reqwest-rustls-tls` features cannot be enabled at the same time."
);

#[cfg(all(
  any(feature = "reqwest-default-tls", feature = "reqwest-native-tls"),
  feature = "reqwest-native-tls-vendored"
))]
compile_error!(
  "`reqwest-default-tls` / `reqwest-native-tls` and `reqwest-native-tls-vendored` \
  features cannot be enabled at the same time."
);

pub type ReqwestClientError = Error;

#[derive(Debug, Clone)]
pub struct ReqwestClient {
  #[allow(dead_code)]
  client: Client,
}

impl Default for ReqwestClient {
  fn default() -> Self {
    let client = ClientBuilder::new()
      .build()
      // building with these options cannot fail
      .unwrap();
    Self { client }
  }
}

impl ReqwestClient {
  async fn request<D>(
    &self,
    method: Method,
    url: &str,
    headers: Option<&Headers>,
    add_data: D,
  ) -> Result<HttpResponse, HttpError<Error>>
  where
    D: Fn(RequestBuilder) -> RequestBuilder,
  {
    let mut request = self.client.request(method.clone(), url);

    if let Some(headers) = headers {
      request = request.headers(headers.try_into().unwrap());
    }

    request = add_data(request);

    log::info!("Making request {:?}", request);
    let response = request.send().await?;

    if response.status().is_success() {
      Ok(HttpResponse::new(
        response.status().as_u16(),
        convert_headers(response.headers()),
        response.text().await?,
      ))
    } else {
      Err(response.into())
    }
  }
}

#[async_impl]
impl crate::common::BaseHttpClient for ReqwestClient {
  type Error = HttpError<Error>;

  #[inline]
  async fn get(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &Query,
  ) -> Result<HttpResponse, Self::Error> {
    self
      .request(Method::GET, url, headers, |req| req.query(payload))
      .await
  }

  #[inline]
  async fn post<T>(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &T,
  ) -> Result<HttpResponse, Self::Error>
  where
    T: Serialize + Send + ?Sized + Sync,
  {
    self
      .request(Method::POST, url, headers, |req| req.json(payload))
      .await
  }

  #[inline]
  async fn put<T>(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &T,
  ) -> Result<HttpResponse, Self::Error>
  where
    T: Serialize + Send + ?Sized + Sync,
  {
    self
      .request(Method::PUT, url, headers, |req| req.json(payload))
      .await
  }

  #[inline]
  async fn delete<T>(
    &self,
    url: &str,
    headers: Option<&Headers>,
    payload: &T,
  ) -> Result<HttpResponse, Self::Error>
  where
    T: Serialize + Send + ?Sized + Sync,
  {
    self
      .request(Method::DELETE, url, headers, |req| req.json(payload))
      .await
  }

  async fn options(&self, url: &str, headers: &Headers) -> Result<HttpResponse, Self::Error> {
    self
      .request(Method::OPTIONS, url, Some(headers), |req| req)
      .await
  }
}
#[cfg(any(
  feature = "reqwest-native-tls",
  feature = "reqwest-native-tls-vendored",
  feature = "reqwest-default-tls",
  feature = "reqwest-rustls-tls"
))]
impl CustomCertHttpClient for ReqwestClient {
  #[cfg(any(
    feature = "reqwest-native-tls",
    feature = "reqwest-native-tls-vendored",
    feature = "reqwest-default-tls"
  ))]
  fn with_custom_cert<CA, CLIENT, KEY>(ca: &CA, client: &CLIENT, key: &KEY) -> Self
  where
    CA: AsRef<[u8]>,
    CLIENT: AsRef<[u8]>,
    KEY: AsRef<[u8]>,
  {
    let identity = reqwest::Identity::from_pkcs8_pem(client.as_ref(), key.as_ref())
      .expect("Failed to read client certificate");

    let root =
      reqwest::Certificate::from_pem(ca.as_ref()).expect("Failed to read CA root certificate");

    let client = Client::builder()
      .identity(identity)
      .add_root_certificate(root)
      .use_native_tls()
      .build()
      .expect("Failed to initialize TLS");

    Self { client }
  }

  #[cfg(feature = "reqwest-rustls-tls")]
  fn with_custom_cert<CA, CLIENT, KEY>(ca: &CA, client: &CLIENT, key: &KEY) -> Self
  where
    CA: AsRef<[u8]>,
    CLIENT: AsRef<[u8]>,
    KEY: AsRef<[u8]>,
  {
    let identity = {
      let mut pem = vec![];
      pem.extend_from_slice(key.as_ref());
      pem.extend_from_slice(client.as_ref());
      reqwest::Identity::from_pem(&pem).expect("Failed to read client certificate")
    };

    let root =
      reqwest::Certificate::from_pem(ca.as_ref()).expect("Failed to read CA root certificate");

    let client = Client::builder()
      .identity(identity)
      .add_root_certificate(root)
      .use_rustls_tls()
      .build()
      .expect("Failed to initialize TLS");

    Self { client }
  }
}

#[cfg(test)]
mod tests {
  use crate::common::{BaseHttpClient, CustomCertHttpClient, HttpError};
  use crate::HttpClient;
  use reqwest::Error;

  async fn create_response_with_custom_cert() -> Result<(), HttpError<Error>> {
    let client = HttpClient::with_custom_cert(
      include_bytes!("../../../../resources/ca.pem"),
      include_bytes!("../../../../resources/client.pem"),
      include_bytes!("../../../../resources/key.pem"),
    );

    let response = client
      .get(
        "https://events2.gameforge.com/",
        Default::default(),
        &Default::default(),
      )
      .await?;

    assert_eq!(response.status(), 200);
    Ok(())
  }

  #[tokio::test]
  #[cfg(feature = "reqwest-rustls-tls")]
  async fn create_client_with_custom_cert_rustls_tls() -> Result<(), HttpError<Error>> {
    println!("Testing reqwest with rustls-tls");
    create_response_with_custom_cert().await
  }

  #[tokio::test]
  #[cfg(any(
    feature = "reqwest-default-tls",
    feature = "reqwest-native-tls",
    feature = "reqwest-native-tls-vendored"
  ))]
  async fn create_client_with_custom_cert_native_tls() -> Result<(), HttpError<Error>> {
    println!("Testing reqwest with native-tls");
    create_response_with_custom_cert().await
  }
}
