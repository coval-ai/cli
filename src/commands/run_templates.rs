use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{CreateRunTemplateRequest, ListParams, UpdateRunTemplateRequest};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum RunTemplateCommands {
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(long)]
    filter: Option<String>,
    #[arg(long, default_value = "50")]
    page_size: u32,
    #[arg(long)]
    order_by: Option<String>,
}

#[derive(Args)]
pub struct GetArgs {
    run_template_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    #[arg(long)]
    name: String,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    agent_id: Option<String>,
    #[arg(long)]
    persona_id: Option<String>,
    #[arg(long)]
    test_set_id: Option<String>,
    #[arg(long, value_delimiter = ',')]
    metric_ids: Option<Vec<String>>,
    #[arg(long, value_delimiter = ',')]
    mutation_ids: Option<Vec<String>>,
    #[arg(long)]
    iteration_count: Option<u32>,
    #[arg(long)]
    concurrency: Option<u32>,
    #[arg(long)]
    sub_sample_size: Option<u32>,
    #[arg(long)]
    sub_sample_seed: Option<u64>,
}

#[derive(Args)]
pub struct UpdateArgs {
    run_template_id: String,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    agent_id: Option<String>,
    #[arg(long)]
    persona_id: Option<String>,
    #[arg(long)]
    test_set_id: Option<String>,
    #[arg(long, value_delimiter = ',')]
    metric_ids: Option<Vec<String>>,
    #[arg(long, value_delimiter = ',')]
    mutation_ids: Option<Vec<String>>,
    #[arg(long)]
    iteration_count: Option<u32>,
    #[arg(long)]
    concurrency: Option<u32>,
    #[arg(long)]
    sub_sample_size: Option<u32>,
    #[arg(long)]
    sub_sample_seed: Option<u64>,
}

#[derive(Args)]
pub struct DeleteArgs {
    run_template_id: String,
}

pub async fn execute(
    cmd: RunTemplateCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        RunTemplateCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.run_templates().list(params).await?;
            print_list(&response.run_templates, format);
        }
        RunTemplateCommands::Get(args) => {
            let template = client.run_templates().get(&args.run_template_id).await?;
            print_one(&template, format);
        }
        RunTemplateCommands::Create(args) => {
            let req = CreateRunTemplateRequest {
                display_name: args.name,
                description: args.description,
                agent_id: args.agent_id,
                persona_id: args.persona_id,
                test_set_id: args.test_set_id,
                metric_ids: args.metric_ids,
                mutation_ids: args.mutation_ids,
                iteration_count: args.iteration_count,
                concurrency: args.concurrency,
                sub_sample_size: args.sub_sample_size,
                sub_sample_seed: args.sub_sample_seed,
                metadata: None,
            };
            let template = client.run_templates().create(req).await?;
            print_one(&template, format);
        }
        RunTemplateCommands::Update(args) => {
            let req = UpdateRunTemplateRequest {
                display_name: args.name,
                description: args.description,
                agent_id: args.agent_id,
                persona_id: args.persona_id,
                test_set_id: args.test_set_id,
                metric_ids: args.metric_ids,
                mutation_ids: args.mutation_ids,
                iteration_count: args.iteration_count,
                concurrency: args.concurrency,
                sub_sample_size: args.sub_sample_size,
                sub_sample_seed: args.sub_sample_seed,
                ..Default::default()
            };
            let template = client
                .run_templates()
                .update(&args.run_template_id, req)
                .await?;
            print_one(&template, format);
        }
        RunTemplateCommands::Delete(args) => {
            client.run_templates().delete(&args.run_template_id).await?;
            print_success("Run template deleted.");
        }
    }
    Ok(())
}
