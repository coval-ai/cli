use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub run_ids: Vec<String>,
    pub compare_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_key: Option<String>,
    pub permissions: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum CompareBy {
    #[serde(rename = "none")]
    #[value(name = "none")]
    None,
    #[serde(rename = "run")]
    #[value(name = "run")]
    Run,
    #[serde(rename = "agent")]
    #[value(name = "agent")]
    Agent,
    #[serde(rename = "mutation")]
    #[value(name = "mutation")]
    Mutation,
    #[serde(rename = "persona")]
    #[value(name = "persona")]
    Persona,
    #[serde(rename = "test_case")]
    #[value(name = "test_case")]
    TestCase,
    #[serde(rename = "metadata")]
    #[value(name = "metadata")]
    Metadata,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum ReportPermission {
    #[serde(rename = "PUBLIC")]
    #[value(name = "PUBLIC")]
    Public,
    #[serde(rename = "PRIVATE")]
    #[value(name = "PRIVATE")]
    Private,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateReportRequest {
    pub name: String,
    pub run_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compare_by: Option<CompareBy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<ReportPermission>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateReportRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compare_by: Option<CompareBy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<ReportPermission>,
}

#[derive(Debug, Deserialize)]
pub struct ListReportsResponse {
    pub reports: Vec<Report>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetReportResponse {
    pub report: Report,
}

#[derive(Debug, Deserialize)]
pub struct CreateReportResponse {
    pub report: Report,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReportResponse {
    pub report: Report,
}

impl Tabular for Report {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "NAME", "COMPARE BY", "RUNS", "PERMISSIONS"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            truncate(&self.name, 30),
            self.compare_by.clone(),
            self.run_ids.len().to_string(),
            self.permissions.clone(),
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
