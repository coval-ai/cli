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
    #[serde(rename = "custom")]
    #[value(name = "custom")]
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum ReportViewMode {
    #[serde(rename = "rows")]
    #[value(name = "rows")]
    Rows,
    #[serde(rename = "grouped")]
    #[value(name = "grouped")]
    Grouped,
}

/// One named bucket of simulations inside a report's custom dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportCustomDimensionGroup {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub simulation_ids: Vec<String>,
}

/// A caller-defined grouping of a report's simulations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportCustomDimension {
    pub id: String,
    pub name: String,
    pub groups: Vec<ReportCustomDimensionGroup>,
    pub hide_unassigned: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum ReportPermission {
    #[serde(rename = "PUBLIC")]
    #[value(name = "public")]
    Public,
    #[serde(rename = "PRIVATE")]
    #[value(name = "private")]
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
    pub custom_dimensions: Option<Vec<ReportCustomDimension>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_dimension_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_mode: Option<ReportViewMode>,
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

/// One simulation in a report, with its metric outputs kept in `extra` for JSON output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRow {
    pub simulation_id: String,
    pub run_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListReportRowsResponse {
    #[serde(default)]
    pub rows: Vec<ReportRow>,
    pub next_page_token: Option<String>,
}

impl Tabular for ReportRow {
    fn headers() -> Vec<&'static str> {
        vec!["SIMULATION ID", "RUN ID", "AGENT", "PERSONA", "STATUS"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.simulation_id.clone(),
            self.run_id.clone(),
            self.agent_id.clone().unwrap_or_default(),
            self.persona_id.clone().unwrap_or_default(),
            self.status.clone().unwrap_or_default(),
        ]
    }
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
