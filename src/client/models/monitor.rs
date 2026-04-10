use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    #[serde(alias = "ulid")]
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: String,
    pub evaluation_type: String,
    pub scope: String,
    pub match_mode: String,
    pub cooldown_seconds: i64,
    pub trigger_count: i64,
    #[serde(default)]
    pub conditions: Vec<serde_json::Value>,
    #[serde(default)]
    pub channels: Vec<serde_json::Value>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorEvent {
    pub id: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListMonitorsResponse {
    pub monitors: Vec<Monitor>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetMonitorResponse {
    pub monitor: Monitor,
}

#[derive(Debug, Deserialize)]
pub struct CreateMonitorResponse {
    pub monitor: Monitor,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMonitorResponse {
    pub monitor: Monitor,
}

#[derive(Debug, Deserialize)]
pub struct ListMonitorEventsResponse {
    pub events: Vec<MonitorEvent>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestEvaluateResponse {
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Tabular for Monitor {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "NAME", "STATUS", "TYPE", "SCOPE", "TRIGGERS", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            truncate(&self.name, 25),
            self.status.clone(),
            self.evaluation_type.clone(),
            self.scope.clone(),
            self.trigger_count.to_string(),
            self.create_time.format("%Y-%m-%d %H:%M").to_string(),
        ]
    }
}

impl Tabular for MonitorEvent {
    fn headers() -> Vec<&'static str> {
        vec!["ID"]
    }

    fn row(&self) -> Vec<String> {
        vec![self.id.clone()]
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
