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
    Resimulate(ResimulateArgs),
    Update(UpdateArgs),
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
    metric_output_id: String,
}

#[derive(Args)]
pub struct ResimulateArgs {
    simulation_id: String,
}

#[derive(Args)]
pub struct UpdateArgs {
    simulation_id: String,
    /// Set public visibility
    #[arg(long)]
    is_public: Option<bool>,
    /// Update notes (max 500 chars)
    #[arg(long)]
    notes: Option<String>,
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
            let response = client.simulations().list(params).await?;
            print_list(&response.simulations, format);
        }
        SimulationCommands::Get(args) => {
            let result = client.simulations().get(&args.simulation_id).await;
            match result {
                Ok(simulation) => print_one(&simulation, format),
                Err(ApiError::NotFound { .. }) => {
                    print_not_found_hint(&args.simulation_id, "conversations");
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
                Ok(()) => print_success("Simulation deleted."),
                Err(ApiError::NotFound { .. }) => {
                    print_not_found_hint(&args.simulation_id, "conversations");
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
            print_list(&response.metrics, format);
        }
        SimulationCommands::MetricDetail(args) => {
            let metric = client
                .simulations()
                .get_metric(&args.simulation_id, &args.metric_output_id)
                .await?;
            print_one(&metric, format);
        }
        SimulationCommands::Resimulate(args) => {
            let result = client
                .simulations()
                .resimulate(&args.simulation_id)
                .await?;
            print_one(&result, format);
        }
        SimulationCommands::Update(args) => {
            let mut body = serde_json::Map::new();
            if let Some(is_public) = args.is_public {
                body.insert("is_public".into(), serde_json::Value::Bool(is_public));
            }
            if let Some(notes) = args.notes {
                body.insert("notes".into(), serde_json::Value::String(notes));
            }
            let result: serde_json::Value = client
                .simulations()
                .update(&args.simulation_id, &serde_json::Value::Object(body))
                .await?;
            print_one(&result, format);
        }
        SimulationCommands::Audio(args) => {
            let audio = client.simulations().audio(&args.simulation_id).await?;

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

fn print_not_found_hint(id: &str, try_command: &str) {
    eprintln!("hint: not found as a simulation. Try `coval {try_command} get {id}` instead.");
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
