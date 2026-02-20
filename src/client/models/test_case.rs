use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_id: Option<String>,
    pub input_str: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_output_str: Option<String>,
    #[serde(default)]
    pub expected_output_json: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,
    #[serde(default)]
    pub simulation_metadata_input: serde_json::Value,
    #[serde(default)]
    pub metric_input: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_notes: Option<String>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct CreateTestCaseRequest {
    pub input_str: String,
    pub test_set_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_behaviors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_output_str: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_output_json: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simulation_metadata_input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_notes: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateTestCaseRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_str: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_behaviors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_output_str: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_output_json: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simulation_metadata_input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListTestCasesResponse {
    pub test_cases: Vec<TestCase>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetTestCaseResponse {
    pub test_case: TestCase,
}

#[derive(Debug, Deserialize)]
pub struct CreateTestCaseResponse {
    pub test_case: TestCase,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTestCaseResponse {
    pub test_case: TestCase,
}

impl Tabular for TestCase {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "INPUT", "TYPE", "TEST SET", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            truncate(&self.input_str, 30),
            self.input_type.clone().unwrap_or_else(|| "SCENARIO".into()),
            self.test_set_id.clone().unwrap_or_else(|| "-".into()),
            self.create_time.format("%Y-%m-%d %H:%M").to_string(),
        ]
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
