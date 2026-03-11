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
    /// Filter expression (supports metric_type, metric_name, create_time)
    #[arg(long)]
    filter: Option<String>,
    /// Results per page (1-100, default 50)
    #[arg(long, default_value = "50")]
    page_size: u32,
    /// Sort field, prefix with - for descending (default: -create_time)
    #[arg(long)]
    order_by: Option<String>,
}

#[derive(Args)]
pub struct GetArgs {
    metric_id: String,
}

#[derive(Args)]
#[command(
    after_help = "Required fields by metric type:\n  llm-binary          --prompt\n  categorical         --prompt --categories\n  numerical           --prompt --min-value --max-value\n  audio-binary        --prompt\n  audio-categorical   --prompt --categories\n  audio-numerical     --prompt --min-value --max-value\n  toolcall            --prompt\n  metadata            --metadata-field-type --metadata-field-key\n  regex               --regex-pattern\n  pause               --min-pause-duration"
)]
pub struct CreateArgs {
    /// Metric name (1-200 characters)
    #[arg(long)]
    name: String,
    /// Metric description (1-1000 characters)
    #[arg(long)]
    description: String,
    /// Metric type (determines required fields, see below)
    #[arg(long, value_enum)]
    r#type: MetricType,
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
}

#[derive(Args)]
pub struct UpdateArgs {
    metric_id: String,
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
            let response = client.metrics().list(params).await?;
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
