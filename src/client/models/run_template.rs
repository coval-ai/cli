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

fn summarize_ids(ids: &[String]) -> String {
    match ids.len() {
        0 => String::new(),
        1 => ids[0].clone(),
        n => format!("{} (+{})", ids[0], n - 1),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunTemplate {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub agent_ids: Vec<String>,
    #[serde(default)]
    pub persona_ids: Vec<String>,
    #[serde(default)]
    pub test_set_ids: Vec<String>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRunTemplateRequest {
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub agent_ids: Vec<String>,
    pub persona_ids: Vec<String>,
    pub test_set_ids: Vec<String>,
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
    /// Tag names. Omitted leaves tags unchanged; an empty list clears them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateRunTemplateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_ids: Option<Vec<String>>,
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
    /// Tag names. Omitted leaves tags unchanged; an empty list clears them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
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
            "AGENTS",
            "PERSONAS",
            "TEST SETS",
            "ITERATIONS",
            "CONCURRENCY",
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            truncate(&self.display_name, 25),
            summarize_ids(&self.agent_ids),
            summarize_ids(&self.persona_ids),
            summarize_ids(&self.test_set_ids),
            self.iteration_count
                .map(|c| c.to_string())
                .unwrap_or_default(),
            self.concurrency.map(|c| c.to_string()).unwrap_or_default(),
        ]
    }
}
