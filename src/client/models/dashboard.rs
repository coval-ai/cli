use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let end: String = s.chars().take(max - 3).collect();
        format!("{}...", end)
    }
}

fn extract_id(name: &str) -> String {
    name.rsplit('/').next().unwrap_or(name).to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_favorite: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDashboardRequest {
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_favorite: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateDashboardRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_favorite: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListDashboardsResponse {
    pub dashboards: Vec<Dashboard>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetDashboardResponse {
    pub dashboard: Dashboard,
}

#[derive(Debug, Deserialize)]
pub struct CreateDashboardResponse {
    pub dashboard: Dashboard,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDashboardResponse {
    pub dashboard: Dashboard,
}

impl Tabular for Dashboard {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "NAME", "DEFAULT", "CREATED", "UPDATED"]
    }

    fn row(&self) -> Vec<String> {
        let updated = self
            .update_time
            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default();
        let default = if self.is_default.unwrap_or(false) {
            "yes".to_string()
        } else {
            String::new()
        };
        vec![
            extract_id(&self.name),
            truncate(self.display_name.as_deref().unwrap_or(""), 30),
            default,
            self.create_time.format("%Y-%m-%d %H:%M").to_string(),
            updated,
        ]
    }
}
