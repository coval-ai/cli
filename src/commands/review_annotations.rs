use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{
    AnnotationPriority, AnnotationStatus, CompletionStatus, CreateReviewAnnotationRequest,
    ListParams, UpdateReviewAnnotationRequest,
};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum ReviewAnnotationCommands {
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
    annotation_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    /// Simulation output ID to link
    #[arg(long)]
    simulation_output_id: String,
    /// Metric ID to link
    #[arg(long)]
    metric_id: String,
    /// Email of the reviewer to assign
    #[arg(long)]
    assignee: String,
    /// Ground truth numeric value (auto-completes annotation)
    #[arg(long)]
    ground_truth_float: Option<f64>,
    /// Ground truth string value (auto-completes annotation)
    #[arg(long)]
    ground_truth_string: Option<String>,
    /// Reviewer notes
    #[arg(long)]
    notes: Option<String>,
    /// Annotation priority
    #[arg(long, value_enum)]
    priority: Option<AnnotationPriority>,
}

#[derive(Args)]
pub struct UpdateArgs {
    annotation_id: String,
    /// Ground truth numeric value (auto-completes annotation)
    #[arg(long)]
    ground_truth_float: Option<f64>,
    /// Ground truth string value (auto-completes annotation)
    #[arg(long)]
    ground_truth_string: Option<String>,
    /// Reviewer notes
    #[arg(long)]
    notes: Option<String>,
    /// Annotation priority
    #[arg(long, value_enum)]
    priority: Option<AnnotationPriority>,
    /// Reassign to a different reviewer
    #[arg(long)]
    assignee: Option<String>,
    /// Completion status
    #[arg(long, value_enum)]
    completion_status: Option<CompletionStatus>,
    /// Annotation status (active or archived)
    #[arg(long, value_enum)]
    status: Option<AnnotationStatus>,
}

#[derive(Args)]
pub struct DeleteArgs {
    annotation_id: String,
}

pub async fn execute(
    cmd: ReviewAnnotationCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        ReviewAnnotationCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.review_annotations().list(params).await?;
            print_list(&response.review_annotations, format);
        }
        ReviewAnnotationCommands::Get(args) => {
            let annotation = client.review_annotations().get(&args.annotation_id).await?;
            print_one(&annotation, format);
        }
        ReviewAnnotationCommands::Create(args) => {
            let req = CreateReviewAnnotationRequest {
                simulation_output_id: args.simulation_output_id,
                metric_id: args.metric_id,
                assignee: args.assignee,
                ground_truth_float_value: args.ground_truth_float,
                ground_truth_string_value: args.ground_truth_string,
                reviewer_notes: args.notes,
                priority: args.priority,
            };
            let annotation = client.review_annotations().create(req).await?;
            print_one(&annotation, format);
        }
        ReviewAnnotationCommands::Update(args) => {
            let req = UpdateReviewAnnotationRequest {
                ground_truth_float_value: args.ground_truth_float,
                ground_truth_string_value: args.ground_truth_string,
                reviewer_notes: args.notes,
                priority: args.priority,
                assignee: args.assignee,
                completion_status: args.completion_status,
                status: args.status,
            };
            let annotation = client
                .review_annotations()
                .update(&args.annotation_id, req)
                .await?;
            print_one(&annotation, format);
        }
        ReviewAnnotationCommands::Delete(args) => {
            client
                .review_annotations()
                .delete(&args.annotation_id)
                .await?;
            print_success("Review annotation deleted.");
        }
    }
    Ok(())
}
