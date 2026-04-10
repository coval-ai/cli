use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub simulation_output_id: String,
    pub message_index: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_text: Option<String>,
    pub content: String,
    pub created_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by_name: Option<String>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_comment_id: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CreateCommentRequest {
    pub content: String,
    pub message_index: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_comment_id: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateCommentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListCommentsResponse {
    pub comments: Vec<Comment>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetCommentResponse {
    pub comment: Comment,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentResponse {
    pub comment: Comment,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentResponse {
    pub comment: Comment,
}

impl Tabular for Comment {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "CONTENT", "AUTHOR", "MSG INDEX", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            truncate(&self.content, 30),
            self.created_by_name
                .as_ref()
                .unwrap_or(&self.created_by)
                .clone(),
            self.message_index.to_string(),
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
