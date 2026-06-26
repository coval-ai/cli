use anyhow::Result;
use clap::Subcommand;

use crate::client::CovalClient;
use crate::output::{emit_list_with_actions, OutputContext};

#[derive(Subcommand)]
pub enum ModelCommands {
    #[command(name = "metric")]
    Metric,
}

impl ModelCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Metric => "metric-models",
        }
    }
}

pub async fn execute(cmd: ModelCommands, client: &CovalClient, ctx: &OutputContext) -> Result<()> {
    match cmd {
        ModelCommands::Metric => {
            let response = client.models().list_metric_models().await?;
            emit_list_with_actions(ctx, "models", "metric-models", &response.models, vec![]);
        }
    }
    Ok(())
}
