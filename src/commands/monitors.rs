use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::ListParams;
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum MonitorCommands {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    Events(EventsArgs),
    #[command(name = "test-evaluate")]
    TestEvaluate(TestEvaluateArgs),
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
    monitor_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    /// JSON configuration for the monitor
    #[arg(long)]
    config: String,
}

#[derive(Args)]
pub struct UpdateArgs {
    monitor_id: String,
    /// JSON configuration for the monitor update
    #[arg(long)]
    config: String,
}

#[derive(Args)]
pub struct DeleteArgs {
    monitor_id: String,
}

#[derive(Args)]
pub struct EventsArgs {
    monitor_id: String,
}

#[derive(Args)]
pub struct TestEvaluateArgs {
    monitor_id: String,
    /// Run ID to test evaluate against
    #[arg(long)]
    run_id: String,
}

pub async fn execute(
    cmd: MonitorCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        MonitorCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.monitors().list(params).await?;
            print_list(&response.monitors, format);
        }
        MonitorCommands::Get(args) => {
            let monitor = client.monitors().get(&args.monitor_id).await?;
            print_one(&monitor, format);
        }
        MonitorCommands::Create(args) => {
            let config: serde_json::Value = serde_json::from_str(&args.config)
                .map_err(|e| anyhow::anyhow!("Invalid JSON for --config: {e}"))?;
            let monitor = client.monitors().create(&config).await?;
            print_one(&monitor, format);
        }
        MonitorCommands::Update(args) => {
            let config: serde_json::Value = serde_json::from_str(&args.config)
                .map_err(|e| anyhow::anyhow!("Invalid JSON for --config: {e}"))?;
            let monitor = client.monitors().update(&args.monitor_id, &config).await?;
            print_one(&monitor, format);
        }
        MonitorCommands::Delete(args) => {
            client.monitors().delete(&args.monitor_id).await?;
            print_success("Monitor deleted.");
        }
        MonitorCommands::Events(args) => {
            let response = client.monitors().events(&args.monitor_id).await?;
            print_list(&response.events, format);
        }
        MonitorCommands::TestEvaluate(args) => {
            let response = client
                .monitors()
                .test_evaluate(&args.monitor_id, &args.run_id)
                .await?;
            print_one(&response, format);
        }
    }
    Ok(())
}
