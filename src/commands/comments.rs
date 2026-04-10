use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{CreateCommentRequest, UpdateCommentRequest};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum CommentCommands {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

#[derive(Args)]
pub struct ListArgs {
    /// Simulation output ID to list comments for
    simulation_output_id: String,
}

#[derive(Args)]
pub struct GetArgs {
    comment_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    /// Simulation output ID to add comment to
    simulation_output_id: String,
    /// Comment content
    #[arg(long)]
    content: String,
    /// Message index in the conversation
    #[arg(long)]
    message_index: i64,
    /// Selected text from the message
    #[arg(long)]
    selected_text: Option<String>,
    /// Parent comment ID for replies
    #[arg(long)]
    parent_comment_id: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    comment_id: String,
    /// Updated comment content
    #[arg(long)]
    content: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    comment_id: String,
}

pub async fn execute(
    cmd: CommentCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        CommentCommands::List(args) => {
            let response = client.comments().list(&args.simulation_output_id).await?;
            print_list(&response.comments, format);
        }
        CommentCommands::Get(args) => {
            let comment = client.comments().get(&args.comment_id).await?;
            print_one(&comment, format);
        }
        CommentCommands::Create(args) => {
            let req = CreateCommentRequest {
                content: args.content,
                message_index: args.message_index,
                selected_text: args.selected_text,
                parent_comment_id: args.parent_comment_id,
            };
            let comment = client
                .comments()
                .create(&args.simulation_output_id, req)
                .await?;
            print_one(&comment, format);
        }
        CommentCommands::Update(args) => {
            let req = UpdateCommentRequest {
                content: args.content,
            };
            let comment = client.comments().update(&args.comment_id, req).await?;
            print_one(&comment, format);
        }
        CommentCommands::Delete(args) => {
            client.comments().delete(&args.comment_id).await?;
            print_success("Comment deleted.");
        }
    }
    Ok(())
}
