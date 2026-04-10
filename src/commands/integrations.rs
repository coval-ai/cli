use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::ConnectSlackRequest;
use crate::client::CovalClient;
use crate::output::{print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum IntegrationCommands {
    #[command(name = "connect-slack")]
    ConnectSlack(ConnectSlackArgs),
    #[command(name = "disconnect-slack")]
    DisconnectSlack,
}

#[derive(Args)]
pub struct ConnectSlackArgs {
    /// OAuth code from Slack
    #[arg(long)]
    code: String,
    /// OAuth redirect URI
    #[arg(long)]
    redirect_uri: String,
}

pub async fn execute(
    cmd: IntegrationCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        IntegrationCommands::ConnectSlack(args) => {
            let req = ConnectSlackRequest {
                code: args.code,
                redirect_uri: args.redirect_uri,
            };
            let response = client.integrations().connect_slack(req).await?;
            print_one(&response, format);
        }
        IntegrationCommands::DisconnectSlack => {
            client.integrations().disconnect_slack().await?;
            print_success("Slack disconnected.");
        }
    }
    Ok(())
}
