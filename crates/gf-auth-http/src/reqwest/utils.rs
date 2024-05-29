use crate::common::Headers;
use reqwest::header::HeaderMap;

pub(crate) fn convert_headers(raw: &HeaderMap) -> Headers {
  raw
    .iter()
    .filter_map(|(key, value)| {
      let value = match value.to_str() {
        Ok(value) => value.to_string(),
        Err(_) => {
          log::error!("malformed header received: {key}");
          return None;
        }
      };

      Some((key.to_string().to_lowercase(), value))
    })
    .collect()
}
