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
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
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
    #[serde(other)]
    #[value(skip)]
    Unknown,
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
            Self::Unknown => write!(f, "BUILT_IN"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_condition: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criteria_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criteria_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criteria: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reporting_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_prompt_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_traces: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_silence_duration_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_silence_gap_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_sentiments: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent_above: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_end_reasons: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observation_name: Option<String>,
    /// Either a string or a JSON object, so it stays untyped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_body: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_volume_change_for_pitch_misalignment: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sql_query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_config: Option<serde_json::Value>,
    /// Tag names. Omitted leaves tags unchanged; an empty list clears them. The API
    /// treats null the same as omitted here, so `Option` carries the whole contract.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
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
    pub criteria_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criteria_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criteria: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reporting_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_prompt_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_traces: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_silence_duration_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_silence_gap_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_sentiments: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent_above: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_end_reasons: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observation_name: Option<String>,
    /// Either a string or a JSON object, so it stays untyped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_body: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_volume_change_for_pitch_misalignment: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sql_query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_config: Option<serde_json::Value>,
    /// Tag names. Omitted leaves tags unchanged; an empty list clears them. The API
    /// treats null the same as omitted here, so `Option` carries the whole contract.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
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

/// Exactly one of `simulation_output_id` or `simulation_output_ids` must be set;
/// the API rejects a request that sets both or neither.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TestMetricRequest {
    /// Deprecated by the API in favor of `simulation_output_ids`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simulation_output_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simulation_output_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TestMetricResponse {
    #[serde(default)]
    pub results: Vec<TestMetricItemResult>,
    /// Deprecated by the API and null for a batch request, which reports per
    /// simulation output in `results` instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric_output_ulid: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestMetricItemResult {
    pub simulation_output_id: String,
    pub status: String,
    #[serde(default)]
    pub metric_output_ulid: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
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
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let end: String = s.chars().take(max - 3).collect();
        format!("{}...", end)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricVersion {
    pub ulid: String,
    pub version_number: i64,
    pub change_type: String,
    #[serde(default)]
    pub metric_type: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub create_time: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Tabular for MetricVersion {
    fn headers() -> Vec<&'static str> {
        vec!["Version ID", "#", "Change Type", "Label"]
    }
    fn row(&self) -> Vec<String> {
        vec![
            self.ulid.clone(),
            self.version_number.to_string(),
            self.change_type.clone(),
            self.label.as_deref().unwrap_or("").to_string(),
        ]
    }
}

#[derive(Debug, Deserialize)]
pub struct ListMetricVersionsResponse {
    pub versions: Vec<MetricVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub ulid: String,
    #[serde(default)]
    pub metric_id: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub sigma_threshold: Option<f64>,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default)]
    pub observation_count: Option<i64>,
    #[serde(default)]
    pub baseline_float: Option<f64>,
    #[serde(default)]
    pub baseline_sigma: Option<f64>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_updated_at: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Tabular for Baseline {
    fn headers() -> Vec<&'static str> {
        vec![
            "ID",
            "Display Name",
            "Status",
            "Direction",
            "Obs",
            "Baseline",
        ]
    }
    fn row(&self) -> Vec<String> {
        vec![
            self.ulid.clone(),
            self.display_name.as_deref().unwrap_or("").to_string(),
            self.status.as_deref().unwrap_or("").to_string(),
            self.direction.as_deref().unwrap_or("").to_string(),
            self.observation_count.unwrap_or(0).to_string(),
            self.baseline_float
                .map(|v| format!("{:.4}", v))
                .unwrap_or_default(),
        ]
    }
}

#[derive(Debug, Deserialize)]
pub struct ListBaselinesResponse {
    pub baselines: Vec<Baseline>,
    #[serde(default)]
    pub next_page_token: Option<String>,
    #[serde(default)]
    pub total_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threshold {
    pub name: String,
    #[serde(default)]
    pub comparison_operator: Option<String>,
    #[serde(default)]
    pub target_float_upper: Option<f64>,
    #[serde(default)]
    pub target_float_lower: Option<f64>,
    #[serde(default)]
    pub target_values: Option<Vec<String>>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub create_time: Option<String>,
    #[serde(default)]
    pub update_time: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Tabular for Threshold {
    fn headers() -> Vec<&'static str> {
        vec!["Name", "Operator", "Upper", "Lower", "Source"]
    }
    fn row(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.comparison_operator
                .as_deref()
                .unwrap_or("")
                .to_string(),
            self.target_float_upper
                .map(|v| v.to_string())
                .unwrap_or_default(),
            self.target_float_lower
                .map(|v| v.to_string())
                .unwrap_or_default(),
            self.source.as_deref().unwrap_or("").to_string(),
        ]
    }
}

#[derive(Debug, Deserialize)]
pub struct ListThresholdsResponse {
    pub thresholds: Vec<Threshold>,
}
