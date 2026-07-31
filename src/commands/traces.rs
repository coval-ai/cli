use anyhow::{Context, Result};
use clap::{Args, Subcommand, ValueEnum};

use crate::client::models::{TraceSearchAttributeFilter, TraceSearchRequest};
use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::{emit_one_with_actions, print_list, NextAction, OutputContext, OutputFormat};

#[derive(Subcommand)]
pub enum TraceCommands {
    Context,
    Search(Box<SearchArgs>),
    Summary(SummaryArgs),
    Spans(SpansArgs),
}

impl TraceCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::Search(_) => "search",
            Self::Summary(_) => "summary",
            Self::Spans(_) => "spans",
        }
    }
}

#[derive(Args)]
pub struct SearchArgs {
    #[command(flatten)]
    input_json: InputJsonArg,
    /// Opaque next_cursor from a previous search response
    #[arg(long)]
    cursor: Option<String>,
    /// Results per page (1-100, default 25)
    #[arg(long, value_parser = parse_search_limit)]
    limit: Option<u32>,
    /// Include spans at or after this ISO 8601 timestamp
    #[arg(long)]
    start_date: Option<String>,
    /// Include spans at or before this ISO 8601 timestamp
    #[arg(long)]
    end_date: Option<String>,
    /// Case-insensitive substring match on span name
    #[arg(long)]
    span_name: Option<String>,
    /// Case-insensitive substring match on provider
    #[arg(long)]
    provider: Option<String>,
    /// Match span status
    #[arg(long, value_enum)]
    status: Option<TraceStatusArg>,
    /// Attribute filter as KEY:OPERATOR[:VALUE]; repeat up to 10 times
    #[arg(long = "attribute-filter")]
    attribute_filters: Vec<String>,
    /// Minimum span duration in milliseconds
    #[arg(long, value_parser = parse_nonnegative_f64)]
    duration_ms_min: Option<f64>,
    /// Maximum span duration in milliseconds
    #[arg(long, value_parser = parse_nonnegative_f64)]
    duration_ms_max: Option<f64>,
    /// Result ordering
    #[arg(long, value_enum)]
    sort_by: Option<TraceSortArg>,
    /// Restrict results to one agent
    #[arg(long)]
    agent_id: Option<String>,
    /// Restrict results to one test set
    #[arg(long)]
    test_set_id: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TraceStatusArg {
    Error,
    Ok,
    Unset,
}

impl TraceStatusArg {
    fn api_value(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Ok => "OK",
            Self::Unset => "UNSET",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TraceSortArg {
    Newest,
    Oldest,
    Slowest,
    Fastest,
}

impl TraceSortArg {
    fn api_value(self) -> &'static str {
        match self {
            Self::Newest => "newest",
            Self::Oldest => "oldest",
            Self::Slowest => "slowest",
            Self::Fastest => "fastest",
        }
    }
}

#[derive(Args)]
pub struct SummaryArgs {
    /// Simulation output to summarize
    #[arg(
        long,
        required_unless_present = "conversation_id",
        conflicts_with = "conversation_id"
    )]
    simulation_id: Option<String>,
    /// Monitoring conversation to summarize
    #[arg(
        long,
        required_unless_present = "simulation_id",
        conflicts_with = "simulation_id"
    )]
    conversation_id: Option<String>,
}

#[derive(Args)]
pub struct SpansArgs {
    /// Simulation output whose raw spans should be returned
    simulation_output_id: String,
    /// Maximum spans to return (1-200, default 50)
    #[arg(long, value_parser = parse_spans_limit)]
    limit: Option<u32>,
    /// Number of spans to skip (0-100000, default 0)
    #[arg(long, value_parser = parse_spans_offset)]
    offset: Option<u32>,
}

pub async fn execute(
    command: TraceCommands,
    client: &CovalClient,
    ctx: &OutputContext,
) -> Result<()> {
    let operation = command.operation();
    match command {
        TraceCommands::Context => return crate::commands::agent::resource_context("traces", ctx),
        TraceCommands::Search(args) => {
            let request = build_search_request(*args)?;
            let response = client.traces().search(request).await?;
            let actions = search_actions(
                response
                    .items
                    .first()
                    .map(|item| item.simulation_output_id.as_str()),
            );
            if ctx.human() {
                print_list(&response.items, OutputFormat::Table);
                println!(
                    "Matching calls: {} ({} errors, {}%)",
                    response.total_count,
                    response.aggregate_stats.error_count,
                    response.aggregate_stats.error_rate
                );
                if let Some(cursor) = &response.next_cursor {
                    println!("Next cursor: {cursor}");
                }
            } else {
                emit_one_with_actions(ctx, "traces", operation, &response, actions);
            }
        }
        TraceCommands::Summary(args) => {
            let response = client
                .traces()
                .summary(
                    args.simulation_id.as_deref(),
                    args.conversation_id.as_deref(),
                )
                .await?;
            emit_one_with_actions(
                ctx,
                "traces",
                operation,
                &response,
                vec![next_actions::context("traces")],
            );
        }
        TraceCommands::Spans(args) => {
            let response = client
                .traces()
                .spans(&args.simulation_output_id, args.limit, args.offset)
                .await?;
            emit_one_with_actions(
                ctx,
                "traces",
                operation,
                &response,
                vec![next_actions::context("traces")],
            );
        }
    }
    Ok(())
}

fn build_search_request(args: SearchArgs) -> Result<TraceSearchRequest> {
    let mut input = args.input_json.object()?;
    input_json::insert(&mut input, "cursor", args.cursor)?;
    input_json::insert(&mut input, "limit", args.limit)?;

    let existing_filters = input
        .remove("filters")
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    let mut filters = existing_filters
        .as_object()
        .cloned()
        .context("filters in --input-json must be a JSON object")?;

    input_json::insert(&mut filters, "start_date", args.start_date)?;
    input_json::insert(&mut filters, "end_date", args.end_date)?;
    input_json::insert(&mut filters, "span_name", args.span_name)?;
    input_json::insert(&mut filters, "provider", args.provider)?;
    input_json::insert(
        &mut filters,
        "status",
        args.status.map(|value| value.api_value().to_string()),
    )?;
    if !args.attribute_filters.is_empty() {
        let parsed = args
            .attribute_filters
            .iter()
            .map(|raw| parse_attribute_filter(raw))
            .collect::<Result<Vec<_>>>()?;
        input_json::insert(&mut filters, "attribute_filters", Some(parsed))?;
    }
    input_json::insert(&mut filters, "duration_ms_min", args.duration_ms_min)?;
    input_json::insert(&mut filters, "duration_ms_max", args.duration_ms_max)?;
    input_json::insert(
        &mut filters,
        "sort_by",
        args.sort_by.map(|value| value.api_value().to_string()),
    )?;
    input_json::insert(&mut filters, "agent_id", args.agent_id)?;
    input_json::insert(&mut filters, "test_set_id", args.test_set_id)?;
    if !filters.is_empty() {
        input.insert("filters".to_string(), serde_json::Value::Object(filters));
    }
    let request: TraceSearchRequest = input_json::finish(input)?;
    if let (Some(minimum), Some(maximum)) = (
        request.filters.duration_ms_min,
        request.filters.duration_ms_max,
    ) {
        anyhow::ensure!(
            minimum <= maximum,
            "trace search duration minimum ({minimum}) must not exceed maximum ({maximum})"
        );
    }
    if let Some(attribute_filters) = &request.filters.attribute_filters {
        anyhow::ensure!(
            attribute_filters.len() <= 10,
            "trace search accepts at most 10 attribute filters, got {}",
            attribute_filters.len()
        );
    }
    Ok(request)
}

fn parse_attribute_filter(raw: &str) -> Result<TraceSearchAttributeFilter> {
    let mut parts = raw.splitn(3, ':');
    let key = parts.next().unwrap_or_default().trim();
    let operator = parts.next().unwrap_or_default().trim();
    let value = parts
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if key.is_empty() || operator.is_empty() {
        anyhow::bail!("--attribute-filter must use KEY:OPERATOR[:VALUE]");
    }
    if !matches!(
        operator,
        "contains" | "eq" | "exists" | "gt" | "gte" | "lt" | "lte"
    ) {
        anyhow::bail!(
            "invalid attribute operator '{operator}'; use contains, eq, exists, gt, gte, lt, or lte"
        );
    }
    if operator == "exists" && value.is_some() {
        anyhow::bail!("the exists attribute operator does not accept a value");
    }
    if operator != "exists" && value.is_none() {
        anyhow::bail!("the {operator} attribute operator requires a value");
    }

    Ok(TraceSearchAttributeFilter {
        key: key.to_string(),
        operator: operator.to_string(),
        value: value.map(str::to_string),
    })
}

fn search_actions(simulation_output_id: Option<&str>) -> Vec<NextAction> {
    let mut actions = vec![next_actions::context("traces")];
    if let Some(simulation_output_id) = simulation_output_id {
        actions.insert(
            0,
            NextAction::new(
                "traces.spans",
                "Inspect matching call spans",
                next_actions::argv(["traces", "spans", simulation_output_id]),
                true,
            )
            .primary(),
        );
    }
    actions
}

fn parse_search_limit(raw: &str) -> std::result::Result<u32, String> {
    parse_bounded_u32(raw, 1, 100, "search limit")
}

fn parse_spans_limit(raw: &str) -> std::result::Result<u32, String> {
    parse_bounded_u32(raw, 1, 200, "spans limit")
}

fn parse_spans_offset(raw: &str) -> std::result::Result<u32, String> {
    parse_bounded_u32(raw, 0, 100_000, "spans offset")
}

fn parse_bounded_u32(
    raw: &str,
    minimum: u32,
    maximum: u32,
    label: &str,
) -> std::result::Result<u32, String> {
    let value = raw
        .parse::<u32>()
        .map_err(|_| format!("{label} must be an integer"))?;
    if !(minimum..=maximum).contains(&value) {
        return Err(format!("{label} must be between {minimum} and {maximum}"));
    }
    Ok(value)
}

fn parse_nonnegative_f64(raw: &str) -> std::result::Result<f64, String> {
    let value = raw
        .parse::<f64>()
        .map_err(|_| "duration must be a number".to_string())?;
    if !value.is_finite() || value < 0.0 {
        return Err("duration must be a finite non-negative number".to_string());
    }
    Ok(value)
}
