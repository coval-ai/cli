mod cli;
mod client;
mod commands;
mod config;
mod output;

use std::process::ExitCode;

use clap::error::ErrorKind;
use clap::Parser;

use crate::client::error::ApiError;
use crate::output::{AgentError, OutputContext};

#[tokio::main]
async fn main() -> ExitCode {
    let agent_requested = std::env::args_os().any(|arg| arg == "--agent");
    let args = match cli::Cli::try_parse() {
        Ok(args) => args,
        Err(err) => match err.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => err.exit(),
            _ if agent_requested => {
                output::emit_error(
                    "cli",
                    "parse",
                    AgentError::new("usage_error", err.to_string()),
                    Vec::new(),
                    Vec::new(),
                );
                return exit_code(err.exit_code());
            }
            _ => err.exit(),
        },
    };

    let resource = args.command.resource();
    let operation = args.command.operation();
    let ctx = OutputContext::new(args.format, args.agent);

    match cli::run(args, &ctx).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            if ctx.agent {
                output::emit_error(
                    resource,
                    operation,
                    AgentError::from_anyhow(&err),
                    Vec::new(),
                    Vec::new(),
                );
            } else {
                eprintln!("Error: {err}");
            }
            exit_code_for_error(&err)
        }
    }
}

fn exit_code_for_error(err: &anyhow::Error) -> ExitCode {
    if let Some(api_error) = err.downcast_ref::<ApiError>() {
        return exit_code(api_error.exit_code());
    }

    exit_code(1)
}

fn exit_code(code: i32) -> ExitCode {
    ExitCode::from(u8::try_from(code).unwrap_or(1))
}
