use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: String,
    #[serde(rename = "type")]
    pub webhook_type: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub create_time: DateTime<Utc>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CreateWebhookRequest {
    #[serde(rename = "type")]
    pub webhook_type: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateWebhookRequest {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub webhook_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListWebhooksResponse {
    pub webhooks: Vec<Webhook>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetWebhookResponse {
    pub webhook: Webhook,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhookResponse {
    pub webhook: Webhook,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWebhookResponse {
    pub webhook: Webhook,
}

impl Tabular for Webhook {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "TYPE", "URL", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.webhook_type.clone(),
            truncate(&self.url, 40),
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
