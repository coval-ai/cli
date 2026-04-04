use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Args, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};

use crate::client::error::ApiError;
use crate::client::models::ListParams;
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum ConversationCommands {
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
    #[arg(long, default_value = "50")]
    page_size: u32,
    #[arg(long)]
    order_by: Option<String>,
}

#[derive(Args)]
pub struct GetArgs {
    conversation_id: String,
}

#[derive(Args)]
pub struct DeleteArgs {
    conversation_id: String,
}

#[derive(Args)]
pub struct AudioArgs {
    conversation_id: String,
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Args)]
pub struct MetricsArgs {
    conversation_id: String,
}

#[derive(Args)]
pub struct MetricDetailArgs {
    conversation_id: String,
    metric_output_id: String,
}

pub async fn execute(
    cmd: ConversationCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        ConversationCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.conversations().list(params).await?;
            print_list(&response.conversations, format);
        }
        ConversationCommands::Get(args) => {
            let result = client.conversations().get(&args.conversation_id).await;
            match result {
                Ok(conversation) => print_one(&conversation, format),
                Err(ApiError::NotFound { .. }) => {
                    print_not_found_hint(&args.conversation_id, "simulations");
                    return Err(ApiError::NotFound {
                        resource: format!("Conversation '{}'", args.conversation_id),
                    }
                    .into());
                }
                Err(e) => return Err(e.into()),
            }
        }
        ConversationCommands::Delete(args) => {
            let result = client.conversations().delete(&args.conversation_id).await;
            match result {
                Ok(()) => print_success("Conversation deleted."),
                Err(ApiError::NotFound { .. }) => {
                    print_not_found_hint(&args.conversation_id, "simulations");
                    return Err(ApiError::NotFound {
                        resource: format!("Conversation '{}'", args.conversation_id),
                    }
                    .into());
                }
                Err(e) => return Err(e.into()),
            }
        }
        ConversationCommands::Metrics(args) => {
            let response = client
                .conversations()
                .list_metrics(&args.conversation_id)
                .await?;
            print_list(&response.metrics, format);
        }
        ConversationCommands::MetricDetail(args) => {
            let metric = client
                .conversations()
                .get_metric(&args.conversation_id, &args.metric_output_id)
                .await?;
            print_one(&metric, format);
        }
        ConversationCommands::Audio(args) => {
            let audio = client.conversations().audio(&args.conversation_id).await?;

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
    eprintln!("hint: not found as a conversation. Try `coval {try_command} get {id}` instead.");
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
