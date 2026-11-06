use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub workspace_type: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_updated_at: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Tabular for Workspace {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Name", "Status", "Type", "Created At"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.display_name.clone(),
            self.status.as_deref().unwrap_or("").to_string(),
            self.workspace_type.as_deref().unwrap_or("").to_string(),
            self.created_at.as_deref().unwrap_or("").to_string(),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListWorkspacesResponse {
    pub workspaces: Vec<Workspace>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceResponse {
    pub workspace: Workspace,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub display_name: String,
}
