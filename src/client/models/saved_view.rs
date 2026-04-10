use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedView {
    pub id: String,
    pub name: String,
    pub page_context: String,
    pub config: serde_json::Value,
    #[serde(default)]
    pub is_default: bool,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CreateSavedViewRequest {
    pub name: String,
    pub page_context: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateSavedViewRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListSavedViewsResponse {
    pub saved_views: Vec<SavedView>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetSavedViewResponse {
    pub saved_view: SavedView,
}

#[derive(Debug, Deserialize)]
pub struct CreateSavedViewResponse {
    pub saved_view: SavedView,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSavedViewResponse {
    pub saved_view: SavedView,
}

impl Tabular for SavedView {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "NAME", "CONTEXT", "DEFAULT", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            truncate(&self.name, 25),
            self.page_context.clone(),
            if self.is_default { "Yes" } else { "No" }.to_string(),
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
