use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct UpdateRequest {
    pub key: String,
    pub secret: String,
}
