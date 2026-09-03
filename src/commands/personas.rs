use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Args, Subcommand};

const BACKGROUND_SOUND_UPLOAD_TIMEOUT_SECONDS: u64 = 300;

use crate::client::models::{
    BackgroundSoundUpdateStatus, CreateBackgroundSoundRequest, CreatePersonaRequest, ListParams,
    UpdateBackgroundSoundRequest, UpdatePersonaRequest,
};
use crate::client::CovalClient;
use crate::input_json::{self, InputJsonArg};
use crate::next_actions;
use crate::output::NextAction;
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
    #[command(name = "background-sounds")]
    BackgroundSounds {
        #[command(subcommand)]
        command: BackgroundSoundCommands,
    },
    #[command(name = "phone-numbers")]
    PhoneNumbers,
    Voices,
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
            Self::BackgroundSounds { command } => command.operation(),
            Self::PhoneNumbers => "phone-numbers",
            Self::Voices => "voices",
        }
    }
}

#[derive(Subcommand)]
pub enum BackgroundSoundCommands {
    List(BackgroundSoundListArgs),
    Upload(BackgroundSoundUploadArgs),
    Update(BackgroundSoundUpdateArgs),
}

impl BackgroundSoundCommands {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::List(_) => "background-sounds.list",
            Self::Upload(_) => "background-sounds.upload",
            Self::Update(_) => "background-sounds.update",
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
    #[command(flatten)]
    input_json: InputJsonArg,
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
    /// Enable multilingual speech-to-text
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    multi_language_stt: Option<bool>,
    /// Background sound volume multiplier (>= 0.0)
    #[arg(long)]
    background_sound_volume: Option<f64>,
    /// Voice gain multiplier (0.0 silent, 1.0 unchanged, 2.0 double)
    #[arg(long)]
    voice_volume: Option<f64>,
    /// Voice speed multiplier (0.25-2.0, 1.0 unchanged)
    #[arg(long)]
    voice_speed: Option<f64>,
    /// Disconnect after this many seconds of no speech (5-300)
    #[arg(long)]
    hold_music_timeout_seconds: Option<f64>,
    /// Placement preset (speakerphone-easy or speakerphone-hard); conflicts with --audio-degradation
    #[arg(long)]
    situate_speaker: Option<String>,
    /// Channel degradation preset id (landline, cell-poor, cell-handoff) or a JSON object; conflicts with --situate-speaker
    #[arg(long)]
    audio_degradation: Option<String>,
    /// Comma-separated tag names; pass an empty value to clear all tags
    #[arg(long, value_delimiter = ',')]
    tags: Option<Vec<String>>,
}

#[derive(Args)]
pub struct UpdateArgs {
    persona_id: String,
    #[command(flatten)]
    input_json: InputJsonArg,
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
    /// Enable or disable multilingual speech-to-text
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    multi_language_stt: Option<bool>,
    /// Background sound volume multiplier (>= 0.0)
    #[arg(long)]
    background_sound_volume: Option<f64>,
    /// Voice gain multiplier (0.0 silent, 1.0 unchanged, 2.0 double)
    #[arg(long)]
    voice_volume: Option<f64>,
    /// Voice speed multiplier (0.25-2.0, 1.0 unchanged)
    #[arg(long)]
    voice_speed: Option<f64>,
    /// Disconnect after this many seconds of no speech (5-300)
    #[arg(long)]
    hold_music_timeout_seconds: Option<f64>,
    /// Placement preset (speakerphone-easy or speakerphone-hard); conflicts with --audio-degradation
    #[arg(long)]
    situate_speaker: Option<String>,
    /// Channel degradation preset id (landline, cell-poor, cell-handoff) or a JSON object; conflicts with --situate-speaker
    #[arg(long)]
    audio_degradation: Option<String>,
    /// Comma-separated tag names; pass an empty value to clear all tags
    #[arg(long, value_delimiter = ',')]
    tags: Option<Vec<String>>,
}

#[derive(Args)]
pub struct DeleteArgs {
    persona_id: String,
}

#[derive(Args)]
pub struct BackgroundSoundListArgs {
    /// Include archived custom background sounds
    #[arg(long)]
    include_archived: bool,
}

#[derive(Args)]
pub struct BackgroundSoundUploadArgs {
    /// Local WAV or MP3 file to upload
    file: PathBuf,
    /// Display name for the custom background sound. Defaults to the filename stem.
    #[arg(long)]
    display_name: Option<String>,
    /// Audio content type. Defaults from file extension (.mp3 or .wav).
    #[arg(long)]
    content_type: Option<String>,
    /// Default volume multiplier for simulations (0.0-1.0)
    #[arg(long)]
    default_volume: Option<f64>,
    /// Acoustic rendering behavior (ambient or point_source)
    #[arg(long)]
    acoustic_source_type: Option<String>,
    /// Metadata as key=value (repeat for multiple)
    #[arg(long = "metadata", value_parser = parse_metadata_kv)]
    metadata: Vec<(String, String)>,
}

#[derive(Args)]
pub struct BackgroundSoundUpdateArgs {
    /// Custom background sound asset id. Accepts either `sound_id` or `custom:sound_id`.
    background_sound_id: String,
    #[command(flatten)]
    input_json: InputJsonArg,
    /// New display name
    #[arg(long)]
    display_name: Option<String>,
    /// New default volume multiplier (0.0-1.0)
    #[arg(long)]
    default_volume: Option<f64>,
    /// New acoustic rendering behavior (ambient or point_source)
    #[arg(long)]
    acoustic_source_type: Option<String>,
    /// Archive or restore the custom background sound
    #[arg(long, value_enum)]
    status: Option<BackgroundSoundUpdateStatus>,
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
            let mut input = args.input_json.object()?;
            normalize_persona_input(&mut input);
            input_json::insert(&mut input, "name", args.name)?;
            input_json::insert(&mut input, "voice_name", args.voice)?;
            input_json::insert(&mut input, "language_code", args.language)?;
            input_json::insert(&mut input, "persona_prompt", args.prompt)?;
            input_json::insert(&mut input, "background_sound", args.background)?;
            input_json::insert(&mut input, "wait_seconds", args.wait_seconds)?;
            input_json::insert(&mut input, "multi_language_stt", args.multi_language_stt)?;
            input_json::insert(
                &mut input,
                "background_sound_volume",
                args.background_sound_volume,
            )?;
            input_json::insert(&mut input, "voice_volume", args.voice_volume)?;
            input_json::insert(&mut input, "voice_speed", args.voice_speed)?;
            input_json::insert(
                &mut input,
                "hold_music_timeout_seconds",
                args.hold_music_timeout_seconds,
            )?;
            input_json::insert(&mut input, "situate_speaker", args.situate_speaker)?;
            input_json::insert(
                &mut input,
                "audio_degradation",
                parse_audio_degradation(args.audio_degradation),
            )?;
            input_json::insert(&mut input, "tags", args.tags)?;
            let req: CreatePersonaRequest = input_json::finish(input)?;
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
            let mut input = args.input_json.object()?;
            normalize_persona_input(&mut input);
            input_json::insert(&mut input, "name", args.name)?;
            input_json::insert(&mut input, "voice_name", args.voice)?;
            input_json::insert(&mut input, "language_code", args.language)?;
            input_json::insert(&mut input, "persona_prompt", args.prompt)?;
            input_json::insert(&mut input, "background_sound", args.background)?;
            input_json::insert(&mut input, "wait_seconds", args.wait_seconds)?;
            input_json::insert(&mut input, "multi_language_stt", args.multi_language_stt)?;
            input_json::insert(
                &mut input,
                "background_sound_volume",
                args.background_sound_volume,
            )?;
            input_json::insert(&mut input, "voice_volume", args.voice_volume)?;
            input_json::insert(&mut input, "voice_speed", args.voice_speed)?;
            input_json::insert(
                &mut input,
                "hold_music_timeout_seconds",
                args.hold_music_timeout_seconds,
            )?;
            input_json::insert(&mut input, "situate_speaker", args.situate_speaker)?;
            input_json::insert(
                &mut input,
                "audio_degradation",
                parse_audio_degradation(args.audio_degradation),
            )?;
            input_json::insert(&mut input, "tags", args.tags)?;
            let req: UpdatePersonaRequest = input_json::finish(input)?;
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
        PersonaCommands::BackgroundSounds { command } => {
            execute_background_sound(command, client, ctx, operation).await?;
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
        PersonaCommands::Voices => {
            let response = client.personas().list_voices().await?;
            emit_list_with_actions(
                ctx,
                "personas",
                operation,
                &response.voices,
                vec![next_actions::context("personas")],
            );
        }
    }
    Ok(())
}

fn normalize_persona_input(input: &mut serde_json::Map<String, serde_json::Value>) {
    if let Some(value) = input.remove("multiLanguageStt") {
        input
            .entry("multi_language_stt".to_string())
            .or_insert(value);
    }
}

async fn execute_background_sound(
    cmd: BackgroundSoundCommands,
    client: &CovalClient,
    ctx: &OutputContext,
    operation: &'static str,
) -> Result<()> {
    match cmd {
        BackgroundSoundCommands::List(args) => {
            let response = client
                .personas()
                .list_background_sounds(args.include_archived)
                .await?;
            emit_list_with_actions(
                ctx,
                "background-sounds",
                operation,
                &response.background_sounds,
                background_sound_list_actions(),
            );
        }
        BackgroundSoundCommands::Upload(args) => {
            let content_type = match args.content_type {
                Some(content_type) => content_type,
                None => infer_audio_content_type(&args.file).ok_or_else(|| {
                    anyhow::anyhow!(
                        "could not infer content type from {}; pass --content-type audio/mpeg or audio/wav",
                        args.file.display()
                    )
                })?,
            };

            let original_filename = file_name(&args.file)?;
            let size = std::fs::metadata(&args.file)
                .map(|metadata| metadata.len())
                .map_err(|err| anyhow::anyhow!("failed to read {}: {err}", args.file.display()))?;
            let display_name = args
                .display_name
                .unwrap_or_else(|| display_name_from_path(&args.file));
            let metadata = metadata_map(args.metadata);
            let create = client
                .personas()
                .create_background_sound(CreateBackgroundSoundRequest {
                    display_name,
                    original_filename: original_filename.clone(),
                    content_type: content_type.clone(),
                    default_volume: args.default_volume,
                    acoustic_source_type: args.acoustic_source_type,
                    metadata,
                })
                .await?;

            if size > create.max_size_bytes {
                let _ = client
                    .personas()
                    .update_background_sound(
                        &create.background_sound.id,
                        UpdateBackgroundSoundRequest {
                            status: Some(BackgroundSoundUpdateStatus::Archived),
                            ..Default::default()
                        },
                    )
                    .await;
                anyhow::bail!(
                    "audio file is too large: {} bytes exceeds max {} bytes",
                    size,
                    create.max_size_bytes
                );
            }
            let bytes = std::fs::read(&args.file)
                .map_err(|err| anyhow::anyhow!("failed to read {}: {err}", args.file.display()))?;

            upload_file_with_signed_post(
                &create.upload_url,
                &create.upload_fields,
                bytes,
                &original_filename,
                &content_type,
            )
            .await?;

            let background_sound = client
                .personas()
                .complete_background_sound_upload(&create.background_sound.id)
                .await?;
            emit_one_with_actions(
                ctx,
                "background-sounds",
                operation,
                &background_sound,
                background_sound_item_actions(&background_sound.value),
            );
        }
        BackgroundSoundCommands::Update(args) => {
            let mut input = args.input_json.object()?;
            input_json::insert(&mut input, "display_name", args.display_name)?;
            input_json::insert(&mut input, "default_volume", args.default_volume)?;
            input_json::insert(
                &mut input,
                "acoustic_source_type",
                args.acoustic_source_type,
            )?;
            input_json::insert(&mut input, "status", args.status)?;
            let req: UpdateBackgroundSoundRequest = input_json::finish(input)?;
            let id = normalize_background_sound_id(&args.background_sound_id);
            let background_sound = client.personas().update_background_sound(&id, req).await?;
            emit_one_with_actions(
                ctx,
                "background-sounds",
                operation,
                &background_sound,
                background_sound_item_actions(&background_sound.value),
            );
        }
    }
    Ok(())
}

/// Accepts either a bare preset id or the full `{"preset": ...}` object the API
/// documents, so the common case does not need JSON on the command line.
fn parse_audio_degradation(raw: Option<String>) -> Option<serde_json::Value> {
    let raw = raw?;
    match serde_json::from_str::<serde_json::Value>(&raw) {
        Ok(value @ serde_json::Value::Object(_)) => Some(value),
        _ => Some(serde_json::json!({ "preset": raw })),
    }
}

fn parse_metadata_kv(raw: &str) -> Result<(String, String), String> {
    let (key, value) = raw
        .split_once('=')
        .ok_or_else(|| format!("expected key=value, got '{raw}'"))?;
    if key.is_empty() {
        return Err(format!("metadata key is empty in '{raw}'"));
    }
    Ok((key.to_string(), value.to_string()))
}

fn metadata_map(values: Vec<(String, String)>) -> Option<BTreeMap<String, String>> {
    if values.is_empty() {
        return None;
    }
    Some(values.into_iter().collect())
}

fn infer_audio_content_type(path: &Path) -> Option<String> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "mp3" => Some("audio/mpeg".to_string()),
        "wav" => Some("audio/wav".to_string()),
        _ => None,
    }
}

fn file_name(path: &Path) -> Result<String> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("file path has no valid filename: {}", path.display()))?;
    Ok(name.to_string())
}

fn display_name_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("Background sound")
        .to_string()
}

async fn upload_file_with_signed_post(
    upload_url: &str,
    fields: &BTreeMap<String, String>,
    bytes: Vec<u8>,
    original_filename: &str,
    content_type: &str,
) -> Result<()> {
    let mut form = reqwest::multipart::Form::new();
    for (key, value) in fields {
        form = form.text(key.clone(), value.clone());
    }
    let file_part = reqwest::multipart::Part::bytes(bytes)
        .file_name(original_filename.to_string())
        .mime_str(content_type)?;
    let response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(
            BACKGROUND_SOUND_UPLOAD_TIMEOUT_SECONDS,
        ))
        .build()?
        .post(upload_url)
        .multipart(form.part("file", file_part))
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("background sound upload failed: HTTP {status}\n{body}");
    }
    Ok(())
}

fn normalize_background_sound_id(value: &str) -> String {
    value.strip_prefix("custom:").unwrap_or(value).to_string()
}

fn background_sound_list_actions() -> Vec<NextAction> {
    vec![NextAction::new(
        "background-sounds.upload",
        "Upload background sound",
        next_actions::argv(["personas", "background-sounds", "upload", "<audio_file>"]),
        false,
    )]
}

fn background_sound_item_actions(value: &str) -> Vec<NextAction> {
    vec![
        NextAction::new(
            "personas.update_background_sound",
            "Use on persona",
            next_actions::argv(["personas", "update", "<persona_id>", "--background", value]),
            false,
        )
        .primary()
        .with_description("Set this custom sound on a persona."),
        NextAction::new(
            "background-sounds.list",
            "List background sounds",
            next_actions::argv(["personas", "background-sounds", "list"]),
            true,
        ),
    ]
}
