use clap::{Parser, Subcommand};

use crate::client::CovalClient;
use crate::commands;
use crate::config::Config;
use crate::output::{OutputContext, OutputFormat};

#[derive(Parser)]
#[command(name = "coval")]
#[command(version, about = "Coval AI evaluation CLI")]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, global = true, default_value = "table", value_enum)]
    pub format: OutputFormat,

    #[arg(long, global = true, env = "COVAL_API_KEY")]
    pub api_key: Option<String>,

    #[arg(long, global = true, env = "COVAL_API_URL")]
    pub api_url: Option<String>,

    #[arg(long, global = true)]
    pub agent: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(name = "agent")]
    Agent {
        #[command(subcommand)]
        command: commands::agent::AgentCommands,
    },
    Login(commands::auth::LoginArgs),
    Whoami,
    Config {
        #[command(subcommand)]
        command: commands::config::ConfigCommands,
    },
    Agents {
        #[command(subcommand)]
        command: commands::agents::AgentCommands,
    },
    Conversations {
        #[command(subcommand)]
        command: commands::conversations::ConversationCommands,
    },
    Runs {
        #[command(subcommand)]
        command: commands::runs::RunCommands,
    },
    Simulations {
        #[command(subcommand)]
        command: commands::simulations::SimulationCommands,
    },
    #[command(name = "test-sets")]
    TestSets {
        #[command(subcommand)]
        command: commands::test_sets::TestSetCommands,
    },
    #[command(name = "test-cases")]
    TestCases {
        #[command(subcommand)]
        command: commands::test_cases::TestCaseCommands,
    },
    Personas {
        #[command(subcommand)]
        command: commands::personas::PersonaCommands,
    },
    Metrics {
        #[command(subcommand)]
        command: commands::metrics::MetricCommands,
    },
    Models {
        #[command(subcommand)]
        command: commands::models::ModelCommands,
    },
    Mutations {
        #[command(subcommand)]
        command: commands::mutations::MutationCommands,
    },
    #[command(name = "api-keys")]
    ApiKeys {
        #[command(subcommand)]
        command: commands::api_keys::ApiKeyCommands,
    },
    #[command(name = "run-templates")]
    RunTemplates {
        #[command(subcommand)]
        command: commands::run_templates::RunTemplateCommands,
    },
    #[command(name = "scheduled-runs")]
    ScheduledRuns {
        #[command(subcommand)]
        command: commands::scheduled_runs::ScheduledRunCommands,
    },
    Dashboards {
        #[command(subcommand)]
        command: commands::dashboards::DashboardCommands,
    },
    #[command(name = "review-annotations")]
    ReviewAnnotations {
        #[command(subcommand)]
        command: commands::review_annotations::ReviewAnnotationCommands,
    },
    #[command(name = "review-projects")]
    ReviewProjects {
        #[command(subcommand)]
        command: commands::review_projects::ReviewProjectCommands,
    },
    Reports {
        #[command(subcommand)]
        command: commands::reports::ReportCommands,
    },
    Monitors {
        #[command(subcommand)]
        command: commands::monitors::MonitorCommands,
    },
    Tags {
        #[command(subcommand)]
        command: commands::tags::TagCommands,
    },
    Traces {
        #[command(subcommand)]
        command: commands::traces::TraceCommands,
    },
}

impl Commands {
    pub fn resource(&self) -> &'static str {
        match self {
            Self::Agent { .. } => "agent",
            Self::Login(_) | Self::Whoami => "auth",
            Self::Config { .. } => "config",
            Self::Agents { .. } => "agents",
            Self::Conversations { .. } => "conversations",
            Self::Runs { .. } => "runs",
            Self::Simulations { .. } => "simulations",
            Self::TestSets { .. } => "test-sets",
            Self::TestCases { .. } => "test-cases",
            Self::Personas { command } => match command {
                commands::personas::PersonaCommands::BackgroundSounds { .. } => "background-sounds",
                _ => "personas",
            },
            Self::Metrics { .. } => "metrics",
            Self::Models { .. } => "models",
            Self::Mutations { .. } => "mutations",
            Self::ApiKeys { .. } => "api-keys",
            Self::RunTemplates { .. } => "run-templates",
            Self::ScheduledRuns { .. } => "scheduled-runs",
            Self::Dashboards { command } => match command {
                commands::dashboards::DashboardCommands::Widgets { .. } => "widgets",
                _ => "dashboards",
            },
            Self::ReviewAnnotations { .. } => "review-annotations",
            Self::ReviewProjects { .. } => "review-projects",
            Self::Reports { .. } => "reports",
            Self::Monitors { .. } => "monitors",
            Self::Tags { .. } => "tags",
            Self::Traces { .. } => "traces",
        }
    }

    pub fn operation(&self) -> &'static str {
        match self {
            Self::Agent { command } => command.operation(),
            Self::Login(_) => "login",
            Self::Whoami => "whoami",
            Self::Config { command } => command.operation(),
            Self::Agents { command } => command.operation(),
            Self::Conversations { command } => command.operation(),
            Self::Runs { command } => command.operation(),
            Self::Simulations { command } => command.operation(),
            Self::TestSets { command } => command.operation(),
            Self::TestCases { command } => command.operation(),
            Self::Personas { command } => command.operation(),
            Self::Metrics { command } => command.operation(),
            Self::Models { command } => command.operation(),
            Self::Mutations { command } => command.operation(),
            Self::ApiKeys { command } => command.operation(),
            Self::RunTemplates { command } => command.operation(),
            Self::ScheduledRuns { command } => command.operation(),
            Self::Dashboards { command } => command.operation(),
            Self::ReviewAnnotations { command } => command.operation(),
            Self::ReviewProjects { command } => command.operation(),
            Self::Reports { command } => command.operation(),
            Self::Monitors { command } => command.operation(),
            Self::Tags { command } => command.operation(),
            Self::Traces { command } => command.operation(),
        }
    }
}

pub async fn run(cli: Cli, ctx: &OutputContext) -> anyhow::Result<()> {
    let config = Config::load().unwrap_or_default();
    let api_key_source = if cli.api_key.is_some() {
        "argument_or_env"
    } else if config.api_key.is_some() {
        "config"
    } else {
        "none"
    };
    let api_url_source = if cli.api_url.is_some() {
        "argument_or_env"
    } else if config.api_url.is_some() {
        "config"
    } else {
        "default"
    };
    let api_key = cli.api_key.or(config.api_key);
    let api_url = cli.api_url.or(config.api_url);

    match cli.command {
        Commands::Agent { command } => {
            commands::agent::execute(
                command,
                api_key.as_deref(),
                api_key_source,
                api_url.as_deref(),
                api_url_source,
                ctx,
            )
            .await
        }
        Commands::Login(args) => commands::auth::login(args, ctx).await,
        Commands::Whoami => {
            commands::auth::whoami(api_key.as_ref(), ctx);
            Ok(())
        }
        Commands::Config { command } => commands::config::execute(command, ctx),
        Commands::Agents {
            command: commands::agents::AgentCommands::Context,
        } => commands::agent::resource_context("agents", ctx),
        Commands::Conversations {
            command: commands::conversations::ConversationCommands::Context,
        } => commands::agent::resource_context("conversations", ctx),
        Commands::Runs {
            command: commands::runs::RunCommands::Context,
        } => commands::agent::resource_context("runs", ctx),
        Commands::Simulations {
            command: commands::simulations::SimulationCommands::Context,
        } => commands::agent::resource_context("simulations", ctx),
        Commands::TestSets {
            command: commands::test_sets::TestSetCommands::Context,
        } => commands::agent::resource_context("test-sets", ctx),
        Commands::TestCases {
            command: commands::test_cases::TestCaseCommands::Context,
        } => commands::agent::resource_context("test-cases", ctx),
        Commands::Personas {
            command: commands::personas::PersonaCommands::Context,
        } => commands::agent::resource_context("personas", ctx),
        Commands::Metrics {
            command: commands::metrics::MetricCommands::Context,
        } => commands::agent::resource_context("metrics", ctx),
        Commands::Mutations {
            command: commands::mutations::MutationCommands::Context,
        } => commands::agent::resource_context("mutations", ctx),
        Commands::ApiKeys {
            command: commands::api_keys::ApiKeyCommands::Context,
        } => commands::agent::resource_context("api-keys", ctx),
        Commands::RunTemplates {
            command: commands::run_templates::RunTemplateCommands::Context,
        } => commands::agent::resource_context("run-templates", ctx),
        Commands::ScheduledRuns {
            command: commands::scheduled_runs::ScheduledRunCommands::Context,
        } => commands::agent::resource_context("scheduled-runs", ctx),
        Commands::Dashboards {
            command: commands::dashboards::DashboardCommands::Context,
        } => commands::agent::resource_context("dashboards", ctx),
        Commands::ReviewAnnotations {
            command: commands::review_annotations::ReviewAnnotationCommands::Context,
        } => commands::agent::resource_context("review-annotations", ctx),
        Commands::ReviewProjects {
            command: commands::review_projects::ReviewProjectCommands::Context,
        } => commands::agent::resource_context("review-projects", ctx),
        Commands::Reports {
            command: commands::reports::ReportCommands::Context,
        } => commands::agent::resource_context("reports", ctx),
        Commands::Monitors {
            command: commands::monitors::MonitorCommands::Context,
        } => commands::agent::resource_context("monitors", ctx),
        Commands::Tags {
            command: commands::tags::TagCommands::Context,
        } => commands::agent::resource_context("tags", ctx),
        Commands::Traces {
            command: commands::traces::TraceCommands::Context,
        } => commands::agent::resource_context("traces", ctx),
        _ => {
            let api_key = api_key.ok_or_else(|| {
                anyhow::anyhow!(
                    "Not authenticated. Run `coval login` or set COVAL_API_KEY environment variable."
                )
            })?;
            let client = CovalClient::new(api_key, api_url.as_deref());

            match cli.command {
                Commands::Agents { command } => {
                    commands::agents::execute(command, &client, ctx).await
                }
                Commands::Conversations { command } => {
                    commands::conversations::execute(command, &client, ctx).await
                }
                Commands::Runs { command } => commands::runs::execute(command, &client, ctx).await,
                Commands::Simulations { command } => {
                    commands::simulations::execute(command, &client, ctx).await
                }
                Commands::TestSets { command } => {
                    commands::test_sets::execute(command, &client, ctx).await
                }
                Commands::TestCases { command } => {
                    commands::test_cases::execute(command, &client, ctx).await
                }
                Commands::Personas { command } => {
                    commands::personas::execute(command, &client, ctx).await
                }
                Commands::Metrics { command } => {
                    commands::metrics::execute(command, &client, ctx).await
                }
                Commands::Models { command } => {
                    commands::models::execute(command, &client, ctx).await
                }
                Commands::Mutations { command } => {
                    commands::mutations::execute(command, &client, ctx).await
                }
                Commands::ApiKeys { command } => {
                    commands::api_keys::execute(command, &client, ctx).await
                }
                Commands::RunTemplates { command } => {
                    commands::run_templates::execute(command, &client, ctx).await
                }
                Commands::ScheduledRuns { command } => {
                    commands::scheduled_runs::execute(command, &client, ctx).await
                }
                Commands::Dashboards { command } => {
                    commands::dashboards::execute(command, &client, ctx).await
                }
                Commands::ReviewAnnotations { command } => {
                    commands::review_annotations::execute(command, &client, ctx).await
                }
                Commands::ReviewProjects { command } => {
                    commands::review_projects::execute(command, &client, ctx).await
                }
                Commands::Reports { command } => {
                    commands::reports::execute(command, &client, ctx).await
                }
                Commands::Monitors { command } => {
                    commands::monitors::execute(command, &client, ctx).await
                }
                Commands::Tags { command } => commands::tags::execute(command, &client, ctx).await,
                Commands::Traces { command } => {
                    commands::traces::execute(command, &client, ctx).await
                }
                _ => unreachable!(),
            }
        }
    }
}
