use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricModel {
    pub model_id: String,
    pub display_name: Option<String>,
    pub provider: Option<String>,
    pub tier: Option<String>,
    #[serde(default)]
    pub supports_thinking: bool,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Tabular for MetricModel {
    fn headers() -> Vec<&'static str> {
        vec!["Model ID", "Display Name", "Provider", "Tier"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.model_id.clone(),
            self.display_name.as_deref().unwrap_or("").to_string(),
            self.provider.as_deref().unwrap_or("").to_string(),
            self.tier.as_deref().unwrap_or("").to_string(),
        ]
    }
}

#[derive(Debug, Deserialize)]
pub struct ListMetricModelsResponse {
    pub models: Vec<MetricModel>,
}
