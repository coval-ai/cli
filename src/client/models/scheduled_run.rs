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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledRun {
    pub id: String,
    pub display_name: String,
    pub run_template_id: String,
    pub schedule_expression: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_timezone: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run_id: Option<String>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CreateScheduledRunRequest {
    pub display_name: String,
    pub run_template_id: String,
    pub schedule_expression: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateScheduledRunRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_expression: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListScheduledRunsResponse {
    pub scheduled_runs: Vec<ScheduledRun>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetScheduledRunResponse {
    pub scheduled_run: ScheduledRun,
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduledRunResponse {
    pub scheduled_run: ScheduledRun,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduledRunResponse {
    pub scheduled_run: ScheduledRun,
}

impl Tabular for ScheduledRun {
    fn headers() -> Vec<&'static str> {
        vec![
            "ID", "NAME", "TEMPLATE", "SCHEDULE", "TIMEZONE", "ENABLED", "LAST RUN",
        ]
    }

    fn row(&self) -> Vec<String> {
        let last_run = self
            .last_run_at
            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "Never".to_string());
        vec![
            self.id.clone(),
            truncate(&self.display_name, 25),
            self.run_template_id.clone(),
            truncate(&self.schedule_expression, 20),
            self.schedule_timezone
                .clone()
                .unwrap_or_else(|| "UTC".to_string()),
            if self.enabled { "Yes" } else { "No" }.to_string(),
            last_run,
        ]
    }
}
