use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{AgentType, CreateAgentRequest, ListParams, UpdateAgentRequest};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum AgentCommands {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    Duplicate(DuplicateArgs),
    #[command(name = "manage-metrics")]
    ManageMetrics(ManageMetricsArgs),
    #[command(name = "manage-test-sets")]
    ManageTestSets(ManageTestSetsArgs),
    #[command(name = "manage-workflows")]
    ManageWorkflows(ManageWorkflowsArgs),
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

#[derive(Args)]
pub struct DuplicateArgs {
    agent_id: String,
}

#[derive(Args)]
pub struct ManageMetricsArgs {
    agent_id: String,
    /// Action to perform: set, add, or remove
    #[arg(long)]
    action: String,
    /// Comma-separated metric IDs
    #[arg(long, value_delimiter = ',')]
    metric_ids: Vec<String>,
}

#[derive(Args)]
pub struct ManageTestSetsArgs {
    agent_id: String,
    /// Action to perform: set, add, or remove
    #[arg(long)]
    action: String,
    /// Comma-separated test set IDs
    #[arg(long, value_delimiter = ',')]
    test_set_ids: Vec<String>,
}

#[derive(Args)]
pub struct ManageWorkflowsArgs {
    agent_id: String,
    /// Workflows JSON object (replaces entire workflows field)
    #[arg(long)]
    workflows: String,
}

pub async fn execute(cmd: AgentCommands, client: &CovalClient, format: OutputFormat) -> Result<()> {
    match cmd {
        AgentCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.agents().list(params).await?;
            print_list(&response.agents, format);
        }
        AgentCommands::Get(args) => {
            let agent = client.agents().get(&args.agent_id).await?;
            print_one(&agent, format);
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
            print_one(&agent, format);
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
            print_one(&agent, format);
        }
        AgentCommands::Delete(args) => {
            client.agents().delete(&args.agent_id).await?;
            print_success("Agent deleted.");
        }
        AgentCommands::Duplicate(args) => {
            let agent = client.agents().duplicate(&args.agent_id).await?;
            print_one(&agent, format);
        }
        AgentCommands::ManageMetrics(args) => {
            let body = serde_json::json!({
                "action": args.action,
                "metric_ids": args.metric_ids,
            });
            let agent = client.agents().manage_metrics(&args.agent_id, &body).await?;
            print_one(&agent, format);
        }
        AgentCommands::ManageTestSets(args) => {
            let body = serde_json::json!({
                "action": args.action,
                "test_set_ids": args.test_set_ids,
            });
            let agent = client
                .agents()
                .manage_test_sets(&args.agent_id, &body)
                .await?;
            print_one(&agent, format);
        }
        AgentCommands::ManageWorkflows(args) => {
            let workflows: serde_json::Value = serde_json::from_str(&args.workflows)
                .map_err(|e| anyhow::anyhow!("Invalid JSON for --workflows: {e}"))?;
            let body = serde_json::json!({ "workflows": workflows });
            let agent = client
                .agents()
                .manage_workflows(&args.agent_id, &body)
                .await?;
            print_one(&agent, format);
        }
    }
    Ok(())
}
