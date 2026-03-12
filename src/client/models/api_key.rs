use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::output::Tabular;

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub key_type: ApiKeyType,
    pub environment: ApiKeyEnvironment,
    pub status: ApiKeyStatus,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(rename = "api_key", default)]
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum ApiKeyType {
    #[serde(rename = "SERVICE")]
    #[value(name = "service")]
    Service,
    #[serde(rename = "USER")]
    #[value(name = "user")]
    User,
}

impl std::fmt::Display for ApiKeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Service => write!(f, "SERVICE"),
            Self::User => write!(f, "USER"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum ApiKeyEnvironment {
    #[serde(rename = "PRODUCTION")]
    #[value(name = "production")]
    Production,
    #[serde(rename = "STAGING")]
    #[value(name = "staging")]
    Staging,
    #[serde(rename = "DEVELOPMENT")]
    #[value(name = "development")]
    Development,
}

impl std::fmt::Display for ApiKeyEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Production => write!(f, "PRODUCTION"),
            Self::Staging => write!(f, "STAGING"),
            Self::Development => write!(f, "DEVELOPMENT"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum ApiKeyStatus {
    #[serde(rename = "ACTIVE")]
    #[value(name = "active")]
    Active,
    #[serde(rename = "REVOKED")]
    #[value(name = "revoked")]
    Revoked,
    #[serde(rename = "SUSPENDED")]
    #[value(name = "suspended")]
    Suspended,
    #[serde(rename = "EXPIRED")]
    #[value(name = "expired")]
    Expired,
}

impl std::fmt::Display for ApiKeyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "ACTIVE"),
            Self::Revoked => write!(f, "REVOKED"),
            Self::Suspended => write!(f, "SUSPENDED"),
            Self::Expired => write!(f, "EXPIRED"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateApiKeyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub key_type: ApiKeyType,
    pub environment: ApiKeyEnvironment,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct UpdateApiKeyRequest {
    pub status: ApiKeyStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListApiKeysResponse {
    pub api_keys: Vec<ApiKey>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyResponse {
    pub api_key: ApiKey,
}

#[derive(Debug, Deserialize)]
pub struct UpdateApiKeyResponse {
    pub api_key: ApiKey,
}

impl Tabular for ApiKey {
    fn headers() -> Vec<&'static str> {
        vec![
            "ID",
            "NAME",
            "TYPE",
            "ENV",
            "STATUS",
            "PERMISSIONS",
            "LAST USED",
        ]
    }

    fn row(&self) -> Vec<String> {
        let perms = if self.permissions.is_empty() {
            "Full Access".to_string()
        } else {
            format!("{} scopes", self.permissions.len())
        };
        let last_used = self
            .last_used_at
            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "Never".to_string());
        vec![
            self.id.clone(),
            truncate(&self.name, 25),
            self.key_type.to_string(),
            self.environment.to_string(),
            self.status.to_string(),
            perms,
            last_used,
        ]
    }
}
