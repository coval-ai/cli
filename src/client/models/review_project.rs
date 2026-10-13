use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewProject {
    pub id: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub assignees: Vec<String>,
    #[serde(default)]
    pub opted_out_assignees: Vec<String>,
    #[serde(default)]
    pub linked_simulation_ids: Vec<String>,
    #[serde(default)]
    pub linked_metric_ids: Vec<String>,
    pub project_type: ProjectType,
    pub notifications: bool,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum ProjectType {
    #[serde(rename = "PROJECT_COLLABORATIVE")]
    #[value(name = "collaborative")]
    Collaborative,
    #[serde(rename = "PROJECT_INDIVIDUAL")]
    #[value(name = "individual")]
    Individual,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Collaborative => write!(f, "COLLABORATIVE"),
            Self::Individual => write!(f, "INDIVIDUAL"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateReviewProjectRequest {
    pub display_name: String,
    /// Reviewer emails. Left out when `assignee_user_ids` selects the reviewers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assignees: Vec<String>,
    /// Reviewer user IDs, instead of `assignees`; the API rejects a request that
    /// sets both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_user_ids: Option<Vec<String>>,
    pub linked_simulation_ids: Vec<String>,
    pub linked_metric_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_type: Option<ProjectType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifications: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_rules: Option<Vec<String>>,
    /// Must be a subset of `linked_metric_ids`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blind_labeling_shown_metric_ids: Option<Vec<String>>,
    /// Only takes effect on a collaborative project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforced_collaboration: Option<bool>,
    /// Lets reviewers skip linked metrics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_metric_skipping: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_label_input_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_label_options: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_label_selection_mode: Option<String>,
    /// False pauses conversation triage while keeping its categories, modes and
    /// saved labels; true enables it. Left out, activation follows the triage
    /// configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_label_enabled: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateReviewProjectRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
    /// Reviewer user IDs, instead of `assignees`; the API rejects a request that
    /// sets both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_user_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_simulation_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_metric_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_type: Option<ProjectType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifications: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opted_out_assignees: Option<Vec<String>>,
    /// Reviewer user IDs opting out of notifications, instead of
    /// `opted_out_assignees`; an empty list clears the opt-outs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opted_out_assignee_user_ids: Option<Vec<String>>,
    /// Adds to the linked simulations instead of replacing them; cannot be combined
    /// with `linked_simulation_ids`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_linked_simulation_ids: Option<Vec<String>>,
    /// Removes from the linked simulations instead of replacing them; cannot be
    /// combined with `linked_simulation_ids`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_linked_simulation_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_rules: Option<Vec<String>>,
    /// Must be a subset of `linked_metric_ids`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blind_labeling_shown_metric_ids: Option<Vec<String>>,
    /// What to do with already-completed assignments when linking a new metric.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_addition_completion_action: Option<String>,
    /// Only takes effect on a collaborative project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforced_collaboration: Option<bool>,
    /// Lets reviewers skip linked metrics. Left out, the saved setting is kept.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_metric_skipping: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_label_input_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_label_options: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_label_selection_mode: Option<String>,
    /// False pauses conversation triage while keeping its categories, modes and
    /// saved labels; true enables it. An explicit null returns activation to the
    /// triage configuration, and leaving it out keeps the current state.
    #[serde(
        default,
        deserialize_with = "super::explicit_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub review_label_enabled: Option<Option<bool>>,
}

#[derive(Debug, Deserialize)]
pub struct ListReviewProjectsResponse {
    pub review_projects: Vec<ReviewProject>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetReviewProjectResponse {
    pub review_project: ReviewProject,
}

#[derive(Debug, Deserialize)]
pub struct CreateReviewProjectResponse {
    pub review_project: ReviewProject,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReviewProjectResponse {
    pub review_project: ReviewProject,
}

impl Tabular for ReviewProject {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "NAME", "TYPE", "ASSIGNEES", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            truncate(&self.display_name, 30),
            self.project_type.to_string(),
            truncate(&self.assignees.join(", "), 40),
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
