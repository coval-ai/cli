use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::CovalClient;
use crate::output::{print_one, OutputFormat};

#[derive(Subcommand)]
pub enum AudioCommands {
    #[command(name = "signed-url")]
    SignedUrl(SignedUrlArgs),
    #[command(name = "peaks-url")]
    PeaksUrl(PeaksUrlArgs),
}

#[derive(Args)]
pub struct SignedUrlArgs {
    /// Simulation output ID
    simulation_output_id: String,
}

#[derive(Args)]
pub struct PeaksUrlArgs {
    /// Simulation output ID
    simulation_output_id: String,
}

pub async fn execute(
    cmd: AudioCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        AudioCommands::SignedUrl(args) => {
            let response = client.audio().signed_url(&args.simulation_output_id).await?;
            print_one(&response, format);
        }
        AudioCommands::PeaksUrl(args) => {
            let response = client.audio().peaks_url(&args.simulation_output_id).await?;
            print_one(&response, format);
        }
    }
    Ok(())
}
