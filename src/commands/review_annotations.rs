use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{
    AnnotationPriority, AnnotationStatus, CompletionStatus, CreateReviewAnnotationRequest,
    ListParams, UpdateReviewAnnotationRequest,
};
use crate::client::CovalClient;
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, OutputContext,
};

#[derive(Subcommand)]
pub enum ReviewAnnotationCommands {
    Context,
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

impl ReviewAnnotationCommands {
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
    /// Ground truth subvalues by timestamp as JSON array (auto-completes annotation)
    #[arg(long)]
    ground_truth_subvalues: Option<String>,
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
    /// Ground truth subvalues by timestamp as JSON array (auto-completes annotation)
    #[arg(long)]
    ground_truth_subvalues: Option<String>,
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
    ctx: &OutputContext,
) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        ReviewAnnotationCommands::Context => {
            return crate::commands::agent::resource_context("review-annotations", ctx);
        }
        ReviewAnnotationCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.review_annotations().list(params).await?;
            emit_list_with_actions(
                ctx,
                "review-annotations",
                operation,
                &response.review_annotations,
                next_actions::list_result(
                    "review-annotations",
                    response
                        .review_annotations
                        .first()
                        .map(|annotation| annotation.id.as_str()),
                ),
            );
        }
        ReviewAnnotationCommands::Get(args) => {
            let annotation = client.review_annotations().get(&args.annotation_id).await?;
            emit_one_with_actions(
                ctx,
                "review-annotations",
                operation,
                &annotation,
                next_actions::item_result("review-annotations", &annotation.id),
            );
        }
        ReviewAnnotationCommands::Create(args) => {
            let subvalues = args
                .ground_truth_subvalues
                .map(|s| serde_json::from_str(&s))
                .transpose()?;
            let req = CreateReviewAnnotationRequest {
                simulation_output_id: args.simulation_output_id,
                metric_id: args.metric_id,
                assignee: args.assignee,
                ground_truth_float_value: args.ground_truth_float,
                ground_truth_string_value: args.ground_truth_string,
                ground_truth_subvalues_by_timestamp: subvalues,
                reviewer_notes: args.notes,
                priority: args.priority,
            };
            let annotation = client.review_annotations().create(req).await?;
            emit_one_with_actions(
                ctx,
                "review-annotations",
                operation,
                &annotation,
                next_actions::item_result("review-annotations", &annotation.id),
            );
        }
        ReviewAnnotationCommands::Update(args) => {
            let subvalues = args
                .ground_truth_subvalues
                .map(|s| serde_json::from_str(&s))
                .transpose()?;
            let req = UpdateReviewAnnotationRequest {
                ground_truth_float_value: args.ground_truth_float,
                ground_truth_string_value: args.ground_truth_string,
                ground_truth_subvalues_by_timestamp: subvalues,
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
            emit_one_with_actions(
                ctx,
                "review-annotations",
                operation,
                &annotation,
                next_actions::item_result("review-annotations", &annotation.id),
            );
        }
        ReviewAnnotationCommands::Delete(args) => {
            client
                .review_annotations()
                .delete(&args.annotation_id)
                .await?;
            emit_success_with_actions(
                ctx,
                "review-annotations",
                operation,
                "Review annotation deleted.",
                next_actions::delete_result("review-annotations"),
            );
        }
    }
    Ok(())
}
