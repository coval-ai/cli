use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub display_name: String,
    pub model_type: AgentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    #[serde(default)]
    pub workflows: serde_json::Value,
    #[serde(default)]
    pub metric_ids: Vec<String>,
    #[serde(default)]
    pub test_set_ids: Vec<String>,
    #[serde(default)]
    pub knowledge_base_ids: Vec<String>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum AgentType {
    #[serde(rename = "MODEL_TYPE_VOICE")]
    #[value(name = "voice")]
    Voice,
    #[serde(rename = "MODEL_TYPE_OUTBOUND_VOICE")]
    #[value(name = "outbound-voice")]
    OutboundVoice,
    #[serde(rename = "MODEL_TYPE_CHAT")]
    #[value(name = "chat")]
    Chat,
    #[serde(rename = "MODEL_TYPE_SMS")]
    #[value(name = "sms")]
    Sms,
    #[serde(rename = "MODEL_TYPE_WEBSOCKET")]
    #[value(name = "websocket")]
    Websocket,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Voice => write!(f, "VOICE"),
            Self::OutboundVoice => write!(f, "OUTBOUND"),
            Self::Chat => write!(f, "CHAT"),
            Self::Sms => write!(f, "SMS"),
            Self::Websocket => write!(f, "WEBSOCKET"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateAgentRequest {
    pub display_name: String,
    pub model_type: AgentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_ids: Option<Vec<String>>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateAgentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_type: Option<AgentType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ListAgentsResponse {
    pub agents: Vec<Agent>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetAgentResponse {
    pub agent: Agent,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentResponse {
    pub agent: Agent,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentResponse {
    pub agent: Agent,
}

impl Tabular for Agent {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "NAME", "TYPE", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            truncate(&self.display_name, 30),
            self.model_type.to_string(),
            self.create_time.format("%Y-%m-%d %H:%M").to_string(),
        ]
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
