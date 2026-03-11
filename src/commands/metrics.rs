use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{CreateMetricRequest, ListParams, MetricType, UpdateMetricRequest};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum MetricCommands {
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
    /// Include built-in metrics (e.g. Turn Count, Audio Duration)
    #[arg(long)]
    include_builtin: bool,
}

#[derive(Args)]
pub struct GetArgs {
    metric_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    #[arg(long)]
    name: String,
    #[arg(long)]
    description: String,
    #[arg(long, value_enum)]
    r#type: MetricType,
    #[arg(long)]
    prompt: Option<String>,
    #[arg(long, value_delimiter = ',')]
    categories: Option<Vec<String>>,
    #[arg(long)]
    min_value: Option<f64>,
    #[arg(long)]
    max_value: Option<f64>,
    #[arg(long)]
    regex_pattern: Option<String>,
    #[arg(long)]
    role: Option<String>,
    #[arg(long)]
    match_mode: Option<String>,
    #[arg(long)]
    position: Option<String>,
    #[arg(long)]
    case_insensitive: Option<bool>,
    #[arg(long)]
    metadata_field_type: Option<String>,
    #[arg(long)]
    metadata_field_key: Option<String>,
    #[arg(long)]
    min_pause_duration: Option<f64>,
    /// JSON string for target condition
    #[arg(long)]
    target_condition: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    metric_id: String,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long, value_enum)]
    r#type: Option<MetricType>,
    #[arg(long)]
    prompt: Option<String>,
    #[arg(long, value_delimiter = ',')]
    categories: Option<Vec<String>>,
    #[arg(long)]
    min_value: Option<f64>,
    #[arg(long)]
    max_value: Option<f64>,
    #[arg(long)]
    metadata_field_type: Option<String>,
    #[arg(long)]
    metadata_field_key: Option<String>,
    #[arg(long)]
    regex_pattern: Option<String>,
    #[arg(long)]
    role: Option<String>,
    #[arg(long)]
    match_mode: Option<String>,
    #[arg(long)]
    position: Option<String>,
    #[arg(long)]
    case_insensitive: Option<bool>,
    #[arg(long)]
    min_pause_duration: Option<f64>,
    /// JSON string for target condition
    #[arg(long)]
    target_condition: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    metric_id: String,
}

pub async fn execute(
    cmd: MetricCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        MetricCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.metrics().list(params, args.include_builtin).await?;
            print_list(&response.metrics, format);
        }
        MetricCommands::Get(args) => {
            let metric = client.metrics().get(&args.metric_id).await?;
            print_one(&metric, format);
        }
        MetricCommands::Create(args) => {
            let target_condition = args
                .target_condition
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(|e| anyhow::anyhow!("Invalid JSON for --target-condition: {e}"))?;

            let req = CreateMetricRequest {
                metric_name: args.name,
                description: args.description,
                metric_type: args.r#type,
                prompt: args.prompt,
                categories: args.categories,
                min_value: args.min_value,
                max_value: args.max_value,
                metadata_field_type: args.metadata_field_type,
                metadata_field_key: args.metadata_field_key,
                regex_pattern: args.regex_pattern,
                role: args.role,
                match_mode: args.match_mode,
                position: args.position,
                case_insensitive: args.case_insensitive,
                min_pause_duration_seconds: args.min_pause_duration,
                target_condition,
            };
            let metric = client.metrics().create(req).await?;
            print_one(&metric, format);
        }
        MetricCommands::Update(args) => {
            let target_condition = args
                .target_condition
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(|e| anyhow::anyhow!("Invalid JSON for --target-condition: {e}"))?;

            let req = UpdateMetricRequest {
                metric_name: args.name,
                description: args.description,
                metric_type: args.r#type,
                prompt: args.prompt,
                categories: args.categories,
                min_value: args.min_value,
                max_value: args.max_value,
                metadata_field_type: args.metadata_field_type,
                metadata_field_key: args.metadata_field_key,
                regex_pattern: args.regex_pattern,
                role: args.role,
                match_mode: args.match_mode,
                position: args.position,
                case_insensitive: args.case_insensitive,
                min_pause_duration_seconds: args.min_pause_duration,
                target_condition,
            };
            let metric = client.metrics().update(&args.metric_id, req).await?;
            print_one(&metric, format);
        }
        MetricCommands::Delete(args) => {
            client.metrics().delete(&args.metric_id).await?;
            print_success("Metric deleted.");
        }
    }
    Ok(())
}
