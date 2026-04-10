use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{CreateBaselineRequest, ListParams};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum BaselineCommands {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Delete(DeleteArgs),
    Archive(ArchiveArgs),
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(long)]
    filter: Option<String>,
    #[arg(long, default_value = "50")]
    page_size: u32,
    #[arg(long)]
    order_by: Option<String>,
}

#[derive(Args)]
pub struct GetArgs {
    baseline_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    /// Metric ID (required)
    #[arg(long)]
    metric_id: String,
    /// Agent ID
    #[arg(long)]
    agent_id: Option<String>,
    /// Test set ID
    #[arg(long)]
    test_set_id: Option<String>,
    /// Display name
    #[arg(long)]
    name: Option<String>,
    /// Sigma threshold (default: 2.0)
    #[arg(long, default_value = "2.0")]
    sigma_threshold: f64,
    /// Direction (default: BOTH)
    #[arg(long, default_value = "BOTH")]
    direction: String,
}

#[derive(Args)]
pub struct DeleteArgs {
    baseline_id: String,
}

#[derive(Args)]
pub struct ArchiveArgs {
    baseline_id: String,
}

pub async fn execute(
    cmd: BaselineCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        BaselineCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.baselines().list(params).await?;
            print_list(&response.baselines, format);
        }
        BaselineCommands::Get(args) => {
            let baseline = client.baselines().get(&args.baseline_id).await?;
            print_one(&baseline, format);
        }
        BaselineCommands::Create(args) => {
            let req = CreateBaselineRequest {
                metric_id: args.metric_id,
                agent_id: args.agent_id,
                test_set_id: args.test_set_id,
                display_name: args.name,
                sigma_threshold: Some(args.sigma_threshold),
                direction: Some(args.direction),
            };
            let baseline = client.baselines().create(req).await?;
            print_one(&baseline, format);
        }
        BaselineCommands::Delete(args) => {
            client.baselines().delete(&args.baseline_id).await?;
            print_success("Baseline deleted.");
        }
        BaselineCommands::Archive(args) => {
            let baseline = client.baselines().archive(&args.baseline_id).await?;
            print_one(&baseline, format);
        }
    }
    Ok(())
}
