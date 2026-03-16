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
    /// Filter expression (supports status, agent_id, persona_id, test_set_id, create_time, tag)
    #[arg(long)]
    filter: Option<String>,
    /// Results per page (1-1000, default 50)
    #[arg(long, default_value = "50")]
    page_size: u32,
    /// Sort field, prefix with - for descending (default: -create_time)
    #[arg(long)]
    order_by: Option<String>,
}

#[derive(Args)]
pub struct GetArgs {
    run_id: String,
}

#[derive(Args)]
pub struct LaunchArgs {
    /// Agent to evaluate (22-char ID)
    #[arg(long)]
    agent_id: String,
    /// Simulated persona for the run (22-char ID)
    #[arg(long)]
    persona_id: String,
    /// Test cases to run against (8-char ID)
    #[arg(long)]
    test_set_id: String,
    /// Comma-separated metric IDs; defaults to agent's metrics
    #[arg(long, value_delimiter = ',')]
    metric_ids: Option<Vec<String>>,
    /// Times to repeat each test case (1-50, default 1)
    #[arg(long)]
    iterations: Option<u32>,
    /// Parallel simulations (1-100, default 1)
    #[arg(long)]
    concurrency: Option<u32>,
    /// Random subset of test cases to run (0 = all)
    #[arg(long)]
    sub_sample_size: Option<u32>,
    /// Seed for reproducible sub-sampling
    #[arg(long)]
    sub_sample_seed: Option<u64>,
    /// Display name for the run
    #[arg(long)]
    name: Option<String>,
    /// Single mutation to apply (26-char ULID)
    #[arg(long)]
    mutation_id: Option<String>,
    /// Comma-separated mutation IDs for multi-mutation runs
    #[arg(long, value_delimiter = ',')]
    mutation_ids: Option<Vec<String>>,
    /// Comma-separated tags for categorizing the run
    #[arg(long, value_delimiter = ',')]
    tags: Option<Vec<String>>,
}

#[derive(Args)]
pub struct WatchArgs {
    run_id: String,
    /// Polling interval in seconds
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
            let options = if args.iterations.is_some()
                || args.concurrency.is_some()
                || args.sub_sample_size.is_some()
                || args.sub_sample_seed.is_some()
            {
                Some(LaunchOptions {
                    iteration_count: args.iterations,
                    concurrency: args.concurrency,
                    sub_sample_size: args.sub_sample_size,
                    sub_sample_seed: args.sub_sample_seed,
                })
            } else {
                None
            };

            let metadata = if args.name.is_some() || args.tags.is_some() {
                Some(LaunchMetadata {
                    display_name: args.name,
                    tags: args.tags,
                    ..Default::default()
                })
            } else {
                None
            };

            let req = LaunchRunRequest {
                agent_id: args.agent_id,
                persona_id: args.persona_id,
                test_set_id: args.test_set_id,
                metric_ids: args.metric_ids,
                mutation_id: args.mutation_id,
                mutation_ids: args.mutation_ids,
                persona_metrics: None,
                options,
                metadata,
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
