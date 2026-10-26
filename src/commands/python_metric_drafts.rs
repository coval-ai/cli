use std::path::PathBuf;

use anyhow::{ensure, Context, Result};
use clap::{Args, Subcommand};

use crate::client::models::{
    CreatePythonMetricDraftRequest, PreviewPythonMetricRequest, PublishPythonMetricRequest,
    PublishPythonMetricResponse, PythonMetricDraftResponse, PythonMetricOutputType,
    PythonMetricPreviewsResponse,
};
use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::{emit_one_with_actions, NextAction, OutputContext};

#[derive(Subcommand)]
pub enum PythonMetricDraftCommands {
    Context,
    Create(CreateArgs),
    Get(DraftIdArgs),
    Publish(PublishArgs),
    Previews {
        draft_id: String,
        #[command(subcommand)]
        command: PreviewCommands,
    },
}

#[derive(Subcommand)]
pub enum PreviewCommands {
    Create(PreviewCreateArgs),
    Get(PreviewIdArgs),
    Cancel(PreviewIdArgs),
}

impl PythonMetricDraftCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::Create(_) => "create",
            Self::Get(_) => "get",
            Self::Publish(_) => "publish",
            Self::Previews { command, .. } => match command {
                PreviewCommands::Create(_) => "previews-create",
                PreviewCommands::Get(_) => "previews-get",
                PreviewCommands::Cancel(_) => "previews-cancel",
            },
        }
    }
}

#[derive(Args)]
pub struct CreateArgs {
    #[command(flatten)]
    input_json: InputJsonArg,
    #[arg(long)]
    display_name: Option<String>,
    #[arg(long)]
    target_metric_id: Option<String>,
    #[arg(long)]
    target_version_id: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long, conflicts_with = "source_file")]
    source: Option<String>,
    #[arg(long)]
    source_file: Option<PathBuf>,
    #[arg(long, value_enum)]
    output_type: Option<PythonMetricOutputType>,
    #[arg(long)]
    unit: Option<String>,
    #[arg(long)]
    requirements: Option<String>,
    #[arg(long)]
    require_audio: Option<bool>,
    #[arg(long)]
    require_traces: Option<bool>,
    #[arg(long)]
    frame_table: Vec<String>,
}

#[derive(Args)]
pub struct DraftIdArgs {
    draft_id: String,
}

#[derive(Args)]
pub struct PublishArgs {
    draft_id: String,
    #[arg(long, required = true, value_delimiter = ',')]
    preview_ids: Vec<String>,
    #[arg(long, required = true)]
    fingerprint: String,
}

#[derive(Args)]
pub struct PreviewCreateArgs {
    #[arg(long, required = true)]
    simulation_output_ids: Vec<String>,
}

#[derive(Args)]
pub struct PreviewIdArgs {
    preview_id: String,
}

pub async fn execute(
    command: PythonMetricDraftCommands,
    client: &CovalClient,
    ctx: &OutputContext,
) -> Result<()> {
    match command {
        PythonMetricDraftCommands::Context => {
            crate::commands::agent::resource_context("python-metric-drafts", ctx)
        }
        PythonMetricDraftCommands::Create(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "display_name", args.display_name)?;
            input_json::insert(&mut input, "target_metric_id", args.target_metric_id)?;
            input_json::insert(&mut input, "target_version_id", args.target_version_id)?;
            input_json::insert(&mut input, "description", args.description)?;
            input_json::insert(&mut input, "source", args.source)?;
            if let Some(path) = args.source_file {
                let source = std::fs::read_to_string(&path)
                    .with_context(|| format!("failed to read --source-file: {}", path.display()))?;
                input.insert("source".to_string(), serde_json::Value::String(source));
            }
            input_json::insert(&mut input, "output_type", args.output_type)?;
            input_json::insert(&mut input, "unit", args.unit)?;
            if args.requirements.is_some()
                || args.require_audio.is_some()
                || args.require_traces.is_some()
                || !args.frame_table.is_empty()
            {
                let mut requirements = match args.requirements {
                    Some(value) => {
                        serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&value)
                            .context("--requirements must be a JSON object")?
                    }
                    None => input
                        .remove("requirements")
                        .map(|value| {
                            value.as_object().cloned().ok_or_else(|| {
                                anyhow::anyhow!(
                                    "requirements from --input-json must be a JSON object"
                                )
                            })
                        })
                        .transpose()?
                        .unwrap_or_default(),
                };
                input_json::insert(&mut requirements, "audio", args.require_audio)?;
                input_json::insert(&mut requirements, "traces", args.require_traces)?;
                if !args.frame_table.is_empty() {
                    requirements.insert(
                        "frame_tables".to_string(),
                        serde_json::to_value(args.frame_table)?,
                    );
                }
                input.insert(
                    "requirements".to_string(),
                    serde_json::Value::Object(requirements),
                );
            }
            let request: CreatePythonMetricDraftRequest = input_json::finish(input)?;
            let response: PythonMetricDraftResponse =
                client.python_metric_drafts().create(request).await?;
            let draft = &response.draft;
            emit_one_with_actions(
                ctx,
                "python-metric-drafts",
                "create",
                &response,
                vec![
                    NextAction::new(
                        "python-metric-drafts.previews.create",
                        "Create a preview for this draft",
                        next_actions::argv([
                            "python-metric-drafts",
                            "previews",
                            &draft.draft_id,
                            "create",
                            "--simulation-output-ids",
                            "<simulation_output_id>",
                        ]),
                        false,
                    )
                    .primary(),
                    next_actions::context("python-metric-drafts"),
                ],
            );
            Ok(())
        }
        PythonMetricDraftCommands::Get(args) => {
            let response = client.python_metric_drafts().get(&args.draft_id).await?;
            emit_one_with_actions(
                ctx,
                "python-metric-drafts",
                "get",
                &response,
                vec![next_actions::context("python-metric-drafts")],
            );
            Ok(())
        }
        PythonMetricDraftCommands::Publish(args) => {
            let request = PublishPythonMetricRequest {
                preview_ids: args.preview_ids,
                fingerprint: args.fingerprint,
            };
            let response: PublishPythonMetricResponse = client
                .python_metric_drafts()
                .publish(&args.draft_id, request)
                .await?;
            emit_one_with_actions(
                ctx,
                "python-metric-drafts",
                "publish",
                &response,
                vec![next_actions::context("python-metric-drafts")],
            );
            Ok(())
        }
        PythonMetricDraftCommands::Previews { draft_id, command } => match command {
            PreviewCommands::Create(args) => {
                ensure!(
                    args.simulation_output_ids.len() == 1,
                    "--simulation-output-ids requires exactly one value"
                );
                let request = PreviewPythonMetricRequest {
                    simulation_output_ids: args.simulation_output_ids,
                };
                let response: PythonMetricPreviewsResponse = client
                    .python_metric_drafts()
                    .create_preview(&draft_id, request)
                    .await?;
                let actions = response
                    .previews
                    .first()
                    .map(|preview| {
                        vec![
                            NextAction::new(
                                "python-metric-drafts.previews.get",
                                "Poll this preview",
                                next_actions::argv([
                                    "python-metric-drafts",
                                    "previews",
                                    &draft_id,
                                    "get",
                                    &preview.preview_id,
                                ]),
                                true,
                            )
                            .primary()
                            .with_description("Preview evaluation is asynchronous; poll until it reaches a terminal state."),
                            next_actions::context("python-metric-drafts"),
                        ]
                    })
                    .unwrap_or_else(|| vec![next_actions::context("python-metric-drafts")]);
                emit_one_with_actions(
                    ctx,
                    "python-metric-drafts",
                    "previews-create",
                    &response,
                    actions,
                );
                Ok(())
            }
            PreviewCommands::Get(args) => {
                let response = client
                    .python_metric_drafts()
                    .get_preview(&draft_id, &args.preview_id)
                    .await?;
                let mut actions = vec![next_actions::context("python-metric-drafts")];
                if let Some(preview) = response.previews.first() {
                    if preview.status.eq_ignore_ascii_case("succeeded") {
                        actions.insert(
                            0,
                            NextAction::new(
                                "python-metric-drafts.publish",
                                "Publish this approved draft",
                                next_actions::argv([
                                    "python-metric-drafts",
                                    "publish",
                                    &draft_id,
                                    "--preview-ids",
                                    &preview.preview_id,
                                    "--fingerprint",
                                    "<draft_fingerprint>",
                                ]),
                                false,
                            )
                            .primary()
                            .with_description("Publish only after the preview succeeds and the exact draft fingerprint is approved."),
                        );
                    } else if preview.error.is_none()
                        && !matches!(
                            preview.status.to_ascii_lowercase().as_str(),
                            "failed" | "cancelled" | "canceled" | "error"
                        )
                    {
                        actions.insert(
                            0,
                            NextAction::new(
                                "python-metric-drafts.previews.get",
                                "Poll this preview",
                                next_actions::argv([
                                    "python-metric-drafts",
                                    "previews",
                                    &draft_id,
                                    "get",
                                    &preview.preview_id,
                                ]),
                                true,
                            )
                            .primary()
                            .with_description("Preview evaluation is asynchronous; poll until it reaches a terminal state."),
                        );
                    }
                }
                emit_one_with_actions(
                    ctx,
                    "python-metric-drafts",
                    "previews-get",
                    &response,
                    actions,
                );
                Ok(())
            }
            PreviewCommands::Cancel(args) => {
                let response = client
                    .python_metric_drafts()
                    .cancel_preview(&draft_id, &args.preview_id)
                    .await?;
                emit_one_with_actions(
                    ctx,
                    "python-metric-drafts",
                    "previews-cancel",
                    &response,
                    vec![next_actions::context("python-metric-drafts")],
                );
                Ok(())
            }
        },
    }
}
