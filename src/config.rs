use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub api_key: Option<String>,
    pub api_url: Option<String>,
}

impl Config {
    pub fn path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("coval")
            .join("config.toml")
    }

    fn legacy_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("coval").join("config.toml"))
    }

    fn migrate_legacy() {
        let new_path = Self::path();
        if new_path.exists() {
            return;
        }
        if let Some(old_path) = Self::legacy_path() {
            if old_path.exists() {
                if let Some(parent) = new_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::copy(&old_path, &new_path);
                let _ = fs::remove_file(&old_path);
            }
        }
    }

    pub fn load() -> Result<Self> {
        Self::migrate_legacy();
        let path = Self::path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }
}
