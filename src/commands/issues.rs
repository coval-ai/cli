//! `coval issues` — the agent improvement loop from the terminal.
//!
//! Mirrors the /v1/issues lifecycle one-to-one so scripts and agents see the same statuses
//! and the same actionable errors as the UI ("Clearance incomplete: 842 of 1,000 ...").

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::{emit_list_with_actions, emit_one_with_actions, OutputContext};

#[derive(Subcommand)]
pub enum IssueCommands {
    Context,
    /// List the issue board; filter with --status, --mine, --agent-id.
    List(ListArgs),
    /// Show one issue with its activity log and clearance attempts.
    Get(GetArgs),
    /// Create an issue, normally from a Sofia finding (--finding-id).
    Create(CreateArgs),
    /// Confirm a needs_review issue and (optionally) set severity / owner.
    Confirm(ConfirmArgs),
    /// Assign an owner.
    Assign(AssignArgs),
    /// Attach the focused test set that asks "did we fix this?".
    AttachSuite(AttachSuiteArgs),
    /// Freeze the clearance tolerance for future attempts.
    SetPolicy(SetPolicyArgs),
    /// Judge a completed run of the attached suite against the frozen policy.
    Clear(ClearArgs),
    /// Any other lifecycle action: record_change, close_manual, dismiss, merge, reopen.
    Action(ActionArgs),
    /// Improvement summary: review / open / resolved counts and resolution history.
    Summary(SummaryArgs),
    /// The regression baseline (suites protecting an agent's expected behavior).
    Regression(RegressionArgs),
}

impl IssueCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::List(_) => "list",
            Self::Get(_) => "get",
            Self::Create(_) => "create",
            Self::Confirm(_) => "confirm",
            Self::Assign(_) => "assign",
            Self::AttachSuite(_) => "attach_suite",
            Self::SetPolicy(_) => "set_clearance_policy",
            Self::Clear(_) => "start_clearance",
            Self::Action(_) => "action",
            Self::Summary(_) => "summary",
            Self::Regression(_) => "regression",
        }
    }
}

#[derive(Args)]
pub struct ListArgs {
    /// Comma-separated statuses, e.g. needs_review,confirmed,clearing
    #[arg(long)]
    status: Option<String>,
    /// Only issues assigned to the calling user.
    #[arg(long)]
    mine: bool,
    #[arg(long)]
    agent_id: Option<String>,
}

#[derive(Args)]
pub struct GetArgs {
    issue_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    #[command(flatten)]
    input_json: InputJsonArg,
    #[arg(long)]
    title: Option<String>,
    #[arg(long)]
    finding_id: Option<String>,
    #[arg(long)]
    agent_id: Option<String>,
    #[arg(long)]
    severity: Option<String>,
}

#[derive(Args)]
pub struct ConfirmArgs {
    issue_id: String,
    #[arg(long)]
    expected_version: u32,
    #[arg(long)]
    severity: Option<String>,
    #[arg(long)]
    owner_user_id: Option<String>,
}

#[derive(Args)]
pub struct AssignArgs {
    issue_id: String,
    #[arg(long)]
    expected_version: u32,
    #[arg(long)]
    owner_user_id: String,
}

#[derive(Args)]
pub struct AttachSuiteArgs {
    issue_id: String,
    #[arg(long)]
    expected_version: u32,
    #[arg(long)]
    test_set_id: String,
}

#[derive(Args)]
pub struct SetPolicyArgs {
    issue_id: String,
    #[arg(long)]
    expected_version: u32,
    /// e.g. 0.98
    #[arg(long)]
    success_rate_min: f64,
    /// e.g. 1000
    #[arg(long)]
    min_valid_simulations: u32,
    /// Comma-separated metric ids that must pass on every counted simulation.
    #[arg(long)]
    required_metric_ids: String,
    /// Comma-separated test case ids that must each be covered.
    #[arg(long)]
    required_test_case_ids: Option<String>,
}

#[derive(Args)]
pub struct ClearArgs {
    issue_id: String,
    #[arg(long)]
    expected_version: u32,
    /// A completed run of the attached suite (launch it with `coval runs launch`).
    #[arg(long)]
    run_id: String,
}

#[derive(Args)]
pub struct ActionArgs {
    issue_id: String,
    #[arg(long)]
    action: String,
    #[arg(long)]
    expected_version: u32,
    #[command(flatten)]
    input_json: InputJsonArg,
    #[arg(long)]
    note: Option<String>,
}

#[derive(Args)]
pub struct SummaryArgs {
    /// week | month | quarter
    #[arg(long, default_value = "week")]
    period: String,
    #[arg(long, default_value_t = 8)]
    buckets: u32,
}

#[derive(Args)]
pub struct RegressionArgs {
    #[arg(long)]
    agent_id: Option<String>,
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

pub async fn execute(cmd: IssueCommands, client: &CovalClient, ctx: &OutputContext) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        IssueCommands::Context => {
            return crate::commands::agent::resource_context("issues", ctx);
        }
        IssueCommands::List(args) => {
            let response = client
                .issues()
                .list(args.status.as_deref(), args.mine, args.agent_id.as_deref())
                .await?;
            emit_list_with_actions(
                ctx,
                "issues",
                operation,
                &response.issues,
                next_actions::list_result("issues", response.issues.first().map(|i| i.id.as_str())),
            );
        }
        IssueCommands::Get(args) => {
            let detail = client.issues().get(&args.issue_id).await?;
            emit_one_with_actions(
                ctx,
                "issues",
                operation,
                &detail,
                next_actions::item_result("issues", &detail.issue.id),
            );
        }
        IssueCommands::Create(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "title", args.title)?;
            input_json::insert(&mut input, "finding_id", args.finding_id)?;
            input_json::insert(&mut input, "agent_id", args.agent_id)?;
            input_json::insert(&mut input, "severity", args.severity)?;
            let response = client.issues().create(input_json::finish(input)?).await?;
            emit_one_with_actions(
                ctx,
                "issues",
                operation,
                &response,
                next_actions::item_result("issues", &response.issue.id),
            );
        }
        IssueCommands::Confirm(args) => {
            let body = serde_json::json!({
                "action": "confirm",
                "expected_version": args.expected_version,
                "severity": args.severity,
                "owner_user_id": args.owner_user_id,
            });
            emit_action(client, ctx, operation, &args.issue_id, body).await?;
        }
        IssueCommands::Assign(args) => {
            let body = serde_json::json!({
                "action": "assign",
                "expected_version": args.expected_version,
                "owner_user_id": args.owner_user_id,
            });
            emit_action(client, ctx, operation, &args.issue_id, body).await?;
        }
        IssueCommands::AttachSuite(args) => {
            let body = serde_json::json!({
                "action": "attach_suite",
                "expected_version": args.expected_version,
                "suite_test_set_id": args.test_set_id,
            });
            emit_action(client, ctx, operation, &args.issue_id, body).await?;
        }
        IssueCommands::SetPolicy(args) => {
            let body = serde_json::json!({
                "action": "set_clearance_policy",
                "expected_version": args.expected_version,
                "clearance_policy": {
                    "success_rate_min": args.success_rate_min,
                    "min_valid_simulations": args.min_valid_simulations,
                    "required_metric_ids": split_csv(&args.required_metric_ids),
                    "required_test_case_ids": args.required_test_case_ids.as_deref().map(split_csv).unwrap_or_default(),
                },
            });
            emit_action(client, ctx, operation, &args.issue_id, body).await?;
        }
        IssueCommands::Clear(args) => {
            let body = serde_json::json!({
                "action": "start_clearance",
                "expected_version": args.expected_version,
                "run_id": args.run_id,
            });
            emit_action(client, ctx, operation, &args.issue_id, body).await?;
        }
        IssueCommands::Action(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "action", Some(args.action))?;
            input_json::insert(&mut input, "note", args.note)?;
            input.insert(
                "expected_version".to_string(),
                serde_json::Value::from(args.expected_version),
            );
            emit_action(
                client,
                ctx,
                operation,
                &args.issue_id,
                input_json::finish(input)?,
            )
            .await?;
        }
        IssueCommands::Summary(args) => {
            let summary = client.issues().summary(&args.period, args.buckets).await?;
            emit_one_with_actions(
                ctx,
                "issues",
                operation,
                &summary,
                next_actions::list_result("issues", None),
            );
        }
        IssueCommands::Regression(args) => {
            let response = client
                .issues()
                .regression_suite(args.agent_id.as_deref())
                .await?;
            emit_list_with_actions(
                ctx,
                "issues",
                operation,
                &response.memberships,
                next_actions::list_result("issues", None),
            );
        }
    }
    Ok(())
}

async fn emit_action(
    client: &CovalClient,
    ctx: &OutputContext,
    operation: &'static str,
    issue_id: &str,
    body: serde_json::Value,
) -> Result<()> {
    let response = client.issues().action(issue_id, &body).await?;
    emit_one_with_actions(
        ctx,
        "issues",
        operation,
        &response,
        next_actions::item_result("issues", &response.issue.id),
    );
    Ok(())
}
