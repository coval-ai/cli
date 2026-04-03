use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub name: String,
    pub conversation_id: String,
    pub status: String,
    pub create_time: DateTime<Utc>,
    #[serde(default)]
    pub external_conversation_id: Option<String>,
    #[serde(default)]
    pub occurred_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub has_audio: bool,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub persona_id: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub metric_ids: Vec<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub progress: Option<ConversationProgress>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationProgress {
    #[serde(default)]
    pub total_metrics: u32,
    #[serde(default)]
    pub completed_metrics: u32,
    #[serde(default)]
    pub failed_metrics: u32,
    #[serde(default)]
    pub in_progress_metrics: u32,
}

#[derive(Debug, Deserialize)]
pub struct GetConversationResponse {
    pub conversation: Conversation,
}

#[derive(Debug, Deserialize)]
pub struct ListConversationsResponse {
    pub conversations: Vec<Conversation>,
    pub next_page_token: Option<String>,
}

impl Tabular for Conversation {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "STATUS", "EXTERNAL ID", "AUDIO", "OCCURRED AT"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.conversation_id.clone(),
            self.status.clone(),
            self.external_conversation_id
                .as_ref()
                .map_or_else(|| "-".into(), |id| truncate(id, 30)),
            if self.has_audio { "Yes" } else { "No" }.to_string(),
            self.occurred_at
                .as_ref()
                .map_or_else(|| "-".into(), |t| t.format("%Y-%m-%d %H:%M").to_string()),
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
