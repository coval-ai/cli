use std::time::Duration;

use anyhow::Result;
use clap::{Args, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};

use crate::client::models::{
    LaunchMetadata, LaunchOptions, LaunchRunRequest, ListParams, Run, RunStatus, UpdateRunRequest,
};
use crate::client::CovalClient;
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, NextAction,
    OutputContext,
};

#[derive(Subcommand)]
pub enum RunCommands {
    Context,
    List(ListArgs),
    Get(GetArgs),
    Launch(LaunchArgs),
    Update(UpdateArgs),
    Watch(WatchArgs),
    Delete(DeleteArgs),
}

impl RunCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::List(_) => "list",
            Self::Get(_) => "get",
            Self::Launch(_) => "launch",
            Self::Update(_) => "update",
            Self::Watch(_) => "watch",
            Self::Delete(_) => "delete",
        }
    }
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
    /// Specific test case IDs to run from the test set (1-100, comma-separated)
    #[arg(
        long,
        value_delimiter = ',',
        value_parser = clap::builder::NonEmptyStringValueParser::new(),
    )]
    test_cases: Option<Vec<String>>,
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
pub struct UpdateArgs {
    run_id: String,
    /// Comma-separated tags; replaces existing tags. Pass an empty string to clear.
    #[arg(long, value_delimiter = ',', required = true)]
    tags: Vec<String>,
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

pub async fn execute(cmd: RunCommands, client: &CovalClient, ctx: &OutputContext) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        RunCommands::Context => return crate::commands::agent::resource_context("runs", ctx),
        RunCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.runs().list(params).await?;
            emit_list_with_actions(
                ctx,
                "runs",
                operation,
                &response.runs,
                list_actions(response.runs.first().map(|run| run.run_id.as_str())),
            );
        }
        RunCommands::Get(args) => {
            let run = client.runs().get(&args.run_id).await?;
            emit_one_with_actions(ctx, "runs", operation, &run, run_actions(&run));
        }
        RunCommands::Launch(args) => {
            if let Some(ref ids) = args.test_cases {
                if ids.len() > 100 {
                    anyhow::bail!("--test-cases accepts 1-100 values, got {}", ids.len());
                }
            }
            let options = if args.iterations.is_some()
                || args.concurrency.is_some()
                || args.sub_sample_size.is_some()
                || args.sub_sample_seed.is_some()
                || args.test_cases.is_some()
            {
                Some(LaunchOptions {
                    iteration_count: args.iterations,
                    concurrency: args.concurrency,
                    sub_sample_size: args.sub_sample_size,
                    sub_sample_seed: args.sub_sample_seed,
                    test_case_ids: args.test_cases,
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
            emit_one_with_actions(ctx, "runs", operation, &run, run_actions(&run));
        }
        RunCommands::Update(args) => {
            let tags = args
                .tags
                .into_iter()
                .filter(|t| !t.trim().is_empty())
                .collect();
            let run = client
                .runs()
                .update(&args.run_id, UpdateRunRequest { tags })
                .await?;
            emit_one_with_actions(ctx, "runs", operation, &run, run_actions(&run));
        }
        RunCommands::Watch(args) => {
            watch_run(client, &args.run_id, args.interval, ctx).await?;
        }
        RunCommands::Delete(args) => {
            client.runs().delete(&args.run_id).await?;
            emit_success_with_actions(
                ctx,
                "runs",
                operation,
                "Run deleted.",
                vec![
                    next_actions::list("runs").primary(),
                    next_actions::context("runs"),
                ],
            );
        }
    }
    Ok(())
}

#[allow(clippy::cast_sign_loss)]
async fn watch_run(
    client: &CovalClient,
    run_id: &str,
    interval_secs: u64,
    ctx: &OutputContext,
) -> Result<()> {
    if ctx.agent {
        loop {
            let run = client.runs().get(run_id).await?;
            match run.status {
                RunStatus::Completed | RunStatus::Cancelled => {
                    emit_one_with_actions(ctx, "runs", "watch", &run, run_actions(&run));
                    break;
                }
                RunStatus::Failed => {
                    let msg = run.error.unwrap_or_else(|| "Unknown error".to_string());
                    anyhow::bail!("Run failed: {msg}");
                }
                _ => {
                    tokio::time::sleep(Duration::from_secs(interval_secs)).await;
                }
            }
        }

        return Ok(());
    }

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

fn list_actions(id: Option<&str>) -> Vec<NextAction> {
    let mut actions = vec![next_actions::context("runs")];
    if let Some(id) = id {
        actions.insert(0, next_actions::get("runs", id).primary());
    }
    actions
}

fn run_actions(run: &Run) -> Vec<NextAction> {
    let mut actions = Vec::new();
    let terminal = matches!(
        run.status,
        RunStatus::Completed | RunStatus::Failed | RunStatus::Cancelled | RunStatus::Deleted
    );
    if terminal {
        actions.push(next_actions::get("runs", &run.run_id).primary());
    } else {
        actions.push(next_actions::runs_watch(&run.run_id));
        actions.push(next_actions::get("runs", &run.run_id));
    }
    actions.push(next_actions::simulations_for_run(&run.run_id));
    actions.push(next_actions::context("runs"));
    actions
}
