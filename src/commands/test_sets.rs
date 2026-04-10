use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{CreateTestSetRequest, ListParams, UpdateTestSetRequest};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum TestSetCommands {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

#[derive(Args)]
pub struct ListArgs {
    /// Filter expression (e.g. test_set_type=SCENARIO)
    #[arg(long)]
    filter: Option<String>,
    /// Results per page (1-100, default 50)
    #[arg(long, default_value = "50")]
    page_size: u32,
    /// Sort field, prefix with - for descending (default: -update_time)
    #[arg(long)]
    order_by: Option<String>,
}

#[derive(Args)]
pub struct GetArgs {
    test_set_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    /// Test set name (1-100 characters)
    #[arg(long)]
    name: String,
    /// URL-friendly identifier; auto-generated if omitted
    #[arg(long)]
    slug: Option<String>,
    /// Human-readable description
    #[arg(long)]
    description: Option<String>,
    /// Test set type (e.g. DEFAULT, SCENARIO, TRANSCRIPT, WORKFLOW)
    #[arg(long)]
    r#type: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    test_set_id: String,
    /// Test set name (1-100 characters)
    #[arg(long)]
    name: Option<String>,
    /// URL-friendly identifier
    #[arg(long)]
    slug: Option<String>,
    /// Human-readable description
    #[arg(long)]
    description: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    test_set_id: String,
}

pub async fn execute(
    cmd: TestSetCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        TestSetCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.test_sets().list(params).await?;
            print_list(&response.test_sets, format);
        }
        TestSetCommands::Get(args) => {
            let test_set = client.test_sets().get(&args.test_set_id).await?;
            print_one(&test_set, format);
        }
        TestSetCommands::Create(args) => {
            let req = CreateTestSetRequest {
                display_name: args.name,
                slug: args.slug,
                description: args.description,
                test_set_type: args.r#type,
                test_set_metadata: None,
                parameters: None,
            };
            let test_set = client.test_sets().create(req).await?;
            print_one(&test_set, format);
        }
        TestSetCommands::Update(args) => {
            let req = UpdateTestSetRequest {
                display_name: args.name,
                slug: args.slug,
                description: args.description,
                ..Default::default()
            };
            let test_set = client.test_sets().update(&args.test_set_id, req).await?;
            print_one(&test_set, format);
        }
        TestSetCommands::Delete(args) => {
            client.test_sets().delete(&args.test_set_id).await?;
            print_success("Test set deleted.");
        }
    }
    Ok(())
}
