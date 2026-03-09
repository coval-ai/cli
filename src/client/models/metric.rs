use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub id: String,
    pub metric_name: String,
    pub description: String,
    pub metric_type: MetricType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_field_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_field_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex_pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_insensitive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_pause_duration_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_condition: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum MetricType {
    #[serde(rename = "METRIC_LLM_BINARY")]
    #[value(name = "llm-binary")]
    LlmBinary,
    #[serde(rename = "METRIC_CATEGORICAL")]
    #[value(name = "categorical")]
    Categorical,
    #[serde(rename = "METRIC_NUMERICAL_LLM_JUDGE")]
    #[value(name = "numerical")]
    Numerical,
    #[serde(rename = "METRIC_AUDIO_LLM_BINARY")]
    #[value(name = "audio-binary")]
    AudioBinary,
    #[serde(rename = "METRIC_AUDIO_LLM_CATEGORICAL")]
    #[value(name = "audio-categorical")]
    AudioCategorical,
    #[serde(rename = "METRIC_AUDIO_LLM_NUMERICAL")]
    #[value(name = "audio-numerical")]
    AudioNumerical,
    #[serde(rename = "METRIC_TOOLCALL")]
    #[value(name = "toolcall")]
    Toolcall,
    #[serde(rename = "METRIC_METADATA_FIELD")]
    #[value(name = "metadata")]
    Metadata,
    #[serde(rename = "METRIC_TRANSCRIPT_REGEX")]
    #[value(name = "regex")]
    Regex,
    #[serde(rename = "METRIC_PAUSE_ANALYSIS")]
    #[value(name = "pause")]
    Pause,
    #[serde(rename = "METRIC_COMPOSITE_EVALUATION")]
    #[value(name = "composite")]
    CompositeEvaluation,
}

impl std::fmt::Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LlmBinary => write!(f, "LLM_BINARY"),
            Self::Categorical => write!(f, "CATEGORICAL"),
            Self::Numerical => write!(f, "NUMERICAL"),
            Self::AudioBinary => write!(f, "AUDIO_BINARY"),
            Self::AudioCategorical => write!(f, "AUDIO_CATEGORICAL"),
            Self::AudioNumerical => write!(f, "AUDIO_NUMERICAL"),
            Self::Toolcall => write!(f, "TOOLCALL"),
            Self::Metadata => write!(f, "METADATA"),
            Self::Regex => write!(f, "REGEX"),
            Self::Pause => write!(f, "PAUSE"),
            Self::CompositeEvaluation => write!(f, "COMPOSITE"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateMetricRequest {
    pub metric_name: String,
    pub description: String,
    pub metric_type: MetricType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_field_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_field_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex_pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_insensitive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_pause_duration_seconds: Option<f64>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateMetricRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_type: Option<MetricType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ListMetricsResponse {
    pub metrics: Vec<Metric>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetMetricResponse {
    pub metric: Metric,
}

#[derive(Debug, Deserialize)]
pub struct CreateMetricResponse {
    pub metric: Metric,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMetricResponse {
    pub metric: Metric,
}

impl Tabular for Metric {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "NAME", "TYPE", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            truncate(&self.metric_name, 25),
            self.metric_type.to_string(),
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
