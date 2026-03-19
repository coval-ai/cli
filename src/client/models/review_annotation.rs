use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewAnnotation {
    pub id: String,
    pub simulation_output_id: String,
    pub metric_id: String,
    pub assignee: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ground_truth_float_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ground_truth_string_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ground_truth_subvalues_by_timestamp: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer_notes: Option<String>,
    pub status: AnnotationStatus,
    pub completion_status: CompletionStatus,
    pub priority: AnnotationPriority,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum AnnotationStatus {
    #[serde(rename = "ACTIVE")]
    #[value(name = "active")]
    Active,
    #[serde(rename = "ARCHIVED")]
    #[value(name = "archived")]
    Archived,
}

impl std::fmt::Display for AnnotationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "ACTIVE"),
            Self::Archived => write!(f, "ARCHIVED"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum CompletionStatus {
    #[serde(rename = "PENDING")]
    #[value(name = "pending")]
    Pending,
    #[serde(rename = "COMPLETED")]
    #[value(name = "completed")]
    Completed,
}

impl std::fmt::Display for CompletionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "PENDING"),
            Self::Completed => write!(f, "COMPLETED"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum AnnotationPriority {
    #[serde(rename = "PRIORITY_PRIMARY")]
    #[value(name = "primary")]
    Primary,
    #[serde(rename = "PRIORITY_STANDARD")]
    #[value(name = "standard")]
    Standard,
}

impl std::fmt::Display for AnnotationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primary => write!(f, "PRIMARY"),
            Self::Standard => write!(f, "STANDARD"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateReviewAnnotationRequest {
    pub simulation_output_id: String,
    pub metric_id: String,
    pub assignee: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ground_truth_float_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ground_truth_string_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<AnnotationPriority>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateReviewAnnotationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ground_truth_float_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ground_truth_string_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<AnnotationPriority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_status: Option<CompletionStatus>,
}

#[derive(Debug, Deserialize)]
pub struct ListReviewAnnotationsResponse {
    pub review_annotations: Vec<ReviewAnnotation>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetReviewAnnotationResponse {
    pub review_annotation: ReviewAnnotation,
}

#[derive(Debug, Deserialize)]
pub struct CreateReviewAnnotationResponse {
    pub review_annotation: ReviewAnnotation,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReviewAnnotationResponse {
    pub review_annotation: ReviewAnnotation,
}

impl Tabular for ReviewAnnotation {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "ASSIGNEE", "STATUS", "COMPLETION", "PRIORITY", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            truncate(&self.assignee, 30),
            self.status.to_string(),
            self.completion_status.to_string(),
            self.priority.to_string(),
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
