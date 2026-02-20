use std::io::{self, Write};

use anyhow::Result;
use clap::Args;

use crate::client::models::ListParams;
use crate::client::CovalClient;
use crate::config::Config;

#[derive(Args)]
pub struct LoginArgs {
    #[arg(long)]
    pub api_key: Option<String>,
}

pub async fn login(args: LoginArgs) -> Result<()> {
    let api_key = if let Some(key) = args.api_key {
        key
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
            println!("Authenticated successfully.");
            println!("Credentials saved to {}", path.display());
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Authentication failed: {e}")
        }
    }
}

pub fn whoami(api_key: Option<&String>) {
    if let Some(key) = api_key {
        let masked = if key.len() > 8 {
            format!("{}...{}", &key[..4], &key[key.len() - 4..])
        } else {
            "****".to_string()
        };
        println!("Authenticated with API key: {masked}");
    } else {
        println!("Not authenticated. Run `coval login` to authenticate.");
    }
}
