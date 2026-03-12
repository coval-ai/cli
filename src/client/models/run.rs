use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub name: String,
    pub run_id: String,
    pub status: RunStatus,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<RunProgress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<RunResults>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RunStatus {
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

impl std::fmt::Display for RunStatus {
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
pub struct RunProgress {
    pub total_test_cases: i32,
    pub completed_test_cases: i32,
    pub failed_test_cases: i32,
    pub in_progress_test_cases: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResults {
    pub output_ids: Vec<String>,
    pub metrics: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct LaunchRunRequest {
    pub agent_id: String,
    pub persona_id: String,
    pub test_set_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona_metrics: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<LaunchOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<LaunchMetadata>,
}

#[derive(Debug, Default, Serialize)]
pub struct LaunchOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iteration_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_sample_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_sample_seed: Option<u64>,
}

#[derive(Debug, Default, Serialize)]
pub struct LaunchMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListRunsResponse {
    pub runs: Vec<Run>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetRunResponse {
    pub run: Run,
}

#[derive(Debug, Deserialize)]
pub struct LaunchRunResponse {
    pub run: Run,
}

impl Tabular for Run {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "STATUS", "PROGRESS", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        let progress = self.progress.as_ref().map_or_else(
            || "-".to_string(),
            |p| format!("{}/{}", p.completed_test_cases, p.total_test_cases),
        );

        vec![
            self.run_id.clone(),
            self.status.to_string(),
            progress,
            self.create_time.format("%Y-%m-%d %H:%M").to_string(),
        ]
    }
}
