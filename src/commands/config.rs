use anyhow::Result;
use clap::Subcommand;

use crate::config::Config;

#[derive(Subcommand)]
pub enum ConfigCommands {
    Path,
    Get { key: String },
    Set { key: String, value: String },
}

pub fn execute(cmd: ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Path => {
            println!("{}", Config::path().display());
        }
        ConfigCommands::Get { key } => {
            let config = Config::load()?;
            match key.as_str() {
                "api_key" => {
                    if let Some(key) = config.api_key {
                        let masked = if key.len() > 8 {
                            format!("{}...{}", &key[..4], &key[key.len() - 4..])
                        } else {
                            "****".to_string()
                        };
                        println!("{masked}");
                    }
                }
                "api_url" => {
                    if let Some(url) = config.api_url {
                        println!("{url}");
                    }
                }
                _ => anyhow::bail!("Unknown config key: {key}"),
            }
        }
        ConfigCommands::Set { key, value } => {
            let mut config = Config::load().unwrap_or_default();
            match key.as_str() {
                "api_key" => config.api_key = Some(value),
                "api_url" => config.api_url = Some(value),
                _ => anyhow::bail!("Unknown config key: {key}"),
            }
            config.save()?;
            println!("Config updated.");
        }
    }
    Ok(())
}
