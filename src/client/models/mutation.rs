use std::collections::HashMap;

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
pub struct Mutation {
    pub id: String,
    pub agent_id: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub config_overrides: serde_json::Value,
    #[serde(default)]
    pub parameter_values: HashMap<String, String>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Tabular for Mutation {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "NAME", "PARAMETERS", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        let params = self
            .parameter_values
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(", ");
        vec![
            self.id.clone(),
            truncate(&self.display_name, 25),
            truncate(&params, 30),
            self.create_time.format("%Y-%m-%d %H:%M").to_string(),
        ]
    }
}

#[derive(Debug, Serialize)]
pub struct CreateMutationRequest {
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_overrides: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_values: Option<HashMap<String, String>>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateMutationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_overrides: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_values: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct ListMutationsResponse {
    pub mutations: Vec<Mutation>,
    pub next_page_token: Option<String>,
    pub total_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct GetMutationResponse {
    pub mutation: Mutation,
}

#[derive(Debug, Deserialize)]
pub struct CreateMutationResponse {
    pub mutation: Mutation,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMutationResponse {
    pub mutation: Mutation,
}
