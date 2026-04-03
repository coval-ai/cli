use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Args, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};

use crate::client::error::ApiError;
use crate::client::models::ListParams;
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum SimulationCommands {
    List(ListArgs),
    Get(GetArgs),
    Delete(DeleteArgs),
    Audio(AudioArgs),
    Metrics(MetricsArgs),
    #[command(name = "metric-detail")]
    MetricDetail(MetricDetailArgs),
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(long)]
    filter: Option<String>,
    #[arg(long)]
    run_id: Option<String>,
    #[arg(long, default_value = "50")]
    page_size: u32,
    #[arg(long)]
    order_by: Option<String>,
    /// Query monitoring conversations instead of simulations
    #[arg(long)]
    monitoring: bool,
}

#[derive(Args)]
pub struct GetArgs {
    simulation_id: String,
    /// Query monitoring conversations instead of simulations
    #[arg(long)]
    monitoring: bool,
}

#[derive(Args)]
pub struct DeleteArgs {
    simulation_id: String,
    /// Query monitoring conversations instead of simulations
    #[arg(long)]
    monitoring: bool,
}

#[derive(Args)]
pub struct AudioArgs {
    simulation_id: String,
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Query monitoring conversations instead of simulations
    #[arg(long)]
    monitoring: bool,
}

#[derive(Args)]
pub struct MetricsArgs {
    simulation_id: String,
    /// Query monitoring conversations instead of simulations
    #[arg(long)]
    monitoring: bool,
}

#[derive(Args)]
pub struct MetricDetailArgs {
    simulation_id: String,
    metric_output_id: String,
    /// Query monitoring conversations instead of simulations
    #[arg(long)]
    monitoring: bool,
}

pub async fn execute(
    cmd: SimulationCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        SimulationCommands::List(args) => {
            let filter = match (args.filter, args.run_id) {
                (Some(f), Some(run_id)) => Some(format!("{f} AND run_id=\"{run_id}\"")),
                (Some(f), None) => Some(f),
                (None, Some(run_id)) => Some(format!("run_id=\"{run_id}\"")),
                (None, None) => None,
            };

            let params = ListParams {
                filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };

            if args.monitoring {
                let response = client.conversations().list(params).await?;
                print_list(&response.conversations, format);
            } else {
                let response = client.simulations().list(params).await?;
                print_list(&response.simulations, format);
            }
        }
        SimulationCommands::Get(args) => {
            if args.monitoring {
                let conversation = client.conversations().get(&args.simulation_id).await?;
                print_one(&conversation, format);
            } else {
                match client.simulations().get(&args.simulation_id).await {
                    Ok(simulation) => print_one(&simulation, format),
                    Err(ApiError::NotFound { .. }) => {
                        let conversation = client.conversations().get(&args.simulation_id).await?;
                        print_one(&conversation, format);
                    }
                    Err(e) => return Err(e.into()),
                }
            }
        }
        SimulationCommands::Delete(args) => {
            if args.monitoring {
                client.conversations().delete(&args.simulation_id).await?;
                print_success("Conversation deleted.");
            } else {
                match client.simulations().delete(&args.simulation_id).await {
                    Ok(()) => print_success("Simulation deleted."),
                    Err(ApiError::NotFound { .. }) => {
                        client.conversations().delete(&args.simulation_id).await?;
                        print_success("Conversation deleted.");
                    }
                    Err(e) => return Err(e.into()),
                }
            }
        }
        SimulationCommands::Metrics(args) => {
            if args.monitoring {
                let response = client
                    .conversations()
                    .list_metrics(&args.simulation_id)
                    .await?;
                print_list(&response.metrics, format);
            } else {
                match client.simulations().list_metrics(&args.simulation_id).await {
                    Ok(response) => print_list(&response.metrics, format),
                    Err(ApiError::NotFound { .. }) => {
                        let response = client
                            .conversations()
                            .list_metrics(&args.simulation_id)
                            .await?;
                        print_list(&response.metrics, format);
                    }
                    Err(e) => return Err(e.into()),
                }
            }
        }
        SimulationCommands::MetricDetail(args) => {
            if args.monitoring {
                let metric = client
                    .conversations()
                    .get_metric(&args.simulation_id, &args.metric_output_id)
                    .await?;
                print_one(&metric, format);
            } else {
                match client
                    .simulations()
                    .get_metric(&args.simulation_id, &args.metric_output_id)
                    .await
                {
                    Ok(metric) => print_one(&metric, format),
                    Err(ApiError::NotFound { .. }) => {
                        let metric = client
                            .conversations()
                            .get_metric(&args.simulation_id, &args.metric_output_id)
                            .await?;
                        print_one(&metric, format);
                    }
                    Err(e) => return Err(e.into()),
                }
            }
        }
        SimulationCommands::Audio(args) => {
            let audio = if args.monitoring {
                client.conversations().audio(&args.simulation_id).await?
            } else {
                match client.simulations().audio(&args.simulation_id).await {
                    Ok(audio) => audio,
                    Err(ApiError::NotFound { .. }) => {
                        client.conversations().audio(&args.simulation_id).await?
                    }
                    Err(e) => return Err(e.into()),
                }
            };

            match args.output {
                Some(path) => {
                    download_audio(&audio.audio_url, &path).await?;
                    print_success(&format!("Audio saved to {}", path.display()));
                }
                None => {
                    println!("{}", audio.audio_url);
                }
            }
        }
    }
    Ok(())
}

async fn download_audio(url: &str, path: &Path) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;

    if !resp.status().is_success() {
        anyhow::bail!("Failed to download audio: HTTP {}", resp.status());
    }

    let total = resp.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes}")?
            .progress_chars("=>-"),
    );

    let bytes = resp.bytes().await?;
    pb.set_position(bytes.len() as u64);
    pb.finish_and_clear();

    std::fs::write(path, &bytes)?;
    Ok(())
}
