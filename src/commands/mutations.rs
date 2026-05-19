use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{CreateMutationRequest, ListParams, UpdateMutationRequest};
use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, OutputContext,
};

#[derive(Subcommand)]
pub enum MutationCommands {
    Context,
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

impl MutationCommands {
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
    /// Parent agent ID (22-char ID)
    #[arg(long)]
    agent_id: String,
    /// Results per page (1-100, default 50)
    #[arg(long, default_value = "50")]
    page_size: u32,
}

#[derive(Args)]
pub struct GetArgs {
    /// Parent agent ID (22-char ID)
    #[arg(long)]
    agent_id: String,
    mutation_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    #[command(flatten)]
    input_json: InputJsonArg,
    /// Parent agent ID (22-char ID)
    #[arg(long)]
    agent_id: String,
    /// Mutation name, unique per agent (1-200 characters)
    #[arg(long)]
    name: Option<String>,
    /// Mutation description (max 2000 characters)
    #[arg(long)]
    description: Option<String>,
    /// JSON string of config overrides to deep-merge with parent agent (max 10KB)
    #[arg(long)]
    config: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    #[command(flatten)]
    input_json: InputJsonArg,
    /// Parent agent ID (22-char ID)
    #[arg(long)]
    agent_id: String,
    mutation_id: String,
    /// Mutation name, unique per agent (1-200 characters)
    #[arg(long)]
    name: Option<String>,
    /// Mutation description (max 2000 characters)
    #[arg(long)]
    description: Option<String>,
    /// JSON string of config overrides to deep-merge with parent agent (max 10KB)
    #[arg(long)]
    config: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// Parent agent ID (22-char ID)
    #[arg(long)]
    agent_id: String,
    mutation_id: String,
}

pub async fn execute(
    cmd: MutationCommands,
    client: &CovalClient,
    ctx: &OutputContext,
) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        MutationCommands::Context => {
            return crate::commands::agent::resource_context("mutations", ctx)
        }
        MutationCommands::List(args) => {
            let params = ListParams {
                page_size: Some(args.page_size),
                ..Default::default()
            };
            let response = client.mutations(&args.agent_id).list(params).await?;
            emit_list_with_actions(
                ctx,
                "mutations",
                operation,
                &response.mutations,
                mutation_list_actions(
                    &args.agent_id,
                    response
                        .mutations
                        .first()
                        .map(|mutation| mutation.id.as_str()),
                ),
            );
        }
        MutationCommands::Get(args) => {
            let mutation = client
                .mutations(&args.agent_id)
                .get(&args.mutation_id)
                .await?;
            emit_one_with_actions(
                ctx,
                "mutations",
                operation,
                &mutation,
                mutation_actions(&mutation.id, &mutation.agent_id),
            );
        }
        MutationCommands::Create(args) => {
            let mut input = args.input_json.object()?;
            let config_overrides: Option<serde_json::Value> = args
                .config
                .as_ref()
                .map(|c| serde_json::from_str(c))
                .transpose()?;
            input_json::insert(&mut input, "display_name", args.name)?;
            input_json::insert(&mut input, "description", args.description)?;
            input_json::insert(&mut input, "config_overrides", config_overrides)?;
            let req: CreateMutationRequest = input_json::finish(input)?;
            let mutation = client.mutations(&args.agent_id).create(req).await?;
            emit_one_with_actions(
                ctx,
                "mutations",
                operation,
                &mutation,
                mutation_actions(&mutation.id, &mutation.agent_id),
            );
        }
        MutationCommands::Update(args) => {
            let mut input = args.input_json.object()?;
            let config_overrides: Option<serde_json::Value> = args
                .config
                .as_ref()
                .map(|c| serde_json::from_str(c))
                .transpose()?;
            input_json::insert(&mut input, "display_name", args.name)?;
            input_json::insert(&mut input, "description", args.description)?;
            input_json::insert(&mut input, "config_overrides", config_overrides)?;
            let req: UpdateMutationRequest = input_json::finish(input)?;
            let mutation = client
                .mutations(&args.agent_id)
                .update(&args.mutation_id, req)
                .await?;
            emit_one_with_actions(
                ctx,
                "mutations",
                operation,
                &mutation,
                mutation_actions(&mutation.id, &mutation.agent_id),
            );
        }
        MutationCommands::Delete(args) => {
            client
                .mutations(&args.agent_id)
                .delete(&args.mutation_id)
                .await?;
            emit_success_with_actions(
                ctx,
                "mutations",
                operation,
                "Mutation deleted.",
                vec![
                    next_actions::mutations_for_agent(&args.agent_id).primary(),
                    next_actions::context("mutations"),
                ],
            );
        }
    }
    Ok(())
}

fn mutation_actions(mutation_id: &str, agent_id: &str) -> Vec<crate::output::NextAction> {
    vec![
        next_actions::mutation_get(agent_id, mutation_id).primary(),
        next_actions::get("agents", agent_id),
        next_actions::context("mutations"),
    ]
}

fn mutation_list_actions(
    agent_id: &str,
    mutation_id: Option<&str>,
) -> Vec<crate::output::NextAction> {
    let mut actions = vec![next_actions::context("mutations")];
    if let Some(mutation_id) = mutation_id {
        actions.insert(
            0,
            next_actions::mutation_get(agent_id, mutation_id).primary(),
        );
    }
    actions
}
