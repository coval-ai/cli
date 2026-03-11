use std::io::{self, BufRead};

use anyhow::Result;
use clap::{Args, Subcommand};
use serde::Deserialize;

use crate::client::models::{CreateTestCaseRequest, ListParams, UpdateTestCaseRequest};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum TestCaseCommands {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
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
    /// Test set to add this case to (8-char ID)
    #[arg(long)]
    test_set_id: String,
    /// Test case input text
    #[arg(long, required_unless_present = "stdin")]
    input: Option<String>,
    /// Expected output text
    #[arg(long)]
    expected: Option<String>,
    /// Human-readable description
    #[arg(long)]
    description: Option<String>,
    /// Read JSON lines from stdin for bulk creation
    #[arg(long, conflicts_with_all = ["input", "expected", "description"])]
    stdin: bool,
}

#[derive(Deserialize)]
struct StdinTestCase {
    input_str: String,
    #[serde(default)]
    expected_output_str: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    test_case_id: String,
    /// Updated test case input text
    #[arg(long)]
    input: Option<String>,
    /// Updated expected output text
    #[arg(long)]
    expected: Option<String>,
    /// Updated description
    #[arg(long)]
    description: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    test_case_id: String,
}

pub async fn execute(
    cmd: TestCaseCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
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
            print_list(&response.test_cases, format);
        }
        TestCaseCommands::Get(args) => {
            let test_case = client.test_cases().get(&args.test_case_id).await?;
            print_one(&test_case, format);
        }
        TestCaseCommands::Create(args) => {
            if args.stdin {
                let mut created = 0;
                let mut failed = 0;

                for line in io::stdin().lock().lines() {
                    let line = line?;
                    if line.trim().is_empty() {
                        continue;
                    }

                    let tc: StdinTestCase = serde_json::from_str(&line)?;
                    let req = CreateTestCaseRequest {
                        test_set_id: args.test_set_id.clone(),
                        input_str: tc.input_str,
                        expected_output_str: tc.expected_output_str,
                        description: tc.description,
                        expected_behaviors: None,
                        expected_output_json: None,
                        input_type: None,
                        simulation_metadata_input: None,
                        metric_input: None,
                        user_notes: None,
                    };

                    match client.test_cases().create(req).await {
                        Ok(_) => created += 1,
                        Err(e) => {
                            eprintln!("Error: {e}");
                            failed += 1;
                        }
                    }
                }

                println!("Created {created} test cases ({failed} failed)");
            } else {
                let req = CreateTestCaseRequest {
                    test_set_id: args.test_set_id,
                    input_str: args.input.unwrap(),
                    expected_output_str: args.expected,
                    description: args.description,
                    expected_behaviors: None,
                    expected_output_json: None,
                    input_type: None,
                    simulation_metadata_input: None,
                    metric_input: None,
                    user_notes: None,
                };
                let test_case = client.test_cases().create(req).await?;
                print_one(&test_case, format);
            }
        }
        TestCaseCommands::Update(args) => {
            let req = UpdateTestCaseRequest {
                input_str: args.input,
                expected_output_str: args.expected,
                description: args.description,
                ..Default::default()
            };
            let test_case = client.test_cases().update(&args.test_case_id, req).await?;
            print_one(&test_case, format);
        }
        TestCaseCommands::Delete(args) => {
            client.test_cases().delete(&args.test_case_id).await?;
            print_success("Test case deleted.");
        }
    }
    Ok(())
}
