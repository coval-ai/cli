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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
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
    #[serde(rename = "MODEL_TYPE_CHAT_A2A")]
    #[value(name = "chat-a2a")]
    ChatA2a,
    #[serde(rename = "MODEL_TYPE_CHAT_WEBSOCKET")]
    #[value(name = "chat-websocket")]
    ChatWebsocket,
    #[serde(rename = "MODEL_TYPE_SMS")]
    #[value(name = "sms")]
    Sms,
    #[serde(rename = "MODEL_TYPE_WEBSOCKET")]
    #[value(name = "websocket")]
    Websocket,
    #[serde(rename = "MODEL_TYPE_LIVEKIT")]
    #[value(name = "livekit")]
    Livekit,
    #[serde(rename = "MODEL_TYPE_DAILY")]
    #[value(name = "pipecat", alias = "daily")]
    Pipecat,
    #[serde(rename = "MODEL_TYPE_OPENAI_REALTIME")]
    #[value(name = "openai-realtime")]
    OpenAiRealtime,
    #[serde(rename = "MODEL_TYPE_GEMINI_REALTIME")]
    #[value(name = "gemini-realtime")]
    GeminiRealtime,
    #[serde(rename = "MODEL_TYPE_GROK_REALTIME")]
    #[value(name = "grok-realtime")]
    GrokRealtime,
    #[serde(other)]
    #[value(skip)]
    Unknown,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Voice => write!(f, "VOICE"),
            Self::OutboundVoice => write!(f, "OUTBOUND"),
            Self::Chat => write!(f, "CHAT"),
            Self::ChatA2a => write!(f, "CHAT_A2A"),
            Self::ChatWebsocket => write!(f, "CHAT_WEBSOCKET"),
            Self::Sms => write!(f, "SMS"),
            Self::Websocket => write!(f, "WEBSOCKET"),
            Self::Livekit => write!(f, "LIVEKIT"),
            Self::Pipecat => write!(f, "PIPECAT"),
            Self::OpenAiRealtime => write!(f, "OPENAI_REALTIME"),
            Self::GeminiRealtime => write!(f, "GEMINI_REALTIME"),
            Self::GrokRealtime => write!(f, "GROK_REALTIME"),
            Self::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_agent_id: Option<String>,
    pub display_name: String,
    pub model_type: AgentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflows: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
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
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let end: String = s.chars().take(max - 3).collect();
        format!("{}...", end)
    }
}

#[cfg(test)]
mod tests {
    use super::AgentType;

    #[test]
    fn serializes_every_creatable_agent_type() {
        let cases = [
            (AgentType::Voice, "MODEL_TYPE_VOICE"),
            (AgentType::OutboundVoice, "MODEL_TYPE_OUTBOUND_VOICE"),
            (AgentType::Chat, "MODEL_TYPE_CHAT"),
            (AgentType::ChatA2a, "MODEL_TYPE_CHAT_A2A"),
            (AgentType::ChatWebsocket, "MODEL_TYPE_CHAT_WEBSOCKET"),
            (AgentType::Sms, "MODEL_TYPE_SMS"),
            (AgentType::Websocket, "MODEL_TYPE_WEBSOCKET"),
            (AgentType::Livekit, "MODEL_TYPE_LIVEKIT"),
            (AgentType::Pipecat, "MODEL_TYPE_DAILY"),
            (AgentType::OpenAiRealtime, "MODEL_TYPE_OPENAI_REALTIME"),
            (AgentType::GeminiRealtime, "MODEL_TYPE_GEMINI_REALTIME"),
            (AgentType::GrokRealtime, "MODEL_TYPE_GROK_REALTIME"),
        ];

        for (agent_type, model_type) in cases {
            assert_eq!(serde_json::to_value(agent_type).unwrap(), model_type);
            assert_eq!(
                serde_json::from_value::<AgentType>(model_type.into()).unwrap(),
                agent_type
            );
        }
    }
}
