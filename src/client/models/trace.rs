use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::output::Tabular;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TraceSearchRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "TraceSearchFilters::is_empty")]
    pub filters: TraceSearchFilters,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TraceSearchFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_filters: Option<Vec<TraceSearchAttributeFilter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms_min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms_max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_set_id: Option<String>,
}

impl TraceSearchFilters {
    fn is_empty(&self) -> bool {
        self.start_date.is_none()
            && self.end_date.is_none()
            && self.span_name.is_none()
            && self.provider.is_none()
            && self.status.is_none()
            && self.attribute_filters.is_none()
            && self.duration_ms_min.is_none()
            && self.duration_ms_max.is_none()
            && self.sort_by.is_none()
            && self.agent_id.is_none()
            && self.test_set_id.is_none()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceSearchAttributeFilter {
    pub key: String,
    #[serde(default = "default_attribute_operator")]
    pub operator: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

fn default_attribute_operator() -> String {
    "contains".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceSearchResponse {
    pub items: Vec<TraceSearchCallResult>,
    pub total_count: u64,
    #[serde(default)]
    pub next_cursor: Option<String>,
    pub aggregate_stats: TraceSearchAggregateStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceSearchAggregateStats {
    pub error_count: u64,
    pub error_rate: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceSearchCallResult {
    pub simulation_output_id: String,
    pub run_id: String,
    pub latest_matched_timestamp_ms: i64,
    pub first_matched_timestamp_ms: i64,
    pub matched_span_count: u64,
    pub total_span_count: u64,
    pub error_span_count: u64,
    pub ok_span_count: u64,
    pub unset_span_count: u64,
    pub overall_status: String,
    #[serde(default)]
    pub matched_span_names: Vec<String>,
    #[serde(default)]
    pub matched_provider_names: Vec<String>,
    #[serde(default)]
    pub matched_service_names: Vec<String>,
    #[serde(default)]
    pub matched_scope_names: Vec<String>,
}

impl Tabular for TraceSearchCallResult {
    fn headers() -> Vec<&'static str> {
        vec![
            "Simulation Output",
            "Run",
            "Status",
            "Matched / Total",
            "Errors",
            "Spans",
            "Providers",
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.simulation_output_id.clone(),
            self.run_id.clone(),
            self.overall_status.clone(),
            format!("{} / {}", self.matched_span_count, self.total_span_count),
            self.error_span_count.to_string(),
            self.matched_span_names.join(", "),
            self.matched_provider_names.join(", "),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceSummaryResponse {
    pub target: TraceSummaryTarget,
    pub trace_summary: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceSummaryTarget {
    #[serde(rename = "type")]
    pub target_type: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceSpansResponse {
    pub traces: Vec<Value>,
    pub total_spans: u64,
}
