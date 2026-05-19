use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{
    CreateReviewProjectRequest, ListParams, ProjectType, UpdateReviewProjectRequest,
};
use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, OutputContext,
};

#[derive(Subcommand)]
pub enum ReviewProjectCommands {
    Context,
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

impl ReviewProjectCommands {
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
    #[arg(long)]
    filter: Option<String>,
    #[arg(long, default_value = "50")]
    page_size: u32,
    #[arg(long)]
    order_by: Option<String>,
}

#[derive(Args)]
pub struct GetArgs {
    project_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    #[command(flatten)]
    input_json: InputJsonArg,
    /// Project name
    #[arg(long)]
    name: Option<String>,
    /// Comma-separated reviewer emails
    #[arg(long, value_delimiter = ',')]
    assignees: Option<Vec<String>>,
    /// Comma-separated simulation output IDs
    #[arg(long, value_delimiter = ',')]
    simulation_ids: Option<Vec<String>>,
    /// Comma-separated metric IDs
    #[arg(long, value_delimiter = ',')]
    metric_ids: Option<Vec<String>>,
    /// Optional project description
    #[arg(long)]
    description: Option<String>,
    /// Project type
    #[arg(long, value_enum)]
    project_type: Option<ProjectType>,
    /// Enable notifications for assignees
    #[arg(long)]
    notifications: Option<bool>,
}

#[derive(Args)]
pub struct UpdateArgs {
    project_id: String,
    #[command(flatten)]
    input_json: InputJsonArg,
    /// Updated project name
    #[arg(long)]
    name: Option<String>,
    /// Updated description
    #[arg(long)]
    description: Option<String>,
    /// Comma-separated updated reviewer emails
    #[arg(long, value_delimiter = ',')]
    assignees: Option<Vec<String>>,
    /// Comma-separated updated simulation output IDs
    #[arg(long, value_delimiter = ',')]
    simulation_ids: Option<Vec<String>>,
    /// Comma-separated updated metric IDs
    #[arg(long, value_delimiter = ',')]
    metric_ids: Option<Vec<String>>,
    /// Updated notification setting
    #[arg(long)]
    notifications: Option<bool>,
    /// Comma-separated emails of assignees to opt out
    #[arg(long, value_delimiter = ',')]
    opted_out_assignees: Option<Vec<String>>,
}

#[derive(Args)]
pub struct DeleteArgs {
    project_id: String,
}

pub async fn execute(
    cmd: ReviewProjectCommands,
    client: &CovalClient,
    ctx: &OutputContext,
) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        ReviewProjectCommands::Context => {
            return crate::commands::agent::resource_context("review-projects", ctx);
        }
        ReviewProjectCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.review_projects().list(params).await?;
            emit_list_with_actions(
                ctx,
                "review-projects",
                operation,
                &response.review_projects,
                next_actions::list_result(
                    "review-projects",
                    response
                        .review_projects
                        .first()
                        .map(|project| project.id.as_str()),
                ),
            );
        }
        ReviewProjectCommands::Get(args) => {
            let project = client.review_projects().get(&args.project_id).await?;
            emit_one_with_actions(
                ctx,
                "review-projects",
                operation,
                &project,
                next_actions::item_result("review-projects", &project.id),
            );
        }
        ReviewProjectCommands::Create(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "display_name", args.name)?;
            input_json::insert(&mut input, "assignees", args.assignees)?;
            input_json::insert(&mut input, "linked_simulation_ids", args.simulation_ids)?;
            input_json::insert(&mut input, "linked_metric_ids", args.metric_ids)?;
            input_json::insert(&mut input, "description", args.description)?;
            input_json::insert(&mut input, "project_type", args.project_type)?;
            input_json::insert(&mut input, "notifications", args.notifications)?;
            let req: CreateReviewProjectRequest = input_json::finish(input)?;
            let project = client.review_projects().create(req).await?;
            emit_one_with_actions(
                ctx,
                "review-projects",
                operation,
                &project,
                next_actions::item_result("review-projects", &project.id),
            );
        }
        ReviewProjectCommands::Update(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "display_name", args.name)?;
            input_json::insert(&mut input, "description", args.description)?;
            input_json::insert(&mut input, "assignees", args.assignees)?;
            input_json::insert(&mut input, "linked_simulation_ids", args.simulation_ids)?;
            input_json::insert(&mut input, "linked_metric_ids", args.metric_ids)?;
            input_json::insert(&mut input, "notifications", args.notifications)?;
            input_json::insert(&mut input, "opted_out_assignees", args.opted_out_assignees)?;
            let req: UpdateReviewProjectRequest = input_json::finish(input)?;
            let project = client
                .review_projects()
                .update(&args.project_id, req)
                .await?;
            emit_one_with_actions(
                ctx,
                "review-projects",
                operation,
                &project,
                next_actions::item_result("review-projects", &project.id),
            );
        }
        ReviewProjectCommands::Delete(args) => {
            client.review_projects().delete(&args.project_id).await?;
            emit_success_with_actions(
                ctx,
                "review-projects",
                operation,
                "Review project deleted.",
                next_actions::delete_result("review-projects"),
            );
        }
    }
    Ok(())
}
