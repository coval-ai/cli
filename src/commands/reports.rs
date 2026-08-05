use std::collections::HashSet;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{
    CompareBy, CreateReportRequest, ReportCustomDimension, ReportCustomDimensionGroup,
    ReportPermission, ReportViewMode, UpdateReportRequest,
};
use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, print_list,
    OutputContext, OutputFormat,
};

/// A merged report carries exactly one dimension, so a fixed ID is unambiguous.
const MERGE_DIMENSION_ID: &str = "merged-reports";
const MERGE_DIMENSION_NAME: &str = "Report";
const MERGE_ROWS_PAGE_SIZE: u32 = 500;
const MERGE_MAX_PAGES_PER_REPORT: usize = 200;
/// Mirrors the API's per-group `simulation_ids` and per-dimension `groups` ceilings.
/// Checked client-side so an oversized merge fails before paging the whole source.
const MERGE_MAX_SIMULATIONS_PER_GROUP: usize = 10_000;
const MERGE_MAX_SOURCE_REPORTS: usize = 500;

#[derive(Subcommand)]
pub enum ReportCommands {
    Context,
    List(ListArgs),
    Get(GetArgs),
    Rows(RowsArgs),
    Create(CreateArgs),
    Merge(MergeArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

impl ReportCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::List(_) => "list",
            Self::Get(_) => "get",
            Self::Rows(_) => "rows",
            Self::Create(_) => "create",
            Self::Merge(_) => "merge",
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
pub struct RowsArgs {
    /// Report ID (26-character ULID)
    report_id: String,
    /// Opaque cursor from a previous response's next_page_token
    #[arg(long)]
    cursor: Option<String>,
    /// Rows per page (1-2000, default 2000)
    #[arg(long)]
    limit: Option<u32>,
    /// Comma-separated metric IDs to include
    #[arg(long, value_delimiter = ',')]
    metric_ids: Option<Vec<String>>,
    /// Comma-separated simulation IDs to restrict the page to
    #[arg(long, value_delimiter = ',')]
    simulation_ids: Option<Vec<String>>,
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
    /// Report layout (default rows)
    #[arg(long, value_enum)]
    view_mode: Option<ReportViewMode>,
    /// Report visibility (default PRIVATE)
    #[arg(long, value_enum)]
    permissions: Option<ReportPermission>,
}

#[derive(Args)]
pub struct MergeArgs {
    /// Comma-separated IDs of the reports to merge (min 2, must be distinct)
    #[arg(long, required = true, value_delimiter = ',')]
    report_ids: Vec<String>,
    /// Display name for the merged report (1-200 characters)
    #[arg(long)]
    name: String,
    /// Label for the generated grouping dimension (default "Report")
    #[arg(long, default_value = MERGE_DIMENSION_NAME)]
    dimension_name: String,
    /// Merged report visibility (default PRIVATE)
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
        ReportCommands::Rows(args) => {
            let response = client
                .reports()
                .rows(
                    &args.report_id,
                    args.cursor.as_deref(),
                    args.limit,
                    args.metric_ids.map(|ids| ids.join(",")).as_deref(),
                    args.simulation_ids.map(|ids| ids.join(",")).as_deref(),
                )
                .await?;
            let actions = next_actions::item_result("reports", &args.report_id);
            if ctx.human() {
                print_list(&response.rows, OutputFormat::Table);
                if let Some(cursor) = &response.next_page_token {
                    println!("Next cursor: {cursor}");
                }
            } else {
                emit_one_with_actions(ctx, "reports", operation, &response, actions);
            }
        }
        ReportCommands::Create(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "name", args.name)?;
            input_json::insert(&mut input, "run_ids", args.run_ids)?;
            input_json::insert(&mut input, "compare_by", args.compare_by)?;
            input_json::insert(&mut input, "metadata_key", args.metadata_key)?;
            input_json::insert(&mut input, "view_mode", args.view_mode)?;
            input_json::insert(&mut input, "permissions", args.permissions)?;
            validate_metadata_key(&input)?;
            validate_custom_dimensions(&input)?;
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
        ReportCommands::Merge(args) => {
            let report = merge_reports(args, client).await?;
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

/// Build one report grouping the source reports' simulations, one group per source.
///
/// Mirrors the app's "Merge reports" action: a simulation is attributed to the first
/// selected report that contains it, so overlapping reports do not double-count.
async fn merge_reports(
    args: MergeArgs,
    client: &CovalClient,
) -> Result<crate::client::models::Report> {
    let mut requested = HashSet::new();
    for report_id in &args.report_ids {
        if !requested.insert(report_id.as_str()) {
            anyhow::bail!("--report-ids contains {report_id} twice; ids must be distinct");
        }
    }
    if args.report_ids.len() < 2 {
        anyhow::bail!("--report-ids requires at least two report IDs to merge");
    }
    if args.report_ids.len() > MERGE_MAX_SOURCE_REPORTS {
        anyhow::bail!(
            "--report-ids has {} reports; a merged report holds at most {MERGE_MAX_SOURCE_REPORTS} groups",
            args.report_ids.len()
        );
    }

    let mut seen_simulation_ids = HashSet::new();
    let mut seen_run_ids = HashSet::new();
    let mut run_ids: Vec<String> = Vec::new();
    let mut groups: Vec<ReportCustomDimensionGroup> = Vec::new();

    for report_id in &args.report_ids {
        let source = client.reports().get(report_id).await?;
        for run_id in &source.run_ids {
            if seen_run_ids.insert(run_id.clone()) {
                run_ids.push(run_id.clone());
            }
        }

        let mut simulation_ids: Vec<String> = Vec::new();
        let mut cursor: Option<String> = None;
        let mut drained = false;
        for _ in 0..MERGE_MAX_PAGES_PER_REPORT {
            let page = client
                .reports()
                .rows(
                    report_id,
                    cursor.as_deref(),
                    Some(MERGE_ROWS_PAGE_SIZE),
                    None,
                    None,
                )
                .await?;
            for row in page.rows {
                if seen_simulation_ids.insert(row.simulation_id.clone()) {
                    simulation_ids.push(row.simulation_id);
                }
            }
            // Checked per page so an oversized source stops here instead of paging to the
            // ceiling and then having the create rejected.
            if simulation_ids.len() > MERGE_MAX_SIMULATIONS_PER_GROUP {
                anyhow::bail!(
                    "report {report_id} contributes more than {MERGE_MAX_SIMULATIONS_PER_GROUP} \
                     simulations; a merged report's group cannot hold more than that"
                );
            }
            match page.next_page_token {
                Some(token) => cursor = Some(token),
                None => {
                    drained = true;
                    break;
                }
            }
        }
        if !drained {
            anyhow::bail!(
                "report {report_id} has more than {} rows; merge cannot page past that",
                MERGE_MAX_PAGES_PER_REPORT as u32 * MERGE_ROWS_PAGE_SIZE
            );
        }

        let name = source.name.trim();
        groups.push(ReportCustomDimensionGroup {
            id: source.id.clone(),
            name: if name.is_empty() {
                "Unnamed report".to_string()
            } else {
                name.to_string()
            },
            simulation_ids,
        });
    }

    if run_ids.is_empty() {
        anyhow::bail!(
            "the selected reports have no runs to merge; a merged report needs at least one run"
        );
    }

    let request = CreateReportRequest {
        name: args.name,
        run_ids,
        compare_by: Some(CompareBy::Custom),
        metadata_key: None,
        custom_dimensions: Some(vec![ReportCustomDimension {
            id: MERGE_DIMENSION_ID.to_string(),
            name: args.dimension_name,
            groups,
            hide_unassigned: false,
        }]),
        custom_dimension_id: Some(MERGE_DIMENSION_ID.to_string()),
        view_mode: Some(ReportViewMode::Grouped),
        permissions: args.permissions,
    };

    Ok(client.reports().create(request).await?)
}

/// Validate the custom_dimensions / custom_dimension_id / compare_by pairing before sending.
///
/// The API requires `custom_dimensions` when `compare_by` is custom and rejects both it and
/// `custom_dimension_id` otherwise. Only `--input-json` can carry them on `reports create`;
/// `reports merge` assembles them itself.
fn validate_custom_dimensions(input: &serde_json::Map<String, serde_json::Value>) -> Result<()> {
    let is_custom = input.get("compare_by").and_then(serde_json::Value::as_str) == Some("custom");
    // An explicit null deserializes to None, so it counts as absent rather than as a value.
    let custom_dimensions = input
        .get("custom_dimensions")
        .filter(|value| !value.is_null());
    let custom_dimension_id = input
        .get("custom_dimension_id")
        .filter(|value| !value.is_null());

    if !is_custom {
        if custom_dimensions.is_some() {
            anyhow::bail!("custom_dimensions can only be set when --compare-by is custom");
        }
        if custom_dimension_id.is_some() {
            anyhow::bail!("custom_dimension_id can only be set when --compare-by is custom");
        }
        return Ok(());
    }

    let Some(custom_dimensions) = custom_dimensions else {
        anyhow::bail!(
            "custom_dimensions is required when --compare-by is custom; use `coval reports merge` \
             to build them from existing reports"
        );
    };
    validate_custom_dimension_id_target(custom_dimensions, custom_dimension_id)
}

/// Check that a supplied custom_dimension_id names one of the supplied dimensions.
///
/// The API defaults the grouping to the first dimension, so an absent ID is valid. Shapes
/// serde will reject anyway are passed through so the type error survives this check.
fn validate_custom_dimension_id_target(
    custom_dimensions: &serde_json::Value,
    custom_dimension_id: Option<&serde_json::Value>,
) -> Result<()> {
    let (Some(dimension_id), Some(dimensions)) = (
        custom_dimension_id.and_then(serde_json::Value::as_str),
        custom_dimensions.as_array(),
    ) else {
        return Ok(());
    };
    let names_a_dimension = dimensions.iter().any(|dimension| {
        dimension.get("id").and_then(serde_json::Value::as_str) == Some(dimension_id)
    });
    if !names_a_dimension {
        anyhow::bail!(
            "custom_dimension_id {dimension_id} does not match the id of any supplied custom dimension"
        );
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
