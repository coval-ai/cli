use clap::ValueEnum;
use serde::Serialize;
use serde_json::json;
use tabled::settings::Style;

use crate::client::error::ApiError;

pub const ACI_VERSION: &str = "0.1";

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
}

#[derive(Debug, Clone, Copy)]
pub struct OutputContext {
    pub format: OutputFormat,
    pub agent: bool,
}

impl OutputContext {
    pub fn new(format: OutputFormat, agent: bool) -> Self {
        Self { format, agent }
    }

    pub fn human(&self) -> bool {
        !self.agent && matches!(self.format, OutputFormat::Table)
    }
}

#[derive(Debug, Serialize)]
pub struct AgentWarning {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remedy: Option<String>,
}

impl AgentWarning {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            remedy: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct NextAction {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub argv: Vec<String>,
    pub safe: bool,
    pub primary: bool,
    pub requires_confirmation: bool,
}

impl NextAction {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        argv: Vec<String>,
        safe: bool,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            argv,
            safe,
            primary: false,
            requires_confirmation: !safe,
        }
    }

    pub fn primary(mut self) -> Self {
        self.primary = true;
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Debug, Serialize)]
pub struct AgentError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remedy: Option<String>,
    pub retryable: bool,
}

impl AgentError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            remedy: None,
            retryable: false,
        }
    }

    pub fn with_remedy(mut self, remedy: impl Into<String>) -> Self {
        self.remedy = Some(remedy.into());
        self
    }

    pub fn retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    pub fn from_anyhow(err: &anyhow::Error) -> Self {
        if let Some(api_error) = err.downcast_ref::<ApiError>() {
            return match api_error {
                ApiError::Unauthenticated { message } => Self::new("unauthenticated", message)
                    .with_remedy("Run `coval login`, pass `--api-key`, or set COVAL_API_KEY."),
                ApiError::NotFound { resource } => Self::new("not_found", resource),
                ApiError::InvalidArgument { message, .. } => Self::new("validation_error", message),
                ApiError::PermissionDenied { message } => Self::new("permission_denied", message),
                ApiError::Internal { message } => Self::new("server_error", message),
                ApiError::Network(network_error) => {
                    Self::new("network_error", network_error.to_string()).retryable(true)
                }
            };
        }

        let message = err.to_string();
        if message.starts_with("Not authenticated.") {
            return Self::new("unauthenticated", message)
                .with_remedy("Run `coval login`, pass `--api-key`, or set COVAL_API_KEY.");
        }

        Self::new("cli_error", message)
    }
}

#[derive(Debug, Serialize)]
struct AgentEnvelope<'a, T: Serialize> {
    aci: &'static str,
    ok: bool,
    resource: &'a str,
    operation: &'a str,
    data: T,
    warnings: Vec<AgentWarning>,
    next_actions: Vec<NextAction>,
}

#[derive(Debug, Serialize)]
struct AgentErrorEnvelope<'a> {
    aci: &'static str,
    ok: bool,
    resource: &'a str,
    operation: &'a str,
    error: AgentError,
    warnings: Vec<AgentWarning>,
    next_actions: Vec<NextAction>,
}

pub trait Tabular {
    fn headers() -> Vec<&'static str>;
    fn row(&self) -> Vec<String>;
}

pub fn print_list<T: Serialize + Tabular>(items: &[T], format: OutputFormat) {
    match format {
        OutputFormat::Table => {
            if items.is_empty() {
                println!("No results found.");
                return;
            }
            let rows: Vec<Vec<String>> = items.iter().map(Tabular::row).collect();
            let headers = T::headers();
            let mut builder = tabled::builder::Builder::new();
            builder.push_record(headers);
            for row in rows {
                builder.push_record(row);
            }
            let table = builder.build().with(Style::rounded()).to_string();
            println!("{table}");
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(items).expect("Failed to serialize");
            println!("{json}");
        }
    }
}

pub fn print_one<T: Serialize>(item: &T, _format: OutputFormat) {
    let json = serde_json::to_string_pretty(item).expect("Failed to serialize");
    println!("{json}");
}

#[allow(dead_code)]
pub fn print_id(id: &str) {
    println!("{id}");
}

pub fn print_success(message: &str) {
    println!("{message}");
}

pub fn emit_list_with_actions<T: Serialize + Tabular>(
    ctx: &OutputContext,
    resource: &'static str,
    operation: &'static str,
    items: &[T],
    next_actions: Vec<NextAction>,
) {
    if ctx.agent {
        emit_agent_data(resource, operation, items, Vec::new(), next_actions);
    } else {
        print_list(items, ctx.format);
    }
}

pub fn emit_one<T: Serialize>(
    ctx: &OutputContext,
    resource: &'static str,
    operation: &'static str,
    item: &T,
) {
    emit_one_with_actions(ctx, resource, operation, item, Vec::new());
}

pub fn emit_one_with_actions<T: Serialize>(
    ctx: &OutputContext,
    resource: &'static str,
    operation: &'static str,
    item: &T,
    next_actions: Vec<NextAction>,
) {
    if ctx.agent {
        emit_agent_data(resource, operation, item, Vec::new(), next_actions);
    } else {
        print_one(item, ctx.format);
    }
}

pub fn emit_one_with_warnings<T: Serialize>(
    ctx: &OutputContext,
    resource: &'static str,
    operation: &'static str,
    item: &T,
    warnings: Vec<AgentWarning>,
) {
    emit_one_with_warnings_and_actions(ctx, resource, operation, item, warnings, Vec::new());
}

pub fn emit_one_with_warnings_and_actions<T: Serialize>(
    ctx: &OutputContext,
    resource: &'static str,
    operation: &'static str,
    item: &T,
    warnings: Vec<AgentWarning>,
    next_actions: Vec<NextAction>,
) {
    if ctx.agent {
        emit_agent_data(resource, operation, item, warnings, next_actions);
    } else {
        print_one(item, ctx.format);
    }
}

pub fn emit_success(
    ctx: &OutputContext,
    resource: &'static str,
    operation: &'static str,
    message: &str,
) {
    emit_success_with_actions(ctx, resource, operation, message, Vec::new());
}

pub fn emit_success_with_actions(
    ctx: &OutputContext,
    resource: &'static str,
    operation: &'static str,
    message: &str,
    next_actions: Vec<NextAction>,
) {
    if ctx.agent {
        emit_agent_data(
            resource,
            operation,
            json!({ "message": message }),
            Vec::new(),
            next_actions,
        );
    } else {
        print_success(message);
    }
}

pub fn emit_error(
    resource: &'static str,
    operation: &'static str,
    error: AgentError,
    warnings: Vec<AgentWarning>,
    next_actions: Vec<NextAction>,
) {
    let envelope = AgentErrorEnvelope {
        aci: ACI_VERSION,
        ok: false,
        resource,
        operation,
        error,
        warnings,
        next_actions,
    };
    print_agent_json(&envelope);
}

fn emit_agent_data<T: Serialize>(
    resource: &'static str,
    operation: &'static str,
    data: T,
    warnings: Vec<AgentWarning>,
    next_actions: Vec<NextAction>,
) {
    let envelope = AgentEnvelope {
        aci: ACI_VERSION,
        ok: true,
        resource,
        operation,
        data,
        warnings,
        next_actions,
    };
    print_agent_json(&envelope);
}

fn print_agent_json<T: Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            let envelope = AgentErrorEnvelope {
                aci: ACI_VERSION,
                ok: false,
                resource: "cli",
                operation: "serialize",
                error: AgentError::new(
                    "serialization_error",
                    format!("Failed to serialize output: {error}"),
                ),
                warnings: Vec::new(),
                next_actions: Vec::new(),
            };
            let json = serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| {
                r#"{"aci":"0.1","ok":false,"resource":"cli","operation":"serialize","error":{"code":"serialization_error","message":"Failed to serialize output","retryable":false},"warnings":[],"next_actions":[]}"#.to_string()
            });
            println!("{json}");
        }
    }
}
