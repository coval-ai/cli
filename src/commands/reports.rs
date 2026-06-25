use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{
    CompareBy, CreateReportRequest, ReportPermission, UpdateReportRequest,
};
use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, OutputContext,
};

#[derive(Subcommand)]
pub enum ReportCommands {
    Context,
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

impl ReportCommands {
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
    /// Opaque cursor from a previous response's next_cursor
    #[arg(long)]
    cursor: Option<String>,
    /// Results per page (1-100, default 50)
    #[arg(long)]
    limit: Option<u32>,
}

#[derive(Args)]
pub struct GetArgs {
    /// Report ID (26-character ULID)
    report_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    #[command(flatten)]
    input_json: InputJsonArg,
    /// Display name for the saved report (1-200 characters)
    #[arg(long)]
    name: Option<String>,
    /// Comma-separated run IDs to include (min 1)
    #[arg(long, value_delimiter = ',')]
    run_ids: Option<Vec<String>>,
    /// Dimension to group and compare runs by (default none)
    #[arg(long, value_enum)]
    compare_by: Option<CompareBy>,
    /// Metadata key to group by (required when --compare-by metadata, rejected otherwise)
    #[arg(long)]
    metadata_key: Option<String>,
    /// Report visibility (default PRIVATE)
    #[arg(long, value_enum)]
    permissions: Option<ReportPermission>,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Report ID (26-character ULID)
    report_id: String,
    #[command(flatten)]
    input_json: InputJsonArg,
    /// Updated display name
    #[arg(long)]
    name: Option<String>,
    /// Updated comma-separated run IDs (replaces existing)
    #[arg(long, value_delimiter = ',')]
    run_ids: Option<Vec<String>>,
    /// Updated compare-by dimension
    #[arg(long, value_enum)]
    compare_by: Option<CompareBy>,
    /// Updated metadata key (only valid when compare-by is metadata)
    #[arg(long)]
    metadata_key: Option<String>,
    /// Updated visibility
    #[arg(long, value_enum)]
    permissions: Option<ReportPermission>,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// Report ID (26-character ULID)
    report_id: String,
}

pub async fn execute(cmd: ReportCommands, client: &CovalClient, ctx: &OutputContext) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        ReportCommands::Context => return crate::commands::agent::resource_context("reports", ctx),
        ReportCommands::List(args) => {
            let response = client
                .reports()
                .list(args.cursor.as_deref(), args.limit)
                .await?;
            emit_list_with_actions(
                ctx,
                "reports",
                operation,
                &response.reports,
                next_actions::list_result(
                    "reports",
                    response.reports.first().map(|report| report.id.as_str()),
                ),
            );
        }
        ReportCommands::Get(args) => {
            let report = client.reports().get(&args.report_id).await?;
            emit_one_with_actions(
                ctx,
                "reports",
                operation,
                &report,
                next_actions::item_result("reports", &report.id),
            );
        }
        ReportCommands::Create(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "name", args.name)?;
            input_json::insert(&mut input, "run_ids", args.run_ids)?;
            input_json::insert(&mut input, "compare_by", args.compare_by)?;
            input_json::insert(&mut input, "metadata_key", args.metadata_key)?;
            input_json::insert(&mut input, "permissions", args.permissions)?;
            validate_metadata_key(&input)?;
            let req: CreateReportRequest = input_json::finish(input)?;
            if req.run_ids.is_empty() {
                anyhow::bail!("--run-ids requires at least one run ID");
            }
            let report = client.reports().create(req).await?;
            emit_one_with_actions(
                ctx,
                "reports",
                operation,
                &report,
                next_actions::item_result("reports", &report.id),
            );
        }
        ReportCommands::Update(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "name", args.name)?;
            input_json::insert(&mut input, "run_ids", args.run_ids)?;
            input_json::insert(&mut input, "compare_by", args.compare_by)?;
            input_json::insert(&mut input, "metadata_key", args.metadata_key)?;
            input_json::insert(&mut input, "permissions", args.permissions)?;
            validate_metadata_key(&input)?;
            let req: UpdateReportRequest = input_json::finish(input)?;
            let report = client.reports().update(&args.report_id, req).await?;
            emit_one_with_actions(
                ctx,
                "reports",
                operation,
                &report,
                next_actions::item_result("reports", &report.id),
            );
        }
        ReportCommands::Delete(args) => {
            client.reports().delete(&args.report_id).await?;
            emit_success_with_actions(
                ctx,
                "reports",
                operation,
                "Report deleted.",
                next_actions::delete_result("reports"),
            );
        }
    }
    Ok(())
}

/// Validate the metadata_key / compare_by pairing before sending.
///
/// The API requires `metadata_key` when `compare_by` is metadata and rejects it
/// otherwise. Only enforced against the fields present in the assembled request,
/// so partial updates that touch neither field are untouched.
fn validate_metadata_key(input: &serde_json::Map<String, serde_json::Value>) -> Result<()> {
    let is_metadata =
        input.get("compare_by").and_then(serde_json::Value::as_str) == Some("metadata");
    let has_metadata_key = input.contains_key("metadata_key");

    if is_metadata && !has_metadata_key {
        anyhow::bail!("--metadata-key is required when --compare-by is metadata");
    }
    // Anything other than compare_by=metadata (including compare_by absent, where
    // the API defaults it to none) rejects a metadata_key, so guard it client-side.
    if !is_metadata && has_metadata_key {
        anyhow::bail!("--metadata-key can only be set when --compare-by is metadata");
    }
    Ok(())
}
