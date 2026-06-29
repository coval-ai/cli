use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{
    CreateMetricRequest, ListParams, MetricType, TestMetricRequest, UpdateMetricRequest,
};
use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, OutputContext,
};

#[derive(Subcommand)]
pub enum MetricCommands {
    Context,
    List(ListArgs),
    Get(GetArgs),
    Create(Box<CreateArgs>),
    Update(Box<UpdateArgs>),
    Delete(DeleteArgs),
    Test(TestArgs),
}

impl MetricCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::List(_) => "list",
            Self::Get(_) => "get",
            Self::Create(_) => "create",
            Self::Update(_) => "update",
            Self::Delete(_) => "delete",
            Self::Test(_) => "test",
        }
    }
}

#[derive(Args)]
pub struct ListArgs {
    /// Filter expression (supports metric_type, metric_name, create_time)
    #[arg(long)]
    filter: Option<String>,
    /// Results per page (1-100, default 50)
    #[arg(long, default_value = "50")]
    page_size: u32,
    /// Sort field, prefix with - for descending (default: -create_time)
    #[arg(long)]
    order_by: Option<String>,
    /// Include built-in metrics (e.g. Turn Count, Audio Duration)
    #[arg(long)]
    include_builtin: bool,
}

#[derive(Args)]
pub struct GetArgs {
    metric_id: String,
}

#[derive(Args)]
#[command(
    after_help = "Required fields by metric type:\n  llm-binary          --prompt\n  categorical         --prompt --categories\n  numerical           --prompt --min-value --max-value\n  audio-binary        --prompt\n  audio-categorical   --prompt --categories\n  audio-numerical     --prompt --min-value --max-value\n  toolcall            --prompt\n  metadata            --metadata-field-type --metadata-field-key\n  regex               --regex-pattern\n  pause               --min-pause-duration\n  composite           --criteria-source test_case --criteria-path expected_behaviors\n                      or --criteria-source metric_metadata --criteria \"a,b,c\""
)]
pub struct CreateArgs {
    #[command(flatten)]
    input_json: InputJsonArg,
    /// Metric name (1-200 characters)
    #[arg(long)]
    name: Option<String>,
    /// Metric description (1-1000 characters)
    #[arg(long)]
    description: Option<String>,
    /// Metric type (determines required fields, see below)
    #[arg(long, value_enum)]
    r#type: Option<MetricType>,
    /// LLM evaluation prompt
    #[arg(long)]
    prompt: Option<String>,
    /// Comma-separated category values (min 2, max 50)
    #[arg(long, value_delimiter = ',')]
    categories: Option<Vec<String>>,
    /// Minimum value for numerical metrics
    #[arg(long)]
    min_value: Option<f64>,
    /// Maximum value for numerical metrics
    #[arg(long)]
    max_value: Option<f64>,
    /// Regex pattern for transcript matching
    #[arg(long)]
    regex_pattern: Option<String>,
    /// Transcript role to evaluate (agent or user)
    #[arg(long)]
    role: Option<String>,
    #[arg(long)]
    match_mode: Option<String>,
    #[arg(long)]
    position: Option<String>,
    /// Case-insensitive regex matching
    #[arg(long)]
    case_insensitive: Option<bool>,
    /// Field type (STRING, NUMBER, or BOOLEAN)
    #[arg(long)]
    metadata_field_type: Option<String>,
    /// Metadata field key to extract
    #[arg(long)]
    metadata_field_key: Option<String>,
    /// Minimum pause duration in seconds (>= 0.5)
    #[arg(long)]
    min_pause_duration: Option<f64>,
    /// JSON string for pass/fail target condition
    #[arg(long)]
    target_condition: Option<String>,
    /// Composite: where per-criterion checks come from (test_case or metric_metadata)
    #[arg(long)]
    criteria_source: Option<String>,
    /// Composite: test-case field holding the criteria list (required when criteria-source=test_case, e.g. expected_behaviors)
    #[arg(long)]
    criteria_path: Option<String>,
    /// Composite: comma-separated static criteria (required when criteria-source=metric_metadata)
    #[arg(long, value_delimiter = ',')]
    criteria: Option<Vec<String>>,
    /// Composite: how to score (percentage_of_criteria_met, count_of_criteria_met, or all_criteria_met)
    #[arg(long)]
    reporting_method: Option<String>,
    /// Composite: optional base prompt template for the judge
    #[arg(long)]
    base_prompt_template: Option<String>,
    /// Composite: include traces in the evaluation
    #[arg(long)]
    include_traces: Option<bool>,
}

#[derive(Args)]
pub struct UpdateArgs {
    metric_id: String,
    #[command(flatten)]
    input_json: InputJsonArg,
    /// Metric name (1-200 characters)
    #[arg(long)]
    name: Option<String>,
    /// Metric description (1-1000 characters)
    #[arg(long)]
    description: Option<String>,
    /// Metric type
    #[arg(long, value_enum)]
    r#type: Option<MetricType>,
    /// LLM evaluation prompt
    #[arg(long)]
    prompt: Option<String>,
    /// Comma-separated category values (min 2, max 50)
    #[arg(long, value_delimiter = ',')]
    categories: Option<Vec<String>>,
    /// Minimum value for numerical metrics
    #[arg(long)]
    min_value: Option<f64>,
    /// Maximum value for numerical metrics
    #[arg(long)]
    max_value: Option<f64>,
    /// Field type (STRING, NUMBER, or BOOLEAN)
    #[arg(long)]
    metadata_field_type: Option<String>,
    /// Metadata field key to extract
    #[arg(long)]
    metadata_field_key: Option<String>,
    /// Regex pattern for transcript matching
    #[arg(long)]
    regex_pattern: Option<String>,
    /// Transcript role to evaluate (agent or user)
    #[arg(long)]
    role: Option<String>,
    #[arg(long)]
    match_mode: Option<String>,
    #[arg(long)]
    position: Option<String>,
    /// Case-insensitive regex matching
    #[arg(long)]
    case_insensitive: Option<bool>,
    /// Minimum pause duration in seconds (>= 0.5)
    #[arg(long)]
    min_pause_duration: Option<f64>,
    /// JSON string for pass/fail target condition
    #[arg(long)]
    target_condition: Option<String>,
    /// Composite: where per-criterion checks come from (test_case or metric_metadata)
    #[arg(long)]
    criteria_source: Option<String>,
    /// Composite: test-case field holding the criteria list (e.g. expected_behaviors)
    #[arg(long)]
    criteria_path: Option<String>,
    /// Composite: comma-separated static criteria (used when criteria-source=metric_metadata)
    #[arg(long, value_delimiter = ',')]
    criteria: Option<Vec<String>>,
    /// Composite: how to score (percentage_of_criteria_met, count_of_criteria_met, or all_criteria_met)
    #[arg(long)]
    reporting_method: Option<String>,
    /// Composite: optional base prompt template for the judge
    #[arg(long)]
    base_prompt_template: Option<String>,
    /// Composite: include traces in the evaluation
    #[arg(long)]
    include_traces: Option<bool>,
}

#[derive(Args)]
pub struct DeleteArgs {
    metric_id: String,
}

#[derive(Args)]
pub struct TestArgs {
    /// The metric ID to test
    metric_id: String,
    /// The simulation output ID to run the metric against
    #[arg(long)]
    simulation_output_id: String,
    /// Optional developer identifier for debugging
    #[arg(long)]
    dev_id: Option<String>,
}

pub async fn execute(cmd: MetricCommands, client: &CovalClient, ctx: &OutputContext) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        MetricCommands::Context => return crate::commands::agent::resource_context("metrics", ctx),
        MetricCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.metrics().list(params, args.include_builtin).await?;
            emit_list_with_actions(
                ctx,
                "metrics",
                operation,
                &response.metrics,
                next_actions::list_result(
                    "metrics",
                    response.metrics.first().map(|metric| metric.id.as_str()),
                ),
            );
        }
        MetricCommands::Get(args) => {
            let metric = client.metrics().get(&args.metric_id).await?;
            emit_one_with_actions(
                ctx,
                "metrics",
                operation,
                &metric,
                next_actions::item_result("metrics", &metric.id),
            );
        }
        MetricCommands::Create(args) => {
            let args = *args;
            let mut input = args.input_json.object()?;
            let target_condition: Option<serde_json::Value> = args
                .target_condition
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(|e| anyhow::anyhow!("Invalid JSON for --target-condition: {e}"))?;

            input_json::insert(&mut input, "metric_name", args.name)?;
            input_json::insert(&mut input, "description", args.description)?;
            input_json::insert(&mut input, "metric_type", args.r#type)?;
            input_json::insert(&mut input, "prompt", args.prompt)?;
            input_json::insert(&mut input, "categories", args.categories)?;
            input_json::insert(&mut input, "min_value", args.min_value)?;
            input_json::insert(&mut input, "max_value", args.max_value)?;
            input_json::insert(&mut input, "metadata_field_type", args.metadata_field_type)?;
            input_json::insert(&mut input, "metadata_field_key", args.metadata_field_key)?;
            input_json::insert(&mut input, "regex_pattern", args.regex_pattern)?;
            input_json::insert(&mut input, "role", args.role)?;
            input_json::insert(&mut input, "match_mode", args.match_mode)?;
            input_json::insert(&mut input, "position", args.position)?;
            input_json::insert(&mut input, "case_insensitive", args.case_insensitive)?;
            input_json::insert(
                &mut input,
                "min_pause_duration_seconds",
                args.min_pause_duration,
            )?;
            input_json::insert(&mut input, "target_condition", target_condition)?;
            input_json::insert(&mut input, "criteria_source", args.criteria_source)?;
            input_json::insert(&mut input, "criteria_path", args.criteria_path)?;
            input_json::insert(&mut input, "criteria", args.criteria)?;
            input_json::insert(&mut input, "reporting_method", args.reporting_method)?;
            input_json::insert(
                &mut input,
                "base_prompt_template",
                args.base_prompt_template,
            )?;
            input_json::insert(&mut input, "include_traces", args.include_traces)?;
            validate_composite(&input)?;
            let req: CreateMetricRequest = input_json::finish(input)?;
            let metric = client.metrics().create(req).await?;
            emit_one_with_actions(
                ctx,
                "metrics",
                operation,
                &metric,
                next_actions::item_result("metrics", &metric.id),
            );
        }
        MetricCommands::Update(args) => {
            let args = *args;
            let mut input = args.input_json.object()?;
            let target_condition: Option<serde_json::Value> = args
                .target_condition
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(|e| anyhow::anyhow!("Invalid JSON for --target-condition: {e}"))?;

            input_json::insert(&mut input, "metric_name", args.name)?;
            input_json::insert(&mut input, "description", args.description)?;
            input_json::insert(&mut input, "metric_type", args.r#type)?;
            input_json::insert(&mut input, "prompt", args.prompt)?;
            input_json::insert(&mut input, "categories", args.categories)?;
            input_json::insert(&mut input, "min_value", args.min_value)?;
            input_json::insert(&mut input, "max_value", args.max_value)?;
            input_json::insert(&mut input, "metadata_field_type", args.metadata_field_type)?;
            input_json::insert(&mut input, "metadata_field_key", args.metadata_field_key)?;
            input_json::insert(&mut input, "regex_pattern", args.regex_pattern)?;
            input_json::insert(&mut input, "role", args.role)?;
            input_json::insert(&mut input, "match_mode", args.match_mode)?;
            input_json::insert(&mut input, "position", args.position)?;
            input_json::insert(&mut input, "case_insensitive", args.case_insensitive)?;
            input_json::insert(
                &mut input,
                "min_pause_duration_seconds",
                args.min_pause_duration,
            )?;
            input_json::insert(&mut input, "target_condition", target_condition)?;
            input_json::insert(&mut input, "criteria_source", args.criteria_source)?;
            input_json::insert(&mut input, "criteria_path", args.criteria_path)?;
            input_json::insert(&mut input, "criteria", args.criteria)?;
            input_json::insert(&mut input, "reporting_method", args.reporting_method)?;
            input_json::insert(
                &mut input,
                "base_prompt_template",
                args.base_prompt_template,
            )?;
            input_json::insert(&mut input, "include_traces", args.include_traces)?;
            let req: UpdateMetricRequest = input_json::finish(input)?;
            let metric = client.metrics().update(&args.metric_id, req).await?;
            emit_one_with_actions(
                ctx,
                "metrics",
                operation,
                &metric,
                next_actions::item_result("metrics", &metric.id),
            );
        }
        MetricCommands::Delete(args) => {
            client.metrics().delete(&args.metric_id).await?;
            emit_success_with_actions(
                ctx,
                "metrics",
                operation,
                "Metric deleted.",
                next_actions::delete_result("metrics"),
            );
        }
        MetricCommands::Test(args) => {
            let req = TestMetricRequest {
                simulation_output_id: args.simulation_output_id,
                dev_id: args.dev_id,
            };
            let response = client.metrics().test(&args.metric_id, req).await?;
            emit_one_with_actions(
                ctx,
                "metrics",
                operation,
                &response,
                next_actions::item_result("metrics", &args.metric_id),
            );
        }
    }
    Ok(())
}

/// Validate the required composite-metric field combinations before sending.
///
/// Only enforced when the assembled request is a composite metric, so other
/// metric types and non-composite `--input-json` payloads are untouched.
fn validate_composite(input: &serde_json::Map<String, serde_json::Value>) -> Result<()> {
    let is_composite = input.get("metric_type").and_then(serde_json::Value::as_str)
        == Some("METRIC_COMPOSITE_EVALUATION");
    if !is_composite {
        return Ok(());
    }

    let source = input
        .get("criteria_source")
        .and_then(serde_json::Value::as_str);
    match source {
        Some("test_case") => {
            if input
                .get("criteria_path")
                .and_then(serde_json::Value::as_str)
                .is_none()
            {
                anyhow::bail!(
                    "--criteria-path is required when --criteria-source is test_case (e.g. expected_behaviors)"
                );
            }
        }
        Some("metric_metadata") => {
            let has_criteria = input
                .get("criteria")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|criteria| !criteria.is_empty());
            if !has_criteria {
                anyhow::bail!("--criteria is required when --criteria-source is metric_metadata");
            }
        }
        Some(other) => {
            anyhow::bail!("--criteria-source must be test_case or metric_metadata, got '{other}'");
        }
        None => {
            anyhow::bail!(
                "composite metrics require --criteria-source (test_case or metric_metadata)"
            );
        }
    }
    Ok(())
}
