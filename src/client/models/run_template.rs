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
pub struct RunTemplate {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_id: Option<String>,
    #[serde(default)]
    pub metric_ids: Vec<String>,
    #[serde(default)]
    pub mutation_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iteration_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_sample_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_sample_seed: Option<u64>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CreateRunTemplateRequest {
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iteration_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_sample_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_sample_seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateRunTemplateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iteration_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_sample_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_sample_seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListRunTemplatesResponse {
    pub run_templates: Vec<RunTemplate>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetRunTemplateResponse {
    pub run_template: RunTemplate,
}

#[derive(Debug, Deserialize)]
pub struct CreateRunTemplateResponse {
    pub run_template: RunTemplate,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRunTemplateResponse {
    pub run_template: RunTemplate,
}

impl Tabular for RunTemplate {
    fn headers() -> Vec<&'static str> {
        vec![
            "ID",
            "NAME",
            "AGENT",
            "PERSONA",
            "TEST SET",
            "ITERATIONS",
            "CONCURRENCY",
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            truncate(&self.display_name, 25),
            self.agent_id.clone().unwrap_or_default(),
            self.persona_id.clone().unwrap_or_default(),
            self.test_set_id.clone().unwrap_or_default(),
            self.iteration_count
                .map(|c| c.to_string())
                .unwrap_or_default(),
            self.concurrency.map(|c| c.to_string()).unwrap_or_default(),
        ]
    }
}
