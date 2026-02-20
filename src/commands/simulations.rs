use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Args, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};

use crate::client::models::ListParams;
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum SimulationCommands {
    List(ListArgs),
    Get(GetArgs),
    Delete(DeleteArgs),
    Audio(AudioArgs),
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
            let simulation = client.simulations().get(&args.simulation_id).await?;
            print_one(&simulation, format);
        }
        SimulationCommands::Delete(args) => {
            client.simulations().delete(&args.simulation_id).await?;
            print_success("Simulation deleted.");
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
