use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::models::{CreatePersonaRequest, ListParams, UpdatePersonaRequest};
use crate::client::CovalClient;
use crate::next_actions;
use crate::output::{
    emit_list_with_actions, emit_one_with_actions, emit_success_with_actions, OutputContext,
};

#[derive(Subcommand)]
pub enum PersonaCommands {
    Context,
    List(ListArgs),
    Get(GetArgs),
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
    #[command(name = "phone-numbers")]
    PhoneNumbers,
}

impl PersonaCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::List(_) => "list",
            Self::Get(_) => "get",
            Self::Create(_) => "create",
            Self::Update(_) => "update",
            Self::Delete(_) => "delete",
            Self::PhoneNumbers => "phone-numbers",
        }
    }
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
    ctx: &OutputContext,
) -> Result<()> {
    let operation = cmd.operation();
    match cmd {
        PersonaCommands::Context => {
            return crate::commands::agent::resource_context("personas", ctx)
        }
        PersonaCommands::List(args) => {
            let params = ListParams {
                filter: args.filter,
                page_size: Some(args.page_size),
                order_by: args.order_by,
                ..Default::default()
            };
            let response = client.personas().list(params).await?;
            emit_list_with_actions(
                ctx,
                "personas",
                operation,
                &response.personas,
                next_actions::list_result(
                    "personas",
                    response.personas.first().map(|persona| persona.id.as_str()),
                ),
            );
        }
        PersonaCommands::Get(args) => {
            let persona = client.personas().get(&args.persona_id).await?;
            emit_one_with_actions(
                ctx,
                "personas",
                operation,
                &persona,
                next_actions::item_result("personas", &persona.id),
            );
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
            emit_one_with_actions(
                ctx,
                "personas",
                operation,
                &persona,
                next_actions::item_result("personas", &persona.id),
            );
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
            emit_one_with_actions(
                ctx,
                "personas",
                operation,
                &persona,
                next_actions::item_result("personas", &persona.id),
            );
        }
        PersonaCommands::Delete(args) => {
            client.personas().delete(&args.persona_id).await?;
            emit_success_with_actions(
                ctx,
                "personas",
                operation,
                "Persona deleted.",
                next_actions::delete_result("personas"),
            );
        }
        PersonaCommands::PhoneNumbers => {
            let response = client.personas().list_phone_numbers().await?;
            emit_list_with_actions(
                ctx,
                "personas",
                operation,
                &response.phone_numbers,
                vec![next_actions::context("personas")],
            );
        }
    }
    Ok(())
}
