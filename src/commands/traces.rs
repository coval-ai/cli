use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use crate::client::CovalClient;

const EXPECTED_VALIDATION_NOT_FOUND: &str = "Simulation output not found";

// ─── Command types ────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum TraceCommands {
    /// Setup Coval traces for LiveKit Agents or Pipecat; other Python voice agents are not currently validated
    Setup(SetupArgs),
    /// Validate your Coval traces configuration by sending a test span
    Validate(ValidateArgs),
}

#[derive(Args)]
pub struct SetupArgs {
    /// Override detected framework
    #[arg(long, value_enum)]
    pub framework: Option<Framework>,

    /// Skip the post-setup validation step
    #[arg(long, default_value = "false")]
    pub no_validate: bool,

    /// Project directory to analyze (defaults to current directory)
    #[arg(long)]
    pub dir: Option<PathBuf>,

    /// Specify the entry-point file (relative to --dir or absolute).
    /// Skips interactive file selection.
    #[arg(long)]
    pub entry_point: Option<PathBuf>,

    /// Auto-confirm setup prompts without interaction.
    /// Answers yes to "Apply changes?", "Validate now?", and "Continue and overwrite?".
    #[arg(long, default_value = "false")]
    pub yes: bool,
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
            Framework::Generic => write!(f, "Generic Python (not currently validated)"),
        }
    }
}

// ─── Entry point ──────────────────────────────────────────────────────────────

pub async fn execute(cmd: TraceCommands, client: Option<&CovalClient>) -> Result<()> {
    match cmd {
        TraceCommands::Setup(args) => setup(args, client).await,
        TraceCommands::Validate(args) => {
            let client = client.ok_or_else(|| {
                anyhow::anyhow!(
                    "Not authenticated. Run `coval login` or set COVAL_API_KEY environment variable."
                )
            })?;
            validate(args, client).await
        }
    }
}

// ─── setup ────────────────────────────────────────────────────────────────────

async fn setup(args: SetupArgs, client: Option<&CovalClient>) -> Result<()> {
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
    if matches!(framework, Framework::Generic) {
        println!(
            "  {} Framework-specific automation is currently supported for {} and {} only.",
            "!".yellow(),
            "LiveKit Agents".bold(),
            "Pipecat".bold()
        );
        println!(
            "    Other Python voice agents are not currently validated and may require manual instrumentation."
        );
    }

    // ── Step 3: Find entry point ──────────────────────────────────────────────
    let entry_path = if let Some(ep) = args.entry_point {
        resolve_entry_point(&dir, &ep)?
    } else {
        pick_entry_point(&dir, framework, args.yes)?
    };
    println!("  {} Selected: {}", "✓".green(), entry_path.display());

    // ── Step 5: Analyze entry file ────────────────────────────────────────────
    println!();
    println!(
        "{}",
        format!("Analyzing {}...", entry_path.display()).cyan()
    );

    let analysis = analyze_entry_point(&entry_path, framework)?;
    print_analysis(&analysis);

    let entry_plan = plan_entry_point_modifications(&analysis, framework)?;
    let tracing_file = dir.join("coval_tracing.py");
    let tracing_content = generate_coval_tracing_py();
    let tracing_file_exists = tracing_file.exists();
    let tracing_needs_update = match fs::read_to_string(&tracing_file) {
        Ok(existing) => existing != tracing_content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => true,
        Err(err) => {
            return Err(err).with_context(|| format!("Failed to read {}", tracing_file.display()));
        }
    };

    if entry_plan.is_empty() && !tracing_needs_update {
        println!();
        println!(
            "  {} Tracing setup is already present and no changes are needed.",
            "✓".green()
        );
        return Ok(());
    }

    if analysis.otel_already_configured {
        println!(
            "\n  {} OpenTelemetry is already configured in this file.",
            "!".yellow()
        );
        let confirmed = if args.yes {
            println!("  {} Auto-confirmed overwrite (--yes)", "✓".green());
            true
        } else {
            Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Continue and overwrite?")
                .default(false)
                .interact()?
        };
        if !confirmed {
            println!("Aborted.");
            return Ok(());
        }
    }

    // ── Step 6: Show plan and confirm ─────────────────────────────────────────
    println!();
    println!("{}", "Here's what I'll do:".bold());

    if tracing_needs_update && tracing_file_exists {
        println!(
            "  {} {} — update existing OTel setup helper",
            "~".yellow(),
            "coval_tracing.py".bold()
        );
        println!(
            "    (backup → {})",
            next_backup_path(&tracing_file).display()
        );
    } else if tracing_needs_update {
        println!(
            "  {} {} — create OTel setup helper",
            "+".green(),
            "coval_tracing.py".bold()
        );
    }

    let entry_name = entry_path.file_name().unwrap_or_default().to_string_lossy();
    if !entry_plan.is_empty() {
        println!(
            "  {} {} — update entry-point tracing hooks",
            "~".yellow(),
            entry_name.bold()
        );
        println!("    (backup → {}.bak)", entry_path.display());
        if !entry_plan.import_lines.is_empty() && analysis.import_line > 0 {
            println!("    - Insert import at line {}", analysis.import_line + 2);
        }
        if !entry_plan.setup_lines.is_empty() {
            if let Some(ref ctx) = analysis.call_context {
                println!("    - Insert setup_coval_tracing() before: {}", ctx.cyan());
            } else if let Some(ref notice) = entry_plan.fallback_notice {
                println!("    - {}", notice);
            }
        }
        if !entry_plan.session_lines.is_empty() {
            println!("    - Add LiveKit session instrumentation after session.start()");
        }
        if !entry_plan.body_lines.is_empty() {
            println!("    - Add Pipecat args.body simulation ID extraction");
        }
        if !entry_plan.dialin_lines.is_empty() {
            println!("    - Add Pipecat on_dialin_connected simulation ID fallback");
        }
        if !entry_plan.task_lines.is_empty() {
            println!("    - Enable Pipecat PipelineTask tracing");
        }
    }
    if matches!(framework, Framework::Generic) && !entry_plan.is_empty() {
        println!("    - Manual review and manual instrumentation may be required before deploy");
    }

    println!();
    let apply = if args.yes {
        println!("  {} Auto-confirmed apply (--yes)", "✓".green());
        true
    } else {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Apply these changes?")
            .default(true)
            .interact()?
    };
    if !apply {
        println!("Aborted. No files were modified.");
        return Ok(());
    }

    // ── Step 7: Write files ───────────────────────────────────────────────────
    let tracing_backup = if tracing_needs_update {
        backup_existing_file(&tracing_file)?
    } else {
        None
    };
    if tracing_needs_update {
        fs::write(&tracing_file, tracing_content)
            .with_context(|| format!("Failed to write {}", tracing_file.display()))?;
    }

    if !entry_plan.is_empty() {
        apply_entry_point_modifications(&entry_path, &analysis, &entry_plan)?;
    }

    println!();
    println!("{}", "Done! Files modified:".bold());
    if tracing_needs_update && tracing_file_exists {
        if let Some(path) = tracing_backup {
            println!(
                "  {} coval_tracing.py (updated, backup at {})",
                "~".yellow(),
                path.display()
            );
        } else {
            println!("  {} coval_tracing.py (updated)", "~".yellow());
        }
    } else if tracing_needs_update {
        println!("  {} coval_tracing.py (created)", "+".green());
    }
    if !entry_plan.is_empty() {
        println!(
            "  {} {} (modified, backup at {}.bak)",
            "~".yellow(),
            entry_name,
            entry_path.display()
        );
    }

    // ── Step 8: Configure LiveKit SIP trunks ───────────────────────────────────
    if matches!(framework, Framework::Livekit) {
        println!();
        println!("{}", "Configuring LiveKit SIP trunks...".cyan());

        match read_livekit_creds(&dir) {
            Some(creds) => match configure_livekit_sip_trunks(&creds).await {
                Ok((updated, total)) => {
                    if total == 0 {
                        println!(
                            "  {} No inbound SIP trunks found — you may need to create one.",
                            "!".yellow()
                        );
                    } else if updated == 0 {
                        println!(
                            "  {} All {} SIP trunk(s) already configured for Coval headers",
                            "✓".green(),
                            total
                        );
                    } else {
                        println!(
                            "  {} Updated {}/{} SIP trunk(s) with Coval header mapping",
                            "✓".green(),
                            updated,
                            total
                        );
                    }
                }
                Err(e) => {
                    println!("  {} Could not configure SIP trunks: {}", "!".yellow(), e);
                    println!(
                        "    You may need to manually add headers_to_attributes to your trunk."
                    );
                    println!("    See: {}", "https://docs.livekit.io/sip/trunk/".bold());
                }
            },
            None => {
                println!(
                    "  {} LiveKit credentials not found (LIVEKIT_URL, LIVEKIT_API_KEY, LIVEKIT_API_SECRET)",
                    "!".yellow()
                );
                println!("    Skipping automatic SIP trunk configuration.");
                println!(
                    "    You must manually add {} to your inbound SIP trunk's headers_to_attributes.",
                    "X-Coval-Simulation-Id".bold()
                );
            }
        }
    }

    // ── Step 9: Next steps ────────────────────────────────────────────────────
    println!();
    println!("{}", "Next steps:".bold());
    println!(
        "  1. Set your API key: {}",
        "export COVAL_API_KEY=<your-key>".bold()
    );
    if matches!(framework, Framework::Generic) {
        println!("  2. Restart (or redeploy) your agent and review the generated changes before testing.");
        println!("     The generated helper buffers spans until the simulation ID is set.");
        println!("     Additional manual instrumentation may still be required.");
    } else {
        println!("  2. Restart (or redeploy) your agent — tracing is now enabled automatically.");
        println!("     Coval sends the simulation ID via SIP header on each call.");
        println!("     Spans are buffered until the ID arrives — no manual config needed.");
    }
    println!(
        "  3. Run a simulation from {} to verify traces are collected.",
        "https://app.coval.dev".bold()
    );
    if matches!(framework, Framework::Generic) {
        println!(
            "     Other Python voice agents are not currently validated; review the generated changes and expect manual instrumentation if traces do not appear."
        );
    }

    if !args.no_validate {
        println!();
        let do_validate = if args.yes {
            println!("  {} Auto-confirmed validate (--yes)", "✓".green());
            true
        } else {
            Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Validate auth now by sending a test trace?")
                .default(true)
                .interact()?
        };
        if do_validate {
            let client = client.ok_or_else(|| {
                anyhow::anyhow!(
                    "Not authenticated. Run `coval login` or set COVAL_API_KEY environment variable before validating traces."
                )
            })?;
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

    let traces_url = client.url("/v1/traces");

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
        .post(traces_url)
        .header("x-api-key", client.api_key())
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
    } else if status == reqwest::StatusCode::NOT_FOUND {
        // 404 "Simulation output not found" means auth succeeded — the endpoint
        // just can't associate the test trace with a real simulation. Treat as OK.
        let body = resp.text().await.unwrap_or_default();
        if is_expected_validation_not_found(&body) {
            println!("{}", "OK".green().bold());
            println!(
                "  {} API key is valid. Ready to collect traces.",
                "✓".green(),
            );
            println!(
                "  Run a simulation from {} to see traces in action.",
                "https://app.coval.dev".bold()
            );
        } else {
            println!("{}", "FAILED".red().bold());
            println!("  {} HTTP {}: {}", "✗".red(), status, body);
            return Err(anyhow::anyhow!("Trace validation failed (HTTP {})", status));
        }
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

fn is_expected_validation_not_found(body: &str) -> bool {
    let trimmed = body.trim();
    if trimmed == EXPECTED_VALIDATION_NOT_FOUND {
        return true;
    }

    let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
        return false;
    };

    value.get("message").and_then(|message| message.as_str()) == Some(EXPECTED_VALIDATION_NOT_FOUND)
        || value
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(|message| message.as_str())
            == Some(EXPECTED_VALIDATION_NOT_FOUND)
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

// ─── Non-interactive helpers ──────────────────────────────────────────────────

fn resolve_entry_point(dir: &Path, entry_point_arg: &Path) -> Result<PathBuf> {
    let p = if entry_point_arg.is_absolute() {
        entry_point_arg.to_path_buf()
    } else {
        dir.join(entry_point_arg)
    };
    if !p.exists() {
        return Err(anyhow::anyhow!("Entry point not found: {}", p.display()));
    }
    println!("  {} Entry point: {}", "✓".green(), p.display());
    Ok(p)
}

// ─── Entry point selection ────────────────────────────────────────────────────

fn pick_entry_point(dir: &Path, framework: Framework, non_interactive: bool) -> Result<PathBuf> {
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

    if candidates.len() == 1 {
        return Ok(candidates.into_iter().next().unwrap());
    }

    if non_interactive {
        return Err(anyhow::anyhow!(
            "Multiple entry point candidates found: {}. Re-run with --entry-point to choose which file to patch.",
            candidates
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let items: Vec<String> = candidates
        .iter()
        .map(|path| path.display().to_string())
        .collect();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Multiple entry point files found — which one should I patch?")
        .items(&items)
        .default(0)
        .interact()?;
    Ok(candidates.into_iter().nth(selection).unwrap())
}

// ─── Entry point analysis ─────────────────────────────────────────────────────

struct EntryPointAnalysis {
    /// 0-indexed line; the import is inserted BEFORE `import_line + 1`
    /// (i.e., immediately after the last existing import)
    import_line: usize,
    /// Whether `import os` is already present in the file
    has_os_import: bool,
    /// 0-indexed line before which to insert the setup_coval_tracing() call
    /// (uses original line numbers, before any insertions are applied)
    call_line: Option<usize>,
    /// Indentation to use for the inserted call block
    call_indent: String,
    /// Human-readable description of what was found at call_line
    call_context: Option<String>,
    /// Whether OpenTelemetry is already configured
    otel_already_configured: bool,
    /// Whether setup_coval_tracing is already imported from coval_tracing
    has_setup_import: bool,
    /// Whether set_simulation_id is already imported from coval_tracing
    has_set_simulation_import: bool,
    /// Whether instrument_session is already imported from coval_tracing
    has_instrument_import: bool,
    /// Whether setup_coval_tracing() is already called anywhere in the file
    has_setup_call: bool,
    /// Whether instrument_session(...) is already called in the file
    has_instrument_call: bool,
    /// Whether Pipecat args.body extraction has already been inserted
    has_pipecat_body_extraction: bool,
    /// Whether Pipecat on_dialin_connected extraction has already been inserted
    has_pipecat_dialin_extraction: bool,
    /// 0-indexed line of `await session.start(...)` for LiveKit session instrumentation.
    /// instrument_session() is inserted AFTER this line.
    session_start_line: Option<usize>,
    /// Indentation at the session.start() line
    session_start_indent: String,
    /// 0-indexed line inside `on_dialin_connected` handler body for Pipecat SIP header extraction.
    dialin_handler_line: Option<usize>,
    /// Indentation inside the dialin handler body
    dialin_handler_indent: String,
    /// 0-indexed line after dialin_settings parsing for Pipecat args.body SIP header extraction.
    body_extraction_line: Option<usize>,
    /// Indentation for body extraction code
    body_extraction_indent: String,
    /// 0-indexed line before the closing `)` of `PipelineTask(...)` for Pipecat tracing enablement.
    pipecat_task_line: Option<usize>,
    /// Indentation for injected Pipecat `PipelineTask(...)` keyword arguments.
    pipecat_task_indent: String,
}

struct EntryPointModificationPlan {
    import_lines: Vec<String>,
    setup_lines: Vec<String>,
    setup_after_imports: bool,
    session_lines: Vec<String>,
    body_lines: Vec<String>,
    dialin_lines: Vec<String>,
    task_lines: Vec<String>,
    fallback_notice: Option<String>,
}

impl EntryPointModificationPlan {
    fn is_empty(&self) -> bool {
        self.import_lines.is_empty()
            && self.setup_lines.is_empty()
            && self.session_lines.is_empty()
            && self.body_lines.is_empty()
            && self.dialin_lines.is_empty()
            && self.task_lines.is_empty()
    }
}

fn analyze_entry_point(path: &Path, framework: Framework) -> Result<EntryPointAnalysis> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let lines: Vec<&str> = content.lines().collect();

    let otel_already_configured = lines.iter().any(|l| {
        l.contains("opentelemetry")
            || l.contains("TracerProvider")
            || l.contains("setup_coval_tracing")
    });
    let has_setup_import = lines.iter().any(|line| {
        line.contains("from coval_tracing import") && line.contains("setup_coval_tracing")
    });
    let has_set_simulation_import = lines.iter().any(|line| {
        line.contains("from coval_tracing import") && line.contains("set_simulation_id")
    });
    let has_instrument_import = lines.iter().any(|line| {
        line.contains("from coval_tracing import") && line.contains("instrument_session")
    });
    let has_setup_call = lines
        .iter()
        .any(|line| line.contains("setup_coval_tracing()"));
    let has_instrument_call = lines
        .iter()
        .any(|line| line.contains("instrument_session("));
    let has_pipecat_body_extraction = lines.iter().any(|line| {
        line.contains("# Coval tracing: extract simulation ID from SIP headers in body")
    });
    let has_pipecat_dialin_extraction = lines
        .iter()
        .any(|line| line.contains("# Coval tracing: extract simulation ID from SIP headers"));

    let import_line = find_last_import_line(&lines);

    let has_os_import = lines
        .iter()
        .any(|l| l.trim() == "import os" || l.trim().starts_with("import os "));

    let (call_line, call_indent, call_context) = match framework {
        Framework::Pipecat => find_pipecat_injection(&lines),
        Framework::Livekit => find_livekit_injection(&lines),
        Framework::Generic => find_generic_injection(&lines),
    };

    let (session_start_line, session_start_indent) = if matches!(framework, Framework::Livekit) {
        find_livekit_session_start(&lines)
    } else {
        (None, String::new())
    };

    let (dialin_handler_line, dialin_handler_indent) = if matches!(framework, Framework::Pipecat) {
        find_pipecat_dialin_handler(&lines)
    } else {
        (None, String::new())
    };

    let (body_extraction_line, body_extraction_indent) = if matches!(framework, Framework::Pipecat)
    {
        find_pipecat_body_extraction(&lines)
    } else {
        (None, String::new())
    };

    let (pipecat_task_line, pipecat_task_indent) = if matches!(framework, Framework::Pipecat) {
        find_pipecat_task_enable_tracing(&lines)
    } else {
        (None, String::new())
    };

    Ok(EntryPointAnalysis {
        import_line,
        has_os_import,
        call_line,
        call_indent,
        call_context,
        otel_already_configured,
        has_setup_import,
        has_set_simulation_import,
        has_instrument_import,
        has_setup_call,
        has_instrument_call,
        has_pipecat_body_extraction,
        has_pipecat_dialin_extraction,
        session_start_line,
        session_start_indent,
        dialin_handler_line,
        dialin_handler_indent,
        body_extraction_line,
        body_extraction_indent,
        pipecat_task_line,
        pipecat_task_indent,
    })
}

fn find_last_import_line(lines: &[&str]) -> usize {
    let mut last = 0;
    for (i, line) in lines.iter().enumerate() {
        // Only consider top-level imports (no leading whitespace).
        // Indented `from x import y` inside function bodies must not be counted.
        let first_char = line.chars().next().unwrap_or(' ');
        if first_char.is_whitespace() {
            continue;
        }
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
    // Prefer: first statement inside the entrypoint function.
    // Detect both `async def entrypoint(` and `@server.rtc_session` decorator patterns.
    for (i, line) in lines.iter().enumerate() {
        let is_entrypoint = line.contains("async def entrypoint");
        let is_rtc_session = line.trim().starts_with("@") && line.contains("rtc_session");

        if is_entrypoint || is_rtc_session {
            // For @server.rtc_session, find the async def line first
            let fn_line = if is_rtc_session {
                let mut found = None;
                for (j, candidate) in lines.iter().enumerate().skip(i + 1) {
                    let t = candidate.trim();
                    if t.starts_with("async def ") {
                        found = Some(j);
                        break;
                    }
                    // Stop if we hit a non-decorator, non-blank line
                    if !t.is_empty() && !t.starts_with('#') && !t.starts_with('@') {
                        break;
                    }
                }
                match found {
                    Some(l) => l,
                    None => continue,
                }
            } else {
                i
            };

            // Find the first real statement in the body
            for (j, candidate) in lines.iter().enumerate().skip(fn_line + 1) {
                let t = candidate.trim();
                if !t.is_empty() && !t.starts_with('#') {
                    return (
                        Some(j),
                        leading_whitespace(candidate),
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

fn find_livekit_session_start(lines: &[&str]) -> (Option<usize>, String) {
    // Find `await session.start(` or `session.start(` and return the line AFTER the
    // entire call completes (tracking parentheses for multi-line calls).
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.contains("session.start(") || (t.contains(".start(") && t.contains("session")) {
            let indent = leading_whitespace(line);
            // Track parentheses to find the end of the call
            let mut depth: i32 = 0;
            for (j, current) in lines.iter().enumerate().skip(i) {
                for ch in current.chars() {
                    if ch == '(' {
                        depth += 1;
                    } else if ch == ')' {
                        depth -= 1;
                    }
                }
                if depth <= 0 {
                    // The call ends on line j; insert after it
                    return (Some(j), indent);
                }
            }
            // Fallback: single-line call
            return (Some(i), indent);
        }
    }
    (None, String::new())
}

fn find_pipecat_body_extraction(lines: &[&str]) -> (Option<usize>, String) {
    // Find where to inject args.body SIP header extraction inside bot().
    // Look for the dialin_settings parsing block (DailyDialinSettings) and insert after it.
    // Also handles simpler patterns like `body = getattr(args, "body"` or `body = args.body`.
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.contains("DailyDialinSettings(")
            || (t.contains("dialin_settings") && t.contains("body"))
        {
            // Find the end of this block — look for the next unindented or less-indented line
            let base_indent = leading_whitespace(line);
            for (j, inner_line) in lines.iter().enumerate().skip(i + 1) {
                let inner = inner_line.trim();
                if inner.is_empty() {
                    // Blank line after the block — insert here
                    let indent = if base_indent.is_empty() {
                        "    ".to_string() // default to 4 spaces
                    } else {
                        base_indent.clone()
                    };
                    return (Some(j + 1), indent);
                }
            }
        }
    }
    (None, String::new())
}

fn find_pipecat_dialin_handler(lines: &[&str]) -> (Option<usize>, String) {
    // Find the `on_dialin_connected` event handler body to inject SIP header extraction.
    // Pattern: @transport.event_handler("on_dialin_connected") followed by async def, then body.
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.contains("on_dialin_connected")
            && (t.contains("event_handler") || t.contains("async def"))
        {
            // Find the async def line (might be the decorator line itself or the next line)
            let fn_line = if t.starts_with("@") {
                // Decorator — find the async def after it
                let mut found = i;
                for (j, candidate) in lines.iter().enumerate().skip(i + 1) {
                    if candidate.trim().starts_with("async def") {
                        found = j;
                        break;
                    }
                }
                found
            } else {
                i
            };

            // Find the first real statement in the body
            for (j, candidate) in lines.iter().enumerate().skip(fn_line + 1) {
                let inner = candidate.trim();
                if !inner.is_empty() && !inner.starts_with('#') {
                    let indent = leading_whitespace(candidate);
                    // Insert AFTER the first statement (usually a logger.info call)
                    return (Some(j + 1), indent);
                }
            }
        }
    }
    (None, String::new())
}

fn find_pipecat_task_enable_tracing(lines: &[&str]) -> (Option<usize>, String) {
    for (i, line) in lines.iter().enumerate() {
        if !line.contains("PipelineTask(") {
            continue;
        }

        let default_indent = format!("{}    ", leading_whitespace(line));
        let mut arg_indent = String::new();
        let mut depth: i32 = 0;
        let mut has_enable_tracing = false;

        for (j, current) in lines.iter().enumerate().skip(i) {
            let trimmed = current.trim();

            if j > i && arg_indent.is_empty() && !trimmed.is_empty() && trimmed != ")" {
                arg_indent = leading_whitespace(current);
            }

            if trimmed.contains("enable_tracing") {
                has_enable_tracing = true;
            }

            for ch in current.chars() {
                if ch == '(' {
                    depth += 1;
                } else if ch == ')' {
                    depth -= 1;
                }
            }

            if depth <= 0 {
                if has_enable_tracing || j == i {
                    return (None, String::new());
                }
                return (
                    Some(j),
                    if arg_indent.is_empty() {
                        default_indent
                    } else {
                        arg_indent
                    },
                );
            }
        }
    }

    (None, String::new())
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
            "  {} No specific injection point found automatically",
            "~".yellow()
        ),
    }
}

// ─── File modification ────────────────────────────────────────────────────────

fn next_backup_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "backup".to_string());
    let parent = path.parent().unwrap_or_else(|| Path::new("."));

    let mut attempt = 0usize;
    loop {
        let candidate = if attempt == 0 {
            parent.join(format!("{file_name}.bak"))
        } else {
            parent.join(format!("{file_name}.bak.{attempt}"))
        };
        if !candidate.exists() {
            return candidate;
        }
        attempt += 1;
    }
}

fn backup_existing_file(path: &Path) -> Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }

    let backup_path = next_backup_path(path);
    fs::copy(path, &backup_path).with_context(|| {
        format!(
            "Failed to create backup {} from {}",
            backup_path.display(),
            path.display()
        )
    })?;
    Ok(Some(backup_path))
}

fn plan_entry_point_modifications(
    analysis: &EntryPointAnalysis,
    framework: Framework,
) -> Result<EntryPointModificationPlan> {
    let mut import_lines = Vec::new();
    if !analysis.has_os_import {
        import_lines.push("import os".to_string());
    }

    let mut missing_imports = Vec::new();
    if !analysis.has_setup_import {
        missing_imports.push("setup_coval_tracing");
    }
    if !analysis.has_set_simulation_import {
        missing_imports.push("set_simulation_id");
    }
    if matches!(framework, Framework::Livekit)
        && analysis.session_start_line.is_some()
        && !analysis.has_instrument_import
    {
        missing_imports.push("instrument_session");
    }
    if !missing_imports.is_empty() {
        import_lines.push(format!(
            "from coval_tracing import {}",
            missing_imports.join(", ")
        ));
    }

    let mut setup_lines = Vec::new();
    let mut setup_after_imports = false;
    let mut fallback_notice = None;
    if matches!(framework, Framework::Pipecat) {
        if !analysis.has_setup_call {
            setup_lines.push("setup_coval_tracing()".to_string());
            setup_after_imports = true;
        }
    } else if !analysis.has_setup_call {
        if analysis.call_line.is_some() {
            setup_lines = generate_call_snippet(framework, &analysis.call_indent);
        } else if matches!(framework, Framework::Generic) {
            fallback_notice = Some(
                "No safe function-level injection point found; insert setup after imports instead."
                    .to_string(),
            );
            setup_lines = generate_call_snippet(framework, "");
            setup_after_imports = true;
        } else {
            return Err(anyhow::anyhow!(
                "Could not find a suitable injection point for setup_coval_tracing() in {} mode. Re-run with --entry-point to choose a different file, or add setup_coval_tracing() manually.",
                framework
            ));
        }
    }

    let session_lines = if analysis.has_instrument_call {
        Vec::new()
    } else if analysis.session_start_line.is_some() {
        vec![format!(
            "{}instrument_session(session)  # Hook Coval trace events",
            analysis.session_start_indent
        )]
    } else {
        Vec::new()
    };

    let body_lines = if analysis.has_pipecat_body_extraction {
        Vec::new()
    } else if analysis.body_extraction_line.is_some() {
        let indent = &analysis.body_extraction_indent;
        vec![
            format!("{indent}# Coval tracing: extract simulation ID from SIP headers in body"),
            format!("{indent}setup_coval_tracing()"),
            format!("{indent}_coval_sim = \"\""),
            format!("{indent}if isinstance(body, dict):"),
            format!("{indent}    _raw_dialin = body.get(\"dialin_settings\") or body.get(\"dialinSettings\") or {{}}"),
            format!("{indent}    if isinstance(_raw_dialin, dict):"),
            format!("{indent}        _sip_h = _raw_dialin.get(\"sip_headers\") or _raw_dialin.get(\"sipHeaders\") or {{}}"),
            format!("{indent}        if isinstance(_sip_h, dict):"),
            format!("{indent}            _coval_sim = _sip_h.get(\"X-Coval-Simulation-Id\") or _sip_h.get(\"x-coval-simulation-id\") or \"\""),
            format!("{indent}if not _coval_sim:"),
            format!("{indent}    _coval_sim = os.environ.get(\"COVAL_SIMULATION_ID\", \"\")"),
            format!("{indent}if _coval_sim:"),
            format!("{indent}    set_simulation_id(_coval_sim)"),
            String::new(),
        ]
    } else {
        Vec::new()
    };

    let dialin_lines = if analysis.has_pipecat_dialin_extraction {
        Vec::new()
    } else if analysis.dialin_handler_line.is_some() {
        let indent = &analysis.dialin_handler_indent;
        vec![
            format!("{indent}# Coval tracing: extract simulation ID from SIP headers"),
            format!("{indent}_coval_sim_id = \"\""),
            format!("{indent}sip_headers = data.get(\"sipHeaders\") or data.get(\"sip_headers\") or {{}}"),
            format!("{indent}if isinstance(sip_headers, dict):"),
            format!("{indent}    _coval_sim_id = sip_headers.get(\"X-Coval-Simulation-Id\") or sip_headers.get(\"x-coval-simulation-id\") or \"\""),
            format!("{indent}if not _coval_sim_id:"),
            format!("{indent}    _coval_sim_id = os.environ.get(\"COVAL_SIMULATION_ID\", \"\")"),
            format!("{indent}if _coval_sim_id:"),
            format!("{indent}    set_simulation_id(_coval_sim_id)"),
        ]
    } else {
        Vec::new()
    };

    let task_lines = if analysis.pipecat_task_line.is_some() {
        vec![format!(
            "{}enable_tracing=True,",
            analysis.pipecat_task_indent
        )]
    } else {
        Vec::new()
    };

    Ok(EntryPointModificationPlan {
        import_lines,
        setup_lines,
        setup_after_imports,
        session_lines,
        body_lines,
        dialin_lines,
        task_lines,
        fallback_notice,
    })
}

fn apply_entry_point_modifications(
    path: &Path,
    analysis: &EntryPointAnalysis,
    plan: &EntryPointModificationPlan,
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

    let import_pos = analysis.import_line + 1;
    if !plan.import_lines.is_empty() || (plan.setup_after_imports && !plan.setup_lines.is_empty()) {
        let mut module_lines = plan.import_lines.clone();
        if plan.setup_after_imports {
            module_lines.extend(plan.setup_lines.clone());
        }
        insertions.push((import_pos, module_lines));
    }

    if !plan.setup_after_imports && !plan.setup_lines.is_empty() {
        let setup_pos = analysis.call_line.unwrap_or(import_pos);
        insertions.push((setup_pos, plan.setup_lines.clone()));
    }

    // LiveKit: insert instrument_session(session) after session.start()
    if let Some(session_line) = analysis.session_start_line {
        if !plan.session_lines.is_empty() {
            insertions.push((session_line + 1, plan.session_lines.clone()));
        }
    }

    // Pipecat: insert args.body SIP header extraction (primary path for PCC)
    if let Some(body_line) = analysis.body_extraction_line {
        if !plan.body_lines.is_empty() {
            insertions.push((body_line, plan.body_lines.clone()));
        }
    }

    // Pipecat: insert SIP header extraction inside on_dialin_connected handler (fallback)
    if let Some(dialin_line) = analysis.dialin_handler_line {
        if !plan.dialin_lines.is_empty() {
            insertions.push((dialin_line, plan.dialin_lines.clone()));
        }
    }

    // Pipecat: enable OpenTelemetry tracing on PipelineTask(...)
    if let Some(task_line) = analysis.pipecat_task_line {
        if !plan.task_lines.is_empty() {
            insertions.push((task_line, plan.task_lines.clone()));
        }
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
    let snippet = match framework {
        Framework::Livekit => format!(
            "{indent}# Coval tracing: extract simulation ID from SIP participant attributes.\n\
             {indent}# Coval sends X-Coval-Simulation-Id as a SIP header when dialing your agent.\n\
             {indent}# LiveKit exposes SIP headers as participant attributes with the sip.h. prefix.\n\
             {indent}setup_coval_tracing()\n\
             {indent}def _coval_extract_sim_id(participant):\n\
             {indent}    sim_id = participant.attributes.get(\"sip.h.X-Coval-Simulation-Id\", \"\")\n\
             {indent}    if not sim_id:\n\
             {indent}        sim_id = os.environ.get(\"COVAL_SIMULATION_ID\", \"\")\n\
             {indent}    if sim_id:\n\
             {indent}        set_simulation_id(sim_id)\n\
             {indent}# Check participants already in the room (SIP caller may already be connected)\n\
             {indent}for _p in ctx.room.remote_participants.values():\n\
             {indent}    _coval_extract_sim_id(_p)\n\
             {indent}# Also listen for new participants and attribute changes\n\
             {indent}ctx.room.on(\"participant_connected\", _coval_extract_sim_id)\n\
             {indent}ctx.room.on(\"participant_attributes_changed\", lambda changed, p: _coval_extract_sim_id(p))"
        ),
        Framework::Pipecat => format!(
            "{indent}# Coval tracing: set up once at startup.\n\
             {indent}# Simulation ID is extracted from SIP headers in on_dialin_connected.\n\
             {indent}setup_coval_tracing()"
        ),
        Framework::Generic => format!(
            "{indent}# Coval tracing: set up once at startup.\n\
             {indent}# The simulation ID is read from the COVAL_SIMULATION_ID env var,\n\
             {indent}# or extracted from SIP headers if available.\n\
             {indent}setup_coval_tracing()\n\
             {indent}_sim_id = os.environ.get(\"COVAL_SIMULATION_ID\", \"\")\n\
             {indent}if _sim_id:\n\
             {indent}    set_simulation_id(_sim_id)"
        ),
    };

    let mut lines: Vec<String> = snippet.lines().map(|l| l.to_string()).collect();
    lines.push(String::new()); // blank line after the block
    lines
}

// ─── coval_tracing.py template ────────────────────────────────────────────────

fn generate_coval_tracing_py() -> String {
    r#"# coval_tracing.py — generated by `coval traces setup`
#
# This module configures OpenTelemetry tracing for Coval evaluation.
# Call setup_coval_tracing() at startup or when a new call/session begins,
# then set_simulation_id() when the Coval simulation ID arrives (typically
# via SIP header or env var).
#
# Spans are buffered until set_simulation_id() is called, so no spans
# are lost even if tracing is initialized before the call connects.
#
# Prefer Coval span naming conventions for the richest UI and metric support:
#   llm           — LLM inference spans
#   tts           — Text-to-speech spans
#   stt           — Speech-to-text spans
#   llm_tool_call — Tool/function call spans
#
# See: https://docs.coval.dev/traces

import logging
import os
import threading
from collections import deque
from contextvars import ContextVar
from typing import Deque, Dict, Optional, Sequence
from uuid import uuid4

from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.http.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import SERVICE_NAME, Resource
from opentelemetry.sdk.trace import ReadableSpan, Span, SpanProcessor, TracerProvider
from opentelemetry.sdk.trace.export import SimpleSpanProcessor, SpanExporter, SpanExportResult

logger = logging.getLogger("coval_tracing")

COVAL_API_KEY = os.environ.get("COVAL_API_KEY", "")
COVAL_TRACES_ENDPOINT = "https://api.coval.dev/v1/traces"
MAX_PREACTIVATION_BUFFER = 1024

_INTERNAL_SIMULATION_ID_ATTR = "coval.internal.simulation_id"
_INTERNAL_BUFFER_KEY_ATTR = "coval.internal.buffer_key"

_router: Optional["_ContextualCovalExporter"] = None
_current_simulation_id: ContextVar[Optional[str]] = ContextVar(
    "coval_simulation_id", default=None
)
_current_buffer_key: ContextVar[Optional[str]] = ContextVar(
    "coval_buffer_key", default=None
)

# ── Span renamer ─────────────────────────────────────────────────────────────
# Remaps span names from framework conventions to Coval conventions on start.

_SPAN_NAME_MAP = {
    "llm_request": "llm",
    "llm_request_run": "llm",
    "tts_request": "tts",
    "tts_request_run": "tts",
    "stt_request": "stt",
    "stt_request_run": "stt",
}


class _CovalSpanRenamer(SpanProcessor):
    """Renames spans on start to match Coval conventions."""

    def on_start(self, span: Span, parent_context=None) -> None:
        name = span.name
        if name in _SPAN_NAME_MAP:
            span._name = _SPAN_NAME_MAP[name]
        elif name.startswith("function_call"):
            span._name = "llm_tool_call"
        simulation_id = _current_simulation_id.get()
        if simulation_id:
            span.set_attribute(_INTERNAL_SIMULATION_ID_ATTR, simulation_id)
        buffer_key = _current_buffer_key.get()
        if buffer_key:
            span.set_attribute(_INTERNAL_BUFFER_KEY_ATTR, buffer_key)

    def on_end(self, span: ReadableSpan) -> None:
        pass

    def shutdown(self) -> None:
        pass

    def force_flush(self, timeout_millis: int = 30000) -> bool:
        return True


# ── Contextual exporter ──────────────────────────────────────────────────────

class _ContextualCovalExporter(SpanExporter):
    """Routes spans to simulation-specific exporters and bounded buffers."""

    def __init__(self, api_key: str):
        self._api_key = api_key
        self._lock = threading.RLock()
        self._exporters: Dict[str, OTLPSpanExporter] = {}
        self._buffers: Dict[str, Deque[ReadableSpan]] = {}

    def _get_or_create_exporter(self, simulation_id: str) -> OTLPSpanExporter:
        exporter = self._exporters.get(simulation_id)
        if exporter is None:
            exporter = OTLPSpanExporter(
                endpoint=COVAL_TRACES_ENDPOINT,
                headers={
                    "x-api-key": self._api_key,
                    "X-Simulation-Id": simulation_id,
                },
                timeout=30,
            )
            self._exporters[simulation_id] = exporter
        return exporter

    def _buffer_for_key(self, buffer_key: str) -> Deque[ReadableSpan]:
        buffer = self._buffers.get(buffer_key)
        if buffer is None:
            buffer = deque(maxlen=MAX_PREACTIVATION_BUFFER)
            self._buffers[buffer_key] = buffer
        return buffer

    def activate(self, simulation_id: str, buffer_key: Optional[str]) -> None:
        """Start exporting spans with the given simulation ID."""
        with self._lock:
            exporter = self._get_or_create_exporter(simulation_id)
            buffered = list(self._buffers.pop(buffer_key, ())) if buffer_key else []
        if buffered:
            logger.info(
                "Flushing %d buffered spans to Coval for simulation_id=%s",
                len(buffered),
                simulation_id,
            )
            exporter.export(buffered)

    def export(self, spans: Sequence[ReadableSpan]) -> SpanExportResult:
        dropped = 0
        batches = []
        with self._lock:
            grouped: Dict[str, list[ReadableSpan]] = {}
            for span in spans:
                attributes = span.attributes or {}
                simulation_id = attributes.get(_INTERNAL_SIMULATION_ID_ATTR)
                if simulation_id:
                    grouped.setdefault(str(simulation_id), []).append(span)
                    continue

                buffer_key = str(
                    attributes.get(_INTERNAL_BUFFER_KEY_ATTR)
                    or _current_buffer_key.get()
                    or "default"
                )
                buffer = self._buffer_for_key(buffer_key)
                before = len(buffer)
                buffer.append(span)
                if before == MAX_PREACTIVATION_BUFFER:
                    dropped += 1

            for simulation_id, batch in grouped.items():
                batches.append((self._get_or_create_exporter(simulation_id), batch))

        if dropped:
            logger.warning(
                "Dropped %d oldest buffered span(s) before simulation ID activation",
                dropped,
            )

        for exporter, batch in batches:
            result = exporter.export(batch)
            if result is not SpanExportResult.SUCCESS:
                return result
        return SpanExportResult.SUCCESS

    def force_flush(self, timeout_millis: int = 30000) -> bool:
        with self._lock:
            exporters = list(self._exporters.values())
        return all(exporter.force_flush(timeout_millis) for exporter in exporters)

    def shutdown(self) -> None:
        with self._lock:
            exporters = list(self._exporters.values())
            self._exporters.clear()
            self._buffers.clear()
        for exporter in exporters:
            exporter.shutdown()


# ── Public API ───────────────────────────────────────────────────────────────

def _begin_tracing_context() -> str:
    buffer_key = uuid4().hex
    _current_buffer_key.set(buffer_key)
    _current_simulation_id.set(None)
    return buffer_key


def setup_coval_tracing(service_name: str = "coval-agent") -> None:
    """Initialize OpenTelemetry tracing for Coval.

    Call at startup or when a new call/session begins. Spans are buffered until
    set_simulation_id() is called. If COVAL_API_KEY is not set, tracing is
    disabled and a warning is logged.
    """
    global _router
    if not COVAL_API_KEY:
        logger.warning("COVAL_API_KEY not set — tracing disabled")
        return
    if _router is None:
        _router = _ContextualCovalExporter(api_key=COVAL_API_KEY)
        resource = Resource.create({SERVICE_NAME: service_name})
        provider = TracerProvider(resource=resource)
        provider.add_span_processor(_CovalSpanRenamer())
        provider.add_span_processor(SimpleSpanProcessor(_router))
        trace.set_tracer_provider(provider)
        logger.info("Coval tracing initialized — waiting for simulation ID")
    else:
        logger.info("Coval tracing reset — waiting for simulation ID")
    _begin_tracing_context()


def set_simulation_id(simulation_id: str) -> None:
    """Activate trace export with the given simulation ID.

    Typically called when the Coval simulation ID arrives via SIP header
    (X-Coval-Simulation-Id) or environment variable (COVAL_SIMULATION_ID).
    """
    if _router and simulation_id:
        buffer_key = _current_buffer_key.get() or _begin_tracing_context()
        _current_simulation_id.set(simulation_id)
        _router.activate(simulation_id, buffer_key)
        logger.info("Coval tracing active for simulation_id=%s", simulation_id)


# ── LiveKit session instrumentation ──────────────────────────────────────────
# Call instrument_session(session) after AgentSession.start() to hook events
# that create Coval-compatible spans for STT, tool calls, and TTFB metrics.

def instrument_session(session) -> None:
    """Hook AgentSession events to create Coval trace spans.

    Must be called after session.start(). Hooks:
    - user_input_transcribed -> creates "stt" spans with transcript attribute
    - function_tools_executed -> creates "llm_tool_call" spans
    - metrics_collected -> creates spans with metrics.ttfb attribute
    """
    tracer = trace.get_tracer("coval.instrumentation")

    def _on_user_input_transcribed(event):
        if not getattr(event, "is_final", False):
            return
        with tracer.start_as_current_span("stt") as span:
            span.set_attribute("transcript", event.transcript or "")

    def _on_function_tools_executed(event):
        for call in getattr(event, "function_calls", []):
            with tracer.start_as_current_span("llm_tool_call") as span:
                span.set_attribute("function.name", getattr(call, "name", ""))
                span.set_attribute("tool_call_id", getattr(call, "call_id", ""))
                span.set_attribute("function.arguments", getattr(call, "arguments", ""))

    def _on_metrics_collected(event):
        metrics = getattr(event, "metrics", None)
        if metrics is None:
            return
        metrics_type = getattr(metrics, "type", "")
        ttfb = getattr(metrics, "ttfb", None)
        if ttfb is None:
            return
        if metrics_type == "llm_metrics":
            span_name = "llm"
        elif metrics_type == "stt_metrics":
            span_name = "stt"
        elif metrics_type == "tts_metrics":
            span_name = "tts"
        else:
            return
        with tracer.start_as_current_span(span_name) as span:
            span.set_attribute("metrics.ttfb", ttfb)

    session.on("user_input_transcribed", _on_user_input_transcribed)
    session.on("function_tools_executed", _on_function_tools_executed)
    session.on("metrics_collected", _on_metrics_collected)
    logger.info("Coval session instrumentation active")


# ── Utility functions for non-LiveKit frameworks ─────────────────────────────

def create_llm_span(input_text: str = "", model: str = "", **kwargs):
    """Create an LLM span. Use as a context manager: with create_llm_span(...) as span: ..."""
    tracer = trace.get_tracer("coval.instrumentation")
    span = tracer.start_span("llm")
    if input_text:
        span.set_attribute("input", input_text)
    if model:
        span.set_attribute("gen_ai.request.model", model)
    for k, v in kwargs.items():
        span.set_attribute(k, v)
    return trace.use_span(span, end_on_exit=True)


def create_stt_span(transcript: str = "", **kwargs):
    """Create an STT span. Use as a context manager."""
    tracer = trace.get_tracer("coval.instrumentation")
    span = tracer.start_span("stt")
    if transcript:
        span.set_attribute("transcript", transcript)
    for k, v in kwargs.items():
        span.set_attribute(k, v)
    return trace.use_span(span, end_on_exit=True)


def create_tts_span(**kwargs):
    """Create a TTS span. Use as a context manager."""
    tracer = trace.get_tracer("coval.instrumentation")
    span = tracer.start_span("tts")
    for k, v in kwargs.items():
        span.set_attribute(k, v)
    return trace.use_span(span, end_on_exit=True)


def create_tool_call_span(name: str = "", call_id: str = "", arguments: str = "", **kwargs):
    """Create a tool call span. Use as a context manager."""
    tracer = trace.get_tracer("coval.instrumentation")
    span = tracer.start_span("llm_tool_call")
    if name:
        span.set_attribute("function.name", name)
    if call_id:
        span.set_attribute("tool_call_id", call_id)
    if arguments:
        span.set_attribute("function.arguments", arguments)
    for k, v in kwargs.items():
        span.set_attribute(k, v)
    return trace.use_span(span, end_on_exit=True)
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

// ─── LiveKit SIP trunk configuration ──────────────────────────────────────────

/// LiveKit credentials read from the project directory.
struct LiveKitCreds {
    url: String,     // e.g. "wss://testproj-idq4nqwp.livekit.cloud"
    api_key: String, // e.g. "APIHSCPz5LgQhut"
    api_secret: String,
}

/// Read LiveKit credentials from .env.local, .env, or environment variables.
fn read_livekit_creds(dir: &Path) -> Option<LiveKitCreds> {
    let mut url = String::new();
    let mut api_key = String::new();
    let mut api_secret = String::new();

    // Try .env.local first, then .env
    for env_file in [".env.local", ".env"] {
        if let Ok(content) = fs::read_to_string(dir.join(env_file)) {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with('#') || !line.contains('=') {
                    continue;
                }
                let (key, val) = match line.split_once('=') {
                    Some((k, v)) => (k.trim(), v.trim().trim_matches(|c| c == '"' || c == '\'')),
                    None => continue,
                };
                match key {
                    "LIVEKIT_URL" => url = val.to_string(),
                    "LIVEKIT_API_KEY" => api_key = val.to_string(),
                    "LIVEKIT_API_SECRET" => api_secret = val.to_string(),
                    _ => {}
                }
            }
        }
    }

    // Fall back to environment variables for any missing values
    if url.is_empty() {
        url = std::env::var("LIVEKIT_URL").unwrap_or_default();
    }
    if api_key.is_empty() {
        api_key = std::env::var("LIVEKIT_API_KEY").unwrap_or_default();
    }
    if api_secret.is_empty() {
        api_secret = std::env::var("LIVEKIT_API_SECRET").unwrap_or_default();
    }

    if url.is_empty() || api_key.is_empty() || api_secret.is_empty() {
        return None;
    }

    Some(LiveKitCreds {
        url,
        api_key,
        api_secret,
    })
}

/// Generate a LiveKit JWT for SIP admin operations.
fn generate_livekit_jwt(api_key: &str, api_secret: &str) -> Result<String> {
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let claims = serde_json::json!({
        "iss": api_key,
        "sub": api_key,
        "iat": now,
        "nbf": now,
        "exp": now + 600,
        "video": { "roomAdmin": true },
        "sip": { "admin": true }
    });

    let header = Header::new(Algorithm::HS256);
    let key = EncodingKey::from_secret(api_secret.as_bytes());
    encode(&header, &claims, &key).context("Failed to generate LiveKit JWT")
}

/// Convert a LiveKit WSS URL to the HTTPS API base URL.
fn livekit_api_url(wss_url: &str) -> String {
    wss_url
        .replace("wss://", "https://")
        .replace("ws://", "http://")
        .trim_end_matches('/')
        .to_string()
}

/// The header mapping we need on the SIP trunk.
const COVAL_SIP_HEADER: &str = "X-Coval-Simulation-Id";
const COVAL_SIP_ATTR: &str = "sip.h.X-Coval-Simulation-Id";

/// List inbound SIP trunks and configure headers_to_attributes on any that are missing it.
async fn configure_livekit_sip_trunks(creds: &LiveKitCreds) -> Result<(usize, usize)> {
    let jwt = generate_livekit_jwt(&creds.api_key, &creds.api_secret)?;
    let base = livekit_api_url(&creds.url);
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    // List inbound trunks
    let list_url = format!("{base}/twirp/livekit.SIP/ListSIPInboundTrunk");
    let resp = http
        .post(&list_url)
        .header("Authorization", format!("Bearer {jwt}"))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({}))
        .send()
        .await
        .context("Failed to list LiveKit SIP trunks")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "LiveKit SIP API error ({}): {}",
            status,
            body
        ));
    }

    let body: serde_json::Value = resp.json().await?;
    let trunks = body["items"].as_array().unwrap_or(&Vec::new()).clone();

    let mut updated = 0;
    let total = trunks.len();

    for trunk in &trunks {
        let trunk_id = trunk["sip_trunk_id"].as_str().unwrap_or("");
        if trunk_id.is_empty() {
            continue;
        }

        // Check if headers_to_attributes already has our header
        let existing = trunk["headers_to_attributes"].as_object();
        let already_configured = existing
            .map(|m| m.contains_key(COVAL_SIP_HEADER))
            .unwrap_or(false);

        if already_configured {
            continue;
        }

        // Build updated headers_to_attributes (preserve existing + add ours)
        let mut headers: HashMap<String, String> = HashMap::new();
        if let Some(existing_map) = existing {
            for (k, v) in existing_map {
                if let Some(s) = v.as_str() {
                    headers.insert(k.clone(), s.to_string());
                }
            }
        }
        headers.insert(COVAL_SIP_HEADER.to_string(), COVAL_SIP_ATTR.to_string());

        // Preserve existing trunk fields so the update doesn't clear them
        let name = trunk["name"].as_str().unwrap_or("");
        let numbers: Vec<&str> = trunk["numbers"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        let allowed_addresses: Vec<&str> = trunk["allowed_addresses"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        let allowed_numbers: Vec<&str> = trunk["allowed_numbers"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        let update_url = format!("{base}/twirp/livekit.SIP/UpdateSIPInboundTrunk");
        // The update API uses camelCase and wraps fields in a "replace" object
        let update_body = serde_json::json!({
            "sipTrunkId": trunk_id,
            "replace": {
                "name": name,
                "numbers": numbers,
                "allowedAddresses": allowed_addresses,
                "allowedNumbers": allowed_numbers,
                "headersToAttributes": headers,
            }
        });

        let update_resp = http
            .post(&update_url)
            .header("Authorization", format!("Bearer {jwt}"))
            .header("Content-Type", "application/json")
            .json(&update_body)
            .send()
            .await;

        match update_resp {
            Ok(r) if r.status().is_success() => {
                let trunk_name = if name.is_empty() {
                    trunk_id.to_string()
                } else {
                    format!("{name} ({trunk_id})")
                };
                println!("    {} Configured SIP trunk: {}", "✓".green(), trunk_name);
                updated += 1;
            }
            Ok(r) => {
                let status = r.status();
                let body = r.text().await.unwrap_or_default();
                println!(
                    "    {} Failed to update trunk {}: {} {}",
                    "✗".red(),
                    trunk_id,
                    status,
                    body
                );
            }
            Err(e) => {
                println!(
                    "    {} Failed to update trunk {}: {}",
                    "✗".red(),
                    trunk_id,
                    e
                );
            }
        }
    }

    Ok((updated, total))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn analysis_with_defaults() -> EntryPointAnalysis {
        EntryPointAnalysis {
            import_line: 0,
            has_os_import: true,
            call_line: None,
            call_indent: String::new(),
            call_context: None,
            otel_already_configured: false,
            has_setup_import: false,
            has_set_simulation_import: false,
            has_instrument_import: false,
            has_setup_call: false,
            has_instrument_call: false,
            has_pipecat_body_extraction: false,
            has_pipecat_dialin_extraction: false,
            session_start_line: None,
            session_start_indent: String::new(),
            dialin_handler_line: None,
            dialin_handler_indent: String::new(),
            body_extraction_line: None,
            body_extraction_indent: String::new(),
            pipecat_task_line: None,
            pipecat_task_indent: String::new(),
        }
    }

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

    #[test]
    fn pick_entry_point_errors_for_multiple_candidates_in_non_interactive_mode() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("bot.py"), "").unwrap();
        fs::write(dir.path().join("agent.py"), "").unwrap();

        let err = pick_entry_point(dir.path(), Framework::Pipecat, true).unwrap_err();
        assert!(err.to_string().contains("--entry-point"));
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
        assert!(joined.contains("setup_coval_tracing"));
        assert!(joined.contains("COVAL_SIMULATION_ID"));
        assert!(!joined.contains("ctx.job.metadata"));
    }

    #[test]
    fn generate_call_snippet_livekit_includes_ctx() {
        let lines = generate_call_snippet(Framework::Livekit, "    ");
        let joined = lines.join("\n");
        assert!(joined.contains("ctx.room"));
        assert!(joined.contains("participant_connected"));
        assert!(joined.contains("participant_attributes_changed"));
        assert!(joined.contains("setup_coval_tracing"));
    }

    #[test]
    fn generate_call_snippet_pipecat() {
        let lines = generate_call_snippet(Framework::Pipecat, "  ");
        let joined = lines.join("\n");
        assert!(joined.contains("setup_coval_tracing"));
        assert!(joined.contains("on_dialin_connected"));
    }

    #[test]
    fn generate_call_snippet_ends_with_blank_line() {
        let lines = generate_call_snippet(Framework::Generic, "");
        assert_eq!(lines.last().unwrap(), "");
    }

    // ── generate_coval_tracing_py ────────────────────────────────────────

    #[test]
    fn generate_coval_tracing_py_contains_expected_elements() {
        let content = generate_coval_tracing_py();
        assert!(content.contains("def setup_coval_tracing"));
        assert!(content.contains("TracerProvider"));
        assert!(content.contains("OTLPSpanExporter"));
        assert!(content.contains("COVAL_API_KEY"));
        assert!(content.contains("MAX_PREACTIVATION_BUFFER"));
    }

    #[test]
    fn generate_coval_tracing_py_valid_python_structure() {
        let content = generate_coval_tracing_py();
        assert!(content.contains("import os"));
        assert!(content.contains("import logging"));
        assert!(content.contains("from opentelemetry"));
        assert!(content.contains("instrument_session"));
        assert!(content.contains("_CovalSpanRenamer"));
        assert!(content.contains("_ContextualCovalExporter"));
        assert!(content.contains("ContextVar"));
    }

    #[test]
    fn is_expected_validation_not_found_supports_plain_text_and_json() {
        assert!(is_expected_validation_not_found(
            "Simulation output not found"
        ));
        assert!(is_expected_validation_not_found(
            r#"{"message":"Simulation output not found"}"#
        ));
        assert!(is_expected_validation_not_found(
            r#"{"error":{"message":"Simulation output not found"}}"#
        ));
        assert!(!is_expected_validation_not_found("not found"));
    }

    #[test]
    fn backup_existing_file_uses_incrementing_suffix() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("coval_tracing.py");
        fs::write(&file, "first").unwrap();
        fs::write(dir.path().join("coval_tracing.py.bak"), "existing").unwrap();

        let backup = backup_existing_file(&file).unwrap().unwrap();
        assert_eq!(backup, dir.path().join("coval_tracing.py.bak.1"));
        assert_eq!(fs::read_to_string(backup).unwrap(), "first");
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
            call_line: Some(3),
            call_context: Some("def main()".to_string()),
            ..analysis_with_defaults()
        };
        let plan = plan_entry_point_modifications(&analysis, Framework::Generic).unwrap();

        apply_entry_point_modifications(&entry, &analysis, &plan).unwrap();

        let result = fs::read_to_string(&entry).unwrap();
        assert!(result.contains("from coval_tracing import setup_coval_tracing"));
        assert!(result.contains("setup_coval_tracing"));
        assert_eq!(result.matches("import os").count(), 1);

        let bak = PathBuf::from(format!("{}.bak", entry.display()));
        assert!(bak.exists());
    }

    #[test]
    fn plan_entry_point_modifications_is_idempotent_when_setup_exists() {
        let analysis = EntryPointAnalysis {
            has_setup_import: true,
            has_set_simulation_import: true,
            has_setup_call: true,
            ..analysis_with_defaults()
        };

        let plan = plan_entry_point_modifications(&analysis, Framework::Generic).unwrap();
        assert!(plan.is_empty());
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
            ..analysis_with_defaults()
        };
        let plan = plan_entry_point_modifications(&analysis, Framework::Pipecat).unwrap();

        apply_entry_point_modifications(&entry, &analysis, &plan).unwrap();

        let result = fs::read_to_string(&entry).unwrap();
        assert!(result.contains("import os"));
        assert!(result.contains("from coval_tracing import setup_coval_tracing"));
    }

    #[test]
    fn apply_entry_point_modifications_preserves_trailing_newline() {
        let dir = TempDir::new().unwrap();
        let entry = dir.path().join("app.py");
        fs::write(&entry, "import os\n\nx = 1\n").unwrap();

        let analysis = EntryPointAnalysis {
            import_line: 0,
            call_line: Some(2),
            ..analysis_with_defaults()
        };
        let plan = plan_entry_point_modifications(&analysis, Framework::Generic).unwrap();

        apply_entry_point_modifications(&entry, &analysis, &plan).unwrap();

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
            ..analysis_with_defaults()
        };
        let plan = plan_entry_point_modifications(&analysis, Framework::Generic).unwrap();

        apply_entry_point_modifications(&entry, &analysis, &plan).unwrap();

        let result = fs::read_to_string(&entry).unwrap();
        assert!(result.contains("from coval_tracing import setup_coval_tracing"));
        assert!(result.contains("setup_coval_tracing()"));
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
            call_line: Some(4),
            call_indent: "    ".to_string(),
            call_context: Some("inside entrypoint()".to_string()),
            ..analysis_with_defaults()
        };
        let plan = plan_entry_point_modifications(&analysis, Framework::Livekit).unwrap();

        apply_entry_point_modifications(&entry, &analysis, &plan).unwrap();

        let result = fs::read_to_string(&entry).unwrap();
        assert!(result.contains("from coval_tracing import setup_coval_tracing"));
        assert!(result.contains("    setup_coval_tracing"));
        assert!(result.contains("ctx"));
    }

    #[test]
    fn find_pipecat_task_enable_tracing_skips_when_already_present() {
        let lines = [
            "task = PipelineTask(",
            "    pipeline,",
            "    enable_tracing=True,",
            ")",
        ];
        let (line, indent) = find_pipecat_task_enable_tracing(&lines);
        assert!(line.is_none());
        assert!(indent.is_empty());
    }

    #[test]
    fn apply_entry_point_modifications_pipecat_adds_task_tracing_and_header_fallbacks() {
        let dir = TempDir::new().unwrap();
        let entry = dir.path().join("bot.py");
        fs::write(
            &entry,
            r#"import asyncio
from loguru import logger
from pipecat.pipeline.task import PipelineParams, PipelineTask
from pipecat.transports.daily.transport import DailyDialinSettings

async def bot(args):
    body = getattr(args, "body", None) or {}
    dialin_settings = None
    if isinstance(body, dict):
        raw = body.get("dialin_settings")
        if raw:
            dialin_settings = DailyDialinSettings(
                call_id=raw.get("callId") or raw.get("call_id", ""),
                call_domain=raw.get("callDomain") or raw.get("call_domain", ""),
            )
            logger.info("dialin ready")

    @transport.event_handler("on_dialin_connected")
    async def on_dialin_connected(transport, data):
        logger.info(f"Dialin CONNECTED — data: {data}")

    task = PipelineTask(
        pipeline,
        params=PipelineParams(
            allow_interruptions=True,
            enable_metrics=True,
        ),
    )
"#,
        )
        .unwrap();

        let analysis = analyze_entry_point(&entry, Framework::Pipecat).unwrap();
        let plan = plan_entry_point_modifications(&analysis, Framework::Pipecat).unwrap();
        apply_entry_point_modifications(&entry, &analysis, &plan).unwrap();

        let result = fs::read_to_string(&entry).unwrap();
        assert!(result.contains("setup_coval_tracing()"));
        assert!(result.contains("body.get(\"dialinSettings\")"));
        assert!(result.contains("os.environ.get(\"COVAL_SIMULATION_ID\", \"\")"));
        assert!(result.contains("data.get(\"sipHeaders\") or data.get(\"sip_headers\")"));
        assert!(result.contains("enable_tracing=True,"));
    }
}
