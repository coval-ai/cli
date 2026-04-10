use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSignedUrlResponse {
    pub url: String,
    pub expires_in: i64,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
