use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

fn extract_id(name: &str) -> String {
    name.rsplit('/').next().unwrap_or(name).to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub name: String,
    pub display_name: String,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct CreateDashboardRequest {
    pub display_name: String,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateDashboardRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
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
        vec!["ID", "NAME", "CREATED", "UPDATED"]
    }

    fn row(&self) -> Vec<String> {
        let updated = self
            .update_time
            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default();
        vec![
            extract_id(&self.name),
            truncate(&self.display_name, 30),
            self.create_time.format("%Y-%m-%d %H:%M").to_string(),
            updated,
        ]
    }
}
