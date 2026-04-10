use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_id: Option<String>,
    pub status: String,
    pub detection_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_float: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_sigma: Option<f64>,
    pub observation_count: i64,
    pub sigma_threshold: f64,
    pub direction: String,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CreateBaselineRequest {
    pub metric_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sigma_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListBaselinesResponse {
    pub baselines: Vec<Baseline>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetBaselineResponse {
    pub baseline: Baseline,
}

#[derive(Debug, Deserialize)]
pub struct CreateBaselineResponse {
    pub baseline: Baseline,
}

impl Tabular for Baseline {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "NAME", "STATUS", "METHOD", "METRIC", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.display_name
                .as_ref()
                .map_or_else(|| "-".into(), |n| truncate(n, 25)),
            self.status.clone(),
            self.detection_method.clone(),
            self.metric_id.clone().unwrap_or_else(|| "-".into()),
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
