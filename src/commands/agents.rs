use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{AgentType, CreateAgentRequest, ListParams, UpdateAgentRequest};
use crate::client::CovalClient;
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, NextAction,
    OutputContext,
};

#[derive(Subcommand)]
pub enum AgentCommands {
    Context,
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

impl AgentCommands {
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
}

#[derive(Args)]
pub struct GetArgs {
    agent_id: String,
}

#[derive(Args)]
#[command(
    after_help = "Required fields by agent type:\n  voice           --phone-number (E.164, e.g. +12345678901)\n  outbound-voice  --endpoint (webhook URL)\n  chat            --metadata '{\"chat_endpoint\": \"https://...\"}'\n  sms             --phone-number (E.164)\n  websocket       --metadata '{\"endpoint\": \"wss://...\", \"initialization_json\": \"...\"}'"
)]
pub struct CreateArgs {
    /// Human-readable agent name
    #[arg(long)]
    name: String,
    /// Agent type (determines required fields, see below)
    #[arg(long, value_enum)]
    r#type: AgentType,
    /// Phone number in E.164 format; required for voice and sms
    #[arg(long)]
    phone_number: Option<String>,
    /// Webhook URL; required for outbound-voice
    #[arg(long)]
    endpoint: Option<String>,
    /// Agent instructions / system prompt
    #[arg(long)]
    prompt: Option<String>,
    /// Comma-separated metric IDs to attach
    #[arg(long, value_delimiter = ',')]
    metric_ids: Option<Vec<String>>,
    /// Comma-separated test set IDs to attach
    #[arg(long, value_delimiter = ',')]
    test_set_ids: Option<Vec<String>>,
    /// JSON string for type-specific config (see required fields below)
    #[arg(long)]
    metadata: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    agent_id: String,
    #[arg(long)]
    name: Option<String>,
    #[arg(long, value_enum)]
    r#type: Option<AgentType>,
    #[arg(long)]
    phone_number: Option<String>,
    #[arg(long)]
    endpoint: Option<String>,
    #[arg(long)]
    prompt: Option<String>,
    #[arg(long, value_delimiter = ',')]
    metric_ids: Option<Vec<String>>,
    #[arg(long, value_delimiter = ',')]
    test_set_ids: Option<Vec<String>>,
    /// JSON string for metadata
    #[arg(long)]
    metadata: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    agent_id: String,
}

pub async fn execute(cmd: AgentCommands, client: &CovalClient, ctx: &OutputContext) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        AgentCommands::Context => return crate::commands::agent::resource_context("agents", ctx),
        AgentCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.agents().list(params).await?;
            emit_list_with_actions(
                ctx,
                "agents",
                operation,
                &response.agents,
                list_actions(response.agents.first().map(|agent| agent.id.as_str())),
            );
        }
        AgentCommands::Get(args) => {
            let agent = client.agents().get(&args.agent_id).await?;
            emit_one_with_actions(ctx, "agents", operation, &agent, agent_actions(&agent.id));
        }
        AgentCommands::Create(args) => {
            let metadata = args
                .metadata
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(|e| anyhow::anyhow!("Invalid JSON for --metadata: {e}"))?;

            let req = CreateAgentRequest {
                display_name: args.name,
                model_type: args.r#type,
                phone_number: args.phone_number,
                endpoint: args.endpoint,
                prompt: args.prompt,
                metadata,
                metric_ids: args.metric_ids,
                test_set_ids: args.test_set_ids,
            };
            let agent = client.agents().create(req).await?;
            emit_one_with_actions(ctx, "agents", operation, &agent, agent_actions(&agent.id));
        }
        AgentCommands::Update(args) => {
            let metadata = args
                .metadata
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(|e| anyhow::anyhow!("Invalid JSON for --metadata: {e}"))?;

            let req = UpdateAgentRequest {
                display_name: args.name,
                model_type: args.r#type,
                phone_number: args.phone_number,
                endpoint: args.endpoint,
                prompt: args.prompt,
                metadata,
                metric_ids: args.metric_ids,
                test_set_ids: args.test_set_ids,
            };
            let agent = client.agents().update(&args.agent_id, req).await?;
            emit_one_with_actions(ctx, "agents", operation, &agent, agent_actions(&agent.id));
        }
        AgentCommands::Delete(args) => {
            client.agents().delete(&args.agent_id).await?;
            emit_success_with_actions(
                ctx,
                "agents",
                operation,
                "Agent deleted.",
                vec![
                    next_actions::list("agents").primary(),
                    next_actions::context("agents"),
                ],
            );
        }
    }
    Ok(())
}

fn list_actions(id: Option<&str>) -> Vec<NextAction> {
    let mut actions = vec![next_actions::context("agents")];
    if let Some(id) = id {
        actions.insert(0, next_actions::get("agents", id).primary());
    }
    actions
}

fn agent_actions(agent_id: &str) -> Vec<NextAction> {
    vec![
        next_actions::get("agents", agent_id).primary(),
        next_actions::mutations_for_agent(agent_id),
        next_actions::runs_for_agent(agent_id),
        next_actions::context("agents"),
    ]
}
