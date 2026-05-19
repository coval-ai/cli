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
}

impl Commands {
    pub fn resource(&self) -> &'static str {
        match self {
            Self::Login(_) | Self::Whoami => "auth",
            Self::Config { .. } => "config",
            Self::Agents { .. } => "agents",
            Self::Conversations { .. } => "conversations",
            Self::Runs { .. } => "runs",
            Self::Simulations { .. } => "simulations",
            Self::TestSets { .. } => "test-sets",
            Self::TestCases { .. } => "test-cases",
            Self::Personas { .. } => "personas",
            Self::Metrics { .. } => "metrics",
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
        }
    }

    pub fn operation(&self) -> &'static str {
        match self {
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
            Self::Mutations { command } => command.operation(),
            Self::ApiKeys { command } => command.operation(),
            Self::RunTemplates { command } => command.operation(),
            Self::ScheduledRuns { command } => command.operation(),
            Self::Dashboards { command } => command.operation(),
            Self::ReviewAnnotations { command } => command.operation(),
            Self::ReviewProjects { command } => command.operation(),
        }
    }
}

pub async fn run(cli: Cli, ctx: &OutputContext) -> anyhow::Result<()> {
    let config = Config::load().unwrap_or_default();
    let api_key = cli.api_key.or(config.api_key);
    let api_url = cli.api_url.or(config.api_url);

    match cli.command {
        Commands::Login(args) => commands::auth::login(args, ctx).await,
        Commands::Whoami => {
            commands::auth::whoami(api_key.as_ref(), ctx);
            Ok(())
        }
        Commands::Config { command } => commands::config::execute(command, ctx),
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
                _ => unreachable!(),
            }
        }
    }
}
