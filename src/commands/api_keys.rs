use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{
    ApiKeyEnvironment, ApiKeyStatus, ApiKeyType, CreateApiKeyRequest, ListParams,
    UpdateApiKeyRequest,
};
use crate::client::CovalClient;
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_one_with_warnings_and_actions,
    emit_success_with_actions, AgentWarning, OutputContext,
};

#[derive(Subcommand)]
pub enum ApiKeyCommands {
    Context,
    List(ListArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

impl ApiKeyCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::List(_) => "list",
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

pub async fn execute(cmd: ApiKeyCommands, client: &CovalClient, ctx: &OutputContext) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        ApiKeyCommands::Context => {
            return crate::commands::agent::resource_context("api-keys", ctx)
        }
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
            emit_list_with_actions(
                ctx,
                "api-keys",
                operation,
                &response.api_keys,
                vec![next_actions::context("api-keys")],
            );
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
                if ctx.agent {
                    emit_one_with_warnings_and_actions(
                        ctx,
                        "api-keys",
                        operation,
                        &api_key,
                        vec![AgentWarning::new(
                            "store_api_key",
                            "Store this key now. It will not be shown again.",
                        )],
                        vec![next_actions::context("api-keys")],
                    );
                } else {
                    eprintln!("WARNING: Store this key now. It will not be shown again.");
                    eprintln!("Key: {}", api_key.key);
                    emit_one_with_actions(ctx, "api-keys", operation, &api_key, Vec::new());
                }
            } else {
                emit_one_with_actions(
                    ctx,
                    "api-keys",
                    operation,
                    &api_key,
                    vec![next_actions::context("api-keys")],
                );
            }
        }
        ApiKeyCommands::Update(args) => {
            let req = UpdateApiKeyRequest {
                status: args.status,
                reason: args.reason,
            };
            let api_key = client.api_keys().update(&args.api_key_id, req).await?;
            emit_one_with_actions(
                ctx,
                "api-keys",
                operation,
                &api_key,
                vec![
                    next_actions::list("api-keys").primary(),
                    next_actions::context("api-keys"),
                ],
            );
        }
        ApiKeyCommands::Delete(args) => {
            client.api_keys().delete(&args.api_key_id).await?;
            emit_success_with_actions(
                ctx,
                "api-keys",
                operation,
                "API key deleted.",
                next_actions::delete_result("api-keys"),
            );
        }
    }
    Ok(())
}
