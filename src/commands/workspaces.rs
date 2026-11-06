use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, NextAction,
    OutputContext,
};

const RESOURCE: &str = "workspaces";

#[derive(Subcommand)]
pub enum WorkspaceCommands {
    Context,
    /// List active and archived workspaces
    List,
    /// Get a workspace by ID
    Get(GetArgs),
    /// Create a custom workspace
    Create(CreateArgs),
    /// Rename a workspace
    Update(UpdateArgs),
    /// Soft-delete a workspace
    Delete(WorkspaceIdArgs),
    /// Archive a workspace
    Archive(WorkspaceIdArgs),
}

impl WorkspaceCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::List => "list",
            Self::Get(_) => "get",
            Self::Create(_) => "create",
            Self::Update(_) => "update",
            Self::Delete(_) => "delete",
            Self::Archive(_) => "archive",
        }
    }
}

#[derive(Args)]
pub struct GetArgs {
    workspace_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    #[command(flatten)]
    input_json: InputJsonArg,
    /// Workspace display name
    #[arg(long, visible_alias = "display-name")]
    name: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    workspace_id: String,
    #[command(flatten)]
    input_json: InputJsonArg,
    /// New workspace display name
    #[arg(long, visible_alias = "display-name")]
    name: Option<String>,
}

#[derive(Args)]
pub struct WorkspaceIdArgs {
    workspace_id: String,
}

pub async fn execute(
    cmd: WorkspaceCommands,
    client: &CovalClient,
    ctx: &OutputContext,
) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        WorkspaceCommands::Context => {
            return crate::commands::agent::resource_context(RESOURCE, ctx);
        }
        WorkspaceCommands::List => {
            let response = client.workspaces().list().await?;
            let first_id = response.workspaces.first().map(|w| w.id.as_str());
            let mut actions = next_actions::list_result(RESOURCE, first_id);
            if let Some(id) = first_id {
                actions.push(scope_action(id));
            }
            emit_list_with_actions(ctx, RESOURCE, operation, &response.workspaces, actions);
        }
        WorkspaceCommands::Get(args) => {
            let workspace = client.workspaces().get(&args.workspace_id).await?;
            emit_one_with_actions(
                ctx,
                RESOURCE,
                operation,
                &workspace,
                item_actions(&workspace.id),
            );
        }
        WorkspaceCommands::Create(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "display_name", args.name)?;
            anyhow::ensure!(
                input.contains_key("display_name"),
                "provide --name (or display_name in --input-json)"
            );
            let req = input_json::finish(input)?;
            let workspace = client.workspaces().create(req).await?;
            emit_one_with_actions(
                ctx,
                RESOURCE,
                operation,
                &workspace,
                item_actions(&workspace.id),
            );
        }
        WorkspaceCommands::Update(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "display_name", args.name)?;
            anyhow::ensure!(
                input.contains_key("display_name"),
                "provide --name (or display_name in --input-json) to update"
            );
            let req = input_json::finish(input)?;
            let workspace = client.workspaces().update(&args.workspace_id, req).await?;
            emit_one_with_actions(
                ctx,
                RESOURCE,
                operation,
                &workspace,
                item_actions(&workspace.id),
            );
        }
        WorkspaceCommands::Delete(args) => {
            client.workspaces().delete(&args.workspace_id).await?;
            emit_success_with_actions(
                ctx,
                RESOURCE,
                operation,
                "Workspace deleted.",
                next_actions::delete_result(RESOURCE),
            );
        }
        WorkspaceCommands::Archive(args) => {
            let workspace = client.workspaces().archive(&args.workspace_id).await?;
            emit_one_with_actions(
                ctx,
                RESOURCE,
                operation,
                &workspace,
                vec![
                    next_actions::list(RESOURCE).primary(),
                    next_actions::context(RESOURCE),
                ],
            );
        }
    }
    Ok(())
}

fn item_actions(workspace_id: &str) -> Vec<NextAction> {
    let mut actions = next_actions::item_result(RESOURCE, workspace_id);
    actions.push(scope_action(workspace_id));
    actions
}

/// Show how to run any other command inside this workspace.
fn scope_action(workspace_id: &str) -> NextAction {
    NextAction::new(
        "workspaces.scope",
        "List agents in this workspace",
        next_actions::argv(["--workspace", workspace_id, "agents", "list"]),
        true,
    )
    .with_description(
        "Pass --workspace (or set COVAL_WORKSPACE_ID) on any command to scope it to this workspace.",
    )
}
