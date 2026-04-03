use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

use super::TranscriptMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub name: Option<String>,
    pub conversation_id: String,
    pub status: ConversationStatus,
    pub create_time: DateTime<Utc>,
    #[serde(default)]
    pub has_audio: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_conversation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurred_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<ConversationProgress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<Vec<TranscriptMessage>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConversationStatus {
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "IN_QUEUE", alias = "IN QUEUE")]
    InQueue,
    #[serde(rename = "IN_PROGRESS", alias = "IN PROGRESS")]
    InProgress,
    #[serde(rename = "COMPLETED")]
    Completed,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "CANCELLED")]
    Cancelled,
    #[serde(rename = "DELETED")]
    Deleted,
}

impl std::fmt::Display for ConversationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "PENDING"),
            Self::InQueue => write!(f, "IN QUEUE"),
            Self::InProgress => write!(f, "IN PROGRESS"),
            Self::Completed => write!(f, "COMPLETED"),
            Self::Failed => write!(f, "FAILED"),
            Self::Cancelled => write!(f, "CANCELLED"),
            Self::Deleted => write!(f, "DELETED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationProgress {
    #[serde(default)]
    pub total_metrics: i32,
    #[serde(default)]
    pub completed_metrics: i32,
    #[serde(default)]
    pub failed_metrics: i32,
    #[serde(default)]
    pub in_progress_metrics: i32,
}

#[derive(Debug, Deserialize)]
pub struct ListConversationsResponse {
    pub conversations: Vec<Conversation>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetConversationResponse {
    pub conversation: Conversation,
}

#[derive(Debug, Deserialize)]
pub struct ConversationAudioUrlResponse {
    pub audio_url: String,
    pub conversation_id: String,
    pub url_expires_in_seconds: i32,
    #[serde(default)]
    pub peaks_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListConversationMetricsResponse {
    pub metrics: Vec<super::SimpleMetricOutput>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetConversationMetricResponse {
    pub metric: super::SimpleMetricOutput,
}

impl Tabular for Conversation {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "STATUS", "EXTERNAL ID", "AUDIO", "OCCURRED AT"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.conversation_id.clone(),
            self.status.to_string(),
            self.external_conversation_id
                .as_deref()
                .unwrap_or("-")
                .to_string(),
            if self.has_audio { "Yes" } else { "No" }.to_string(),
            self.occurred_at
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "-".into()),
        ]
    }
}
