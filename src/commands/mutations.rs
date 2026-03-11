use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{CreateMutationRequest, ListParams, UpdateMutationRequest};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum MutationCommands {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
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
    /// Parent agent ID (22-char ID)
    #[arg(long)]
    agent_id: String,
    /// Mutation name, unique per agent (1-200 characters)
    #[arg(long)]
    name: String,
    /// Mutation description (max 2000 characters)
    #[arg(long)]
    description: Option<String>,
    /// JSON string of config overrides to deep-merge with parent agent (max 10KB)
    #[arg(long)]
    config: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
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
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        MutationCommands::List(args) => {
            let params = ListParams {
                page_size: Some(args.page_size),
                ..Default::default()
            };
            let response = client.mutations(&args.agent_id).list(params).await?;
            print_list(&response.mutations, format);
        }
        MutationCommands::Get(args) => {
            let mutation = client
                .mutations(&args.agent_id)
                .get(&args.mutation_id)
                .await?;
            print_one(&mutation, format);
        }
        MutationCommands::Create(args) => {
            let config_overrides = args
                .config
                .as_ref()
                .map(|c| serde_json::from_str(c))
                .transpose()?;
            let req = CreateMutationRequest {
                display_name: args.name,
                description: args.description,
                config_overrides,
                parameter_values: None,
            };
            let mutation = client.mutations(&args.agent_id).create(req).await?;
            print_one(&mutation, format);
        }
        MutationCommands::Update(args) => {
            let config_overrides = args
                .config
                .as_ref()
                .map(|c| serde_json::from_str(c))
                .transpose()?;
            let req = UpdateMutationRequest {
                display_name: args.name,
                description: args.description,
                config_overrides,
                parameter_values: None,
            };
            let mutation = client
                .mutations(&args.agent_id)
                .update(&args.mutation_id, req)
                .await?;
            print_one(&mutation, format);
        }
        MutationCommands::Delete(args) => {
            client
                .mutations(&args.agent_id)
                .delete(&args.mutation_id)
                .await?;
            print_success("Mutation deleted.");
        }
    }
    Ok(())
}
