use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, OutputContext,
};

#[derive(Subcommand)]
pub enum TagCommands {
    Context,
    List,
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

impl TagCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::List => "list",
            Self::Get(_) => "get",
            Self::Create(_) => "create",
            Self::Update(_) => "update",
            Self::Delete(_) => "delete",
        }
    }
}

#[derive(Args)]
pub struct GetArgs {
    tag_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    #[command(flatten)]
    input_json: InputJsonArg,
    #[arg(long)]
    tag_name: Option<String>,
    #[arg(long)]
    color: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    tag_id: String,
    #[command(flatten)]
    input_json: InputJsonArg,
    #[arg(long)]
    tag_name: Option<String>,
    #[arg(long)]
    color: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    tag_id: String,
}

pub async fn execute(cmd: TagCommands, client: &CovalClient, ctx: &OutputContext) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        TagCommands::Context => {
            return crate::commands::agent::resource_context("tags", ctx);
        }
        TagCommands::List => {
            let response = client.tags().list().await?;
            emit_list_with_actions(
                ctx,
                "tags",
                operation,
                &response.tags,
                next_actions::list_result("tags", response.tags.first().map(|t| t.id.as_str())),
            );
        }
        TagCommands::Get(args) => {
            let tag = client.tags().get(&args.tag_id).await?;
            emit_one_with_actions(
                ctx,
                "tags",
                operation,
                &tag,
                next_actions::item_result("tags", &tag.id),
            );
        }
        TagCommands::Create(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "tag_name", args.tag_name)?;
            input_json::insert(&mut input, "color", args.color)?;
            let req = input_json::finish(input)?;
            let tag = client.tags().create(req).await?;
            emit_one_with_actions(
                ctx,
                "tags",
                operation,
                &tag,
                next_actions::item_result("tags", &tag.id),
            );
        }
        TagCommands::Update(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "tag_name", args.tag_name)?;
            input_json::insert(&mut input, "color", args.color)?;
            anyhow::ensure!(
                !input.is_empty(),
                "provide at least --tag-name or --color to update"
            );
            let req = input_json::finish(input)?;
            let tag = client.tags().update(&args.tag_id, req).await?;
            emit_one_with_actions(
                ctx,
                "tags",
                operation,
                &tag,
                next_actions::item_result("tags", &tag.id),
            );
        }
        TagCommands::Delete(args) => {
            client.tags().delete(&args.tag_id).await?;
            emit_success_with_actions(
                ctx,
                "tags",
                operation,
                "Tag deleted.",
                next_actions::delete_result("tags"),
            );
        }
    }
    Ok(())
}
