use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Args, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};

use crate::client::error::ApiError;
use crate::client::models::{ListParams, MetricDetailResponse};
use crate::client::CovalClient;
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, NextAction,
    OutputContext,
};

#[derive(Subcommand)]
pub enum SimulationCommands {
    Context,
    Resimulate(ResimulateArgs),
    Update(UpdateArgs),
    List(ListArgs),
    Get(GetArgs),
    Delete(DeleteArgs),
    Audio(AudioArgs),
    Metrics(MetricsArgs),
    #[command(name = "metric-detail")]
    MetricDetail(MetricDetailArgs),
}

impl SimulationCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::List(_) => "list",
            Self::Get(_) => "get",
            Self::Delete(_) => "delete",
            Self::Audio(_) => "audio",
            Self::Metrics(_) => "metrics",
            Self::MetricDetail(_) => "metric-detail",
            Self::Resimulate(_) => "resimulate",
            Self::Update(_) => "update",
        }
    }
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
    metric_id: String,
}

#[derive(Args)]
pub struct ResimulateArgs {
    simulation_id: String,
    #[arg(long)]
    dev_id: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    simulation_id: String,
    #[arg(long)]
    notes: Option<String>,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    is_public: bool,
}

#[derive(Clone, Copy)]
enum SimulationCollection {
    SimulatedConversations,
    Legacy,
}

impl SimulationCollection {
    fn resource(self) -> &'static str {
        match self {
            Self::SimulatedConversations => "simulated-conversations",
            Self::Legacy => "simulations",
        }
    }

    fn alternate_resource(self) -> &'static str {
        match self {
            Self::SimulatedConversations => "uploaded-conversations",
            Self::Legacy => "conversations",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::SimulatedConversations => "Simulated conversation",
            Self::Legacy => "Simulation",
        }
    }
}

pub async fn execute_simulated_conversations(
    cmd: SimulationCommands,
    client: &CovalClient,
    ctx: &OutputContext,
) -> Result<()> {
    execute_for(
        cmd,
        client,
        ctx,
        SimulationCollection::SimulatedConversations,
    )
    .await
}

pub async fn execute_legacy(
    cmd: SimulationCommands,
    client: &CovalClient,
    ctx: &OutputContext,
) -> Result<()> {
    execute_for(cmd, client, ctx, SimulationCollection::Legacy).await
}

async fn execute_for(
    cmd: SimulationCommands,
    client: &CovalClient,
    ctx: &OutputContext,
    collection: SimulationCollection,
) -> Result<()> {
    let operation = cmd.operation();
    let resource = collection.resource();
    match cmd {
        SimulationCommands::Context => {
            return crate::commands::agent::resource_context(resource, ctx);
        }
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
            let response = match collection {
                SimulationCollection::SimulatedConversations => {
                    client.simulated_conversations().list(params).await?
                }
                SimulationCollection::Legacy => client.simulations().list(params).await?,
            };
            emit_list_with_actions(
                ctx,
                resource,
                operation,
                &response.simulations,
                list_actions(
                    resource,
                    response
                        .simulations
                        .first()
                        .map(|simulation| simulation.simulation_id.as_str()),
                ),
            );
        }
        SimulationCommands::Get(args) => {
            let result = match collection {
                SimulationCollection::SimulatedConversations => {
                    client
                        .simulated_conversations()
                        .get(&args.simulation_id)
                        .await
                }
                SimulationCollection::Legacy => client.simulations().get(&args.simulation_id).await,
            };
            match result {
                Ok(simulation) => emit_one_with_actions(
                    ctx,
                    resource,
                    operation,
                    &simulation,
                    simulation_actions(resource, &simulation.simulation_id),
                ),
                Err(ApiError::NotFound { .. }) => {
                    if !ctx.agent {
                        print_not_found_hint(
                            &args.simulation_id,
                            collection.description(),
                            collection.alternate_resource(),
                        );
                    }
                    return Err(ApiError::NotFound {
                        resource: format!("{} '{}'", collection.description(), args.simulation_id),
                    }
                    .into());
                }
                Err(e) => return Err(e.into()),
            }
        }
        SimulationCommands::Delete(args) => {
            let result = match collection {
                SimulationCollection::SimulatedConversations => {
                    client
                        .simulated_conversations()
                        .delete(&args.simulation_id)
                        .await
                }
                SimulationCollection::Legacy => {
                    client.simulations().delete(&args.simulation_id).await
                }
            };
            match result {
                Ok(()) => emit_success_with_actions(
                    ctx,
                    resource,
                    operation,
                    &format!("{} deleted.", collection.description()),
                    vec![
                        next_actions::list(resource).primary(),
                        next_actions::context(resource),
                    ],
                ),
                Err(ApiError::NotFound { .. }) => {
                    if !ctx.agent {
                        print_not_found_hint(
                            &args.simulation_id,
                            collection.description(),
                            collection.alternate_resource(),
                        );
                    }
                    return Err(ApiError::NotFound {
                        resource: format!("{} '{}'", collection.description(), args.simulation_id),
                    }
                    .into());
                }
                Err(e) => return Err(e.into()),
            }
        }
        SimulationCommands::Metrics(args) => {
            let response = match collection {
                SimulationCollection::SimulatedConversations => {
                    client
                        .simulated_conversations()
                        .list_metrics(&args.simulation_id)
                        .await?
                }
                SimulationCollection::Legacy => {
                    client
                        .simulations()
                        .list_metrics(&args.simulation_id)
                        .await?
                }
            };
            emit_list_with_actions(
                ctx,
                resource,
                operation,
                &response.metrics,
                vec![
                    next_actions::get(resource, &args.simulation_id).primary(),
                    next_actions::context(resource),
                ],
            );
        }
        SimulationCommands::MetricDetail(args) => {
            let response = match collection {
                SimulationCollection::SimulatedConversations => {
                    client
                        .simulated_conversations()
                        .get_metric(&args.simulation_id, &args.metric_id)
                        .await?
                }
                SimulationCollection::Legacy => {
                    client
                        .simulations()
                        .get_metric(&args.simulation_id, &args.metric_id)
                        .await?
                }
            };
            match response {
                MetricDetailResponse::Single { metric } => emit_one_with_actions(
                    ctx,
                    resource,
                    operation,
                    &metric,
                    vec![next_actions::get(resource, &args.simulation_id).primary()],
                ),
                MetricDetailResponse::Collection { metric_outputs } => emit_list_with_actions(
                    ctx,
                    resource,
                    operation,
                    &metric_outputs,
                    vec![next_actions::get(resource, &args.simulation_id).primary()],
                ),
            }
        }
        SimulationCommands::Audio(args) => {
            let audio = match collection {
                SimulationCollection::SimulatedConversations => {
                    client
                        .simulated_conversations()
                        .audio(&args.simulation_id)
                        .await?
                }
                SimulationCollection::Legacy => {
                    client.simulations().audio(&args.simulation_id).await?
                }
            };

            match args.output {
                Some(path) => {
                    download_audio(&audio.audio_url, &path, !ctx.agent).await?;
                    emit_success_with_actions(
                        ctx,
                        resource,
                        operation,
                        &format!("Audio saved to {}", path.display()),
                        vec![
                            next_actions::get(resource, &args.simulation_id).primary(),
                            next_actions::metrics(resource, &args.simulation_id),
                        ],
                    );
                }
                None => {
                    if ctx.agent {
                        emit_one_with_actions(
                            ctx,
                            resource,
                            operation,
                            &audio,
                            vec![
                                next_actions::get(resource, &args.simulation_id).primary(),
                                next_actions::metrics(resource, &args.simulation_id),
                            ],
                        );
                    } else {
                        println!("{}", audio.audio_url);
                    }
                }
            }
        }
        SimulationCommands::Resimulate(args) => {
            let simulation = match collection {
                SimulationCollection::SimulatedConversations => {
                    client
                        .simulated_conversations()
                        .resimulate(&args.simulation_id, args.dev_id.as_deref())
                        .await?
                }
                SimulationCollection::Legacy => {
                    client
                        .simulations()
                        .resimulate(&args.simulation_id, args.dev_id.as_deref())
                        .await?
                }
            };
            emit_one_with_actions(
                ctx,
                resource,
                operation,
                &simulation,
                vec![next_actions::get(resource, &args.simulation_id).primary()],
            );
        }
        SimulationCommands::Update(args) => {
            let simulation = match collection {
                SimulationCollection::SimulatedConversations => {
                    client
                        .simulated_conversations()
                        .update(
                            &args.simulation_id,
                            args.notes,
                            args.is_public.then_some(true),
                        )
                        .await?
                }
                SimulationCollection::Legacy => {
                    client
                        .simulations()
                        .update(
                            &args.simulation_id,
                            args.notes,
                            args.is_public.then_some(true),
                        )
                        .await?
                }
            };
            emit_one_with_actions(
                ctx,
                resource,
                operation,
                &simulation,
                vec![next_actions::get(resource, &args.simulation_id).primary()],
            );
        }
    }
    Ok(())
}

fn list_actions(resource: &str, id: Option<&str>) -> Vec<NextAction> {
    let mut actions = vec![next_actions::context(resource)];
    if let Some(id) = id {
        actions.insert(0, next_actions::get(resource, id).primary());
    }
    actions
}

fn simulation_actions(resource: &str, simulation_id: &str) -> Vec<NextAction> {
    vec![
        next_actions::metrics(resource, simulation_id).primary(),
        next_actions::audio(resource, simulation_id),
        next_actions::context(resource),
    ]
}

fn print_not_found_hint(id: &str, description: &str, alternate_resource: &str) {
    eprintln!(
        "hint: not found as a {}. Try `coval {alternate_resource} get {id}` instead.",
        description.to_lowercase()
    );
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
