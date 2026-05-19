use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{CreateTestSetRequest, ListParams, UpdateTestSetRequest};
use crate::client::CovalClient;
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, NextAction,
    OutputContext,
};

#[derive(Subcommand)]
pub enum TestSetCommands {
    Context,
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

impl TestSetCommands {
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
    /// Filter expression (e.g. test_set_type=SCENARIO)
    #[arg(long)]
    filter: Option<String>,
    /// Results per page (1-100, default 50)
    #[arg(long, default_value = "50")]
    page_size: u32,
    /// Sort field, prefix with - for descending (default: -update_time)
    #[arg(long)]
    order_by: Option<String>,
}

#[derive(Args)]
pub struct GetArgs {
    test_set_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    /// Test set name (1-100 characters)
    #[arg(long)]
    name: String,
    /// URL-friendly identifier; auto-generated if omitted
    #[arg(long)]
    slug: Option<String>,
    /// Human-readable description
    #[arg(long)]
    description: Option<String>,
    /// Test set type (e.g. DEFAULT, SCENARIO, TRANSCRIPT, WORKFLOW)
    #[arg(long)]
    r#type: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    test_set_id: String,
    /// Test set name (1-100 characters)
    #[arg(long)]
    name: Option<String>,
    /// URL-friendly identifier
    #[arg(long)]
    slug: Option<String>,
    /// Human-readable description
    #[arg(long)]
    description: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    test_set_id: String,
}

pub async fn execute(
    cmd: TestSetCommands,
    client: &CovalClient,
    ctx: &OutputContext,
) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        TestSetCommands::Context => {
            return crate::commands::agent::resource_context("test-sets", ctx)
        }
        TestSetCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.test_sets().list(params).await?;
            emit_list_with_actions(
                ctx,
                "test-sets",
                operation,
                &response.test_sets,
                list_actions(
                    response
                        .test_sets
                        .first()
                        .map(|test_set| test_set.id.as_str()),
                ),
            );
        }
        TestSetCommands::Get(args) => {
            let test_set = client.test_sets().get(&args.test_set_id).await?;
            emit_one_with_actions(
                ctx,
                "test-sets",
                operation,
                &test_set,
                test_set_actions(&test_set.id),
            );
        }
        TestSetCommands::Create(args) => {
            let req = CreateTestSetRequest {
                display_name: args.name,
                slug: args.slug,
                description: args.description,
                test_set_type: args.r#type,
                test_set_metadata: None,
                parameters: None,
            };
            let test_set = client.test_sets().create(req).await?;
            emit_one_with_actions(
                ctx,
                "test-sets",
                operation,
                &test_set,
                test_set_actions(&test_set.id),
            );
        }
        TestSetCommands::Update(args) => {
            let req = UpdateTestSetRequest {
                display_name: args.name,
                slug: args.slug,
                description: args.description,
                ..Default::default()
            };
            let test_set = client.test_sets().update(&args.test_set_id, req).await?;
            emit_one_with_actions(
                ctx,
                "test-sets",
                operation,
                &test_set,
                test_set_actions(&test_set.id),
            );
        }
        TestSetCommands::Delete(args) => {
            client.test_sets().delete(&args.test_set_id).await?;
            emit_success_with_actions(
                ctx,
                "test-sets",
                operation,
                "Test set deleted.",
                vec![
                    next_actions::list("test-sets").primary(),
                    next_actions::context("test-sets"),
                ],
            );
        }
    }
    Ok(())
}

fn list_actions(id: Option<&str>) -> Vec<NextAction> {
    let mut actions = vec![next_actions::context("test-sets")];
    if let Some(id) = id {
        actions.insert(0, next_actions::get("test-sets", id).primary());
    }
    actions
}

fn test_set_actions(test_set_id: &str) -> Vec<NextAction> {
    vec![
        next_actions::test_cases_for_set(test_set_id).primary(),
        next_actions::context("test-sets"),
    ]
}
