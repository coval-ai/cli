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
}

#[derive(Args)]
pub struct UpdateArgs {
    metric_id: String,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    prompt: Option<String>,
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
            let req = CreateMetricRequest {
                metric_name: args.name,
                description: args.description,
                metric_type: args.r#type,
                prompt: args.prompt,
                categories: args.categories,
                min_value: args.min_value,
                max_value: args.max_value,
                metadata_field_type: None,
                metadata_field_key: None,
                regex_pattern: None,
                role: None,
                min_pause_duration_seconds: None,
            };
            let metric = client.metrics().create(req).await?;
            print_one(&metric, format);
        }
        MetricCommands::Update(args) => {
            let req = UpdateMetricRequest {
                metric_name: args.name,
                description: args.description,
                prompt: args.prompt,
                ..Default::default()
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
