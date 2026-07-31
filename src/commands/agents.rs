use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{AgentType, CreateAgentRequest, ListParams, UpdateAgentRequest};
use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
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
    after_help = "Required fields by agent type:\n  voice             --phone-number (E.164 or SIP)\n  outbound-voice    --endpoint (webhook URL)\n  chat              --metadata '{\"chat_endpoint\":\"https://...\"}'\n  chat-a2a          --metadata '{\"chat_endpoint\":\"https://...\"}'\n  chat-websocket    --metadata '{\"endpoint\":\"wss://...\"}'\n  sms               --phone-number (E.164)\n  websocket         --metadata '{\"endpoint\":\"wss://...\"}'\n  livekit           --metadata '{\"generate_token_endpoint\":\"https://...\",\"livekit_url\":\"wss://...\"}'\n  pipecat           --metadata '{\"pipecat_api_key\":\"...\",\"agent_name\":\"...\"}'\n  openai-realtime   --metadata '{\"openai_realtime_api_key\":\"...\"}'\n  gemini-realtime   --metadata '{\"gemini_realtime_api_key\":\"...\"}'\n  grok-realtime     --metadata '{\"grok_realtime_api_key\":\"...\"}'"
)]
pub struct CreateArgs {
    #[command(flatten)]
    input_json: InputJsonArg,
    /// Human-readable agent name
    #[arg(long)]
    name: Option<String>,
    /// Your own stable identifier for the agent
    #[arg(long)]
    customer_agent_id: Option<String>,
    /// Agent type (determines required fields, see below)
    #[arg(long, value_enum)]
    r#type: Option<AgentType>,
    /// Phone number in E.164 format; required for voice and sms
    #[arg(long)]
    phone_number: Option<String>,
    /// Webhook URL; required for outbound-voice
    #[arg(long)]
    endpoint: Option<String>,
    /// Agent instructions / system prompt
    #[arg(long)]
    prompt: Option<String>,
    /// Primary agent language
    #[arg(long)]
    language: Option<String>,
    /// JSON object of free-form agent attributes
    #[arg(long)]
    attributes: Option<String>,
    /// Comma-separated metric IDs to attach
    #[arg(long, value_delimiter = ',')]
    metric_ids: Option<Vec<String>>,
    /// Comma-separated test set IDs to attach
    #[arg(long, value_delimiter = ',')]
    test_set_ids: Option<Vec<String>>,
    /// JSON string for type-specific config (see required fields below)
    #[arg(long)]
    metadata: Option<String>,
    /// JSON object containing workflow configuration
    #[arg(long)]
    workflows: Option<String>,
    /// Comma-separated tag names
    #[arg(long, value_delimiter = ',')]
    tags: Option<Vec<String>>,
}

#[derive(Args)]
pub struct UpdateArgs {
    agent_id: String,
    #[command(flatten)]
    input_json: InputJsonArg,
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
            let mut input = args.input_json.object()?;
            let metadata = parse_json_argument(args.metadata, "metadata")?;
            let attributes = parse_json_argument(args.attributes, "attributes")?;
            let workflows = parse_json_argument(args.workflows, "workflows")?;

            input_json::insert(&mut input, "display_name", args.name)?;
            input_json::insert(&mut input, "customer_agent_id", args.customer_agent_id)?;
            input_json::insert(&mut input, "model_type", args.r#type)?;
            input_json::insert(&mut input, "phone_number", args.phone_number)?;
            input_json::insert(&mut input, "endpoint", args.endpoint)?;
            input_json::insert(&mut input, "prompt", args.prompt)?;
            input_json::insert(&mut input, "language", args.language)?;
            input_json::insert(&mut input, "attributes", attributes)?;
            input_json::insert(&mut input, "metadata", metadata)?;
            input_json::insert(&mut input, "workflows", workflows)?;
            input_json::insert(&mut input, "metric_ids", args.metric_ids)?;
            input_json::insert(&mut input, "test_set_ids", args.test_set_ids)?;
            input_json::insert(&mut input, "tags", args.tags)?;
            let req: CreateAgentRequest = input_json::finish(input)?;
            let agent = client.agents().create(req).await?;
            emit_one_with_actions(ctx, "agents", operation, &agent, agent_actions(&agent.id));
        }
        AgentCommands::Update(args) => {
            let mut input = args.input_json.object()?;
            let metadata: Option<serde_json::Value> = args
                .metadata
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(|e| anyhow::anyhow!("Invalid JSON for --metadata: {e}"))?;

            input_json::insert(&mut input, "display_name", args.name)?;
            input_json::insert(&mut input, "model_type", args.r#type)?;
            input_json::insert(&mut input, "phone_number", args.phone_number)?;
            input_json::insert(&mut input, "endpoint", args.endpoint)?;
            input_json::insert(&mut input, "prompt", args.prompt)?;
            input_json::insert(&mut input, "metadata", metadata)?;
            input_json::insert(&mut input, "metric_ids", args.metric_ids)?;
            input_json::insert(&mut input, "test_set_ids", args.test_set_ids)?;
            let req: UpdateAgentRequest = input_json::finish(input)?;
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

fn parse_json_argument(raw: Option<String>, flag_name: &str) -> Result<Option<serde_json::Value>> {
    raw.map(|value| {
        serde_json::from_str(&value)
            .map_err(|error| anyhow::anyhow!("Invalid JSON for --{flag_name}: {error}"))
    })
    .transpose()
}
