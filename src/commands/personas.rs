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
    #[command(name = "phone-numbers")]
    PhoneNumbers,
}

#[derive(Args)]
pub struct ListArgs {
    /// Filter expression (supports name, create_time, update_time)
    #[arg(long)]
    filter: Option<String>,
    /// Results per page (1-100, default 50)
    #[arg(long, default_value = "50")]
    page_size: u32,
    /// Sort field, prefix with - for descending (default: -create_time)
    #[arg(long)]
    order_by: Option<String>,
}

#[derive(Args)]
pub struct GetArgs {
    persona_id: String,
}

#[derive(Args)]
pub struct CreateArgs {
    /// Persona name (1-200 characters)
    #[arg(long)]
    name: String,
    /// Voice name for speech synthesis
    #[arg(long)]
    voice: String,
    /// Language code in BCP-47 format (e.g. en-US)
    #[arg(long)]
    language: String,
    /// Persona behavior instructions
    #[arg(long)]
    prompt: Option<String>,
    /// Background sound (e.g. office, cafe, airport)
    #[arg(long)]
    background: Option<String>,
    /// Seconds to wait before speaking (0.1-2.0)
    #[arg(long)]
    wait_seconds: Option<f32>,
}

#[derive(Args)]
pub struct UpdateArgs {
    persona_id: String,
    /// Persona name (1-200 characters)
    #[arg(long)]
    name: Option<String>,
    /// Voice name for speech synthesis
    #[arg(long)]
    voice: Option<String>,
    /// Language code in BCP-47 format (e.g. en-US)
    #[arg(long)]
    language: Option<String>,
    /// Persona behavior instructions
    #[arg(long)]
    prompt: Option<String>,
    /// Background sound (e.g. office, cafe, airport)
    #[arg(long)]
    background: Option<String>,
    /// Seconds to wait before speaking (0.1-2.0)
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
        PersonaCommands::PhoneNumbers => {
            let response = client.personas().list_phone_numbers().await?;
            print_list(&response.phone_numbers, format);
        }
    }
    Ok(())
}
