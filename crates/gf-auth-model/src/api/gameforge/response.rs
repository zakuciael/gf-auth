use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GameforgeAuthResponse {
  pub token: Uuid,
}
