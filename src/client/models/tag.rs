use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub tag_name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub create_time: Option<String>,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Tabular for Tag {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Name", "Color", "Created By", "Created At"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.tag_name.clone(),
            self.color.as_deref().unwrap_or("").to_string(),
            self.created_by.as_deref().unwrap_or("").to_string(),
            self.create_time.as_deref().unwrap_or("").to_string(),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListTagsResponse {
    pub tags: Vec<Tag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTagResponse {
    pub tag: Tag,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTagRequest {
    pub tag_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTagRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}
