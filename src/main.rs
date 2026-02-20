mod cli;
mod client;
mod commands;
mod config;
mod output;

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    cli::run(args).await
}
