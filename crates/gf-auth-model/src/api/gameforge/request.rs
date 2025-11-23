use serde::Serialize;

use crate::Blackbox;

#[derive(Serialize, Debug)]
pub struct GameforgeAuthRequest<'a> {
  pub blackbox: Blackbox,
  pub email: &'a str,
  pub password: &'a str,
  pub locale: &'a str,
}
