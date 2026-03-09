use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{
    ApiKeyEnvironment, ApiKeyStatus, ApiKeyType, CreateApiKeyRequest, ListParams,
    UpdateApiKeyRequest,
};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum ApiKeyCommands {
    List(ListArgs),
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
    #[arg(long, value_enum)]
    status: Option<ApiKeyStatus>,
    #[arg(long, value_enum)]
    environment: Option<ApiKeyEnvironment>,
}

#[derive(Args)]
pub struct CreateArgs {
    #[arg(long)]
    name: String,
    #[arg(long)]
    description: Option<String>,
    #[arg(long, value_enum)]
    r#type: ApiKeyType,
    #[arg(long, value_enum)]
    environment: ApiKeyEnvironment,
    #[arg(long, value_delimiter = ',')]
    permissions: Option<Vec<String>>,
}

#[derive(Args)]
pub struct UpdateArgs {
    api_key_id: String,
    #[arg(long, value_enum)]
    status: ApiKeyStatus,
    #[arg(long)]
    reason: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    api_key_id: String,
}

pub async fn execute(
    cmd: ApiKeyCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        ApiKeyCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client
                .api_keys()
                .list(params, args.status, args.environment)
                .await?;
            print_list(&response.api_keys, format);
        }
        ApiKeyCommands::Create(args) => {
            let req = CreateApiKeyRequest {
                name: Some(args.name),
                description: args.description,
                key_type: args.r#type,
                environment: args.environment,
                permissions: args.permissions,
            };
            let api_key = client.api_keys().create(req).await?;
            if !api_key.key.is_empty() && !api_key.key.contains("***") {
                eprintln!("WARNING: Store this key now. It will not be shown again.");
                eprintln!("Key: {}", api_key.key);
            }
            print_one(&api_key, format);
        }
        ApiKeyCommands::Update(args) => {
            let req = UpdateApiKeyRequest {
                status: args.status,
                reason: args.reason,
            };
            let api_key = client.api_keys().update(&args.api_key_id, req).await?;
            print_one(&api_key, format);
        }
        ApiKeyCommands::Delete(args) => {
            client.api_keys().delete(&args.api_key_id).await?;
            print_success("API key deleted.");
        }
    }
    Ok(())
}
