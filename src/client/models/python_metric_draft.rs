use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum PythonMetricOutputType {
    Number,
    Boolean,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PythonMetricInputRequirements {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traces: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_tables: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CreatePythonMetricDraftRequest {
    pub display_name: String,
    pub source: String,
    pub output_type: PythonMetricOutputType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_metric_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_version_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements: Option<PythonMetricInputRequirements>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PythonMetricDraft {
    pub draft_id: String,
    pub version_id: String,
    pub display_name: String,
    pub description: String,
    pub source: String,
    pub output_type: PythonMetricOutputType,
    pub unit: Option<String>,
    pub requirements: PythonMetricInputRequirements,
    pub fingerprint: String,
    pub target_metric_id: Option<String>,
    pub target_version_id: Option<String>,
    pub published: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PythonMetricDraftResponse {
    pub draft: PythonMetricDraft,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PreviewPythonMetricRequest {
    pub simulation_output_ids: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PublishPythonMetricRequest {
    pub preview_ids: Vec<String>,
    pub fingerprint: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PythonMetricPreview {
    pub preview_id: String,
    pub simulation_output_id: String,
    pub status: String,
    #[serde(default)]
    pub value: Option<serde_json::Value>,
    #[serde(default)]
    pub explanation: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PythonMetricPreviewsResponse {
    pub previews: Vec<PythonMetricPreview>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PublishPythonMetricResponse {
    pub metric_id: String,
    pub version_id: String,
}
