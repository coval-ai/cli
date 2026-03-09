use std::time::Duration;

use anyhow::Result;
use clap::{Args, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};

use crate::client::models::{
    LaunchMetadata, LaunchOptions, LaunchRunRequest, ListParams, RunStatus,
};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum RunCommands {
    List(ListArgs),
    Get(GetArgs),
    Launch(LaunchArgs),
    Watch(WatchArgs),
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
    run_id: String,
}

#[derive(Args)]
pub struct LaunchArgs {
    #[arg(long)]
    agent_id: String,
    #[arg(long)]
    persona_id: String,
    #[arg(long)]
    test_set_id: String,
    #[arg(long)]
    iterations: Option<u32>,
    #[arg(long)]
    concurrency: Option<u32>,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    mutation_id: Option<String>,
    #[arg(long, value_delimiter = ',')]
    mutation_ids: Option<Vec<String>>,
}

#[derive(Args)]
pub struct WatchArgs {
    run_id: String,
    #[arg(long, default_value = "2")]
    interval: u64,
}

#[derive(Args)]
pub struct DeleteArgs {
    run_id: String,
}

pub async fn execute(cmd: RunCommands, client: &CovalClient, format: OutputFormat) -> Result<()> {
    match cmd {
        RunCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.runs().list(params).await?;
            print_list(&response.runs, format);
        }
        RunCommands::Get(args) => {
            let run = client.runs().get(&args.run_id).await?;
            print_one(&run, format);
        }
        RunCommands::Launch(args) => {
            let options = if args.iterations.is_some() || args.concurrency.is_some() {
                Some(LaunchOptions {
                    iteration_count: args.iterations,
                    concurrency: args.concurrency,
                    ..Default::default()
                })
            } else {
                None
            };

            let metadata = args.name.map(|name| LaunchMetadata {
                display_name: Some(name),
                ..Default::default()
            });

            let req = LaunchRunRequest {
                agent_id: args.agent_id,
                persona_id: args.persona_id,
                test_set_id: args.test_set_id,
                mutation_id: args.mutation_id,
                mutation_ids: args.mutation_ids,
                persona_metrics: None,
                options,
                metadata,
                metric_ids: None,
            };
            let run = client.runs().launch(req).await?;
            print_one(&run, format);
        }
        RunCommands::Watch(args) => {
            watch_run(client, &args.run_id, args.interval).await?;
        }
        RunCommands::Delete(args) => {
            client.runs().delete(&args.run_id).await?;
            print_success("Run deleted.");
        }
    }
    Ok(())
}

#[allow(clippy::cast_sign_loss)]
async fn watch_run(client: &CovalClient, run_id: &str, interval_secs: u64) -> Result<()> {
    let run = client.runs().get(run_id).await?;
    let total = run
        .progress
        .as_ref()
        .map_or(100, |p| p.total_test_cases as u64);

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")?
            .progress_chars("=>-"),
    );

    loop {
        let run = client.runs().get(run_id).await?;

        if let Some(progress) = &run.progress {
            pb.set_length(progress.total_test_cases as u64);
            pb.set_position(progress.completed_test_cases as u64);
        }

        pb.set_message(run.status.to_string());

        match run.status {
            RunStatus::Completed => {
                pb.finish_with_message("COMPLETED");
                println!("\nRun completed successfully.");
                if let Some(results) = &run.results {
                    println!("Simulations: {}", results.output_ids.len());
                }
                break;
            }
            RunStatus::Failed => {
                pb.finish_with_message("FAILED");
                let msg = run.error.unwrap_or_else(|| "Unknown error".to_string());
                anyhow::bail!("Run failed: {msg}");
            }
            RunStatus::Cancelled => {
                pb.finish_with_message("CANCELLED");
                println!("\nRun was cancelled.");
                break;
            }
            _ => {
                tokio::time::sleep(Duration::from_secs(interval_secs)).await;
            }
        }
    }

    Ok(())
}
