use anyhow::Result;
use clap::Subcommand;
use serde::Serialize;

use crate::agent_discovery;
use crate::client::models::ListParams;
use crate::client::{CovalClient, DEFAULT_BASE_URL};
use crate::config::Config;
use crate::output::{emit_one, emit_one_with_warnings, AgentWarning, OutputContext};

#[derive(Subcommand)]
pub enum AgentCommands {
    Doctor,
    Manifest,
}

impl AgentCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Doctor => "doctor",
            Self::Manifest => "manifest",
        }
    }
}

#[derive(Debug, Serialize)]
struct DoctorReport {
    version: &'static str,
    agent_mode: bool,
    config: ConfigReport,
    auth: AuthReport,
    api: ApiReport,
    skills: SkillsReport,
}

#[derive(Debug, Serialize)]
struct ConfigReport {
    path: String,
    exists: bool,
}

#[derive(Debug, Serialize)]
struct AuthReport {
    authenticated: bool,
    source: &'static str,
}

#[derive(Debug, Serialize)]
struct ApiReport {
    url: String,
    source: &'static str,
    connectivity: ConnectivityReport,
}

#[derive(Debug, Serialize)]
struct ConnectivityReport {
    checked: bool,
    ok: Option<bool>,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct SkillsReport {
    implemented: bool,
    source: Option<&'static str>,
}

pub async fn execute(
    command: AgentCommands,
    api_key: Option<&str>,
    api_key_source: &'static str,
    api_url: Option<&str>,
    api_url_source: &'static str,
    ctx: &OutputContext,
) -> Result<()> {
    match command {
        AgentCommands::Doctor => {
            doctor(api_key, api_key_source, api_url, api_url_source, ctx).await
        }
        AgentCommands::Manifest => {
            let manifest = agent_discovery::manifest();
            emit_one(ctx, "agent", "manifest", &manifest);
            Ok(())
        }
    }
}

pub fn resource_context(resource: &'static str, ctx: &OutputContext) -> Result<()> {
    let context = agent_discovery::resource_context(resource)
        .ok_or_else(|| anyhow::anyhow!("Unknown resource context: {resource}"))?;
    emit_one(ctx, resource, "context", &context);
    Ok(())
}

async fn doctor(
    api_key: Option<&str>,
    api_key_source: &'static str,
    api_url: Option<&str>,
    api_url_source: &'static str,
    ctx: &OutputContext,
) -> Result<()> {
    let config_path = Config::path();
    let mut warnings = Vec::new();
    let connectivity = if let Some(key) = api_key {
        let client = CovalClient::new(key.to_string(), api_url);
        match client
            .agents()
            .list(ListParams {
                page_size: Some(1),
                ..Default::default()
            })
            .await
        {
            Ok(_) => ConnectivityReport {
                checked: true,
                ok: Some(true),
                message: None,
            },
            Err(err) => {
                warnings.push(AgentWarning::new(
                    "api_connectivity_failed",
                    "API connectivity check failed.",
                ));
                ConnectivityReport {
                    checked: true,
                    ok: Some(false),
                    message: Some(err.to_string()),
                }
            }
        }
    } else {
        warnings.push(AgentWarning::new(
            "not_authenticated",
            "No API key was found; API connectivity was not checked.",
        ));
        ConnectivityReport {
            checked: false,
            ok: None,
            message: None,
        }
    };

    let report = DoctorReport {
        version: env!("CARGO_PKG_VERSION"),
        agent_mode: ctx.agent,
        config: ConfigReport {
            path: config_path.display().to_string(),
            exists: config_path.exists(),
        },
        auth: AuthReport {
            authenticated: api_key.is_some(),
            source: api_key_source,
        },
        api: ApiReport {
            url: api_url.unwrap_or(DEFAULT_BASE_URL).to_string(),
            source: api_url_source,
            connectivity,
        },
        skills: SkillsReport {
            implemented: false,
            source: None,
        },
    };

    emit_one_with_warnings(ctx, "agent", "doctor", &report, warnings);
    Ok(())
}
