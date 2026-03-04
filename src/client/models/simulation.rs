use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Simulation {
    pub name: String,
    pub simulation_id: String,
    pub run_id: String,
    pub status: SimulationStatus,
    pub create_time: DateTime<Utc>,
    #[serde(default)]
    pub test_case_id: Option<String>,
    #[serde(default)]
    pub has_audio: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<Vec<TranscriptMessage>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SimulationStatus {
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "IN QUEUE")]
    InQueue,
    #[serde(rename = "IN PROGRESS")]
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

impl std::fmt::Display for SimulationStatus {
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
pub struct TranscriptMessage {
    pub role: String,
    pub content: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_timestamp: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_timestamp: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListSimulationsResponse {
    pub simulations: Vec<Simulation>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetSimulationResponse {
    pub simulation: Simulation,
}

#[derive(Debug, Deserialize)]
pub struct AudioUrlResponse {
    pub audio_url: String,
    pub simulation_id: String,
    pub url_expires_in_seconds: i32,
}

impl Tabular for Simulation {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "STATUS", "RUN", "TEST CASE", "AUDIO"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.simulation_id.clone(),
            self.status.to_string(),
            self.run_id.clone(),
            self.test_case_id
                .as_ref()
                .map_or_else(|| "-".into(), |id| truncate(id, 20)),
            if self.has_audio { "Yes" } else { "No" }.to_string(),
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
