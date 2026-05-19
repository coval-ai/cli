use std::io::{self, Write};

use anyhow::Result;
use clap::Args;
use serde_json::json;

use crate::client::models::ListParams;
use crate::client::CovalClient;
use crate::config::{mask_api_key, Config};
use crate::output::{emit_one, OutputContext};

#[derive(Args)]
pub struct LoginArgs {
    #[arg(long)]
    pub api_key: Option<String>,
}

pub async fn login(args: LoginArgs, ctx: &OutputContext) -> Result<()> {
    let api_key = if let Some(key) = args.api_key {
        key
    } else if ctx.agent {
        anyhow::bail!("API key is required in agent mode. Pass `coval login --api-key <key>`.")
    } else {
        print!("Enter your Coval API key: ");
        io::stdout().flush()?;
        let mut key = String::new();
        io::stdin().read_line(&mut key)?;
        key.trim().to_string()
    };

    if api_key.is_empty() {
        anyhow::bail!("API key cannot be empty");
    }

    let client = CovalClient::new(api_key.clone(), None);
    let agents = client.agents().list(ListParams::default()).await;

    match agents {
        Ok(_) => {
            let mut config = Config::load().unwrap_or_default();
            config.api_key = Some(api_key);
            config.save()?;

            let path = Config::path();
            if ctx.agent {
                emit_one(
                    ctx,
                    "auth",
                    "login",
                    &json!({
                        "authenticated": true,
                        "config_path": path.display().to_string(),
                    }),
                );
            } else {
                println!("Authenticated successfully.");
                println!("Credentials saved to {}", path.display());
            }
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Authentication failed: {e}")
        }
    }
}

pub fn whoami(api_key: Option<&String>, ctx: &OutputContext) {
    if let Some(key) = api_key {
        let masked = mask_api_key(key);
        if ctx.agent {
            emit_one(
                ctx,
                "auth",
                "whoami",
                &json!({
                    "authenticated": true,
                    "api_key": masked,
                }),
            );
        } else {
            println!("Authenticated with API key: {masked}");
        }
    } else if ctx.agent {
        emit_one(
            ctx,
            "auth",
            "whoami",
            &json!({
                "authenticated": false,
                "api_key": null,
            }),
        );
    } else {
        println!("Not authenticated. Run `coval login` to authenticate.");
    }
}
