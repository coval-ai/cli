use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{CreateScheduledRunRequest, ListParams, UpdateScheduledRunRequest};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum ScheduledRunCommands {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(long)]
    filter: Option<String>,
    #[arg(long, default_value = "50")]
    page_size: u32,
    #[arg(long)]
    order_by: Option<String>,
    #[arg(long)]
    enabled: Option<bool>,
    #[arg(long)]
    template_id: Option<String>,
}

#[derive(Args)]
pub struct GetArgs {
    scheduled_run_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    #[arg(long)]
    name: String,
    #[arg(long)]
    template_id: String,
    #[arg(long)]
    schedule: String,
    #[arg(long)]
    timezone: Option<String>,
    #[arg(long)]
    enabled: Option<bool>,
}

#[derive(Args)]
pub struct UpdateArgs {
    scheduled_run_id: String,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    schedule: Option<String>,
    #[arg(long)]
    timezone: Option<String>,
    #[arg(long)]
    enabled: Option<bool>,
}

#[derive(Args)]
pub struct DeleteArgs {
    scheduled_run_id: String,
}

pub async fn execute(
    cmd: ScheduledRunCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        ScheduledRunCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client
                .scheduled_runs()
                .list(params, args.enabled, args.template_id.as_deref())
                .await?;
            print_list(&response.scheduled_runs, format);
        }
        ScheduledRunCommands::Get(args) => {
            let run = client.scheduled_runs().get(&args.scheduled_run_id).await?;
            print_one(&run, format);
        }
        ScheduledRunCommands::Create(args) => {
            let req = CreateScheduledRunRequest {
                display_name: args.name,
                run_template_id: args.template_id,
                schedule_expression: args.schedule,
                schedule_timezone: args.timezone,
                enabled: args.enabled,
            };
            let run = client.scheduled_runs().create(req).await?;
            print_one(&run, format);
        }
        ScheduledRunCommands::Update(args) => {
            let req = UpdateScheduledRunRequest {
                display_name: args.name,
                schedule_expression: args.schedule,
                schedule_timezone: args.timezone,
                enabled: args.enabled,
            };
            let run = client
                .scheduled_runs()
                .update(&args.scheduled_run_id, req)
                .await?;
            print_one(&run, format);
        }
        ScheduledRunCommands::Delete(args) => {
            client
                .scheduled_runs()
                .delete(&args.scheduled_run_id)
                .await?;
            print_success("Scheduled run deleted.");
        }
    }
    Ok(())
}
