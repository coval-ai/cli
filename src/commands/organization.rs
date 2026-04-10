use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::UpdateOrganizationRequest;
use crate::client::CovalClient;
use crate::output::{print_one, OutputFormat};

#[derive(Subcommand)]
pub enum OrganizationCommands {
    Get,
    Update(UpdateArgs),
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Organization display name
    #[arg(long)]
    name: Option<String>,
    /// JSON string for metadata
    #[arg(long)]
    metadata: Option<String>,
    /// Comma-separated list of enabled metric IDs
    #[arg(long, value_delimiter = ',')]
    enabled_metrics: Option<Vec<String>>,
}

pub async fn execute(
    cmd: OrganizationCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        OrganizationCommands::Get => {
            let org = client.organization().get().await?;
            print_one(&org, format);
        }
        OrganizationCommands::Update(args) => {
            let metadata = args
                .metadata
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(|e| anyhow::anyhow!("Invalid JSON for --metadata: {e}"))?;

            let req = UpdateOrganizationRequest {
                display_name: args.name,
                metadata,
                enabled_metrics: args.enabled_metrics,
            };
            let org = client.organization().update(req).await?;
            print_one(&org, format);
        }
    }
    Ok(())
}
