use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{CreateWebhookRequest, ListParams, UpdateWebhookRequest};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum WebhookCommands {
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
}

#[derive(Args)]
pub struct GetArgs {
    webhook_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    /// Webhook type (required)
    #[arg(long)]
    r#type: String,
    /// Webhook URL (required)
    #[arg(long)]
    url: String,
    /// JSON string for metadata
    #[arg(long)]
    metadata: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    webhook_id: String,
    /// Webhook type
    #[arg(long)]
    r#type: Option<String>,
    /// Webhook URL
    #[arg(long)]
    url: Option<String>,
    /// JSON string for metadata
    #[arg(long)]
    metadata: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    webhook_id: String,
}

pub async fn execute(
    cmd: WebhookCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        WebhookCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.webhooks().list(params).await?;
            print_list(&response.webhooks, format);
        }
        WebhookCommands::Get(args) => {
            let webhook = client.webhooks().get(&args.webhook_id).await?;
            print_one(&webhook, format);
        }
        WebhookCommands::Create(args) => {
            let metadata = args
                .metadata
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(|e| anyhow::anyhow!("Invalid JSON for --metadata: {e}"))?;

            let req = CreateWebhookRequest {
                webhook_type: args.r#type,
                url: args.url,
                metadata,
            };
            let webhook = client.webhooks().create(req).await?;
            print_one(&webhook, format);
        }
        WebhookCommands::Update(args) => {
            let metadata = args
                .metadata
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(|e| anyhow::anyhow!("Invalid JSON for --metadata: {e}"))?;

            let req = UpdateWebhookRequest {
                webhook_type: args.r#type,
                url: args.url,
                metadata,
            };
            let webhook = client.webhooks().update(&args.webhook_id, req).await?;
            print_one(&webhook, format);
        }
        WebhookCommands::Delete(args) => {
            client.webhooks().delete(&args.webhook_id).await?;
            print_success("Webhook deleted.");
        }
    }
    Ok(())
}
