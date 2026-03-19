use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{
    CreateReviewProjectRequest, ListParams, ProjectType, UpdateReviewProjectRequest,
};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum ReviewProjectCommands {
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
    project_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    /// Project name
    #[arg(long)]
    name: String,
    /// Comma-separated reviewer emails
    #[arg(long, value_delimiter = ',')]
    assignees: Vec<String>,
    /// Comma-separated simulation output IDs
    #[arg(long, value_delimiter = ',')]
    simulation_ids: Vec<String>,
    /// Comma-separated metric IDs
    #[arg(long, value_delimiter = ',')]
    metric_ids: Vec<String>,
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
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        ReviewProjectCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.review_projects().list(params).await?;
            print_list(&response.review_projects, format);
        }
        ReviewProjectCommands::Get(args) => {
            let project = client.review_projects().get(&args.project_id).await?;
            print_one(&project, format);
        }
        ReviewProjectCommands::Create(args) => {
            let req = CreateReviewProjectRequest {
                display_name: args.name,
                assignees: args.assignees,
                linked_simulation_ids: args.simulation_ids,
                linked_metric_ids: args.metric_ids,
                description: args.description,
                project_type: args.project_type,
                notifications: args.notifications,
            };
            let project = client.review_projects().create(req).await?;
            print_one(&project, format);
        }
        ReviewProjectCommands::Update(args) => {
            let req = UpdateReviewProjectRequest {
                display_name: args.name,
                description: args.description,
                assignees: args.assignees,
                linked_simulation_ids: args.simulation_ids,
                linked_metric_ids: args.metric_ids,
                notifications: args.notifications,
                opted_out_assignees: args.opted_out_assignees,
            };
            let project = client
                .review_projects()
                .update(&args.project_id, req)
                .await?;
            print_one(&project, format);
        }
        ReviewProjectCommands::Delete(args) => {
            client.review_projects().delete(&args.project_id).await?;
            print_success("Review project deleted.");
        }
    }
    Ok(())
}
