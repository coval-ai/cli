use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{CreateSavedViewRequest, ListParams, UpdateSavedViewRequest};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum SavedViewCommands {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    #[command(name = "set-default")]
    SetDefault(SetDefaultArgs),
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
    view_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    /// View name
    #[arg(long)]
    name: String,
    /// Page context identifier
    #[arg(long)]
    page_context: String,
    /// JSON configuration for the view
    #[arg(long)]
    config: String,
}

#[derive(Args)]
pub struct UpdateArgs {
    view_id: String,
    /// View name
    #[arg(long)]
    name: Option<String>,
    /// Page context identifier
    #[arg(long)]
    page_context: Option<String>,
    /// JSON configuration for the view
    #[arg(long)]
    config: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    view_id: String,
}

#[derive(Args)]
pub struct SetDefaultArgs {
    view_id: String,
}

pub async fn execute(
    cmd: SavedViewCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        SavedViewCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.saved_views().list(params).await?;
            print_list(&response.saved_views, format);
        }
        SavedViewCommands::Get(args) => {
            let view = client.saved_views().get(&args.view_id).await?;
            print_one(&view, format);
        }
        SavedViewCommands::Create(args) => {
            let config: serde_json::Value = serde_json::from_str(&args.config)
                .map_err(|e| anyhow::anyhow!("Invalid JSON for --config: {e}"))?;
            let req = CreateSavedViewRequest {
                name: args.name,
                page_context: args.page_context,
                config,
            };
            let view = client.saved_views().create(req).await?;
            print_one(&view, format);
        }
        SavedViewCommands::Update(args) => {
            let config = args
                .config
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(|e| anyhow::anyhow!("Invalid JSON for --config: {e}"))?;
            let req = UpdateSavedViewRequest {
                name: args.name,
                page_context: args.page_context,
                config,
            };
            let view = client.saved_views().update(&args.view_id, req).await?;
            print_one(&view, format);
        }
        SavedViewCommands::Delete(args) => {
            client.saved_views().delete(&args.view_id).await?;
            print_success("Saved view deleted.");
        }
        SavedViewCommands::SetDefault(args) => {
            let view = client.saved_views().set_default(&args.view_id).await?;
            print_one(&view, format);
        }
    }
    Ok(())
}
