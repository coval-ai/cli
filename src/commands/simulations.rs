use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Args, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};

use crate::client::error::ApiError;
use crate::client::models::{ListParams, MetricDetailResponse};
use crate::client::CovalClient;
use crate::output::{emit_list, emit_one, emit_success, OutputContext};

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

impl SimulationCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::List(_) => "list",
            Self::Get(_) => "get",
            Self::Delete(_) => "delete",
            Self::Audio(_) => "audio",
            Self::Metrics(_) => "metrics",
            Self::MetricDetail(_) => "metric-detail",
        }
    }
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
}

#[derive(Args)]
pub struct GetArgs {
    simulation_id: String,
}

#[derive(Args)]
pub struct DeleteArgs {
    simulation_id: String,
}

#[derive(Args)]
pub struct AudioArgs {
    simulation_id: String,
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Args)]
pub struct MetricsArgs {
    simulation_id: String,
}

#[derive(Args)]
pub struct MetricDetailArgs {
    simulation_id: String,
    metric_id: String,
}

pub async fn execute(
    cmd: SimulationCommands,
    client: &CovalClient,
    ctx: &OutputContext,
) -> Result<()> {
    let operation = cmd.operation();
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
            let response = client.simulations().list(params).await?;
            emit_list(ctx, "simulations", operation, &response.simulations);
        }
        SimulationCommands::Get(args) => {
            let result = client.simulations().get(&args.simulation_id).await;
            match result {
                Ok(simulation) => emit_one(ctx, "simulations", operation, &simulation),
                Err(ApiError::NotFound { .. }) => {
                    if !ctx.agent {
                        print_not_found_hint(&args.simulation_id, "conversations");
                    }
                    return Err(ApiError::NotFound {
                        resource: format!("Simulation '{}'", args.simulation_id),
                    }
                    .into());
                }
                Err(e) => return Err(e.into()),
            }
        }
        SimulationCommands::Delete(args) => {
            let result = client.simulations().delete(&args.simulation_id).await;
            match result {
                Ok(()) => emit_success(ctx, "simulations", operation, "Simulation deleted."),
                Err(ApiError::NotFound { .. }) => {
                    if !ctx.agent {
                        print_not_found_hint(&args.simulation_id, "conversations");
                    }
                    return Err(ApiError::NotFound {
                        resource: format!("Simulation '{}'", args.simulation_id),
                    }
                    .into());
                }
                Err(e) => return Err(e.into()),
            }
        }
        SimulationCommands::Metrics(args) => {
            let response = client
                .simulations()
                .list_metrics(&args.simulation_id)
                .await?;
            emit_list(ctx, "simulations", operation, &response.metrics);
        }
        SimulationCommands::MetricDetail(args) => {
            let response = client
                .simulations()
                .get_metric(&args.simulation_id, &args.metric_id)
                .await?;
            match response {
                MetricDetailResponse::Single { metric } => {
                    emit_one(ctx, "simulations", operation, &metric)
                }
                MetricDetailResponse::Collection { metric_outputs } => {
                    emit_list(ctx, "simulations", operation, &metric_outputs)
                }
            }
        }
        SimulationCommands::Audio(args) => {
            let audio = client.simulations().audio(&args.simulation_id).await?;

            match args.output {
                Some(path) => {
                    download_audio(&audio.audio_url, &path, !ctx.agent).await?;
                    emit_success(
                        ctx,
                        "simulations",
                        operation,
                        &format!("Audio saved to {}", path.display()),
                    );
                }
                None => {
                    if ctx.agent {
                        emit_one(ctx, "simulations", operation, &audio);
                    } else {
                        println!("{}", audio.audio_url);
                    }
                }
            }
        }
    }
    Ok(())
}

fn print_not_found_hint(id: &str, try_command: &str) {
    eprintln!("hint: not found as a simulation. Try `coval {try_command} get {id}` instead.");
}

async fn download_audio(url: &str, path: &Path, show_progress: bool) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;

    if !resp.status().is_success() {
        anyhow::bail!("Failed to download audio: HTTP {}", resp.status());
    }

    let total = resp.content_length().unwrap_or(0);
    let bytes = resp.bytes().await?;
    if show_progress {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes}")?
                .progress_chars("=>-"),
        );
        pb.set_position(bytes.len() as u64);
        pb.finish_and_clear();
    }

    std::fs::write(path, &bytes)?;
    Ok(())
}
