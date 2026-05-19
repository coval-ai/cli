use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{CreateScheduledRunRequest, ListParams, UpdateScheduledRunRequest};
use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, OutputContext,
};

#[derive(Subcommand)]
pub enum ScheduledRunCommands {
    Context,
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

impl ScheduledRunCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::List(_) => "list",
            Self::Get(_) => "get",
            Self::Create(_) => "create",
            Self::Update(_) => "update",
            Self::Delete(_) => "delete",
        }
    }
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
    #[command(flatten)]
    input_json: InputJsonArg,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    template_id: Option<String>,
    #[arg(long)]
    schedule: Option<String>,
    #[arg(long)]
    timezone: Option<String>,
    #[arg(long)]
    enabled: Option<bool>,
}

#[derive(Args)]
pub struct UpdateArgs {
    scheduled_run_id: String,
    #[command(flatten)]
    input_json: InputJsonArg,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    template_id: Option<String>,
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
    ctx: &OutputContext,
) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        ScheduledRunCommands::Context => {
            return crate::commands::agent::resource_context("scheduled-runs", ctx);
        }
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
            emit_list_with_actions(
                ctx,
                "scheduled-runs",
                operation,
                &response.scheduled_runs,
                next_actions::list_result(
                    "scheduled-runs",
                    response.scheduled_runs.first().map(|run| run.id.as_str()),
                ),
            );
        }
        ScheduledRunCommands::Get(args) => {
            let run = client.scheduled_runs().get(&args.scheduled_run_id).await?;
            emit_one_with_actions(
                ctx,
                "scheduled-runs",
                operation,
                &run,
                next_actions::item_result("scheduled-runs", &run.id),
            );
        }
        ScheduledRunCommands::Create(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "display_name", args.name)?;
            input_json::insert(&mut input, "run_template_id", args.template_id)?;
            input_json::insert(&mut input, "schedule_expression", args.schedule)?;
            input_json::insert(&mut input, "schedule_timezone", args.timezone)?;
            input_json::insert(&mut input, "enabled", args.enabled)?;
            let req: CreateScheduledRunRequest = input_json::finish(input)?;
            let run = client.scheduled_runs().create(req).await?;
            emit_one_with_actions(
                ctx,
                "scheduled-runs",
                operation,
                &run,
                next_actions::item_result("scheduled-runs", &run.id),
            );
        }
        ScheduledRunCommands::Update(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "display_name", args.name)?;
            input_json::insert(&mut input, "run_template_id", args.template_id)?;
            input_json::insert(&mut input, "schedule_expression", args.schedule)?;
            input_json::insert(&mut input, "schedule_timezone", args.timezone)?;
            input_json::insert(&mut input, "enabled", args.enabled)?;
            let req: UpdateScheduledRunRequest = input_json::finish(input)?;
            let run = client
                .scheduled_runs()
                .update(&args.scheduled_run_id, req)
                .await?;
            emit_one_with_actions(
                ctx,
                "scheduled-runs",
                operation,
                &run,
                next_actions::item_result("scheduled-runs", &run.id),
            );
        }
        ScheduledRunCommands::Delete(args) => {
            client
                .scheduled_runs()
                .delete(&args.scheduled_run_id)
                .await?;
            emit_success_with_actions(
                ctx,
                "scheduled-runs",
                operation,
                "Scheduled run deleted.",
                next_actions::delete_result("scheduled-runs"),
            );
        }
    }
    Ok(())
}
