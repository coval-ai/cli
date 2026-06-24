use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::output::Tabular;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub resource_name: String,
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_sound: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_seconds: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_initiation: Option<String>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum VoiceName {
    Aria,
    Ashwin,
    Autumn,
    Brynn,
    Callum,
    Caspian,
    Corwin,
    Darrow,
    Delphine,
    Dorian,
    Elara,
    Kieran,
    Lysander,
    Marina,
    Naveen,
    Orion,
    Rowan,
    Skye,
    Soren,
    Vera,
    Alejandro,
    Erika,
    Monika,
    Mark,
    Angela,
    Raju,
    Harry,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePersonaRequest {
    pub name: String,
    pub voice_name: String,
    pub language_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_sound: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_seconds: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_initiation: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdatePersonaRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_sound: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_seconds: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_initiation: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListPersonasResponse {
    pub personas: Vec<Persona>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetPersonaResponse {
    pub persona: Persona,
}

#[derive(Debug, Deserialize)]
pub struct CreatePersonaResponse {
    pub persona: Persona,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePersonaResponse {
    pub persona: Persona,
}

impl Tabular for Persona {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "NAME", "VOICE", "LANGUAGE", "CREATED"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            truncate(&self.name, 25),
            self.voice_name.clone().unwrap_or_else(|| "-".into()),
            self.language_code.clone().unwrap_or_else(|| "-".into()),
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
pub struct PhoneNumberMapping {
    pub index: u32,
    pub phone_number: String,
}

#[derive(Debug, Deserialize)]
pub struct ListPhoneNumbersResponse {
    pub phone_numbers: Vec<PhoneNumberMapping>,
}

impl Tabular for PhoneNumberMapping {
    fn headers() -> Vec<&'static str> {
        vec!["INDEX", "PHONE NUMBER"]
    }

    fn row(&self) -> Vec<String> {
        vec![self.index.to_string(), self.phone_number.clone()]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundSoundResource {
    pub id: String,
    pub value: String,
    pub source: String,
    pub display_name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url_expires_at: Option<DateTime<Utc>>,
    pub default_volume: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated_at: Option<DateTime<Utc>>,
}

impl Tabular for BackgroundSoundResource {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "VALUE", "SOURCE", "STATUS", "DISPLAY NAME", "VOLUME"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.value.clone(),
            self.source.clone(),
            self.status.clone(),
            truncate(&self.display_name, 25),
            format!("{:.2}", self.default_volume),
        ]
    }
}

#[derive(Debug, Deserialize)]
pub struct ListBackgroundSoundsResponse {
    pub background_sounds: Vec<BackgroundSoundResource>,
}

#[derive(Debug, Serialize)]
pub struct CreateBackgroundSoundRequest {
    pub display_name: String,
    pub original_filename: String,
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_volume: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBackgroundSoundResponse {
    pub background_sound: BackgroundSoundResource,
    pub upload_url: String,
    pub upload_fields: BTreeMap<String, String>,
    pub expires_at: DateTime<Utc>,
    pub max_size_bytes: u64,
}

#[derive(Debug, Deserialize)]
pub struct CompleteBackgroundSoundResponse {
    pub background_sound: BackgroundSoundResource,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum BackgroundSoundUpdateStatus {
    Active,
    Archived,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateBackgroundSoundRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_volume: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<BackgroundSoundUpdateStatus>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBackgroundSoundResponse {
    pub background_sound: BackgroundSoundResource,
}
