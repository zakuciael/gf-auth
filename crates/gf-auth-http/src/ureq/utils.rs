use crate::common::Headers;
use ureq::Response;

pub(crate) fn convert_headers(response: &Response) -> Headers {
  response
    .headers_names()
    .iter()
    .filter_map(|key| {
      let value = match response.header(key) {
        Some(value) => value.to_string(),
        None => {
          log::error!("malformed header received: {key}");
          return None;
        }
      };

      Some((key.to_string().to_lowercase(), value))
    })
    .collect()
}
