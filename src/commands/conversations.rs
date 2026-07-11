use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::{DateTime, Utc};
use clap::{Args, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};

use crate::client::error::ApiError;
use crate::client::models::{ListParams, MetricDetailResponse, SubmitConversationRequest};
use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, NextAction,
    OutputContext,
};

#[derive(Subcommand)]
pub enum ConversationCommands {
    Context,
    List(ListArgs),
    Get(GetArgs),
    Delete(DeleteArgs),
    Audio(AudioArgs),
    Submit(SubmitArgs),
    Metrics(MetricsArgs),
    #[command(name = "metric-detail")]
    MetricDetail(MetricDetailArgs),
    Patch(PatchArgs),
}

impl ConversationCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::List(_) => "list",
            Self::Get(_) => "get",
            Self::Delete(_) => "delete",
            Self::Audio(_) => "audio",
            Self::Submit(_) => "submit",
            Self::Metrics(_) => "metrics",
            Self::MetricDetail(_) => "metric-detail",
            Self::Patch(_) => "patch",
        }
    }
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(long)]
    filter: Option<String>,
    #[arg(long, default_value = "50")]
    page_size: u32,
    #[arg(long)]
    order_by: Option<String>,
    /// Include full outputs for this metric on every conversation in the page
    #[arg(long, value_name = "METRIC_ID")]
    include_metric_outputs: Option<String>,
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
pub struct SubmitArgs {
    #[command(flatten)]
    input_json: InputJsonArg,
    /// Path to JSON file containing the transcript (array of message objects)
    #[arg(long)]
    transcript_file: Option<PathBuf>,
    /// Path to local audio file; will be base64-encoded into the request body
    #[arg(long)]
    audio_file: Option<PathBuf>,
    /// Presigned URL to audio (S3, GCS, Azure Blob, or any HTTPS URL)
    #[arg(long)]
    audio_url: Option<String>,
    /// Reference to a prior `POST /v1/audio:upload` (`upl_<26-char ULID>`)
    #[arg(long)]
    upload_id: Option<String>,
    /// Metric ID to evaluate (repeat for multiple). Defaults to org's monitoring metrics.
    #[arg(long = "metric")]
    metrics: Vec<String>,
    /// Custom metadata as `key=value` (repeat for multiple)
    #[arg(long = "metadata", value_parser = parse_kv)]
    metadata: Vec<(String, String)>,
    /// External conversation ID from your system
    #[arg(long)]
    external_id: Option<String>,
    /// Agent ID to associate with this conversation (22-char ID)
    #[arg(long)]
    agent_id: Option<String>,
    /// When the conversation occurred (ISO 8601, e.g. 2026-05-05T12:34:56Z)
    #[arg(long)]
    occurred_at: Option<DateTime<Utc>>,
}

fn parse_kv(raw: &str) -> Result<(String, String), String> {
    let (key, value) = raw
        .split_once('=')
        .ok_or_else(|| format!("expected key=value, got '{raw}'"))?;
    if key.is_empty() {
        return Err(format!("metadata key is empty in '{raw}'"));
    }
    Ok((key.to_string(), value.to_string()))
}

#[derive(Args)]
pub struct MetricsArgs {
    conversation_id: String,
}

#[derive(Args)]
pub struct MetricDetailArgs {
    conversation_id: String,
    metric_id: String,
}

#[derive(Args)]
pub struct PatchArgs {
    conversation_id: String,
    #[arg(long)]
    audio_url: Option<String>,
    #[arg(long)]
    audio_file: Option<PathBuf>,
}

pub async fn execute(
    cmd: ConversationCommands,
    client: &CovalClient,
    ctx: &OutputContext,
) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        ConversationCommands::Context => {
            return crate::commands::agent::resource_context("conversations", ctx);
        }
        ConversationCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = match args.include_metric_outputs {
                Some(metric_id) => {
                    client
                        .conversations()
                        .list_with_metric_outputs(params, &metric_id)
                        .await?
                }
                None => client.conversations().list(params).await?,
            };
            emit_list_with_actions(
                ctx,
                "conversations",
                operation,
                &response.conversations,
                list_actions(
                    response
                        .conversations
                        .first()
                        .map(|conversation| conversation.conversation_id.as_str()),
                ),
            );
        }
        ConversationCommands::Get(args) => {
            let result = client.conversations().get(&args.conversation_id).await;
            match result {
                Ok(conversation) => emit_one_with_actions(
                    ctx,
                    "conversations",
                    operation,
                    &conversation,
                    conversation_actions(&conversation.conversation_id),
                ),
                Err(ApiError::NotFound { .. }) => {
                    if !ctx.agent {
                        print_not_found_hint(&args.conversation_id, "simulations");
                    }
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
                Ok(()) => emit_success_with_actions(
                    ctx,
                    "conversations",
                    operation,
                    "Conversation deleted.",
                    vec![
                        next_actions::list("conversations").primary(),
                        next_actions::context("conversations"),
                    ],
                ),
                Err(ApiError::NotFound { .. }) => {
                    if !ctx.agent {
                        print_not_found_hint(&args.conversation_id, "simulations");
                    }
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
            emit_list_with_actions(
                ctx,
                "conversations",
                operation,
                &response.metrics,
                vec![
                    next_actions::get("conversations", &args.conversation_id).primary(),
                    next_actions::context("conversations"),
                ],
            );
        }
        ConversationCommands::MetricDetail(args) => {
            let response = client
                .conversations()
                .get_metric(&args.conversation_id, &args.metric_id)
                .await?;
            match response {
                MetricDetailResponse::Single { metric } => emit_one_with_actions(
                    ctx,
                    "conversations",
                    operation,
                    &metric,
                    vec![next_actions::get("conversations", &args.conversation_id).primary()],
                ),
                MetricDetailResponse::Collection { metric_outputs } => emit_list_with_actions(
                    ctx,
                    "conversations",
                    operation,
                    &metric_outputs,
                    vec![next_actions::get("conversations", &args.conversation_id).primary()],
                ),
            }
        }
        ConversationCommands::Submit(args) => {
            let req = build_submit_request(args)?;
            let conversation = client.conversations().submit(req).await?;
            emit_one_with_actions(
                ctx,
                "conversations",
                operation,
                &conversation,
                conversation_actions(&conversation.conversation_id),
            );
        }
        ConversationCommands::Patch(args) => {
            use crate::client::models::PatchConversationRequest;
            if args.audio_file.is_some() && args.audio_url.is_some() {
                anyhow::bail!("--audio-file and --audio-url are mutually exclusive");
            }
            if args.audio_file.is_none() && args.audio_url.is_none() {
                anyhow::bail!("must provide at least one of: --audio-file, --audio-url");
            }
            let audio_b64 = match args.audio_file {
                Some(path) => {
                    let data = std::fs::read(&path).with_context(|| {
                        format!("Failed to read audio file: {}", path.display())
                    })?;
                    Some(BASE64.encode(&data))
                }
                None => None,
            };
            let req = PatchConversationRequest {
                audio: audio_b64,
                audio_url: args.audio_url,
                audio_reference: None,
            };
            match client
                .conversations()
                .patch(&args.conversation_id, req)
                .await
            {
                Ok(conversation) => emit_one_with_actions(
                    ctx,
                    "conversations",
                    operation,
                    &conversation,
                    conversation_actions(&conversation.conversation_id),
                ),
                Err(ApiError::NotFound { .. }) => {
                    if !ctx.agent {
                        print_not_found_hint(&args.conversation_id, "simulations");
                    }
                    return Err(ApiError::NotFound {
                        resource: format!("Conversation '{}'", args.conversation_id),
                    }
                    .into());
                }
                Err(e) => return Err(e.into()),
            }
        }
        ConversationCommands::Audio(args) => {
            let audio = client.conversations().audio(&args.conversation_id).await?;

            match args.output {
                Some(path) => {
                    download_audio(&audio.audio_url, &path, !ctx.agent).await?;
                    emit_success_with_actions(
                        ctx,
                        "conversations",
                        operation,
                        &format!("Audio saved to {}", path.display()),
                        vec![
                            next_actions::get("conversations", &args.conversation_id).primary(),
                            next_actions::conversation_metrics(&args.conversation_id),
                        ],
                    );
                }
                None => {
                    if ctx.human() {
                        println!("{}", audio.audio_url);
                    } else {
                        emit_one_with_actions(
                            ctx,
                            "conversations",
                            operation,
                            &audio,
                            vec![
                                next_actions::get("conversations", &args.conversation_id).primary(),
                                next_actions::conversation_metrics(&args.conversation_id),
                            ],
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

fn list_actions(id: Option<&str>) -> Vec<NextAction> {
    let mut actions = vec![next_actions::context("conversations")];
    if let Some(id) = id {
        actions.insert(0, next_actions::get("conversations", id).primary());
    }
    actions
}

fn conversation_actions(conversation_id: &str) -> Vec<NextAction> {
    vec![
        next_actions::conversation_metrics(conversation_id).primary(),
        next_actions::context("conversations"),
    ]
}

fn print_not_found_hint(id: &str, try_command: &str) {
    eprintln!("hint: not found as a conversation. Try `coval {try_command} get {id}` instead.");
}

fn build_submit_request(args: SubmitArgs) -> Result<SubmitConversationRequest> {
    let mut input = args.input_json.object()?;
    let audio_sources = [
        args.audio_file.is_some(),
        args.audio_url.is_some(),
        args.upload_id.is_some(),
    ]
    .iter()
    .filter(|present| **present)
    .count();
    if audio_sources > 1 {
        anyhow::bail!("--audio-file, --audio-url, and --upload-id are mutually exclusive");
    }

    if args.transcript_file.is_none() && audio_sources == 0 {
        let has_input_source = ["transcript", "audio", "audio_url", "upload_id"]
            .iter()
            .any(|key| input.contains_key(*key));
        if !has_input_source {
            anyhow::bail!(
                "must provide at least one of: --transcript-file, --audio-file, --audio-url, --upload-id"
            );
        }
    }

    let transcript = if let Some(path) = args.transcript_file.as_ref() {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read transcript file: {}", path.display()))?;
        let parsed: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("transcript file is not valid JSON: {}", path.display()))?;
        if !parsed.is_array() {
            anyhow::bail!(
                "transcript file must contain a JSON array of message objects: {}",
                path.display()
            );
        }
        Some(parsed)
    } else {
        None
    };

    let audio = if let Some(path) = args.audio_file.as_ref() {
        let bytes = std::fs::read(path)
            .with_context(|| format!("failed to read audio file: {}", path.display()))?;
        Some(BASE64.encode(&bytes))
    } else {
        None
    };

    let metadata = if args.metadata.is_empty() {
        None
    } else {
        let mut map = BTreeMap::new();
        for (key, value) in args.metadata {
            map.insert(key, value);
        }
        Some(map)
    };

    let metrics = if args.metrics.is_empty() {
        None
    } else {
        Some(args.metrics)
    };

    input_json::insert(&mut input, "transcript", transcript)?;
    input_json::insert(&mut input, "audio", audio)?;
    input_json::insert(&mut input, "audio_url", args.audio_url)?;
    input_json::insert(&mut input, "upload_id", args.upload_id)?;
    input_json::insert(&mut input, "metrics", metrics)?;
    input_json::insert(&mut input, "metadata", metadata)?;
    input_json::insert(&mut input, "external_conversation_id", args.external_id)?;
    input_json::insert(&mut input, "occurred_at", args.occurred_at)?;
    input_json::insert(&mut input, "agent_id", args.agent_id)?;
    input_json::finish(input)
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
