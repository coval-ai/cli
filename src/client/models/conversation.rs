use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SubmitConversationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_conversation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurred_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitConversationResponse {
    pub conversation: Conversation,
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

#[derive(Debug)]
pub struct FailureBreakdownParams {
    pub metric_id: String,
    pub group_by_metadata: Option<String>,
    pub failure_query: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub max_examples_per_group: u8,
}

impl FailureBreakdownParams {
    pub fn apply_to(&self, url: &mut Url) {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("view", "failure_breakdown");
        pairs.append_pair("metric_id", &self.metric_id);
        if let Some(ref group_by_metadata) = self.group_by_metadata {
            pairs.append_pair("group_by_metadata", group_by_metadata);
        }
        if let Some(ref failure_query) = self.failure_query {
            pairs.append_pair("failure_query", failure_query);
        }
        if let Some(start_date) = self.start_date {
            pairs.append_pair(
                "start_date",
                &start_date.to_rfc3339_opts(SecondsFormat::AutoSi, true),
            );
        }
        if let Some(end_date) = self.end_date {
            pairs.append_pair(
                "end_date",
                &end_date.to_rfc3339_opts(SecondsFormat::AutoSi, true),
            );
        }
        pairs.append_pair(
            "max_examples_per_group",
            &self.max_examples_per_group.to_string(),
        );
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailureBreakdownResponse {
    pub view: String,
    pub metric_id: String,
    #[serde(default)]
    pub tree_definition: Vec<serde_json::Value>,
    pub group_by_metadata: Option<String>,
    pub failure_query: Option<String>,
    pub scope: FailureBreakdownScope,
    #[serde(default)]
    pub breakdown: Vec<FailureBreakdownRow>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailureBreakdownScope {
    pub requested_start_date: Option<String>,
    pub requested_end_date: Option<String>,
    pub observed_start_date: Option<DateTime<Utc>>,
    pub observed_end_date: Option<DateTime<Utc>>,
    pub total_scored_conversations: u64,
    pub structured_result_conversations: u64,
    pub critical_failure_conversations: u64,
    pub non_critical_failure_conversations: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailureBreakdownRow {
    pub failure_type: String,
    pub node_id: String,
    pub node_label: String,
    pub metadata_value: Option<String>,
    pub conversation_count: u64,
    pub occurrence_count: u64,
    #[serde(default)]
    pub examples: Vec<FailureBreakdownExample>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailureBreakdownExample {
    pub conversation_id: String,
    pub failure: String,
    pub expected_bot_response: Option<String>,
    pub message_index: Option<i64>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationAudioUrlResponse {
    pub audio_url: String,
    pub conversation_id: String,
    pub url_expires_in_seconds: i32,
    #[serde(default)]
    pub peaks_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PatchConversationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_reference: Option<serde_json::Value>,
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
