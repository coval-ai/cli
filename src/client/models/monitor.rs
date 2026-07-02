use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    #[serde(alias = "ulid")]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub status: String,
    pub evaluation_type: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub match_mode: Option<String>,
    #[serde(default)]
    pub cooldown_seconds: Option<i64>,
    #[serde(default)]
    pub custom_message_template: Option<String>,
    #[serde(default)]
    pub agent_ids: Option<Vec<String>>,
    #[serde(default)]
    pub required_tags: Option<Vec<String>>,
    #[serde(default)]
    pub scheduled_run_ids: Option<Vec<String>>,
    #[serde(default)]
    pub trigger_count: Option<i64>,
    #[serde(default)]
    pub last_triggered_at: Option<String>,
    #[serde(default)]
    pub conditions: Option<Vec<serde_json::Value>>,
    #[serde(default, deserialize_with = "deserialize_redacted_json_values")]
    pub channels: Option<Vec<Value>>,
    pub create_time: String,
    pub update_time: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Tabular for Monitor {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Name", "Status", "Scope", "Evaluation", "Triggers"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.name.clone(),
            self.status.clone(),
            self.scope.as_deref().unwrap_or("ALL").to_string(),
            self.evaluation_type.clone(),
            self.trigger_count.unwrap_or(0).to_string(),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListMonitorsResponse {
    pub monitors: Vec<Monitor>,
    #[serde(default)]
    pub next_page_token: Option<String>,
    #[serde(default)]
    pub total_count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetMonitorResponse {
    pub monitor: Monitor,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMonitorRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub evaluation_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cooldown_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_message_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_run_ids: Option<Vec<String>>,
    pub conditions: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMonitorRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluation_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cooldown_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_message_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_run_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestEvaluateRequest {
    pub run_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestEvaluateResponse {
    pub monitor_id: String,
    pub run_id: String,
    pub triggered: bool,
    #[serde(default)]
    pub suppressed: bool,
    #[serde(default)]
    pub condition_results: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub dispatch_results: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorEvent {
    #[serde(alias = "ulid")]
    pub id: String,
    #[serde(alias = "monitor_ulid")]
    pub monitor_id: String,
    #[serde(default)]
    pub run_id: Option<String>,
    pub outcome: String,
    #[serde(default)]
    pub condition_results: Option<Vec<serde_json::Value>>,
    #[serde(default, deserialize_with = "deserialize_redacted_json_values")]
    pub dispatched_channels: Option<Vec<Value>>,
    #[serde(default)]
    pub message_sent: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Tabular for MonitorEvent {
    fn headers() -> Vec<&'static str> {
        vec!["Event ID", "Monitor ID", "Run ID", "Outcome"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.monitor_id.clone(),
            self.run_id.as_deref().unwrap_or("").to_string(),
            self.outcome.clone(),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListMonitorEventsResponse {
    pub events: Vec<MonitorEvent>,
    #[serde(default)]
    pub next_page_token: Option<String>,
    #[serde(default)]
    pub total_count: Option<i64>,
}

fn deserialize_redacted_json_values<'de, D>(deserializer: D) -> Result<Option<Vec<Value>>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut values = Option::<Vec<Value>>::deserialize(deserializer)?;
    if let Some(items) = values.as_mut() {
        for item in items {
            redact_auth_tokens(item);
        }
    }
    Ok(values)
}

fn redact_auth_tokens(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if map.contains_key("auth_token") {
                map.insert(
                    "auth_token".to_string(),
                    Value::String("<redacted>".to_string()),
                );
            }
            for child in map.values_mut() {
                redact_auth_tokens(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_auth_tokens(item);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Monitor, MonitorEvent};

    #[test]
    fn monitor_channels_redact_auth_tokens() {
        let monitor: Monitor = serde_json::from_value(json!({
            "ulid": "mon_123",
            "name": "Webhook monitor",
            "status": "ACTIVE",
            "evaluation_type": "RUN",
            "channels": [{
                "type": "WEBHOOK",
                "config": {
                    "url": "https://example.com/webhook",
                    "auth_token": "secret-token"
                }
            }],
            "create_time": "2026-01-01T00:00:00Z",
            "update_time": "2026-01-01T00:00:00Z"
        }))
        .unwrap();

        let rendered = serde_json::to_string(&monitor).unwrap();
        assert!(!rendered.contains("secret-token"));
        assert!(rendered.contains("<redacted>"));
    }

    #[test]
    fn monitor_events_redact_dispatched_channel_auth_tokens() {
        let event: MonitorEvent = serde_json::from_value(json!({
            "ulid": "evt_123",
            "monitor_ulid": "mon_123",
            "outcome": "TRIGGERED",
            "dispatched_channels": [{
                "config": {
                    "auth_token": "event-secret"
                }
            }]
        }))
        .unwrap();

        let rendered = serde_json::to_string(&event).unwrap();
        assert!(!rendered.contains("event-secret"));
        assert!(rendered.contains("<redacted>"));
    }
}
