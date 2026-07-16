//! Codegen xtask: regenerate the do-not-edit API model modules
//! (`src/client/generated/<resource>.rs`) from the external OpenAPI specs, so the
//! CLI's types stay in lockstep with the public `/v1/openapi` surface.
//!
//! Pipeline (mirrors the design in API-400):
//!   1. Load the `x-visibility: external` specs (from a local dir or the live API).
//!   2. Strip client-irrelevant keywords (validation constraints, `default`, and
//!      component-level `nullable`) so progenitor emits plain, ergonomic types
//!      instead of validated newtypes / non-optional-with-default fields.
//!   3. Generate types with the `progenitor` library, keep only its `types`
//!      module, and format with stable `rustfmt` (no nightly toolchain needed).
//!
//! Usage:
//!   cargo xtask --from-dir <backend>/docs/api/openapi          # regenerate from local specs
//!   cargo xtask --resource runs --from-dir <dir>               # just one resource
//!   cargo xtask --check --from-dir <dir>                       # CI: fail if committed output is stale

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use progenitor::{GenerationSettings, Generator, InterfaceStyle};
use quote::quote;
use serde_yaml::Value;

/// Keywords stripped from every schema node. They make progenitor emit
/// constrained newtypes (`RunRunId`) or non-optional fields with serde defaults;
/// the CLI doesn't validate client-side, so plain `String`/`Option<T>` is what we
/// want. `format: date-time` is kept (maps to `chrono::DateTime`).
const STRIP_KEYWORDS: &[&str] = &[
    "minLength",
    "maxLength",
    "pattern",
    "minItems",
    "maxItems",
    "uniqueItems",
    "minimum",
    "maximum",
    "exclusiveMinimum",
    "exclusiveMaximum",
    "multipleOf",
    "example",
    "examples",
    "default",
];

#[derive(Parser)]
#[command(about = "Regenerate generated API model modules from the external OpenAPI specs")]
struct Args {
    /// Read specs from this directory (files named `<resource>-v1.yaml`). Takes precedence over --from-api.
    #[arg(long)]
    from_dir: Option<PathBuf>,
    /// Fetch specs from this API base (`GET {base}/v1/openapi`).
    #[arg(long, default_value = "https://api.coval.dev")]
    from_api: String,
    /// Limit to these resources (spec basenames, e.g. `runs`). Default: all external specs.
    #[arg(long = "resource")]
    resources: Vec<String>,
    /// Output directory for the generated modules.
    #[arg(long, default_value = "src/client/generated")]
    out_dir: PathBuf,
    /// Don't write; exit non-zero if committed output differs (CI lockstep check).
    #[arg(long)]
    check: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let specs = load_specs(&args)?;
    if specs.is_empty() {
        bail!("no external specs matched (dir/api empty, or --resource filtered everything out)");
    }

    let mut stale = Vec::new();
    for (name, text) in specs {
        let module = module_name(&name);
        let generated = generate_module(&name, &text)
            .with_context(|| format!("generating module for {name}"))?;
        let path = args.out_dir.join(format!("{module}.rs"));
        if args.check {
            let existing = std::fs::read_to_string(&path).unwrap_or_default();
            if existing != generated {
                stale.push(module);
            }
        } else {
            std::fs::write(&path, &generated)
                .with_context(|| format!("writing {}", path.display()))?;
            println!("wrote {}", path.display());
        }
    }

    if args.check && !stale.is_empty() {
        bail!(
            "generated modules are out of date with the specs: {}. Run `cargo xtask`.",
            stale.join(", ")
        );
    }
    Ok(())
}

/// (spec basename, raw YAML text) for each external spec selected by the args.
fn load_specs(args: &Args) -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    if let Some(dir) = &args.from_dir {
        for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
            let path = entry?.path();
            let Some(base) = path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_suffix("-v1.yaml"))
            else {
                continue;
            };
            if !args.resources.is_empty() && !args.resources.iter().any(|r| r == base) {
                continue;
            }
            let text = std::fs::read_to_string(&path)?;
            if is_external(&text) {
                out.push((base.to_string(), text));
            }
        }
    } else {
        for name in fetch_spec_names(&args.from_api)? {
            if !args.resources.is_empty() && !args.resources.iter().any(|r| r == &name) {
                continue;
            }
            let url = format!("{}/v1/openapi/{name}", args.from_api.trim_end_matches('/'));
            let text = ureq::get(&url)
                .call()
                .with_context(|| format!("fetching {url}"))?
                .into_string()?;
            // The public endpoint only serves external specs, so no re-filter needed.
            out.push((name, text));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

/// List spec names from `GET {base}/v1/openapi`. Lenient about the envelope shape.
fn fetch_spec_names(base: &str) -> Result<Vec<String>> {
    let url = format!("{}/v1/openapi", base.trim_end_matches('/'));
    let body: serde_json::Value = ureq::get(&url)
        .call()
        .with_context(|| format!("fetching {url}"))?
        .into_json()?;
    let arr = body
        .get("specs")
        .or_else(|| body.get("openapi"))
        .unwrap_or(&body);
    let names = arr
        .as_array()
        .ok_or_else(|| anyhow!("unexpected /v1/openapi response shape"))?
        .iter()
        .filter_map(|item| {
            item.as_str().map(str::to_string).or_else(|| {
                item.get("name")
                    .and_then(|n| n.as_str())
                    .map(str::to_string)
            })
        })
        // Endpoint serves e.g. "runs.yaml"; normalize to the basename.
        .map(|n| {
            n.trim_end_matches(".yaml")
                .trim_end_matches("-v1")
                .to_string()
        })
        .collect();
    Ok(names)
}

fn is_external(spec_text: &str) -> bool {
    serde_yaml::from_str::<Value>(spec_text)
        .ok()
        .and_then(|v| {
            v.get("x-visibility")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .as_deref()
        == Some("external")
}

/// `runs-v1.yaml` basename `runs` -> module `runs`; `api-keys` -> `api_keys`.
fn module_name(base: &str) -> String {
    base.replace('-', "_")
}

fn generate_module(name: &str, spec_text: &str) -> Result<String> {
    let mut doc: Value = serde_yaml::from_str(spec_text)?;
    strip(&mut doc);
    strip_component_nullable(&mut doc);

    let api: openapiv3::OpenAPI =
        serde_yaml::from_value(doc).context("spec is not a valid OpenAPI 3.0 document")?;

    let mut generator =
        Generator::new(GenerationSettings::default().with_interface(InterfaceStyle::Builder));
    let tokens = generator
        .generate_tokens(&api)
        .map_err(|e| anyhow!("progenitor codegen failed: {e}"))?;

    // Keep only progenitor's `types` module; discard its client/builder/prelude.
    let file: syn::File = syn::parse2(tokens).context("parsing generated tokens")?;
    let items = file
        .items
        .into_iter()
        .find_map(|item| match item {
            syn::Item::Mod(m) if m.ident == "types" => m.content.map(|(_, items)| items),
            _ => None,
        })
        .ok_or_else(|| anyhow!("generated output had no `types` module"))?;
    let body: proc_macro2::TokenStream = items.iter().map(|item| quote!(#item)).collect();

    let header = format!(
        "// @generated from the external OpenAPI spec {name}-v1.yaml — DO NOT EDIT BY HAND.\n\
         // Regenerate with `cargo xtask` (see API-400). Presentation/behavior impls\n\
         // (Tabular, etc.) live in src/client/models/, not here.\n\
         #![allow(clippy::all, clippy::pedantic, dead_code, unused_imports, non_snake_case)]\n\n"
    );
    rustfmt(&format!("{header}{body}"))
}

/// Recursively drop the client-irrelevant validation keywords from every node.
fn strip(value: &mut Value) {
    match value {
        Value::Mapping(map) => {
            for kw in STRIP_KEYWORDS {
                map.remove(Value::from(*kw));
            }
            let keep_format =
                matches!(map.get("format"), Some(Value::String(f)) if f == "date-time");
            if !keep_format {
                map.remove(Value::from("format"));
            }
            for (_key, val) in map.iter_mut() {
                strip(val);
            }
        }
        Value::Sequence(seq) => {
            for item in seq.iter_mut() {
                strip(item);
            }
        }
        _ => {}
    }
}

/// A *named* component schema being `nullable` is redundant (every referencing
/// field is already optional) and makes progenitor wrap it in `Newtype(Option<_>)`.
fn strip_component_nullable(doc: &mut Value) {
    let Some(schemas) = doc
        .get_mut("components")
        .and_then(|c| c.get_mut("schemas"))
        .and_then(Value::as_mapping_mut)
    else {
        return;
    };
    for (_name, schema) in schemas.iter_mut() {
        if let Some(map) = schema.as_mapping_mut() {
            map.remove(Value::from("nullable"));
        }
    }
}

/// Format source with stable `rustfmt` so output matches CI's `cargo fmt --check`.
fn rustfmt(src: &str) -> Result<String> {
    let mut child = Command::new("rustfmt")
        .args(["--edition", "2021"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawning rustfmt")?;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(src.as_bytes())
        .context("writing to rustfmt")?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        bail!(
            "rustfmt failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?)
}
