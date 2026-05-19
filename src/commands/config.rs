use anyhow::Result;
use clap::Subcommand;
use serde_json::json;

use crate::config::{mask_api_key, Config};
use crate::output::{emit_one, emit_success, OutputContext};

#[derive(Subcommand)]
pub enum ConfigCommands {
    Path,
    Get { key: String },
    Set { key: String, value: String },
}

impl ConfigCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Get { .. } => "get",
            Self::Set { .. } => "set",
        }
    }
}

pub fn execute(cmd: ConfigCommands, ctx: &OutputContext) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        ConfigCommands::Path => {
            let path = Config::path().display().to_string();
            if ctx.agent {
                emit_one(ctx, "config", operation, &json!({ "path": path }));
            } else {
                println!("{path}");
            }
        }
        ConfigCommands::Get { key } => {
            let config = Config::load()?;
            let value = match key.as_str() {
                "api_key" => config.api_key.map(|key| mask_api_key(&key)),
                "api_url" => config.api_url,
                _ => anyhow::bail!("Unknown config key: {key}"),
            };

            if ctx.agent {
                emit_one(
                    ctx,
                    "config",
                    operation,
                    &json!({
                        "key": key,
                        "value": value,
                    }),
                );
            } else if let Some(value) = value {
                println!("{value}");
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
            emit_success(ctx, "config", operation, "Config updated.");
        }
    }
    Ok(())
}
