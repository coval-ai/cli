use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectSlackResponse {
    pub team_name: String,
    pub connected: bool,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ConnectSlackRequest {
    pub code: String,
    pub redirect_uri: String,
}
