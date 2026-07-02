use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::ListParams;
use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, OutputContext,
};

#[derive(Subcommand)]
pub enum MonitorCommands {
    Context,
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    #[command(name = "test-evaluate")]
    TestEvaluate(TestEvaluateArgs),
    Events {
        monitor_id: String,
        #[command(subcommand)]
        command: Option<EventCommands>,
    },
}

#[derive(Subcommand)]
pub enum EventCommands {
    List(ListEventArgs),
}

impl MonitorCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::List(_) => "list",
            Self::Get(_) => "get",
            Self::Create(_) => "create",
            Self::Update(_) => "update",
            Self::Delete(_) => "delete",
            Self::TestEvaluate(_) => "test-evaluate",
            Self::Events { command, .. } => match command {
                Some(EventCommands::List(_)) => "list-events",
                None => "list-events",
            },
        }
    }
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(long)]
    scope: Option<String>,
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
    #[command(flatten)]
    input_json: InputJsonArg,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long, default_value = "ON_RUN_COMPLETE")]
    evaluation_type: String,
    #[arg(long)]
    scope: Option<String>,
    #[arg(long)]
    match_mode: Option<String>,
    #[arg(long)]
    cooldown_seconds: Option<i64>,
    #[arg(long)]
    custom_message_template: Option<String>,
    #[arg(long)]
    agent_ids: Option<String>,
    #[arg(long)]
    required_tags: Option<String>,
    #[arg(long)]
    scheduled_run_ids: Option<String>,
    #[arg(long)]
    conditions: Option<String>,
    #[arg(long)]
    channels: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    monitor_id: String,
    #[command(flatten)]
    input_json: InputJsonArg,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    scope: Option<String>,
    #[arg(long)]
    match_mode: Option<String>,
    #[arg(long)]
    cooldown_seconds: Option<i64>,
    #[arg(long)]
    custom_message_template: Option<String>,
    #[arg(long)]
    agent_ids: Option<String>,
    #[arg(long)]
    required_tags: Option<String>,
    #[arg(long)]
    scheduled_run_ids: Option<String>,
    #[arg(long)]
    conditions: Option<String>,
    #[arg(long)]
    channels: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    monitor_id: String,
}

#[derive(Args)]
pub struct TestEvaluateArgs {
    monitor_id: String,
    #[arg(long)]
    run_id: String,
}

#[derive(Args)]
pub struct ListEventArgs {
    #[arg(long)]
    outcome: Option<String>,
    #[arg(long, default_value = "50")]
    page_size: u32,
}

pub async fn execute(
    cmd: MonitorCommands,
    client: &CovalClient,
    ctx: &OutputContext,
) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        MonitorCommands::Context => {
            return crate::commands::agent::resource_context("monitors", ctx);
        }
        MonitorCommands::List(args) => {
            let params = ListParams {
                filter: args.scope.map(|s| format!("scope:{s}")),
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.monitors().list(params).await?;
            emit_list_with_actions(
                ctx,
                "monitors",
                operation,
                &response.monitors,
                next_actions::list_result(
                    "monitors",
                    response.monitors.first().map(|m| m.id.as_str()),
                ),
            );
        }
        MonitorCommands::Get(args) => {
            let monitor = client.monitors().get(&args.monitor_id).await?;
            emit_one_with_actions(
                ctx,
                "monitors",
                operation,
                &monitor,
                next_actions::item_result("monitors", &monitor.id),
            );
        }
        MonitorCommands::Create(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "name", args.name)?;
            input_json::insert(&mut input, "description", args.description)?;
            input_json::insert(&mut input, "evaluation_type", Some(args.evaluation_type))?;
            input_json::insert(&mut input, "scope", args.scope)?;
            input_json::insert(&mut input, "match_mode", args.match_mode)?;
            input_json::insert(&mut input, "cooldown_seconds", args.cooldown_seconds)?;
            input_json::insert(
                &mut input,
                "custom_message_template",
                args.custom_message_template,
            )?;
            if let Some(ref ids) = args.agent_ids {
                let v: Vec<String> = ids.split(',').map(|s| s.trim().to_string()).collect();
                input_json::insert(&mut input, "agent_ids", Some(v))?;
            }
            if let Some(ref ids) = args.required_tags {
                let v: Vec<String> = ids.split(',').map(|s| s.trim().to_string()).collect();
                input_json::insert(&mut input, "required_tags", Some(v))?;
            }
            if let Some(ref ids) = args.scheduled_run_ids {
                let v: Vec<String> = ids.split(',').map(|s| s.trim().to_string()).collect();
                input_json::insert(&mut input, "scheduled_run_ids", Some(v))?;
            }
            if let Some(ref c) = args.conditions {
                let v: serde_json::Value = serde_json::from_str(c)?;
                input_json::insert(&mut input, "conditions", Some(v))?;
            }
            if let Some(ref c) = args.channels {
                let v: serde_json::Value = serde_json::from_str(c)?;
                input_json::insert(&mut input, "channels", Some(v))?;
            }
            if input
                .get("name")
                .and_then(serde_json::Value::as_str)
                .is_none()
            {
                anyhow::bail!("--name is required (or provide it via --input-json)");
            }
            let has_conditions = input
                .get("conditions")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|c| !c.is_empty());
            if !has_conditions {
                anyhow::bail!("--conditions is required (or provide it via --input-json)");
            }
            let req = input_json::finish(input)?;
            let monitor = client.monitors().create(req).await?;
            emit_one_with_actions(
                ctx,
                "monitors",
                operation,
                &monitor,
                next_actions::item_result("monitors", &monitor.id),
            );
        }
        MonitorCommands::Update(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "name", args.name)?;
            input_json::insert(&mut input, "description", args.description)?;
            input_json::insert(&mut input, "scope", args.scope)?;
            input_json::insert(&mut input, "match_mode", args.match_mode)?;
            input_json::insert(&mut input, "cooldown_seconds", args.cooldown_seconds)?;
            input_json::insert(
                &mut input,
                "custom_message_template",
                args.custom_message_template,
            )?;
            if let Some(ref ids) = args.agent_ids {
                let v: Vec<String> = ids.split(',').map(|s| s.trim().to_string()).collect();
                input_json::insert(&mut input, "agent_ids", Some(v))?;
            }
            if let Some(ref ids) = args.required_tags {
                let v: Vec<String> = ids.split(',').map(|s| s.trim().to_string()).collect();
                input_json::insert(&mut input, "required_tags", Some(v))?;
            }
            if let Some(ref ids) = args.scheduled_run_ids {
                let v: Vec<String> = ids.split(',').map(|s| s.trim().to_string()).collect();
                input_json::insert(&mut input, "scheduled_run_ids", Some(v))?;
            }
            if let Some(ref c) = args.conditions {
                let v: serde_json::Value = serde_json::from_str(c)?;
                input_json::insert(&mut input, "conditions", Some(v))?;
            }
            if let Some(ref c) = args.channels {
                let v: serde_json::Value = serde_json::from_str(c)?;
                input_json::insert(&mut input, "channels", Some(v))?;
            }
            let req = input_json::finish(input)?;
            let monitor = client.monitors().update(&args.monitor_id, req).await?;
            emit_one_with_actions(
                ctx,
                "monitors",
                operation,
                &monitor,
                next_actions::item_result("monitors", &monitor.id),
            );
        }
        MonitorCommands::Delete(args) => {
            client.monitors().delete(&args.monitor_id).await?;
            emit_success_with_actions(
                ctx,
                "monitors",
                operation,
                "Monitor deleted.",
                next_actions::delete_result("monitors"),
            );
        }
        MonitorCommands::TestEvaluate(args) => {
            let result = client
                .monitors()
                .test_evaluate(&args.monitor_id, &args.run_id)
                .await?;
            emit_one_with_actions(
                ctx,
                "monitors",
                operation,
                &result,
                next_actions::item_result("monitors", &args.monitor_id),
            );
        }
        MonitorCommands::Events {
            monitor_id,
            command,
        } => {
            let list_args = match command {
                Some(EventCommands::List(args)) => args,
                None => ListEventArgs {
                    outcome: None,
                    page_size: 50,
                },
            };
            let params = ListParams {
                filter: list_args.outcome.map(|o| format!("outcome:{o}")),
                page_size: Some(list_args.page_size),
                ..Default::default()
            };
            let response = client.monitors().list_events(&monitor_id, params).await?;
            emit_list_with_actions(
                ctx,
                "monitor-events",
                "list-events",
                &response.events,
                next_actions::list_result(
                    "monitor-events",
                    response.events.first().map(|e| e.id.as_str()),
                ),
            );
        }
    }
    Ok(())
}
