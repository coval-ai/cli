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

fn extract_widget_id(name: &str) -> String {
    name.rsplit('/').next().unwrap_or(name).to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub name: String,
    pub display_name: String,
    pub widget_type: WidgetType,
    #[serde(default)]
    pub config: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_height: Option<u32>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum WidgetType {
    #[serde(rename = "CHART")]
    #[value(name = "chart")]
    Chart,
    #[serde(rename = "TABLE")]
    #[value(name = "table")]
    Table,
    #[serde(rename = "TEXT")]
    #[value(name = "text")]
    Text,
}

impl std::fmt::Display for WidgetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chart => write!(f, "CHART"),
            Self::Table => write!(f, "TABLE"),
            Self::Text => write!(f, "TEXT"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateWidgetRequest {
    pub display_name: String,
    pub widget_type: WidgetType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_height: Option<u32>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateWidgetRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widget_type: Option<WidgetType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_height: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ListWidgetsResponse {
    pub widgets: Vec<Widget>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetWidgetResponse {
    pub widget: Widget,
}

#[derive(Debug, Deserialize)]
pub struct CreateWidgetResponse {
    pub widget: Widget,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWidgetResponse {
    pub widget: Widget,
}

impl Tabular for Widget {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "NAME", "TYPE", "GRID", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        let grid = match (self.grid_width, self.grid_height) {
            (Some(w), Some(h)) => format!("{w}x{h}"),
            _ => String::new(),
        };
        vec![
            extract_widget_id(&self.name),
            truncate(&self.display_name, 25),
            self.widget_type.to_string(),
            grid,
            self.create_time.format("%Y-%m-%d %H:%M").to_string(),
        ]
    }
}
