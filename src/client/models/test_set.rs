use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSet {
    pub name: String,
    pub id: String,
    pub slug: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_type: Option<String>,
    #[serde(default)]
    pub test_set_metadata: serde_json::Value,
    #[serde(default)]
    pub parameters: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_case_count: Option<i32>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CreateTestSetRequest {
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateTestSetRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListTestSetsResponse {
    pub test_sets: Vec<TestSet>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetTestSetResponse {
    pub test_set: TestSet,
}

#[derive(Debug, Deserialize)]
pub struct CreateTestSetResponse {
    pub test_set: TestSet,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTestSetResponse {
    pub test_set: TestSet,
}

impl Tabular for TestSet {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "NAME", "TYPE", "CASES", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            truncate(&self.display_name, 25),
            self.test_set_type.clone().unwrap_or_else(|| "-".into()),
            self.test_case_count
                .map_or_else(|| "-".into(), |c| c.to_string()),
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
