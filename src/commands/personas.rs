use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{CreatePersonaRequest, ListParams, UpdatePersonaRequest};
use crate::client::CovalClient;
use crate::output::{print_list, print_one, print_success, OutputFormat};

#[derive(Subcommand)]
pub enum PersonaCommands {
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
    persona_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    #[arg(long)]
    name: String,
    #[arg(long)]
    voice: String,
    #[arg(long)]
    language: String,
    #[arg(long)]
    prompt: Option<String>,
    #[arg(long)]
    background: Option<String>,
    #[arg(long)]
    wait_seconds: Option<f32>,
}

#[derive(Args)]
pub struct UpdateArgs {
    persona_id: String,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    voice: Option<String>,
    #[arg(long)]
    language: Option<String>,
    #[arg(long)]
    prompt: Option<String>,
    #[arg(long)]
    background: Option<String>,
    #[arg(long)]
    wait_seconds: Option<f32>,
}

#[derive(Args)]
pub struct DeleteArgs {
    persona_id: String,
}

pub async fn execute(
    cmd: PersonaCommands,
    client: &CovalClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd {
        PersonaCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.personas().list(params).await?;
            print_list(&response.personas, format);
        }
        PersonaCommands::Get(args) => {
            let persona = client.personas().get(&args.persona_id).await?;
            print_one(&persona, format);
        }
        PersonaCommands::Create(args) => {
            let req = CreatePersonaRequest {
                name: args.name,
                voice_name: args.voice,
                language_code: args.language,
                persona_prompt: args.prompt,
                background_sound: args.background,
                wait_seconds: args.wait_seconds,
                conversation_initiation: None,
            };
            let persona = client.personas().create(req).await?;
            print_one(&persona, format);
        }
        PersonaCommands::Update(args) => {
            let req = UpdatePersonaRequest {
                name: args.name,
                voice_name: args.voice,
                language_code: args.language,
                persona_prompt: args.prompt,
                background_sound: args.background,
                wait_seconds: args.wait_seconds,
                ..Default::default()
            };
            let persona = client.personas().update(&args.persona_id, req).await?;
            print_one(&persona, format);
        }
        PersonaCommands::Delete(args) => {
            client.personas().delete(&args.persona_id).await?;
            print_success("Persona deleted.");
        }
    }
    Ok(())
}
