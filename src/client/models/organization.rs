use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub slug: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_metrics: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateOrganizationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_metrics: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct GetOrganizationResponse {
    pub organization: Organization,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrganizationResponse {
    pub organization: Organization,
}

impl Tabular for Organization {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "SLUG", "NAME", "PLAN", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.slug.clone(),
            truncate(&self.display_name, 25),
            self.plan.clone().unwrap_or_else(|| "-".into()),
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
