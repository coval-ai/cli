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

#[derive(Debug, Serialize)]
pub struct CreateReviewProjectRequest {
    pub display_name: String,
    pub assignees: Vec<String>,
    pub linked_simulation_ids: Vec<String>,
    pub linked_metric_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_type: Option<ProjectType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifications: Option<bool>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateReviewProjectRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_simulation_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_metric_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifications: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opted_out_assignees: Option<Vec<String>>,
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
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
