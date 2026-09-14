use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub status: String,
    pub severity: String,
    #[serde(default)]
    pub owner_user_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub finding_ids: Vec<String>,
    #[serde(default)]
    pub suite_test_set_id: Option<String>,
    #[serde(default)]
    pub confirmed_at: Option<String>,
    #[serde(default)]
    pub resolved_at: Option<String>,
    #[serde(default)]
    pub resolution_count: u32,
    #[serde(default)]
    pub recurrence_count: u32,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

fn default_version() -> u32 {
    1
}

impl Tabular for Issue {
    fn headers() -> Vec<&'static str> {
        vec![
            "ID", "Status", "Sev", "Title", "Owner", "Suite", "Resolved", "Recur", "Ver",
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.status.clone(),
            self.severity.clone(),
            self.title.clone(),
            self.owner_user_id.as_deref().unwrap_or("").to_string(),
            self.suite_test_set_id.as_deref().unwrap_or("").to_string(),
            self.resolution_count.to_string(),
            self.recurrence_count.to_string(),
            self.version.to_string(),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListIssuesResponse {
    pub issues: Vec<Issue>,
    #[serde(default)]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueDetailResponse {
    pub issue: Issue,
    #[serde(default)]
    pub events: Vec<serde_json::Value>,
    #[serde(default)]
    pub clearance_attempts: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueActionResponse {
    pub issue: Issue,
    pub event: serde_json::Value,
    #[serde(default)]
    pub clearance_attempt: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionMembership {
    pub id: String,
    pub agent_id: String,
    pub test_set_id: String,
    pub provenance: String,
    pub status: String,
    #[serde(default)]
    pub issue_id: Option<String>,
    #[serde(default)]
    pub proven_test_set_version_id: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Tabular for RegressionMembership {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Agent", "Test Set", "Provenance", "Issue", "Status"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.agent_id.clone(),
            self.test_set_id.clone(),
            self.provenance.clone(),
            self.issue_id.as_deref().unwrap_or("").to_string(),
            self.status.clone(),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRegressionSuiteResponse {
    pub memberships: Vec<RegressionMembership>,
}
