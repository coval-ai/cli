use std::io::{self, BufRead};

use anyhow::Result;
use clap::{Args, Subcommand};
use serde::Deserialize;
use serde_json::json;

use crate::client::models::{CreateTestCaseRequest, ListParams, UpdateTestCaseRequest};
use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, NextAction,
    OutputContext,
};

#[derive(Subcommand)]
pub enum TestCaseCommands {
    Context,
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    #[command(name = "media-upload-url")]
    MediaUploadUrl(MediaUploadUrlArgs),
}

impl TestCaseCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::List(_) => "list",
            Self::Get(_) => "get",
            Self::Create(_) => "create",
            Self::Update(_) => "update",
            Self::Delete(_) => "delete",
            Self::MediaUploadUrl(_) => "media-upload-url",
        }
    }
}

#[derive(Args)]
pub struct ListArgs {
    /// Filter expression (e.g. test_set_id=abc12345)
    #[arg(long)]
    filter: Option<String>,
    /// Filter by test set ID (8-char ID)
    #[arg(long)]
    test_set_id: Option<String>,
    /// Results per page (1-100, default 50)
    #[arg(long, default_value = "50")]
    page_size: u32,
    /// Sort field, prefix with - for descending (default: -create_time)
    #[arg(long)]
    order_by: Option<String>,
}

#[derive(Args)]
pub struct GetArgs {
    test_case_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    #[command(flatten)]
    input_json: InputJsonArg,
    /// Test set to add this case to (8-char ID)
    #[arg(long)]
    test_set_id: Option<String>,
    /// Test case input text
    #[arg(long)]
    input: Option<String>,
    /// Single expected output text (deprecated; prefer --expected-behavior)
    #[arg(long)]
    expected: Option<String>,
    /// Expected behavior; repeat the flag for multiple behaviors
    #[arg(long = "expected-behavior")]
    expected_behavior: Vec<String>,
    /// Human-readable description
    #[arg(long)]
    description: Option<String>,
    /// Input type (default SCENARIO)
    #[arg(long)]
    input_type: Option<String>,
    /// Read JSON lines from stdin for bulk creation
    #[arg(long, conflicts_with_all = ["input", "expected", "expected_behavior", "description", "input_type"])]
    stdin: bool,
}

#[derive(Deserialize)]
struct StdinTestCase {
    input_str: String,
    #[serde(default)]
    expected_output_str: Option<String>,
    #[serde(default)]
    expected_behaviors: Option<Vec<String>>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    input_type: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    test_case_id: String,
    #[command(flatten)]
    input_json: InputJsonArg,
    /// Updated test case input text
    #[arg(long)]
    input: Option<String>,
    /// Updated single expected output text (deprecated; prefer --expected-behavior)
    #[arg(long)]
    expected: Option<String>,
    /// Updated expected behavior; repeat the flag to replace with multiple behaviors
    #[arg(long = "expected-behavior")]
    expected_behavior: Vec<String>,
    /// Updated description
    #[arg(long)]
    description: Option<String>,
    /// Updated input type
    #[arg(long)]
    input_type: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    test_case_id: String,
}

#[derive(Args)]
pub struct MediaUploadUrlArgs {
    test_case_id: String,
    #[arg(long)]
    filename: String,
    #[arg(long, default_value = "image/png")]
    mime_type: String,
}

pub async fn execute(
    cmd: TestCaseCommands,
    client: &CovalClient,
    ctx: &OutputContext,
) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        TestCaseCommands::Context => {
            return crate::commands::agent::resource_context("test-cases", ctx);
        }
        TestCaseCommands::List(args) => {
            let filter = match (args.filter, args.test_set_id) {
                (Some(f), Some(ts)) => Some(format!("{f} AND test_set_id=\"{ts}\"")),
                (Some(f), None) => Some(f),
                (None, Some(ts)) => Some(format!("test_set_id=\"{ts}\"")),
                (None, None) => None,
            };

            let params = ListParams {
                filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.test_cases().list(params).await?;
            emit_list_with_actions(
                ctx,
                "test-cases",
                operation,
                &response.test_cases,
                list_actions(
                    response
                        .test_cases
                        .first()
                        .map(|test_case| test_case.id.as_str()),
                ),
            );
        }
        TestCaseCommands::Get(args) => {
            let test_case = client.test_cases().get(&args.test_case_id).await?;
            emit_one_with_actions(
                ctx,
                "test-cases",
                operation,
                &test_case,
                item_actions(&test_case.id, test_case.test_set_id.as_deref()),
            );
        }
        TestCaseCommands::Create(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "test_set_id", args.test_set_id)?;
            if args.stdin {
                let test_set_id = input
                    .get("test_set_id")
                    .and_then(|value| value.as_str())
                    .ok_or_else(|| anyhow::anyhow!("missing field `test_set_id`"))?
                    .to_string();
                let mut created = 0;
                let mut failed = 0;

                for line in io::stdin().lock().lines() {
                    let line = line?;
                    if line.trim().is_empty() {
                        continue;
                    }

                    let tc: StdinTestCase = serde_json::from_str(&line)?;
                    let req = CreateTestCaseRequest {
                        test_set_id: test_set_id.clone(),
                        input_str: tc.input_str,
                        expected_output_str: tc.expected_output_str,
                        description: tc.description,
                        expected_behaviors: tc.expected_behaviors,
                        expected_output_json: None,
                        input_type: tc.input_type,
                        simulation_metadata_input: None,
                        metric_input: None,
                        user_notes: None,
                    };

                    match client.test_cases().create(req).await {
                        Ok(_) => created += 1,
                        Err(e) => {
                            if !ctx.agent {
                                eprintln!("Error: {e}");
                            }
                            failed += 1;
                        }
                    }
                }

                if ctx.human() {
                    println!("Created {created} test cases ({failed} failed)");
                } else {
                    emit_one_with_actions(
                        ctx,
                        "test-cases",
                        operation,
                        &json!({ "created": created, "failed": failed }),
                        vec![
                            next_actions::list("test-cases").primary(),
                            next_actions::context("test-cases"),
                        ],
                    );
                }
            } else {
                input_json::insert(&mut input, "input_str", args.input)?;
                input_json::insert(&mut input, "expected_output_str", args.expected)?;
                input_json::insert(
                    &mut input,
                    "expected_behaviors",
                    vec_to_option(args.expected_behavior),
                )?;
                input_json::insert(&mut input, "description", args.description)?;
                input_json::insert(&mut input, "input_type", args.input_type)?;
                let req: CreateTestCaseRequest = input_json::finish(input)?;
                let test_case = client.test_cases().create(req).await?;
                emit_one_with_actions(
                    ctx,
                    "test-cases",
                    operation,
                    &test_case,
                    item_actions(&test_case.id, test_case.test_set_id.as_deref()),
                );
            }
        }
        TestCaseCommands::Update(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "input_str", args.input)?;
            input_json::insert(&mut input, "expected_output_str", args.expected)?;
            input_json::insert(
                &mut input,
                "expected_behaviors",
                vec_to_option(args.expected_behavior),
            )?;
            input_json::insert(&mut input, "description", args.description)?;
            input_json::insert(&mut input, "input_type", args.input_type)?;
            let req: UpdateTestCaseRequest = input_json::finish(input)?;
            let test_case = client.test_cases().update(&args.test_case_id, req).await?;
            emit_one_with_actions(
                ctx,
                "test-cases",
                operation,
                &test_case,
                item_actions(&test_case.id, test_case.test_set_id.as_deref()),
            );
        }
        TestCaseCommands::Delete(args) => {
            client.test_cases().delete(&args.test_case_id).await?;
            emit_success_with_actions(
                ctx,
                "test-cases",
                operation,
                "Test case deleted.",
                vec![
                    next_actions::list("test-cases").primary(),
                    next_actions::context("test-cases"),
                ],
            );
        }
        TestCaseCommands::MediaUploadUrl(args) => {
            let result = client
                .test_cases()
                .media_upload_url(&args.test_case_id, &args.filename, &args.mime_type)
                .await?;
            emit_one_with_actions(ctx, "test-cases", operation, &result, vec![]);
        }
    }
    Ok(())
}

fn list_actions(id: Option<&str>) -> Vec<NextAction> {
    let mut actions = vec![next_actions::context("test-cases")];
    if let Some(id) = id {
        actions.insert(0, next_actions::get("test-cases", id).primary());
    }
    actions
}

fn vec_to_option(values: Vec<String>) -> Option<Vec<String>> {
    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

fn item_actions(test_case_id: &str, test_set_id: Option<&str>) -> Vec<NextAction> {
    let mut actions = vec![
        next_actions::get("test-cases", test_case_id).primary(),
        next_actions::context("test-cases"),
    ];
    if let Some(test_set_id) = test_set_id {
        actions.insert(1, next_actions::test_cases_for_set(test_set_id));
    }
    actions
}
