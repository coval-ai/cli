use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use clap::{Args, Subcommand, ValueEnum};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use crate::client::models::ListParams;
use crate::client::CovalClient;

// ─── Command types ────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum TraceCommands {
    /// Interactive setup wizard — instruments your agent for Coval traces
    Setup(SetupArgs),
    /// Validate your Coval traces configuration by sending a test span
    Validate(ValidateArgs),
}

#[derive(Args)]
pub struct SetupArgs {
    /// Pre-select an agent by ID (skips interactive agent selection)
    #[arg(long)]
    pub agent_id: Option<String>,

    /// Override detected framework
    #[arg(long, value_enum)]
    pub framework: Option<Framework>,

    /// Skip the post-setup validation step
    #[arg(long, default_value = "false")]
    pub no_validate: bool,

    /// Project directory to analyze (defaults to current directory)
    #[arg(long)]
    pub dir: Option<PathBuf>,
}

#[derive(Args)]
pub struct ValidateArgs {
    /// Simulation ID to include in the test trace (uses "cli-test" if omitted)
    #[arg(long)]
    pub simulation_id: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Framework {
    Pipecat,
    Livekit,
    Generic,
}

impl std::fmt::Display for Framework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Framework::Pipecat => write!(f, "Pipecat"),
            Framework::Livekit => write!(f, "LiveKit Agents"),
            Framework::Generic => write!(f, "Generic Python"),
        }
    }
}

// ─── Entry point ──────────────────────────────────────────────────────────────

pub async fn execute(cmd: TraceCommands, client: &CovalClient) -> Result<()> {
    match cmd {
        TraceCommands::Setup(args) => setup(args, client).await,
        TraceCommands::Validate(args) => validate(args, client).await,
    }
}

// ─── setup ────────────────────────────────────────────────────────────────────

async fn setup(args: SetupArgs, client: &CovalClient) -> Result<()> {
    let dir = args
        .dir
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    println!();
    println!("{}", "Coval Traces Setup Wizard".bold());
    println!();

    // ── Step 1: Detect Python project ────────────────────────────────────────
    println!("{}", "Detecting your project...".cyan());

    let project_file = detect_python_project(&dir).ok_or_else(|| {
        anyhow::anyhow!(
            "No Python project found in {}.\n\
             Expected pyproject.toml, requirements.txt, Pipfile, or setup.py.",
            dir.display()
        )
    })?;
    println!("  {} Found Python project ({})", "✓".green(), project_file);

    // ── Step 2: Detect framework ──────────────────────────────────────────────
    let (framework, detection_note) = if let Some(f) = args.framework {
        (f, "(--framework flag)".to_string())
    } else {
        let (f, note) = detect_framework(&dir);
        (
            f,
            note.unwrap_or_else(|| "no framework-specific marker found".to_string()),
        )
    };
    println!(
        "  {} Detected {} ({})",
        "✓".green(),
        framework.to_string().bold(),
        detection_note
    );

    // ── Step 3: Pick agent ────────────────────────────────────────────────────
    println!();
    let agent_id = match args.agent_id {
        Some(id) => {
            println!("  {} Using agent ID: {}", "✓".green(), id.bold());
            id
        }
        None => pick_agent(client).await?,
    };

    // ── Step 4: Find entry point ──────────────────────────────────────────────
    let entry_path = pick_entry_point(&dir, framework)?;
    println!("  {} Selected: {}", "✓".green(), entry_path.display());

    // ── Step 5: Analyze entry file ────────────────────────────────────────────
    println!();
    println!(
        "{}",
        format!("Analyzing {}...", entry_path.display()).cyan()
    );

    let analysis = analyze_entry_point(&entry_path, framework)?;
    print_analysis(&analysis);

    if analysis.otel_already_configured {
        println!(
            "\n  {} OpenTelemetry is already configured in this file.",
            "!".yellow()
        );
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Continue and overwrite?")
            .default(false)
            .interact()?
        {
            println!("Aborted.");
            return Ok(());
        }
    }

    // ── Step 6: Show plan and confirm ─────────────────────────────────────────
    let tracing_file = dir.join("coval_tracing.py");
    println!();
    println!("{}", "Here's what I'll do:".bold());

    let tracing_file_exists = tracing_file.exists();
    if tracing_file_exists {
        println!(
            "  {} {} — update existing OTel setup helper",
            "~".yellow(),
            "coval_tracing.py".bold()
        );
    } else {
        println!(
            "  {} {} — create OTel setup helper",
            "+".green(),
            "coval_tracing.py".bold()
        );
    }

    let entry_name = entry_path.file_name().unwrap_or_default().to_string_lossy();
    println!(
        "  {} {} — add import + configure_coval_tracing() call",
        "~".yellow(),
        entry_name.bold()
    );
    println!("    (backup → {}.bak)", entry_path.display());

    if analysis.import_line > 0 {
        println!("    - Insert import at line {}", analysis.import_line + 2);
    }
    if let Some(ref ctx) = analysis.call_context {
        println!(
            "    - Insert configure_coval_tracing() before: {}",
            ctx.cyan()
        );
    }

    println!();
    if !Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Apply these changes?")
        .default(true)
        .interact()?
    {
        println!("Aborted. No files were modified.");
        return Ok(());
    }

    // ── Step 7: Write files ───────────────────────────────────────────────────
    let tracing_content = generate_coval_tracing_py(client.api_key());
    fs::write(&tracing_file, tracing_content)
        .with_context(|| format!("Failed to write {}", tracing_file.display()))?;

    apply_entry_point_modifications(&entry_path, &analysis, framework)?;

    println!();
    println!("{}", "Done! Files modified:".bold());
    if tracing_file_exists {
        println!("  {} coval_tracing.py (updated)", "~".yellow());
    } else {
        println!("  {} coval_tracing.py (created)", "+".green());
    }
    println!(
        "  {} {} (modified, backup at {}.bak)",
        "~".yellow(),
        entry_name,
        entry_path.display()
    );

    // ── Step 8: Next steps ────────────────────────────────────────────────────
    println!();
    println!("{}", "Next steps:".bold());
    println!(
        "  1. Set your API key: {}",
        "export COVAL_API_KEY=<your-key>".bold()
    );
    println!("  2. Get the simulation_output_id from the Coval API when a run starts.");
    println!("     See the TODO comment in {} for details.", entry_name);
    println!(
        "  3. Run a simulation — traces appear at {}",
        format!("https://app.coval.dev/runs/{agent_id}").bold()
    );

    if !args.no_validate {
        println!();
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Validate auth now by sending a test trace?")
            .default(true)
            .interact()?
        {
            validate(
                ValidateArgs {
                    simulation_id: None,
                },
                client,
            )
            .await?;
        }
    }

    Ok(())
}

// ─── validate ─────────────────────────────────────────────────────────────────

async fn validate(args: ValidateArgs, client: &CovalClient) -> Result<()> {
    let sim_id = args.simulation_id.unwrap_or_else(|| "cli-test".to_string());

    println!();
    print!(
        "  {} Sending test trace (simulation_id={})... ",
        "→".cyan(),
        sim_id
    );
    // Flush stdout so the message appears before the async operation
    use std::io::Write;
    let _ = std::io::stdout().flush();

    let b64_key = BASE64.encode(client.api_key());
    // Ensure base URL ends without trailing slash before joining path
    let base = client.base_url().trim_end_matches('/');
    let traces_url = format!("{base}/v1/traces");

    let (trace_id, span_id, now_ns) = generate_trace_ids();

    let payload = serde_json::json!({
        "resourceSpans": [{
            "resource": {
                "attributes": [
                    {"key": "service.name", "value": {"stringValue": "coval-cli-validation"}},
                    {"key": "coval.test", "value": {"boolValue": true}}
                ]
            },
            "scopeSpans": [{
                "scope": {
                    "name": "coval.traces.validate",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "spans": [{
                    "traceId": trace_id,
                    "spanId": span_id,
                    "name": "coval.validate.test_span",
                    "kind": 1,
                    "startTimeUnixNano": now_ns.to_string(),
                    "endTimeUnixNano": (now_ns + 1_000_000u128).to_string(),
                    "status": {"code": 1, "message": "OK"},
                    "attributes": [
                        {"key": "coval.test", "value": {"boolValue": true}}
                    ]
                }]
            }]
        }]
    });

    let http = reqwest::Client::builder()
        .user_agent(concat!("coval-cli/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let resp = http
        .post(&traces_url)
        .header("Authorization", format!("Basic {b64_key}"))
        .header("X-Simulation-Id", &sim_id)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("Failed to reach Coval traces endpoint")?;

    let status = resp.status();
    if status.is_success() {
        println!("{}", "OK".green().bold());
        println!(
            "  {} Test trace received! Check your runs at {}",
            "✓".green(),
            "https://app.coval.dev".bold()
        );
    } else {
        let body = resp.text().await.unwrap_or_default();
        println!("{}", "FAILED".red().bold());
        println!("  {} HTTP {}: {}", "✗".red(), status, body);
        println!();
        println!("  Troubleshooting:");
        println!("  - Ensure COVAL_API_KEY is set correctly");
        println!("  - Run `coval whoami` to verify authentication");
        return Err(anyhow::anyhow!("Trace validation failed (HTTP {})", status));
    }

    Ok(())
}

// ─── Detection helpers ────────────────────────────────────────────────────────

fn detect_python_project(dir: &Path) -> Option<String> {
    for file in ["pyproject.toml", "requirements.txt", "Pipfile", "setup.py"] {
        if dir.join(file).exists() {
            return Some(file.to_string());
        }
    }
    None
}

fn detect_framework(dir: &Path) -> (Framework, Option<String>) {
    // Check pyproject.toml
    if let Ok(content) = fs::read_to_string(dir.join("pyproject.toml")) {
        if content.contains("pipecat-ai") || content.contains("pipecat_ai") {
            return (
                Framework::Pipecat,
                Some("pipecat-ai in pyproject.toml".to_string()),
            );
        }
        if content.contains("livekit-agents") || content.contains("livekit_agents") {
            return (
                Framework::Livekit,
                Some("livekit-agents in pyproject.toml".to_string()),
            );
        }
    }

    // Check requirements.txt
    if let Ok(content) = fs::read_to_string(dir.join("requirements.txt")) {
        let lower = content.to_lowercase();
        if lower.contains("pipecat") {
            return (
                Framework::Pipecat,
                Some("pipecat in requirements.txt".to_string()),
            );
        }
        if lower.contains("livekit") {
            return (
                Framework::Livekit,
                Some("livekit in requirements.txt".to_string()),
            );
        }
    }

    // Scan likely Python source files for import clues
    if let Some(result) = scan_python_imports(dir) {
        return result;
    }

    (Framework::Generic, None)
}

fn scan_python_imports(dir: &Path) -> Option<(Framework, Option<String>)> {
    for name in ["bot.py", "agent.py", "server.py", "main.py", "app.py"] {
        if let Ok(content) = fs::read_to_string(dir.join(name)) {
            if content.contains("from pipecat") || content.contains("import pipecat") {
                return Some((
                    Framework::Pipecat,
                    Some(format!("pipecat import in {name}")),
                ));
            }
            if content.contains("from livekit") || content.contains("import livekit") {
                return Some((
                    Framework::Livekit,
                    Some(format!("livekit import in {name}")),
                ));
            }
        }
    }
    None
}

// ─── Entry point selection ────────────────────────────────────────────────────

fn pick_entry_point(dir: &Path, framework: Framework) -> Result<PathBuf> {
    let preferred: &[&str] = match framework {
        Framework::Pipecat => &["bot.py", "agent.py", "server.py", "main.py", "app.py"],
        Framework::Livekit => &["agent.py", "bot.py", "server.py", "main.py", "app.py"],
        Framework::Generic => &["main.py", "app.py", "agent.py", "bot.py", "server.py"],
    };

    let candidates: Vec<PathBuf> = preferred
        .iter()
        .map(|f| dir.join(f))
        .filter(|p| p.exists())
        .collect();

    if candidates.is_empty() {
        println!(
            "  {} No common entry point files found (bot.py, agent.py, etc.)",
            "!".yellow()
        );
        let path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter path to your agent's main Python file")
            .interact_text()?;
        let p = if Path::new(&path).is_absolute() {
            PathBuf::from(&path)
        } else {
            dir.join(&path)
        };
        if !p.exists() {
            return Err(anyhow::anyhow!("File not found: {}", p.display()));
        }
        return Ok(p);
    }

    let mut items: Vec<String> = candidates
        .iter()
        .map(|p| {
            p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        })
        .collect();
    items.push("Enter path manually".to_string());

    println!();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Which file is your main agent/bot entry point?")
        .items(&items)
        .default(0)
        .interact()?;

    if selection == candidates.len() {
        let path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter path to your agent's main Python file")
            .interact_text()?;
        let p = if Path::new(&path).is_absolute() {
            PathBuf::from(&path)
        } else {
            dir.join(&path)
        };
        if !p.exists() {
            return Err(anyhow::anyhow!("File not found: {}", p.display()));
        }
        Ok(p)
    } else {
        Ok(candidates.into_iter().nth(selection).unwrap())
    }
}

// ─── Agent selection ──────────────────────────────────────────────────────────

async fn pick_agent(client: &CovalClient) -> Result<String> {
    let agents = client
        .agents()
        .list(ListParams {
            page_size: Some(50),
            ..Default::default()
        })
        .await
        .context("Failed to list agents")?;

    if agents.agents.is_empty() {
        return Err(anyhow::anyhow!(
            "No agents found. Create one first with `coval agents create`."
        ));
    }

    let items: Vec<String> = agents
        .agents
        .iter()
        .map(|a| format!("{:<30} (id: {})", a.display_name, a.id))
        .collect();

    println!();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Which agent?")
        .items(&items)
        .default(0)
        .interact()?;

    Ok(agents.agents[selection].id.clone())
}

// ─── Entry point analysis ─────────────────────────────────────────────────────

struct EntryPointAnalysis {
    /// 0-indexed line; the import is inserted BEFORE `import_line + 1`
    /// (i.e., immediately after the last existing import)
    import_line: usize,
    /// Whether `import os` is already present in the file
    has_os_import: bool,
    /// 0-indexed line before which to insert the configure_coval_tracing() call
    /// (uses original line numbers, before any insertions are applied)
    call_line: Option<usize>,
    /// Indentation to use for the inserted call block
    call_indent: String,
    /// Human-readable description of what was found at call_line
    call_context: Option<String>,
    /// Whether OpenTelemetry is already configured
    otel_already_configured: bool,
}

fn analyze_entry_point(path: &Path, framework: Framework) -> Result<EntryPointAnalysis> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let lines: Vec<&str> = content.lines().collect();

    let otel_already_configured = lines.iter().any(|l| {
        l.contains("opentelemetry")
            || l.contains("TracerProvider")
            || l.contains("configure_coval_tracing")
    });

    let import_line = find_last_import_line(&lines);

    let has_os_import = lines
        .iter()
        .any(|l| l.trim() == "import os" || l.trim().starts_with("import os "));

    let (call_line, call_indent, call_context) = match framework {
        Framework::Pipecat => find_pipecat_injection(&lines),
        Framework::Livekit => find_livekit_injection(&lines),
        Framework::Generic => find_generic_injection(&lines),
    };

    Ok(EntryPointAnalysis {
        import_line,
        has_os_import,
        call_line,
        call_indent,
        call_context,
        otel_already_configured,
    })
}

fn find_last_import_line(lines: &[&str]) -> usize {
    let mut last = 0;
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.starts_with("import ") || t.starts_with("from ") {
            last = i;
        }
    }
    last
}

fn find_pipecat_injection(lines: &[&str]) -> (Option<usize>, String, Option<String>) {
    for (i, line) in lines.iter().enumerate() {
        if line.contains("PipelineTask(")
            || line.contains("PipelineRunner(")
            || line.contains("Pipeline(")
        {
            return (
                Some(i),
                leading_whitespace(line),
                Some(line.trim().chars().take(60).collect()),
            );
        }
    }
    // Fallback: runner.run() pattern
    for (i, line) in lines.iter().enumerate() {
        if line.contains("runner.run(") {
            return (
                Some(i),
                leading_whitespace(line),
                Some(line.trim().chars().take(60).collect()),
            );
        }
    }
    find_generic_injection(lines)
}

fn find_livekit_injection(lines: &[&str]) -> (Option<usize>, String, Option<String>) {
    // Prefer: first statement inside the entrypoint function
    for (i, line) in lines.iter().enumerate() {
        if line.contains("async def entrypoint") {
            for (j, inner_line) in lines.iter().enumerate().skip(i + 1) {
                let t = inner_line.trim();
                if !t.is_empty() && !t.starts_with('#') {
                    return (
                        Some(j),
                        leading_whitespace(inner_line),
                        Some(format!("inside entrypoint() at line {}", j + 1)),
                    );
                }
            }
        }
    }
    // Fallback: before AgentSession or VoicePipelineAgent
    for (i, line) in lines.iter().enumerate() {
        if line.contains("AgentSession(") || line.contains("VoicePipelineAgent(") {
            return (
                Some(i),
                leading_whitespace(line),
                Some(line.trim().chars().take(60).collect()),
            );
        }
    }
    find_generic_injection(lines)
}

fn find_generic_injection(lines: &[&str]) -> (Option<usize>, String, Option<String>) {
    // First non-import, non-blank, non-comment top-level statement after imports
    let last_import = find_last_import_line(lines);
    for (i, line) in lines.iter().enumerate().skip(last_import + 1) {
        let t = line.trim();
        if !t.is_empty() && !t.starts_with('#') {
            return (
                Some(i),
                leading_whitespace(line),
                Some(format!(
                    "line {}: {}",
                    i + 1,
                    t.chars().take(50).collect::<String>()
                )),
            );
        }
    }
    (None, String::new(), None)
}

fn leading_whitespace(s: &str) -> String {
    s.chars().take_while(|c| c.is_whitespace()).collect()
}

fn print_analysis(a: &EntryPointAnalysis) {
    if a.otel_already_configured {
        println!(
            "  {} Existing OpenTelemetry configuration detected",
            "!".yellow()
        );
    } else {
        println!("  {} No existing OTel configuration detected", "✓".green());
    }
    match &a.call_context {
        Some(ctx) => println!("  {} Injection point: {}", "✓".green(), ctx.cyan()),
        None => println!(
            "  {} No specific injection point found — will insert after imports",
            "~".yellow()
        ),
    }
}

// ─── File modification ────────────────────────────────────────────────────────

fn apply_entry_point_modifications(
    path: &Path,
    analysis: &EntryPointAnalysis,
    framework: Framework,
) -> Result<()> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;

    // Write backup by appending ".bak" to the full path string
    let bak_path = PathBuf::from(format!("{}.bak", path.display()));
    fs::write(&bak_path, &content)
        .with_context(|| format!("Failed to write backup {}", bak_path.display()))?;

    let lines: Vec<&str> = content.lines().collect();
    let trailing_newline = content.ends_with('\n');

    // Build insertions as (insert_before_this_index, lines_to_insert).
    // We use ORIGINAL line indices and sort descending so each insertion
    // doesn't affect the positions of earlier (lower-indexed) insertions.
    let mut insertions: Vec<(usize, Vec<String>)> = Vec::new();

    // Import block: insert after the last import line
    let import_pos = analysis.import_line + 1;
    let mut import_lines = Vec::new();
    if !analysis.has_os_import {
        import_lines.push("import os".to_string());
    }
    import_lines.push("from coval_tracing import configure_coval_tracing".to_string());
    insertions.push((import_pos, import_lines));

    // configure_coval_tracing() call block
    if let Some(call_line) = analysis.call_line {
        let snippet = generate_call_snippet(framework, &analysis.call_indent);
        insertions.push((call_line, snippet));
    }

    // Apply in descending order: higher line numbers first so earlier indices stay stable
    insertions.sort_by_key(|(pos, _)| std::cmp::Reverse(*pos));

    let mut result: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    for (pos, new_lines) in insertions {
        let insert_at = pos.min(result.len());
        for (offset, line) in new_lines.into_iter().enumerate() {
            result.insert(insert_at + offset, line);
        }
    }

    let mut output = result.join("\n");
    if trailing_newline {
        output.push('\n');
    }

    fs::write(path, output).with_context(|| format!("Failed to write {}", path.display()))?;

    Ok(())
}

fn generate_call_snippet(framework: Framework, indent: &str) -> Vec<String> {
    let comment = match framework {
        Framework::Livekit => format!(
            "{indent}# TODO: Replace with actual simulation_output_id from the Coval API response.\n\
             {indent}# For LiveKit, it may be passed via job metadata:\n\
             {indent}#   simulation_output_id = ctx.job.metadata or os.environ.get(\"COVAL_SIMULATION_ID\", \"\")"
        ),
        _ => format!(
            "{indent}# TODO: Replace with actual simulation_output_id from the Coval API response.\n\
             {indent}# Example: simulation_output_id = coval_run[\"simulations\"][0][\"id\"]"
        ),
    };

    let call = if matches!(framework, Framework::Livekit) {
        format!(
            "{indent}simulation_output_id = getattr(getattr(ctx, \"job\", None), \"metadata\", None) \
             or os.environ.get(\"COVAL_SIMULATION_ID\", \"\")\n\
             {indent}configure_coval_tracing(simulation_output_id)"
        )
    } else {
        format!(
            "{indent}simulation_output_id = os.environ.get(\"COVAL_SIMULATION_ID\", \"\")\n\
             {indent}configure_coval_tracing(simulation_output_id)"
        )
    };

    let full = format!("{comment}\n{call}");
    let mut lines: Vec<String> = full.lines().map(|l| l.to_string()).collect();
    lines.push(String::new()); // blank line after the block
    lines
}

// ─── coval_tracing.py template ────────────────────────────────────────────────

fn generate_coval_tracing_py(api_key: &str) -> String {
    // Use a placeholder comment for the API key rather than embedding the real key,
    // so the file is safe to commit. The real key is read from COVAL_API_KEY env var.
    let _ = api_key; // reserved for future per-customer defaults
    r#"# coval_tracing.py — generated by `coval traces setup`
#
# This module configures OpenTelemetry tracing for Coval evaluation.
# Span names must follow Coval conventions:
#   llm           — LLM inference spans
#   tts           — Text-to-speech spans
#   stt           — Speech-to-text spans
#   llm_tool_call — Tool/function call spans
#
# See: https://docs.coval.dev/traces

import os
import base64
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import SimpleSpanProcessor
from opentelemetry.exporter.otlp.proto.http.trace_exporter import OTLPSpanExporter

# Set your Coval API key via the environment variable
COVAL_API_KEY = os.environ.get("COVAL_API_KEY", "")
_b64_key = base64.b64encode(COVAL_API_KEY.encode()).decode()


def configure_coval_tracing(simulation_output_id: str) -> None:
    """Configure OpenTelemetry to export traces to Coval.

    Call this once per simulation run, BEFORE your agent pipeline starts.
    The simulation_output_id is returned by the Coval API when a run is triggered.

    Example:
        # From your Coval run trigger response:
        simulation_output_id = run_response["simulations"][0]["id"]
        configure_coval_tracing(simulation_output_id)

    IMPORTANT: The OTel provider is configured per-call (not once at startup)
    because X-Simulation-Id is unique to each simulation run.
    """
    provider = TracerProvider()
    exporter = OTLPSpanExporter(
        endpoint="https://api.coval.dev/v1/traces",
        headers={
            "Authorization": f"Basic {_b64_key}",
            "X-Simulation-Id": simulation_output_id,
        },
    )
    provider.add_span_processor(SimpleSpanProcessor(exporter))
    trace.set_tracer_provider(provider)
"#
    .to_string()
}

// ─── Trace ID generation ──────────────────────────────────────────────────────

fn generate_trace_ids() -> (String, String, u128) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let now_ns = now.as_nanos();
    let secs = now.as_secs() as u128;
    // Combine secs (upper 64 bits) and sub-second nanos (lower 64 bits) for the trace ID
    let trace_val = (secs << 64) | (now_ns & 0xFFFF_FFFF_FFFF_FFFF);
    let trace_id = format!("{trace_val:032x}");
    let span_id = format!("{:016x}", now_ns & 0xFFFF_FFFF_FFFF_FFFF);
    (trace_id, span_id, now_ns)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ── detect_python_project ────────────────────────────────────────────

    #[test]
    fn detect_python_project_finds_pyproject_toml() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("pyproject.toml"), "").unwrap();
        assert_eq!(
            detect_python_project(dir.path()),
            Some("pyproject.toml".to_string())
        );
    }

    #[test]
    fn detect_python_project_finds_requirements_txt() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("requirements.txt"), "flask\n").unwrap();
        assert_eq!(
            detect_python_project(dir.path()),
            Some("requirements.txt".to_string())
        );
    }

    #[test]
    fn detect_python_project_finds_pipfile() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Pipfile"), "").unwrap();
        assert_eq!(
            detect_python_project(dir.path()),
            Some("Pipfile".to_string())
        );
    }

    #[test]
    fn detect_python_project_finds_setup_py() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("setup.py"), "").unwrap();
        assert_eq!(
            detect_python_project(dir.path()),
            Some("setup.py".to_string())
        );
    }

    #[test]
    fn detect_python_project_returns_none_for_empty_dir() {
        let dir = TempDir::new().unwrap();
        assert_eq!(detect_python_project(dir.path()), None);
    }

    #[test]
    fn detect_python_project_prefers_pyproject_over_requirements() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("pyproject.toml"), "").unwrap();
        fs::write(dir.path().join("requirements.txt"), "").unwrap();
        assert_eq!(
            detect_python_project(dir.path()),
            Some("pyproject.toml".to_string())
        );
    }

    // ── detect_framework ─────────────────────────────────────────────────

    #[test]
    fn detect_framework_pipecat_from_pyproject() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("pyproject.toml"),
            "[project]\ndependencies = [\"pipecat-ai>=0.1\"]\n",
        )
        .unwrap();
        let (fw, note) = detect_framework(dir.path());
        assert!(matches!(fw, Framework::Pipecat));
        assert!(note.unwrap().contains("pyproject.toml"));
    }

    #[test]
    fn detect_framework_livekit_from_pyproject() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("pyproject.toml"),
            "[project]\ndependencies = [\"livekit-agents\"]\n",
        )
        .unwrap();
        let (fw, note) = detect_framework(dir.path());
        assert!(matches!(fw, Framework::Livekit));
        assert!(note.unwrap().contains("pyproject.toml"));
    }

    #[test]
    fn detect_framework_pipecat_from_requirements() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("requirements.txt"), "pipecat-ai==0.1\n").unwrap();
        let (fw, _) = detect_framework(dir.path());
        assert!(matches!(fw, Framework::Pipecat));
    }

    #[test]
    fn detect_framework_livekit_from_requirements() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("requirements.txt"), "livekit-agents\n").unwrap();
        let (fw, _) = detect_framework(dir.path());
        assert!(matches!(fw, Framework::Livekit));
    }

    #[test]
    fn detect_framework_generic_for_empty_dir() {
        let dir = TempDir::new().unwrap();
        let (fw, note) = detect_framework(dir.path());
        assert!(matches!(fw, Framework::Generic));
        assert!(note.is_none());
    }

    // ── scan_python_imports ──────────────────────────────────────────────

    #[test]
    fn scan_python_imports_finds_pipecat() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("bot.py"),
            "from pipecat.pipeline import Pipeline\n",
        )
        .unwrap();
        let result = scan_python_imports(dir.path());
        assert!(result.is_some());
        let (fw, note) = result.unwrap();
        assert!(matches!(fw, Framework::Pipecat));
        assert!(note.unwrap().contains("bot.py"));
    }

    #[test]
    fn scan_python_imports_finds_livekit() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("agent.py"),
            "from livekit.agents import AgentSession\n",
        )
        .unwrap();
        let result = scan_python_imports(dir.path());
        assert!(result.is_some());
        let (fw, _) = result.unwrap();
        assert!(matches!(fw, Framework::Livekit));
    }

    #[test]
    fn scan_python_imports_returns_none_for_no_matches() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("main.py"),
            "import flask\napp = flask.Flask()\n",
        )
        .unwrap();
        assert!(scan_python_imports(dir.path()).is_none());
    }

    #[test]
    fn scan_python_imports_returns_none_for_empty_dir() {
        let dir = TempDir::new().unwrap();
        assert!(scan_python_imports(dir.path()).is_none());
    }

    // ── find_last_import_line ────────────────────────────────────────────

    #[test]
    fn find_last_import_line_basic() {
        let lines = vec!["import os", "import sys", "", "x = 1"];
        assert_eq!(find_last_import_line(&lines), 1);
    }

    #[test]
    fn find_last_import_line_with_from_imports() {
        let lines = vec!["import os", "from pathlib import Path", "", "def main():"];
        assert_eq!(find_last_import_line(&lines), 1);
    }

    #[test]
    fn find_last_import_line_no_imports() {
        let lines = vec!["x = 1", "y = 2"];
        assert_eq!(find_last_import_line(&lines), 0);
    }

    #[test]
    fn find_last_import_line_empty() {
        let lines: Vec<&str> = vec![];
        assert_eq!(find_last_import_line(&lines), 0);
    }

    #[test]
    fn find_last_import_line_imports_scattered() {
        let lines = vec![
            "import os",
            "",
            "# comment",
            "from sys import argv",
            "",
            "x = 1",
        ];
        assert_eq!(find_last_import_line(&lines), 3);
    }

    // ── find_pipecat_injection ───────────────────────────────────────────

    #[test]
    fn find_pipecat_injection_pipeline_task() {
        let lines = vec![
            "import os",
            "from pipecat import Pipeline",
            "",
            "async def main():",
            "    task = PipelineTask(pipeline)",
        ];
        let (line, indent, ctx) = find_pipecat_injection(&lines);
        assert_eq!(line, Some(4));
        assert_eq!(indent, "    ");
        assert!(ctx.unwrap().contains("PipelineTask"));
    }

    #[test]
    fn find_pipecat_injection_runner_run() {
        let lines = vec![
            "import os",
            "",
            "async def main():",
            "    await runner.run(task)",
        ];
        let (line, indent, _) = find_pipecat_injection(&lines);
        assert_eq!(line, Some(3));
        assert_eq!(indent, "    ");
    }

    #[test]
    fn find_pipecat_injection_falls_back_to_generic() {
        let lines = vec!["import os", "", "print('hello')"];
        let (line, _, _) = find_pipecat_injection(&lines);
        assert_eq!(line, Some(2));
    }

    // ── find_livekit_injection ───────────────────────────────────────────

    #[test]
    fn find_livekit_injection_entrypoint() {
        let lines = vec![
            "import os",
            "",
            "async def entrypoint(ctx):",
            "    session = AgentSession()",
        ];
        let (line, indent, ctx) = find_livekit_injection(&lines);
        assert_eq!(line, Some(3));
        assert_eq!(indent, "    ");
        assert!(ctx.unwrap().contains("entrypoint"));
    }

    #[test]
    fn find_livekit_injection_skips_comments() {
        let lines = vec![
            "import os",
            "",
            "async def entrypoint(ctx):",
            "    # this is a comment",
            "    session = AgentSession()",
        ];
        let (line, _, _) = find_livekit_injection(&lines);
        assert_eq!(line, Some(4));
    }

    #[test]
    fn find_livekit_injection_agent_session_fallback() {
        let lines = vec![
            "import os",
            "",
            "async def main():",
            "    session = AgentSession(config)",
        ];
        let (line, _, ctx) = find_livekit_injection(&lines);
        assert_eq!(line, Some(3));
        assert!(ctx.unwrap().contains("AgentSession"));
    }

    #[test]
    fn find_livekit_injection_voice_pipeline_agent() {
        let lines = vec![
            "import os",
            "",
            "async def run():",
            "    agent = VoicePipelineAgent(llm=llm)",
        ];
        let (line, _, _) = find_livekit_injection(&lines);
        assert_eq!(line, Some(3));
    }

    // ── find_generic_injection ───────────────────────────────────────────

    #[test]
    fn find_generic_injection_after_imports() {
        let lines = vec!["import os", "import sys", "", "app = Flask()"];
        let (line, indent, _) = find_generic_injection(&lines);
        assert_eq!(line, Some(3));
        assert_eq!(indent, "");
    }

    #[test]
    fn find_generic_injection_skips_comments_and_blanks() {
        let lines = vec!["import os", "", "# config section", "", "app = Flask()"];
        let (line, _, _) = find_generic_injection(&lines);
        assert_eq!(line, Some(4));
    }

    #[test]
    fn find_generic_injection_empty_file() {
        let lines: Vec<&str> = vec![];
        let (line, indent, ctx) = find_generic_injection(&lines);
        assert_eq!(line, None);
        assert_eq!(indent, "");
        assert!(ctx.is_none());
    }

    #[test]
    fn find_generic_injection_only_imports() {
        let lines = vec!["import os", "import sys"];
        let (line, _, _) = find_generic_injection(&lines);
        assert_eq!(line, None);
    }

    // ── leading_whitespace ───────────────────────────────────────────────

    #[test]
    fn leading_whitespace_spaces() {
        assert_eq!(leading_whitespace("    hello"), "    ");
    }

    #[test]
    fn leading_whitespace_tabs() {
        assert_eq!(leading_whitespace("\t\thello"), "\t\t");
    }

    #[test]
    fn leading_whitespace_none() {
        assert_eq!(leading_whitespace("hello"), "");
    }

    #[test]
    fn leading_whitespace_empty_string() {
        assert_eq!(leading_whitespace(""), "");
    }

    #[test]
    fn leading_whitespace_all_whitespace() {
        assert_eq!(leading_whitespace("   "), "   ");
    }

    // ── generate_call_snippet ────────────────────────────────────────────

    #[test]
    fn generate_call_snippet_generic() {
        let lines = generate_call_snippet(Framework::Generic, "");
        let joined = lines.join("\n");
        assert!(joined.contains("configure_coval_tracing(simulation_output_id)"));
        assert!(joined.contains("COVAL_SIMULATION_ID"));
        assert!(!joined.contains("ctx.job.metadata"));
    }

    #[test]
    fn generate_call_snippet_livekit_includes_ctx() {
        let lines = generate_call_snippet(Framework::Livekit, "    ");
        let joined = lines.join("\n");
        assert!(joined.contains("ctx"));
        assert!(joined.contains("job"));
        assert!(joined.contains("configure_coval_tracing"));
    }

    #[test]
    fn generate_call_snippet_pipecat() {
        let lines = generate_call_snippet(Framework::Pipecat, "  ");
        let joined = lines.join("\n");
        assert!(joined.contains("configure_coval_tracing"));
        assert!(joined.contains("COVAL_SIMULATION_ID"));
    }

    #[test]
    fn generate_call_snippet_ends_with_blank_line() {
        let lines = generate_call_snippet(Framework::Generic, "");
        assert_eq!(lines.last().unwrap(), "");
    }

    // ── generate_coval_tracing_py ────────────────────────────────────────

    #[test]
    fn generate_coval_tracing_py_contains_expected_elements() {
        let content = generate_coval_tracing_py("test-key");
        assert!(content.contains("def configure_coval_tracing"));
        assert!(content.contains("TracerProvider"));
        assert!(content.contains("OTLPSpanExporter"));
        assert!(content.contains("COVAL_API_KEY"));
        // Should NOT embed the actual key
        assert!(!content.contains("test-key"));
    }

    #[test]
    fn generate_coval_tracing_py_valid_python_structure() {
        let content = generate_coval_tracing_py("");
        assert!(content.contains("import os"));
        assert!(content.contains("import base64"));
        assert!(content.contains("from opentelemetry"));
    }

    // ── generate_trace_ids ───────────────────────────────────────────────

    #[test]
    fn generate_trace_ids_correct_lengths() {
        let (trace_id, span_id, now_ns) = generate_trace_ids();
        assert_eq!(trace_id.len(), 32, "trace_id should be 32 hex chars");
        assert_eq!(span_id.len(), 16, "span_id should be 16 hex chars");
        assert!(now_ns > 0);
    }

    #[test]
    fn generate_trace_ids_valid_hex() {
        let (trace_id, span_id, _) = generate_trace_ids();
        assert!(
            u128::from_str_radix(&trace_id, 16).is_ok(),
            "trace_id should be valid hex"
        );
        assert!(
            u64::from_str_radix(&span_id, 16).is_ok(),
            "span_id should be valid hex"
        );
    }

    #[test]
    fn generate_trace_ids_unique_across_calls() {
        // Add a small sleep to guarantee different nanosecond timestamps.
        let (t1, s1, _) = generate_trace_ids();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let (t2, s2, _) = generate_trace_ids();
        assert!(t1 != t2 || s1 != s2);
    }

    // ── apply_entry_point_modifications ──────────────────────────────────

    #[test]
    fn apply_entry_point_modifications_generic() {
        let dir = TempDir::new().unwrap();
        let entry = dir.path().join("main.py");
        fs::write(
            &entry,
            "import os\nimport sys\n\ndef main():\n    print('hello')\n",
        )
        .unwrap();

        let analysis = EntryPointAnalysis {
            import_line: 1,
            has_os_import: true,
            call_line: Some(3),
            call_indent: String::new(),
            call_context: Some("def main()".to_string()),
            otel_already_configured: false,
        };

        apply_entry_point_modifications(&entry, &analysis, Framework::Generic).unwrap();

        let result = fs::read_to_string(&entry).unwrap();
        assert!(result.contains("from coval_tracing import configure_coval_tracing"));
        assert!(result.contains("configure_coval_tracing(simulation_output_id)"));
        assert_eq!(result.matches("import os").count(), 1);

        let bak = PathBuf::from(format!("{}.bak", entry.display()));
        assert!(bak.exists());
    }

    #[test]
    fn apply_entry_point_modifications_adds_os_import_when_missing() {
        let dir = TempDir::new().unwrap();
        let entry = dir.path().join("bot.py");
        fs::write(
            &entry,
            "from pipecat import Pipeline\n\ntask = PipelineTask(p)\n",
        )
        .unwrap();

        let analysis = EntryPointAnalysis {
            import_line: 0,
            has_os_import: false,
            call_line: Some(2),
            call_indent: String::new(),
            call_context: None,
            otel_already_configured: false,
        };

        apply_entry_point_modifications(&entry, &analysis, Framework::Pipecat).unwrap();

        let result = fs::read_to_string(&entry).unwrap();
        assert!(result.contains("import os"));
        assert!(result.contains("from coval_tracing import configure_coval_tracing"));
    }

    #[test]
    fn apply_entry_point_modifications_preserves_trailing_newline() {
        let dir = TempDir::new().unwrap();
        let entry = dir.path().join("app.py");
        fs::write(&entry, "import os\n\nx = 1\n").unwrap();

        let analysis = EntryPointAnalysis {
            import_line: 0,
            has_os_import: true,
            call_line: Some(2),
            call_indent: String::new(),
            call_context: None,
            otel_already_configured: false,
        };

        apply_entry_point_modifications(&entry, &analysis, Framework::Generic).unwrap();

        let result = fs::read_to_string(&entry).unwrap();
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn apply_entry_point_modifications_no_call_line() {
        let dir = TempDir::new().unwrap();
        let entry = dir.path().join("empty_ish.py");
        fs::write(&entry, "import os\n").unwrap();

        let analysis = EntryPointAnalysis {
            import_line: 0,
            has_os_import: true,
            call_line: None,
            call_indent: String::new(),
            call_context: None,
            otel_already_configured: false,
        };

        apply_entry_point_modifications(&entry, &analysis, Framework::Generic).unwrap();

        let result = fs::read_to_string(&entry).unwrap();
        assert!(result.contains("from coval_tracing import configure_coval_tracing"));
        assert!(!result.contains("configure_coval_tracing(simulation_output_id)"));
    }

    #[test]
    fn apply_entry_point_modifications_livekit_indented() {
        let dir = TempDir::new().unwrap();
        let entry = dir.path().join("agent.py");
        fs::write(
            &entry,
            "import os\nfrom livekit.agents import AgentSession\n\nasync def entrypoint(ctx):\n    session = AgentSession()\n",
        )
        .unwrap();

        let analysis = EntryPointAnalysis {
            import_line: 1,
            has_os_import: true,
            call_line: Some(4),
            call_indent: "    ".to_string(),
            call_context: Some("inside entrypoint()".to_string()),
            otel_already_configured: false,
        };

        apply_entry_point_modifications(&entry, &analysis, Framework::Livekit).unwrap();

        let result = fs::read_to_string(&entry).unwrap();
        assert!(result.contains("from coval_tracing import configure_coval_tracing"));
        assert!(result.contains("    configure_coval_tracing(simulation_output_id)"));
        assert!(result.contains("ctx"));
    }
}
