use clap::{Parser, Subcommand};

use crate::client::CovalClient;
use crate::commands;
use crate::config::Config;
use crate::output::OutputFormat;

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
    Traces {
        #[command(subcommand)]
        command: commands::traces::TraceCommands,
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
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    let config = Config::load().unwrap_or_default();
    let api_key = cli.api_key.or(config.api_key);
    let api_url = cli.api_url.or(config.api_url);

    match cli.command {
        Commands::Login(args) => commands::auth::login(args).await,
        Commands::Whoami => {
            commands::auth::whoami(api_key.as_ref());
            Ok(())
        }
        Commands::Config { command } => commands::config::execute(command),
        Commands::Traces {
            command: commands::traces::TraceCommands::Setup(args),
        } if args.no_validate => commands::traces::execute(
            commands::traces::TraceCommands::Setup(args),
            None,
        )
        .await,
        _ => {
            let api_key = api_key.ok_or_else(|| {
                anyhow::anyhow!(
                    "Not authenticated. Run `coval login` or set COVAL_API_KEY environment variable."
                )
            })?;
            let client = CovalClient::new(api_key, api_url.as_deref());

            match cli.command {
                Commands::Agents { command } => {
                    commands::agents::execute(command, &client, cli.format).await
                }
                Commands::Runs { command } => {
                    commands::runs::execute(command, &client, cli.format).await
                }
                Commands::Simulations { command } => {
                    commands::simulations::execute(command, &client, cli.format).await
                }
                Commands::TestSets { command } => {
                    commands::test_sets::execute(command, &client, cli.format).await
                }
                Commands::TestCases { command } => {
                    commands::test_cases::execute(command, &client, cli.format).await
                }
                Commands::Personas { command } => {
                    commands::personas::execute(command, &client, cli.format).await
                }
                Commands::Metrics { command } => {
                    commands::metrics::execute(command, &client, cli.format).await
                }
                Commands::Mutations { command } => {
                    commands::mutations::execute(command, &client, cli.format).await
                }
                Commands::Traces { command } => {
                    commands::traces::execute(command, Some(&client)).await
                }
                Commands::ApiKeys { command } => {
                    commands::api_keys::execute(command, &client, cli.format).await
                }
                Commands::RunTemplates { command } => {
                    commands::run_templates::execute(command, &client, cli.format).await
                }
                Commands::ScheduledRuns { command } => {
                    commands::scheduled_runs::execute(command, &client, cli.format).await
                }
                Commands::Dashboards { command } => {
                    commands::dashboards::execute(command, &client, cli.format).await
                }
                _ => unreachable!(),
            }
        }
    }
}
