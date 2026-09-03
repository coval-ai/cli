use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::{json, Value};
use wiremock::matchers::{body_partial_json, header, method, path, query_param};
use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

fn coval() -> Command {
    #[allow(deprecated)]
    let mut command = Command::cargo_bin("coval").unwrap();
    command.env("COVAL_NO_UPDATE_CHECK", "1");
    command
}

fn coval_with_api(mock_server: &MockServer) -> Command {
    let mut command = coval();
    command
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri());
    command
}

fn stdout_json(assert: assert_cmd::assert::Assert) -> Value {
    serde_json::from_slice(&assert.get_output().stdout).unwrap()
}

struct BodyExcludes(&'static str);

impl Match for BodyExcludes {
    fn matches(&self, request: &Request) -> bool {
        std::str::from_utf8(&request.body)
            .map(|body| !body.contains(self.0))
            .unwrap_or(false)
    }
}

struct QueryParamAbsent(&'static str);

impl Match for QueryParamAbsent {
    fn matches(&self, request: &Request) -> bool {
        !request
            .url
            .query_pairs()
            .any(|(key, _)| key.as_ref() == self.0)
    }
}

#[derive(Clone, Default)]
struct BodyCapture {
    body: std::sync::Arc<std::sync::Mutex<Option<Value>>>,
}

impl BodyCapture {
    fn take(&self) -> Value {
        self.body.lock().unwrap().take().unwrap()
    }
}

impl Match for BodyCapture {
    fn matches(&self, request: &Request) -> bool {
        if let Ok(parsed) = serde_json::from_slice::<Value>(&request.body) {
            *self.body.lock().unwrap() = Some(parsed);
        }
        true
    }
}

#[derive(Clone, Default)]
struct RequestCounter {
    count: std::sync::Arc<std::sync::atomic::AtomicU32>,
}

impl RequestCounter {
    fn count(&self) -> u32 {
        self.count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Match for RequestCounter {
    fn matches(&self, _request: &Request) -> bool {
        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        true
    }
}

fn write_skill(root: &std::path::Path, id: &str, description: &str) {
    let skill_dir = root.join("skills").join(id);
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        format!(
            "---\nname: {}\ndescription: {}\n---\n",
            id.replace('/', "-"),
            description
        ),
    )
    .unwrap();
}

const AGENT_RESOURCES: &[&str] = &[
    "agents",
    "uploaded-conversations",
    "runs",
    "simulated-conversations",
    "test-sets",
    "test-cases",
    "personas",
    "metrics",
    "mutations",
    "api-keys",
    "run-templates",
    "scheduled-runs",
    "dashboards",
    "review-annotations",
    "review-projects",
    "reports",
    "monitors",
    "tags",
    "traces",
];

const INPUT_JSON_HELP_COMMANDS: &[&[&str]] = &[
    &["agents", "create", "--help"],
    &["agents", "update", "--help"],
    &["uploaded-conversations", "submit", "--help"],
    &["runs", "launch", "--help"],
    &["runs", "update", "--help"],
    &["test-sets", "create", "--help"],
    &["test-sets", "update", "--help"],
    &["test-cases", "create", "--help"],
    &["test-cases", "update", "--help"],
    &["personas", "create", "--help"],
    &["personas", "update", "--help"],
    &["personas", "background-sounds", "update", "--help"],
    &["metrics", "create", "--help"],
    &["metrics", "update", "--help"],
    &["metrics", "baselines", "metric123", "create", "--help"],
    &[
        "metrics",
        "baselines",
        "metric123",
        "update",
        "baseline123",
        "--help",
    ],
    &["metrics", "thresholds", "metric123", "create", "--help"],
    &["metrics", "thresholds", "metric123", "update", "--help"],
    &["mutations", "create", "--help"],
    &["mutations", "update", "--help"],
    &["api-keys", "create", "--help"],
    &["api-keys", "update", "--help"],
    &["run-templates", "create", "--help"],
    &["run-templates", "update", "--help"],
    &["scheduled-runs", "create", "--help"],
    &["scheduled-runs", "update", "--help"],
    &["dashboards", "create", "--help"],
    &["dashboards", "update", "--help"],
    &["dashboards", "widgets", "create", "--help"],
    &["dashboards", "widgets", "update", "--help"],
    &["review-annotations", "create", "--help"],
    &["review-annotations", "update", "--help"],
    &["review-projects", "create", "--help"],
    &["review-projects", "update", "--help"],
    &["reports", "create", "--help"],
    &["reports", "update", "--help"],
    &["monitors", "create", "--help"],
    &["monitors", "update", "--help"],
    &["tags", "create", "--help"],
    &["tags", "update", "--help"],
    &["traces", "search", "--help"],
];

#[test]
fn test_help() {
    coval()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Coval AI evaluation CLI"));
}

#[test]
fn test_help_prefers_canonical_conversation_commands() {
    coval()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("uploaded-conversations"))
        .stdout(predicate::str::contains("simulated-conversations"))
        .stdout(predicate::str::contains("\n  conversations ").not())
        .stdout(predicate::str::contains("\n  simulations ").not());

    coval()
        .args(["conversations", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Legacy command for uploaded conversations",
        ));
    coval()
        .args(["simulations", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Legacy command for simulated conversations",
        ));
}

#[test]
fn test_version() {
    coval()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("coval"));
}

#[test]
fn test_agent_manifest_no_auth() {
    let temp_dir = tempfile::tempdir().unwrap();
    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("agent")
            .arg("manifest")
            .env_remove("COVAL_API_KEY")
            .env("HOME", temp_dir.path())
            .env("XDG_CONFIG_HOME", temp_dir.path())
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], true);
    assert_eq!(value["resource"], "agent");
    assert_eq!(value["operation"], "manifest");
    assert_eq!(value["data"]["name"], "coval");
    assert!(value["data"]["description"]
        .as_str()
        .unwrap()
        .contains("simulate interactions between your agent and personas"));
    assert_eq!(
        value["data"]["agent_mode"]["argv_prefix"],
        json!(["coval", "--agent"])
    );
    assert!(value["data"]["agent_mode"]["argv"].is_null());
    assert_eq!(value["data"]["help_argv"], json!(["coval", "--help"]));
    assert_eq!(value["data"]["profiles"]["discovery"], true);
    assert_eq!(value["data"]["profiles"]["structured_input"], true);
    assert_eq!(value["data"]["profiles"]["skills"], true);
    assert_eq!(
        value["data"]["resources"].as_array().unwrap().len(),
        AGENT_RESOURCES.len()
    );
    assert!(value["data"]["resources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|resource| resource["name"] == "runs"
            && resource["help_argv"] == json!(["coval", "runs", "--help"])
            && resource["commands"]
                .as_array()
                .unwrap()
                .iter()
                .any(|command| command == "context")));
}

#[test]
fn test_agent_manifest_skills_require_explicit_local_source() {
    let temp_dir = tempfile::tempdir().unwrap();
    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("agent")
            .arg("manifest")
            .env_remove("COVAL_API_KEY")
            .env("HOME", temp_dir.path())
            .env("XDG_CONFIG_HOME", temp_dir.path())
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    let skills = &value["data"]["skills"];
    assert_eq!(skills["implemented"], true);
    assert!(skills["source"].is_null());
    assert_eq!(
        skills["list_argv"],
        json!(["coval", "--agent", "agent", "skills", "list", "--source", "<path>"])
    );
    assert_eq!(
        skills["install_argv"],
        json!([
            "coval",
            "--agent",
            "agent",
            "skills",
            "install",
            "<skill-id>",
            "--source",
            "<path>",
            "--dest",
            "<path>"
        ])
    );

    let manifest = value["data"].to_string();
    assert!(!manifest.contains("github.com"));
    assert!(!manifest.contains("coval-external-skills"));
}

#[test]
fn test_agent_parse_errors_use_structured_envelope() {
    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("agents")
            .assert()
            .failure()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["aci"], "0.1");
    assert_eq!(value["ok"], false);
    assert_eq!(value["resource"], "cli");
    assert_eq!(value["operation"], "parse");
    assert_eq!(value["error"]["code"], "usage_error");
    assert!(value["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Usage: coval agents"));
}

#[test]
fn test_agent_skills_list_requires_explicit_source() {
    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("agent")
            .arg("skills")
            .arg("list")
            .env_remove("COVAL_SKILLS_SOURCE")
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], true);
    assert_eq!(value["operation"], "skills.list");
    assert_eq!(value["data"]["skills"].as_array().unwrap().len(), 0);
    assert_eq!(value["warnings"][0]["code"], "skills_source_required");
}

#[test]
fn test_agent_skills_list_local_source() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("source");
    write_skill(&source, "runs/launch-run", "Launch a Coval run.");

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("agent")
            .arg("skills")
            .arg("list")
            .arg("--source")
            .arg(&source)
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["skills"][0]["id"], "runs/launch-run");
    assert_eq!(
        value["data"]["skills"][0]["description"],
        "Launch a Coval run."
    );
    assert_eq!(value["warnings"].as_array().unwrap().len(), 0);
    assert_eq!(value["next_actions"][0]["id"], "agent.skills.install");
    assert_eq!(value["next_actions"][0]["safe"], false);
    assert_eq!(value["next_actions"][0]["requires_confirmation"], true);
}

#[test]
fn test_agent_skills_list_reports_installed_state() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("source");
    let dest = temp_dir.path().join("dest");
    write_skill(&source, "agents/create-agent", "Create a Coval agent.");
    write_skill(&source, "runs/launch-run", "Launch a Coval run.");
    let installed_skill = dest.join("runs").join("launch-run");
    std::fs::create_dir_all(&installed_skill).unwrap();
    std::fs::write(
        installed_skill.join("SKILL.md"),
        "---\nname: launch-run\ndescription: Launch a Coval run.\n---\n",
    )
    .unwrap();

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("agent")
            .arg("skills")
            .arg("list")
            .arg("--source")
            .arg(&source)
            .arg("--dest")
            .arg(&dest)
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    let skills = value["data"]["skills"].as_array().unwrap();
    let create_agent = skills
        .iter()
        .find(|skill| skill["id"] == "agents/create-agent")
        .unwrap();
    let launch_run = skills
        .iter()
        .find(|skill| skill["id"] == "runs/launch-run")
        .unwrap();

    assert_eq!(create_agent["installed"], false);
    assert_eq!(launch_run["installed"], true);
    assert_eq!(value["next_actions"].as_array().unwrap().len(), 0);
}

#[test]
fn test_agent_skills_install_local_source() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("source");
    let dest = temp_dir.path().join("dest");
    write_skill(&source, "runs/launch-run", "Launch a Coval run.");

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("agent")
            .arg("skills")
            .arg("install")
            .arg("runs/launch-run")
            .arg("--source")
            .arg(&source)
            .arg("--dest")
            .arg(&dest)
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], true);
    assert_eq!(value["operation"], "skills.install");
    assert_eq!(value["data"]["id"], "runs/launch-run");
    assert!(dest
        .join("runs")
        .join("launch-run")
        .join("SKILL.md")
        .exists());
}

#[test]
fn test_agent_skills_reject_remote_source() {
    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("agent")
            .arg("skills")
            .arg("list")
            .arg("--source")
            .arg("https://github.com/coval-ai/coval-external-skills")
            .assert()
            .failure()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], false);
    assert_eq!(value["operation"], "skills.list");
    assert_eq!(value["error"]["code"], "cli_error");
}

#[test]
fn test_agent_skills_install_requires_force_for_existing_skill() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("source");
    let dest = temp_dir.path().join("dest");
    write_skill(&source, "runs/launch-run", "Launch a Coval run.");
    let installed_skill = dest.join("runs").join("launch-run");
    std::fs::create_dir_all(&installed_skill).unwrap();
    std::fs::write(
        installed_skill.join("SKILL.md"),
        "---\nname: launch-run\ndescription: Old instructions.\n---\n",
    )
    .unwrap();

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("agent")
            .arg("skills")
            .arg("install")
            .arg("runs/launch-run")
            .arg("--source")
            .arg(&source)
            .arg("--dest")
            .arg(&dest)
            .assert()
            .failure()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], false);
    assert_eq!(value["operation"], "skills.install");
    assert_eq!(value["error"]["code"], "cli_error");
    assert!(value["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Skill already exists"));
}

#[test]
fn test_agent_skills_install_rejects_path_traversal_id() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("source");
    let dest = temp_dir.path().join("dest");
    let source_escape = source.join("outside");
    let target_escape = temp_dir.path().join("outside");
    std::fs::create_dir_all(&source_escape).unwrap();
    std::fs::create_dir_all(&target_escape).unwrap();
    std::fs::write(
        source_escape.join("SKILL.md"),
        "---\nname: outside\ndescription: Outside source.\n---\n",
    )
    .unwrap();
    std::fs::write(
        target_escape.join("SKILL.md"),
        "---\nname: outside\ndescription: Outside target.\n---\n",
    )
    .unwrap();

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("agent")
            .arg("skills")
            .arg("install")
            .arg("../outside")
            .arg("--source")
            .arg(&source)
            .arg("--dest")
            .arg(&dest)
            .arg("--force")
            .assert()
            .failure()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], false);
    assert_eq!(value["operation"], "skills.install");
    assert_eq!(value["error"]["code"], "cli_error");
    assert!(value["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Invalid skill id"));
    assert!(target_escape.join("SKILL.md").exists());
}

#[cfg(unix)]
#[test]
fn test_agent_skills_install_rejects_symlink_source() {
    use std::os::unix::fs::symlink;

    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("source");
    let dest = temp_dir.path().join("dest");
    write_skill(&source, "runs/launch-run", "Launch a Coval run.");
    let outside = temp_dir.path().join("outside.txt");
    std::fs::write(&outside, "secret").unwrap();
    symlink(
        &outside,
        source
            .join("skills")
            .join("runs")
            .join("launch-run")
            .join("outside.txt"),
    )
    .unwrap();

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("agent")
            .arg("skills")
            .arg("install")
            .arg("runs/launch-run")
            .arg("--source")
            .arg(&source)
            .arg("--dest")
            .arg(&dest)
            .assert()
            .failure()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], false);
    assert_eq!(value["operation"], "skills.install");
    assert_eq!(value["error"]["code"], "cli_error");
    assert!(value["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unsupported symlink"));
    assert!(!dest
        .join("runs")
        .join("launch-run")
        .join("outside.txt")
        .exists());
}

#[cfg(unix)]
#[test]
fn test_agent_skills_install_rejects_symlink_skills_root() {
    use std::os::unix::fs::symlink;

    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("source");
    let external = temp_dir.path().join("external");
    let dest = temp_dir.path().join("dest");
    std::fs::create_dir_all(&source).unwrap();
    write_skill(&external, "runs/launch-run", "Launch a Coval run.");
    symlink(external.join("skills"), source.join("skills")).unwrap();

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("agent")
            .arg("skills")
            .arg("install")
            .arg("runs/launch-run")
            .arg("--source")
            .arg(&source)
            .arg("--dest")
            .arg(&dest)
            .assert()
            .failure()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], false);
    assert_eq!(value["operation"], "skills.install");
    assert_eq!(value["error"]["code"], "cli_error");
    assert!(value["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unsupported symlink"));
    assert!(!dest
        .join("runs")
        .join("launch-run")
        .join("SKILL.md")
        .exists());
}

#[cfg(unix)]
#[test]
fn test_agent_skills_install_rejects_symlink_namespace() {
    use std::os::unix::fs::symlink;

    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("source");
    let external = temp_dir.path().join("external");
    let dest = temp_dir.path().join("dest");
    std::fs::create_dir_all(source.join("skills")).unwrap();
    write_skill(&external, "runs/launch-run", "Launch a Coval run.");
    symlink(
        external.join("skills").join("runs"),
        source.join("skills").join("runs"),
    )
    .unwrap();

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("agent")
            .arg("skills")
            .arg("install")
            .arg("runs/launch-run")
            .arg("--source")
            .arg(&source)
            .arg("--dest")
            .arg(&dest)
            .assert()
            .failure()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], false);
    assert_eq!(value["operation"], "skills.install");
    assert_eq!(value["error"]["code"], "cli_error");
    assert!(value["error"]["message"]
        .as_str()
        .unwrap()
        .contains("escapes skills root"));
    assert!(!dest
        .join("runs")
        .join("launch-run")
        .join("SKILL.md")
        .exists());
}

#[test]
fn test_input_json_help_coverage() {
    for args in INPUT_JSON_HELP_COMMANDS {
        coval()
            .args(*args)
            .assert()
            .success()
            .stdout(predicate::str::contains("--input-json"));
    }
}

#[test]
fn test_resource_contexts_agent_mode_no_auth() {
    for resource in AGENT_RESOURCES
        .iter()
        .copied()
        .chain(["conversations", "simulations"])
    {
        let canonical_resource = match resource {
            "conversations" => "uploaded-conversations",
            "simulations" => "simulated-conversations",
            _ => resource,
        };
        let temp_dir = tempfile::tempdir().unwrap();
        let value = stdout_json(
            coval()
                .arg("--agent")
                .arg(resource)
                .arg("context")
                .env_remove("COVAL_API_KEY")
                .env("HOME", temp_dir.path())
                .env("XDG_CONFIG_HOME", temp_dir.path())
                .assert()
                .success()
                .stderr(predicate::str::is_empty()),
        );

        assert_eq!(value["ok"], true);
        assert_eq!(value["resource"], resource);
        assert_eq!(value["operation"], "context");
        assert_eq!(value["data"]["name"], canonical_resource);
        assert_eq!(
            value["data"]["help_argv"],
            json!(["coval", canonical_resource, "--help"])
        );
        assert_eq!(value["data"]["commands"][0]["name"], "context");
        assert_eq!(value["data"]["commands"][0]["requires_auth"], false);
        assert_eq!(
            value["data"]["commands"][0]["help_argv"],
            json!(["coval", canonical_resource, "context", "--help"])
        );
        if resource == "runs" {
            assert_eq!(
                value["data"]["workflows"][0]["argv"],
                json!([
                    "coval",
                    "--agent",
                    "runs",
                    "launch",
                    "--agent-id",
                    "<agent_id>",
                    "--persona-id",
                    "<persona_id>",
                    "--test-set-id",
                    "<test_set_id>"
                ])
            );
        }
    }
}

#[test]
fn test_agent_doctor_no_auth() {
    let temp_dir = tempfile::tempdir().unwrap();
    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("agent")
            .arg("doctor")
            .env_remove("COVAL_API_KEY")
            .env("HOME", temp_dir.path())
            .env("XDG_CONFIG_HOME", temp_dir.path())
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], true);
    assert_eq!(value["resource"], "agent");
    assert_eq!(value["operation"], "doctor");
    assert_eq!(value["data"]["auth"]["authenticated"], false);
    assert_eq!(value["data"]["api"]["connectivity"]["checked"], false);
    assert_eq!(value["warnings"][0]["code"], "not_authenticated");
}

#[tokio::test]
async fn test_agent_doctor_with_connectivity() {
    let mock_server = MockServer::start().await;
    let temp_dir = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/v1/agents"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agents": []
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("agent")
            .arg("doctor")
            .env("HOME", temp_dir.path())
            .env("XDG_CONFIG_HOME", temp_dir.path())
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["auth"]["authenticated"], true);
    assert_eq!(value["data"]["auth"]["source"], "argument_or_env");
    assert_eq!(value["data"]["api"]["source"], "argument_or_env");
    assert_eq!(value["data"]["api"]["connectivity"]["checked"], true);
    assert_eq!(value["data"]["api"]["connectivity"]["ok"], true);
    assert_eq!(value["warnings"].as_array().unwrap().len(), 0);
}

#[test]
fn test_missing_api_key() {
    let temp_dir = tempfile::tempdir().unwrap();
    coval()
        .arg("agents")
        .arg("list")
        .env_remove("COVAL_API_KEY")
        .env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not authenticated"));
}

#[test]
fn test_missing_api_key_agent_mode() {
    let temp_dir = tempfile::tempdir().unwrap();
    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("agents")
            .arg("list")
            .env_remove("COVAL_API_KEY")
            .env("HOME", temp_dir.path())
            .env("XDG_CONFIG_HOME", temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["aci"], "0.1");
    assert_eq!(value["ok"], false);
    assert_eq!(value["resource"], "agents");
    assert_eq!(value["operation"], "list");
    assert_eq!(value["error"]["code"], "unauthenticated");
}

#[test]
fn test_config_get_masks_unicode_api_key() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("coval");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(
        config_dir.join("config.toml"),
        r#"api_key = "🔑🔑🔑🔑abcdEFGH""#,
    )
    .unwrap();

    coval()
        .arg("config")
        .arg("get")
        .arg("api_key")
        .env_remove("COVAL_API_KEY")
        .env("HOME", temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("🔑🔑🔑🔑...EFGH"));
}

#[test]
fn test_whoami_masks_unicode_api_key() {
    coval()
        .arg("--api-key")
        .arg("🔑🔑🔑🔑abcdEFGH")
        .arg("whoami")
        .assert()
        .success()
        .stdout(predicate::str::contains("🔑🔑🔑🔑...EFGH"));
}

fn update_check_env<'a>(
    command: &'a mut Command,
    home: &std::path::Path,
    url: &str,
) -> &'a mut Command {
    command
        .env_remove("COVAL_NO_UPDATE_CHECK")
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env("COVAL_UPDATE_CHECK_URL", url)
}

async fn mount_latest_release(mock_server: &MockServer, tag: &str) -> RequestCounter {
    let counter = RequestCounter::default();
    Mock::given(method("GET"))
        .and(path("/release-latest"))
        .and(counter.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "tag_name": tag })))
        .mount(mock_server)
        .await;
    counter
}

#[tokio::test]
async fn test_update_check_notifies_when_outdated() {
    let mock_server = MockServer::start().await;
    let temp_dir = tempfile::tempdir().unwrap();
    mount_latest_release(&mock_server, "v99.9.9").await;

    update_check_env(
        coval().arg("--api-key").arg("test_key").arg("whoami"),
        temp_dir.path(),
        &format!("{}/release-latest", mock_server.uri()),
    )
    .assert()
    .success()
    .stderr(predicate::str::contains(
        "A newer version of coval is available: v99.9.9",
    ));

    assert!(temp_dir.path().join(".config/coval/update-check").exists());
}

#[tokio::test]
async fn test_update_check_silent_when_current() {
    let mock_server = MockServer::start().await;
    let temp_dir = tempfile::tempdir().unwrap();
    mount_latest_release(&mock_server, concat!("v", env!("CARGO_PKG_VERSION"))).await;

    update_check_env(
        coval().arg("--api-key").arg("test_key").arg("whoami"),
        temp_dir.path(),
        &format!("{}/release-latest", mock_server.uri()),
    )
    .assert()
    .success()
    .stderr(predicate::str::is_empty());
}

#[tokio::test]
async fn test_update_check_skips_when_recently_checked() {
    let mock_server = MockServer::start().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let counter = mount_latest_release(&mock_server, "v99.9.9").await;

    let state_dir = temp_dir.path().join(".config").join("coval");
    std::fs::create_dir_all(&state_dir).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    std::fs::write(state_dir.join("update-check"), now.to_string()).unwrap();

    update_check_env(
        coval().arg("--api-key").arg("test_key").arg("whoami"),
        temp_dir.path(),
        &format!("{}/release-latest", mock_server.uri()),
    )
    .assert()
    .success()
    .stderr(predicate::str::is_empty());

    assert_eq!(counter.count(), 0);
}

#[tokio::test]
async fn test_update_check_disabled_by_env_is_silent_and_makes_no_request() {
    let mock_server = MockServer::start().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let counter = RequestCounter::default();
    Mock::given(method("GET"))
        .and(path("/release-latest"))
        .and(counter.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "tag_name": "v99.9.9" })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("whoami")
        .env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path())
        .env(
            "COVAL_UPDATE_CHECK_URL",
            format!("{}/release-latest", mock_server.uri()),
        )
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    assert_eq!(counter.count(), 0);
}

#[tokio::test]
async fn test_update_check_suppressed_in_agent_mode() {
    let mock_server = MockServer::start().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let counter = mount_latest_release(&mock_server, "v99.9.9").await;

    update_check_env(
        coval()
            .arg("--agent")
            .arg("--api-key")
            .arg("test_key")
            .arg("whoami"),
        temp_dir.path(),
        &format!("{}/release-latest", mock_server.uri()),
    )
    .assert()
    .success()
    .stderr(predicate::str::is_empty());

    assert_eq!(counter.count(), 0);
}

#[tokio::test]
async fn test_update_check_silent_when_fetch_fails() {
    let temp_dir = tempfile::tempdir().unwrap();

    update_check_env(
        coval().arg("--api-key").arg("test_key").arg("whoami"),
        temp_dir.path(),
        "http://127.0.0.1:9/release-latest",
    )
    .assert()
    .success()
    .stderr(predicate::str::is_empty());
}

#[test]
fn test_agents_help() {
    coval()
        .arg("agents")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"));
}

#[test]
fn test_agents_create_help_lists_all_agent_types() {
    coval()
        .arg("agents")
        .arg("create")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("chat-a2a"))
        .stdout(predicate::str::contains("chat-websocket"))
        .stdout(predicate::str::contains("livekit"))
        .stdout(predicate::str::contains("pipecat"))
        .stdout(predicate::str::contains("openai-realtime"))
        .stdout(predicate::str::contains("gemini-realtime"))
        .stdout(predicate::str::contains("grok-realtime"));
}

#[tokio::test]
async fn test_agents_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/agents"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agents": [
                {
                    "id": "abc123",
                    "display_name": "Test Agent",
                    "model_type": "MODEL_TYPE_VOICE",
                    "create_time": "2025-01-15T10:30:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("agents")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("abc123"))
        .stdout(predicate::str::contains("Test Agent"));
}

#[tokio::test]
async fn test_agents_list_json() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/agents"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agents": [
                {
                    "id": "abc123",
                    "display_name": "Test Agent",
                    "model_type": "MODEL_TYPE_VOICE",
                    "create_time": "2025-01-15T10:30:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("--format")
        .arg("json")
        .arg("agents")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": \"abc123\""));
}

#[tokio::test]
async fn test_agents_list_agent_mode() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/agents"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agents": [
                {
                    "id": "abc123",
                    "display_name": "Test Agent",
                    "model_type": "MODEL_TYPE_VOICE",
                    "create_time": "2025-01-15T10:30:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("agents")
            .arg("list")
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["aci"], "0.1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["resource"], "agents");
    assert_eq!(value["operation"], "list");
    assert_eq!(value["data"][0]["id"], "abc123");
    assert_eq!(value["warnings"].as_array().unwrap().len(), 0);
    assert_eq!(value["next_actions"][0]["id"], "agents.get");
    assert_eq!(
        value["next_actions"][0]["argv"],
        json!(["coval", "--agent", "agents", "get", "abc123"])
    );
}

#[tokio::test]
async fn test_agent_mode_overrides_json_format() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/agents"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agents": [
                {
                    "id": "abc123",
                    "display_name": "Test Agent",
                    "model_type": "MODEL_TYPE_VOICE",
                    "create_time": "2025-01-15T10:30:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("--format")
            .arg("json")
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("agents")
            .arg("list")
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], true);
    assert_eq!(value["resource"], "agents");
    assert_eq!(value["operation"], "list");
    assert_eq!(value["data"][0]["id"], "abc123");
    assert_eq!(value["next_actions"][0]["id"], "agents.get");
}

#[tokio::test]
async fn test_agents_list_json_is_not_agent_enveloped() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/agents"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agents": [
                {
                    "id": "abc123",
                    "display_name": "Test Agent",
                    "model_type": "MODEL_TYPE_VOICE",
                    "create_time": "2025-01-15T10:30:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let output = coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("--format")
        .arg("json")
        .arg("agents")
        .arg("list")
        .output()
        .unwrap();

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(value.is_array());
}

#[tokio::test]
async fn test_agents_get() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/agents/abc123"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agent": {
                "id": "abc123",
                "display_name": "Test Agent",
                "model_type": "MODEL_TYPE_VOICE",
                "create_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("agents")
        .arg("get")
        .arg("abc123")
        .assert()
        .success()
        .stdout(predicate::str::contains("abc123"));
}

#[tokio::test]
async fn test_agents_delete() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/agents/abc123"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("agents")
        .arg("delete")
        .arg("abc123")
        .assert()
        .success()
        .stdout(predicate::str::contains("deleted"));
}

#[tokio::test]
async fn test_agents_delete_agent_mode_next_actions_are_safe() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/agents/abc123"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("agents")
            .arg("delete")
            .arg("abc123")
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], true);
    assert_eq!(value["next_actions"][0]["id"], "agents.list");
    assert_eq!(value["next_actions"][0]["safe"], true);
    assert_eq!(value["next_actions"][0]["requires_confirmation"], false);
}

#[tokio::test]
async fn test_runs_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/runs"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "runs": [
                {
                    "name": "Test Run",
                    "run_id": "run123",
                    "status": "COMPLETED",
                    "create_time": "2025-01-15T10:30:00Z",
                    "progress": {
                        "total_test_cases": 10,
                        "completed_test_cases": 10,
                        "failed_test_cases": 0,
                        "in_progress_test_cases": 0
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("runs")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("run123"))
        .stdout(predicate::str::contains("COMPLETED"));
}

#[tokio::test]
async fn test_runs_update_tags() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/runs/run123"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "run": {
                "name": "Test Run",
                "run_id": "run123",
                "status": "COMPLETED",
                "create_time": "2025-01-15T10:30:00Z",
                "tags": ["baseline", "prod"]
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("runs")
        .arg("update")
        .arg("run123")
        .arg("--tags")
        .arg("baseline,prod")
        .assert()
        .success()
        .stdout(predicate::str::contains("run123"));
}

#[tokio::test]
async fn test_api_error_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/agents/notfound"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": {
                "code": "NOT_FOUND",
                "message": "Agent not found",
                "details": []
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("agents")
        .arg("get")
        .arg("notfound")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[tokio::test]
async fn test_api_error_handling_agent_mode() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/agents/notfound"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": {
                "code": "NOT_FOUND",
                "message": "Agent not found",
                "details": []
            }
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("agents")
            .arg("get")
            .arg("notfound")
            .assert()
            .failure()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["aci"], "0.1");
    assert_eq!(value["ok"], false);
    assert_eq!(value["resource"], "agents");
    assert_eq!(value["operation"], "get");
    assert_eq!(value["error"]["code"], "not_found");
    assert_eq!(value["error"]["message"], "Agent not found");
}

fn persona_response(multi_language_stt: bool) -> Value {
    json!({
        "persona": {
            "resource_name": "personas/persona1",
            "id": "persona1",
            "name": "Multilingual caller",
            "voice_name": "marina",
            "language_code": "en-US",
            "multi_language_stt": multi_language_stt,
            "create_time": "2025-01-15T10:30:00Z"
        }
    })
}

#[tokio::test]
async fn test_personas_create_preserves_multilingual_stt_input_json() {
    for key in ["multi_language_stt", "multiLanguageStt"] {
        let mock_server = MockServer::start().await;
        let body_capture = BodyCapture::default();

        Mock::given(method("POST"))
            .and(path("/v1/personas"))
            .and(header("X-API-Key", "test_key"))
            .and(body_capture.clone())
            .respond_with(ResponseTemplate::new(200).set_body_json(persona_response(true)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let value = stdout_json(
            coval()
                .arg("--api-key")
                .arg("test_key")
                .arg("--api-url")
                .arg(mock_server.uri())
                .arg("--format")
                .arg("json")
                .arg("personas")
                .arg("create")
                .arg("--input-json")
                .arg(
                    json!({
                        "name": "Multilingual caller",
                        "voice_name": "marina",
                        "language_code": "en-US",
                        key: true
                    })
                    .to_string(),
                )
                .assert()
                .success()
                .stderr(predicate::str::is_empty()),
        );

        assert_eq!(body_capture.take()["multi_language_stt"], true);
        assert_eq!(value["multi_language_stt"], true);
    }
}

#[tokio::test]
async fn test_personas_create_multilingual_stt_flag_sends_true() {
    let mock_server = MockServer::start().await;
    let body_capture = BodyCapture::default();

    Mock::given(method("POST"))
        .and(path("/v1/personas"))
        .and(header("X-API-Key", "test_key"))
        .and(body_capture.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(persona_response(true)))
        .expect(1)
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("personas")
        .arg("create")
        .arg("--input-json")
        .arg(json!({ "multiLanguageStt": false }).to_string())
        .arg("--name")
        .arg("Multilingual caller")
        .arg("--voice")
        .arg("marina")
        .arg("--language")
        .arg("en-US")
        .arg("--multi-language-stt")
        .assert()
        .success();

    assert_eq!(body_capture.take()["multi_language_stt"], true);
}

#[tokio::test]
async fn test_personas_update_multilingual_stt_flag_sends_false() {
    let mock_server = MockServer::start().await;
    let body_capture = BodyCapture::default();

    Mock::given(method("PATCH"))
        .and(path("/v1/personas/persona1"))
        .and(header("X-API-Key", "test_key"))
        .and(body_capture.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(persona_response(false)))
        .expect(1)
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("personas")
        .arg("update")
        .arg("persona1")
        .arg("--multi-language-stt=false")
        .assert()
        .success();

    assert_eq!(body_capture.take()["multi_language_stt"], false);
}

#[tokio::test]
async fn test_personas_background_sounds_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/personas/background-sounds"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "background_sounds": [
                {
                    "id": "off",
                    "value": "off",
                    "source": "built_in",
                    "display_name": "Off",
                    "status": "active",
                    "default_volume": 1.0
                },
                {
                    "id": "sound1",
                    "value": "custom:sound1",
                    "source": "custom",
                    "display_name": "Lobby Noise",
                    "status": "active",
                    "preview_url": "https://preview.example/sound1.mp3",
                    "preview_url_expires_at": "2026-01-01T00:00:00Z",
                    "default_volume": 0.3,
                    "content_type": "audio/mpeg",
                    "original_filename": "lobby-noise.mp3",
                    "metadata": { "size_bytes": 12 },
                    "created_at": "2025-01-15T10:30:00Z",
                    "last_updated_at": "2025-01-15T10:30:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("personas")
        .arg("background-sounds")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("custom:sound1"))
        .stdout(predicate::str::contains("Lobby Noise"));
}

#[tokio::test]
async fn test_personas_background_sounds_upload() {
    let mock_server = MockServer::start().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let audio_path = temp_dir.path().join("lobby-noise.mp3");
    std::fs::write(&audio_path, b"ID3 test audio").unwrap();

    Mock::given(method("POST"))
        .and(path("/v1/personas/background-sounds"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "display_name": "Lobby Noise",
            "original_filename": "lobby-noise.mp3",
            "content_type": "audio/mpeg",
            "default_volume": 0.42,
            "metadata": { "source": "cli-test" }
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "background_sound": {
                "id": "sound1",
                "value": "custom:sound1",
                "source": "custom",
                "display_name": "Lobby Noise",
                "status": "pending_upload",
                "default_volume": 0.42,
                "content_type": "audio/mpeg",
                "original_filename": "lobby-noise.mp3"
            },
            "upload_url": format!("{}/s3-upload", mock_server.uri()),
            "upload_fields": {
                "key": "background-sounds/org/sound1/lobby-noise.mp3",
                "Content-Type": "audio/mpeg",
                "policy": "policy",
                "x-amz-signature": "signature"
            },
            "expires_at": "2026-01-01T00:00:00Z",
            "max_size_bytes": 52428800
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/s3-upload"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/personas/background-sounds/sound1/complete"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "background_sound": {
                "id": "sound1",
                "value": "custom:sound1",
                "source": "custom",
                "display_name": "Lobby Noise",
                "status": "active",
                "default_volume": 0.42,
                "content_type": "audio/mpeg",
                "original_filename": "lobby-noise.mp3",
                "created_at": "2025-01-15T10:30:00Z",
                "last_updated_at": "2025-01-15T10:31:00Z"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("personas")
        .arg("background-sounds")
        .arg("upload")
        .arg(&audio_path)
        .arg("--display-name")
        .arg("Lobby Noise")
        .arg("--default-volume")
        .arg("0.42")
        .arg("--metadata")
        .arg("source=cli-test")
        .assert()
        .success()
        .stdout(predicate::str::contains("custom:sound1"))
        .stdout(predicate::str::contains("active"));
}

#[tokio::test]
async fn test_personas_background_sounds_update_accepts_custom_value() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/personas/background-sounds/sound1"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "display_name": "Archived Lobby Noise",
            "status": "archived"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "background_sound": {
                "id": "sound1",
                "value": "custom:sound1",
                "source": "custom",
                "display_name": "Archived Lobby Noise",
                "status": "archived",
                "default_volume": 0.42,
                "content_type": "audio/mpeg",
                "original_filename": "lobby-noise.mp3"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("personas")
        .arg("background-sounds")
        .arg("update")
        .arg("custom:sound1")
        .arg("--display-name")
        .arg("Archived Lobby Noise")
        .arg("--status")
        .arg("archived")
        .assert()
        .success()
        .stdout(predicate::str::contains("Archived Lobby Noise"))
        .stdout(predicate::str::contains("archived"));
}

#[tokio::test]
async fn test_api_key_create_warning_agent_mode() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/api-keys"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "api_key": {
                "id": "key123",
                "name": "Agent Key",
                "description": "",
                "key_type": "SERVICE",
                "environment": "DEVELOPMENT",
                "status": "ACTIVE",
                "permissions": [],
                "api_key": "coval_secret_123",
                "create_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("api-keys")
            .arg("create")
            .arg("--name")
            .arg("Agent Key")
            .arg("--type")
            .arg("service")
            .arg("--environment")
            .arg("development")
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], true);
    assert_eq!(value["resource"], "api-keys");
    assert_eq!(value["operation"], "create");
    assert_eq!(value["warnings"][0]["code"], "store_api_key");
    assert_eq!(value["data"]["api_key"], "coval_secret_123");
}

#[tokio::test]
async fn test_dashboard_widgets_list_agent_mode_resource() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/dashboards/dash123/widgets"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "widgets": [
                {
                    "name": "dashboards/dash123/widgets/widget123",
                    "display_name": "Overview",
                    "type": "chart",
                    "create_time": "2025-01-15T10:30:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("dashboards")
            .arg("widgets")
            .arg("list")
            .arg("dash123")
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], true);
    assert_eq!(value["resource"], "widgets");
    assert_eq!(value["operation"], "widgets.list");
    assert_eq!(
        value["data"][0]["name"],
        "dashboards/dash123/widgets/widget123"
    );
}

#[test]
fn test_dashboard_widgets_agent_mode_error_resource() {
    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("--api-key")
            .arg("test_key")
            .arg("dashboards")
            .arg("widgets")
            .arg("create")
            .arg("dash123")
            .arg("--name")
            .arg("Overview")
            .arg("--type")
            .arg("chart")
            .arg("--config")
            .arg("{")
            .assert()
            .failure()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], false);
    assert_eq!(value["resource"], "widgets");
    assert_eq!(value["operation"], "widgets.create");
    assert_eq!(value["error"]["code"], "cli_error");
}

#[tokio::test]
async fn test_conversations_audio_json_output() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/conversations/conv123/audio"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "audio_url": "https://storage.example.com/conversation.wav",
            "conversation_id": "conv123",
            "url_expires_in_seconds": 3600
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("--format")
            .arg("json")
            .arg("conversations")
            .arg("audio")
            .arg("conv123")
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(
        value["audio_url"],
        "https://storage.example.com/conversation.wav"
    );
    assert_eq!(value["conversation_id"], "conv123");
}

#[tokio::test]
async fn test_conversations_list_can_include_full_metric_outputs() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/conversations"))
        .and(header("X-API-Key", "test_key"))
        .and(query_param("include", "metric_outputs"))
        .and(query_param("metric_id", "metric123"))
        .and(query_param("page_size", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "conversations": [{
                "name": "conversations/conversation123",
                "conversation_id": "conversation123",
                "status": "COMPLETED",
                "create_time": "2026-04-09T12:00:00Z",
                "occurred_at": "2026-04-09T11:55:00Z",
                "has_audio": true,
                "metadata": {"nlp_provider": "sierra"},
                "metric_outputs": [{
                    "metric_output_id": "01JMETRICOUTPUT00000000000",
                    "metric_id": "metric123",
                    "metric_version_ulid": "01JMETRICVERSION0000000000",
                    "status": "COMPLETED",
                    "value": 0.5,
                    "result": {
                        "raw_values": {
                            "critical_failures": [{
                                "node_id": "dispute_status_flow",
                                "failure": "Wrong status guidance.",
                                "message_index": 9
                            }],
                            "non_critical_failures": []
                        }
                    }
                }]
            }],
            "next_page_token": "page-2"
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("--format")
            .arg("json")
            .arg("conversations")
            .arg("list")
            .arg("--include-metric-outputs")
            .arg("metric123")
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value[0]["conversation_id"], "conversation123");
    assert_eq!(value[0]["metric_outputs"][0]["metric_id"], "metric123");
    assert_eq!(
        value[0]["metric_outputs"][0]["result"]["raw_values"]["critical_failures"][0]["node_id"],
        "dispute_status_flow"
    );
}

#[tokio::test]
async fn test_test_cases_stdin_json_summary() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/test-cases"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "test_set_id": "ts123",
            "input_str": "hello"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "test_case": {
                "name": "testCases/tc123",
                "id": "tc123",
                "test_set_id": "ts123",
                "input_str": "hello",
                "create_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("--format")
            .arg("json")
            .arg("test-cases")
            .arg("create")
            .arg("--test-set-id")
            .arg("ts123")
            .arg("--stdin")
            .write_stdin(r#"{"input_str":"hello"}"#)
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["created"], 1);
    assert_eq!(value["failed"], 0);
}

#[tokio::test]
async fn test_runs_watch_agent_mode() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/runs/run123"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "run": {
                "name": "Test Run",
                "run_id": "run123",
                "status": "COMPLETED",
                "create_time": "2025-01-15T10:30:00Z",
                "progress": {
                    "total_test_cases": 10,
                    "completed_test_cases": 10,
                    "failed_test_cases": 0,
                    "in_progress_test_cases": 0
                },
                "results": {
                    "output_ids": ["sim123"],
                    "metrics": {}
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("runs")
            .arg("watch")
            .arg("run123")
            .arg("--interval")
            .arg("0")
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], true);
    assert_eq!(value["resource"], "runs");
    assert_eq!(value["operation"], "watch");
    assert_eq!(value["data"]["run_id"], "run123");
    assert_eq!(value["data"]["status"], "COMPLETED");
    assert_eq!(value["next_actions"][0]["id"], "runs.get");
    assert_eq!(
        value["next_actions"][1]["argv"],
        json!([
            "coval",
            "--agent",
            "simulated-conversations",
            "list",
            "--run-id",
            "run123"
        ])
    );
}

#[tokio::test]
async fn test_simulations_audio_url() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/simulations/sim123/audio"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "audio_url": "https://storage.example.com/audio.wav",
            "simulation_id": "sim123",
            "url_expires_in_seconds": 3600
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("simulations")
        .arg("audio")
        .arg("sim123")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "https://storage.example.com/audio.wav",
        ));
}

#[tokio::test]
async fn test_simulations_update_notes_does_not_send_is_public() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/simulations/sim123"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "notes": "Updated notes"
        })))
        .and(BodyExcludes("is_public"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "simulation": {
                "name": "Simulation 123",
                "simulation_id": "sim123",
                "run_id": "run123",
                "status": "COMPLETED",
                "create_time": "2026-01-01T00:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("simulations")
        .arg("update")
        .arg("sim123")
        .arg("--notes")
        .arg("Updated notes")
        .assert()
        .success()
        .stdout(predicate::str::contains("sim123"));
}

#[tokio::test]
async fn test_simulations_update_is_public_sends_true() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/simulations/sim123"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "is_public": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "simulation": {
                "name": "Simulation 123",
                "simulation_id": "sim123",
                "run_id": "run123",
                "status": "COMPLETED",
                "create_time": "2026-01-01T00:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("simulations")
        .arg("update")
        .arg("sim123")
        .arg("--is-public")
        .assert()
        .success()
        .stdout(predicate::str::contains("sim123"));
}

#[tokio::test]
async fn test_simulations_metrics_with_subvalues() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/simulations/sim123/metrics"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metrics": [
                {
                    "metric_output_id": "mo123",
                    "metric_id": "met456",
                    "metric_version_ulid": "01JMETRICVERSION000000001",
                    "status": "COMPLETED",
                    "value": 0.95,
                    "explanation": "The agent skipped the identity check.",
                    "subvalues_by_timestamp": [
                        {
                            "start_offset": 0.0,
                            "end_offset": 5.0,
                            "output_type": "float",
                            "float_value": 0.8,
                            "string_value": "",
                            "role": "agent",
                            "message_index": 1
                        },
                        {
                            "start_offset": 5.0,
                            "end_offset": 10.0,
                            "output_type": "float",
                            "float_value": 0.9,
                            "string_value": "",
                            "role": null,
                            "message_index": null
                        }
                    ],
                    "subvalues_by_timestamp_truncated": false,
                    "subvalues_by_timestamp_total_count": 2,
                    "result": {
                        "unit": "score",
                        "raw_values": {
                            "critical_failures": [
                                {
                                    "node_id": "identity_check",
                                    "failure": "The bot skipped the identity check.",
                                    "message_index": 1
                                }
                            ],
                            "non_critical_failures": [
                                {
                                    "node_id": "wrap_up",
                                    "failure": "The bot repeated the closing prompt.",
                                    "message_index": 3
                                }
                            ]
                        }
                    },
                    "runtime_metadata": {"model_version": "openai:gpt-4.1-2025-04-14"}
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("simulations")
        .arg("metrics")
        .arg("sim123")
        .assert()
        .success()
        .stdout(predicate::str::contains("mo123"))
        .stdout(predicate::str::contains("met456"))
        .stdout(predicate::str::contains("COMPLETED"))
        .stdout(predicate::str::contains("2"));

    let value = stdout_json(
        coval()
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("--format")
            .arg("json")
            .arg("simulations")
            .arg("metrics")
            .arg("sim123")
            .assert()
            .success(),
    );
    assert_eq!(
        value[0]["metric_version_ulid"],
        json!("01JMETRICVERSION000000001")
    );
    assert_eq!(
        value[0]["explanation"],
        json!("The agent skipped the identity check.")
    );
    assert_eq!(
        value[0]["result"]["raw_values"]["critical_failures"][0]["node_id"],
        json!("identity_check")
    );
    assert_eq!(
        value[0]["result"]["raw_values"]["non_critical_failures"][0]["node_id"],
        json!("wrap_up")
    );
    assert_eq!(
        value[0]["runtime_metadata"]["model_version"],
        json!("openai:gpt-4.1-2025-04-14")
    );
}

#[tokio::test]
async fn test_simulations_metrics_without_subvalues() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/simulations/sim456/metrics"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metrics": [
                {
                    "metric_output_id": "mo789",
                    "metric_id": "met101",
                    "status": "COMPLETED",
                    "value": 0.75
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("simulations")
        .arg("metrics")
        .arg("sim456")
        .assert()
        .success()
        .stdout(predicate::str::contains("mo789"))
        .stdout(predicate::str::contains("met101"))
        .stdout(predicate::str::contains("-"));
}

#[tokio::test]
async fn test_simulations_metric_detail_by_ulid() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/v1/simulations/sim123/metrics/01ARZ3NDEKTSV4RRFFQ69G5FAV",
        ))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metric": {
                "metric_output_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
                "metric_id": "29BlkepvvX19ebbLDB0y6Q",
                "status": "COMPLETED",
                "value": 2.35,
                "result": {
                    "raw_values": {
                        "critical_failures": [
                            {
                                "node_id": "existing_dispute_check",
                                "failure": "Bot did not clarify whether the dispute was new or existing.",
                                "message_index": 7
                            }
                        ],
                        "non_critical_failures": []
                    }
                },
                "runtime_metadata": {"model_version": "openai:gpt-4.1-2025-04-14"}
            }
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("simulations")
            .arg("metric-detail")
            .arg("sim123")
            .arg("01ARZ3NDEKTSV4RRFFQ69G5FAV")
            .assert()
            .success(),
    );
    assert_eq!(
        value["metric_output_id"],
        json!("01ARZ3NDEKTSV4RRFFQ69G5FAV")
    );
    assert_eq!(value["metric_id"], json!("29BlkepvvX19ebbLDB0y6Q"));
    assert_eq!(value["status"], json!("COMPLETED"));
    assert_eq!(
        value["result"]["raw_values"]["critical_failures"][0]["node_id"],
        json!("existing_dispute_check")
    );
    assert_eq!(
        value["runtime_metadata"]["model_version"],
        json!("openai:gpt-4.1-2025-04-14")
    );
}

#[tokio::test]
async fn test_simulations_metric_detail_by_metric_id() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/v1/simulations/sim123/metrics/29BlkepvvX19ebbLDB0y6Q",
        ))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metric_outputs": [
                {
                    "metric_output_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
                    "metric_id": "29BlkepvvX19ebbLDB0y6Q",
                    "status": "COMPLETED",
                    "value": 2.35
                },
                {
                    "metric_output_id": "01ARZ3NDEKTSV4RRFFQ69OTHER",
                    "metric_id": "29BlkepvvX19ebbLDB0y6Q",
                    "status": "COMPLETED",
                    "value": 2.41
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("simulations")
        .arg("metric-detail")
        .arg("sim123")
        .arg("29BlkepvvX19ebbLDB0y6Q")
        .assert()
        .success()
        .stdout(predicate::str::contains("01ARZ3NDEKTSV4RRFFQ69G5FAV"))
        .stdout(predicate::str::contains("01ARZ3NDEKTSV4RRFFQ69OTHER"));
}

#[tokio::test]
async fn test_conversations_metric_detail_by_metric_id() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/v1/conversations/conv123/metrics/4HTX6gnqXtpexWSLNaKdC4",
        ))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metric_outputs": [
                {
                    "metric_output_id": "01KKWQYSF737ZN6X1Q1RYX8M2D",
                    "metric_id": "4HTX6gnqXtpexWSLNaKdC4",
                    "status": "COMPLETED",
                    "value": "YES",
                    "result": {
                        "unit": "score",
                        "raw_values": {
                            "critical_failures": [
                                {
                                    "node_id": "existing_dispute_check",
                                    "failure": "Bot did not clarify whether the dispute was new or existing.",
                                    "message_index": 7
                                }
                            ],
                            "non_critical_failures": [
                                {
                                    "node_id": "existing_dispute_check",
                                    "failure": "Bot repeated the question instead of streamlining the path.",
                                    "message_index": 8
                                }
                            ]
                        }
                    },
                    "runtime_metadata": {"model_version": "openai:gpt-4.1-2025-04-14"}
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("--format")
            .arg("json")
            .arg("conversations")
            .arg("metric-detail")
            .arg("conv123")
            .arg("4HTX6gnqXtpexWSLNaKdC4")
            .assert()
            .success(),
    );
    assert_eq!(
        value[0]["metric_output_id"],
        json!("01KKWQYSF737ZN6X1Q1RYX8M2D")
    );
    assert_eq!(value[0]["metric_id"], json!("4HTX6gnqXtpexWSLNaKdC4"));
    assert_eq!(value[0]["value"], json!("YES"));
    assert_eq!(
        value[0]["result"]["raw_values"]["critical_failures"][0]["node_id"],
        json!("existing_dispute_check")
    );
    assert_eq!(
        value[0]["result"]["raw_values"]["non_critical_failures"][0]["message_index"],
        json!(8)
    );
}

#[tokio::test]
async fn test_mutations_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/agents/agent123/mutations"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "mutations": [
                {
                    "id": "mut123",
                    "agent_id": "agent123",
                    "display_name": "GPT-4 Fast",
                    "description": "",
                    "config_overrides": {"model": "gpt-4"},
                    "parameter_values": {"model": "gpt-4"},
                    "create_time": "2025-01-15T10:30:00Z"
                }
            ],
            "total_count": 1
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("mutations")
        .arg("list")
        .arg("--agent-id")
        .arg("agent123")
        .assert()
        .success()
        .stdout(predicate::str::contains("mut123"))
        .stdout(predicate::str::contains("GPT-4 Fast"));
}

#[tokio::test]
async fn test_run_templates_list_hyphenated_path() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/run-templates"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "run_templates": [
                {
                    "id": "rt123",
                    "display_name": "My Template",
                    "metric_ids": [],
                    "mutation_ids": [],
                    "metadata": {},
                    "create_time": "2025-01-15T10:30:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("run-templates")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("rt123"))
        .stdout(predicate::str::contains("My Template"));
}

#[tokio::test]
async fn test_input_json_file() {
    let mock_server = MockServer::start().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let input_path = temp_dir.path().join("test-set.json");
    std::fs::write(
        &input_path,
        r#"{"display_name":"From File","slug":"from-file"}"#,
    )
    .unwrap();

    Mock::given(method("POST"))
        .and(path("/v1/test-sets"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "display_name": "From File",
            "slug": "from-file"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "test_set": {
                "name": "testSets/ts123",
                "id": "ts123",
                "slug": "from-file",
                "display_name": "From File",
                "create_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("test-sets")
        .arg("create")
        .arg("--input-json")
        .arg(format!("@{}", input_path.display()))
        .assert()
        .success()
        .stdout(predicate::str::contains("ts123"));
}

#[tokio::test]
async fn test_input_json_stdin() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/dashboards"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "display_name": "Ops"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "dashboard": {
                "name": "dashboards/dash123",
                "display_name": "Ops",
                "create_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("dashboards")
        .arg("create")
        .arg("--input-json")
        .arg("-")
        .write_stdin(r#"{"display_name":"Ops"}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("dash123"));
}

#[tokio::test]
async fn test_monitors_create_preserves_input_json_evaluation_type() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/monitors"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "name": "Scheduled monitor",
            "evaluation_type": "SCHEDULED",
            "conditions": [{"type": "metric"}]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "mon123",
            "name": "Scheduled monitor",
            "status": "ACTIVE",
            "evaluation_type": "SCHEDULED",
            "conditions": [{"type": "metric"}],
            "create_time": "2026-01-01T00:00:00Z",
            "update_time": "2026-01-01T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("monitors")
        .arg("create")
        .arg("--input-json")
        .arg(
            r#"{"name":"Scheduled monitor","evaluation_type":"SCHEDULED","conditions":[{"type":"metric"}]}"#,
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("mon123"));
}

#[tokio::test]
async fn test_monitors_create_evaluation_type_flag_overrides_input_json() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/monitors"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "name": "Run monitor",
            "evaluation_type": "ON_RUN_COMPLETE",
            "conditions": [{"type": "metric"}]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "mon456",
            "name": "Run monitor",
            "status": "ACTIVE",
            "evaluation_type": "ON_RUN_COMPLETE",
            "conditions": [{"type": "metric"}],
            "create_time": "2026-01-01T00:00:00Z",
            "update_time": "2026-01-01T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("monitors")
        .arg("create")
        .arg("--input-json")
        .arg(
            r#"{"name":"Run monitor","evaluation_type":"SCHEDULED","conditions":[{"type":"metric"}]}"#,
        )
        .arg("--evaluation-type")
        .arg("ON_RUN_COMPLETE")
        .assert()
        .success()
        .stdout(predicate::str::contains("mon456"));
}

#[tokio::test]
async fn test_dashboard_create_with_full_fields() {
    let mock_server = MockServer::start().await;

    // body_partial_json asserts the new flags are serialized into the request body;
    // a mismatch yields no matching mock (404) and the command fails.
    Mock::given(method("POST"))
        .and(path("/v1/dashboards"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "display_name": "Ops",
            "description": "desc",
            "is_favorite": true,
            "is_default": true,
            "position": 3,
            "config": {"date_preferences": {"preset": "last-7-days"}}
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "dashboard": {
                "name": "dashboards/dash123",
                "display_name": "Ops",
                "description": "desc",
                "is_default": true,
                "is_favorite": true,
                "position": 3,
                "config": {"date_preferences": {"preset": "last-7-days"}},
                "create_time": "2025-01-15T10:30:00Z",
                "update_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("dashboards")
        .arg("create")
        .arg("--name")
        .arg("Ops")
        .arg("--description")
        .arg("desc")
        .arg("--favorite")
        .arg("true")
        .arg("--default")
        .arg("true")
        .arg("--position")
        .arg("3")
        .arg("--config")
        .arg(r#"{"date_preferences":{"preset":"last-7-days"}}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("dash123"));
}

#[tokio::test]
async fn test_dashboard_update_sets_default_and_position() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/dashboards/dash123"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "is_default": true,
            "position": 5
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "dashboard": {
                "name": "dashboards/dash123",
                "display_name": "Ops",
                "is_default": true,
                "position": 5,
                "create_time": "2025-01-15T10:30:00Z",
                "update_time": "2025-01-16T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("dashboards")
        .arg("update")
        .arg("dash123")
        .arg("--default")
        .arg("true")
        .arg("--position")
        .arg("5")
        .assert()
        .success()
        .stdout(predicate::str::contains("dash123"));
}

#[test]
fn test_dashboard_create_rejects_negative_position() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg("http://localhost:1")
        .arg("dashboards")
        .arg("create")
        .arg("--name")
        .arg("Ops")
        .arg("--position=-1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--position must be >= 0"));
}

#[test]
fn test_dashboard_update_rejects_negative_position() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg("http://localhost:1")
        .arg("dashboards")
        .arg("update")
        .arg("dash123")
        .arg("--position=-1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--position must be >= 0"));
}

#[test]
fn test_input_json_invalid_agent_error() {
    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg("http://localhost:1")
            .arg("agents")
            .arg("create")
            .arg("--input-json")
            .arg("{")
            .assert()
            .failure()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["ok"], false);
    assert_eq!(value["resource"], "agents");
    assert_eq!(value["operation"], "create");
    assert_eq!(value["error"]["code"], "cli_error");
}

#[tokio::test]
async fn test_scheduled_runs_list_hyphenated_path() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/scheduled-runs"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "scheduled_runs": [
                {
                    "id": "sr123",
                    "display_name": "Daily Run",
                    "run_template_id": "rt123",
                    "schedule_expression": "rate(1 day)",
                    "enabled": true,
                    "create_time": "2025-01-15T10:30:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("scheduled-runs")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("sr123"))
        .stdout(predicate::str::contains("Daily Run"));
}

#[tokio::test]
async fn test_agents_create_with_metadata() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/agents"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agent": {
                "id": "new123",
                "display_name": "Bot",
                "model_type": "MODEL_TYPE_CHAT",
                "metadata": {"chat_endpoint": "https://example.com"},
                "create_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("agents")
        .arg("create")
        .arg("--name")
        .arg("Bot")
        .arg("--type")
        .arg("chat")
        .arg("--metadata")
        .arg(r#"{"chat_endpoint":"https://example.com"}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("new123"));
}

#[tokio::test]
async fn test_agents_create_livekit_with_all_common_fields() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/agents"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "customer_agent_id": "speak-language-tutor",
            "display_name": "Language Tutor",
            "model_type": "MODEL_TYPE_LIVEKIT",
            "prompt": "Teach conversational English.",
            "language": "en",
            "attributes": {"customer": "Speak"},
            "metadata": {
                "generate_token_endpoint": "https://api.example.com/livekit/token",
                "livekit_url": "wss://example.livekit.cloud",
                "livekit_agent_name": "language-tutor"
            },
            "workflows": {"dispatch": {"enabled": true}},
            "metric_ids": ["metric-one", "metric-two"],
            "test_set_ids": ["pilot-suite"],
            "tags": ["pilot", "livekit"]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "agent": {
                "id": "livekit123",
                "display_name": "Language Tutor",
                "model_type": "MODEL_TYPE_LIVEKIT",
                "metadata": {
                    "generate_token_endpoint": "https://api.example.com/livekit/token",
                    "livekit_url": "wss://example.livekit.cloud"
                },
                "create_time": "2026-07-31T18:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("agents")
        .arg("create")
        .arg("--customer-agent-id")
        .arg("speak-language-tutor")
        .arg("--name")
        .arg("Language Tutor")
        .arg("--type")
        .arg("livekit")
        .arg("--prompt")
        .arg("Teach conversational English.")
        .arg("--language")
        .arg("en")
        .arg("--attributes")
        .arg(r#"{"customer":"Speak"}"#)
        .arg("--metadata")
        .arg(
            r#"{"generate_token_endpoint":"https://api.example.com/livekit/token","livekit_url":"wss://example.livekit.cloud","livekit_agent_name":"language-tutor"}"#,
        )
        .arg("--workflows")
        .arg(r#"{"dispatch":{"enabled":true}}"#)
        .arg("--metric-ids")
        .arg("metric-one,metric-two")
        .arg("--test-set-ids")
        .arg("pilot-suite")
        .arg("--tags")
        .arg("pilot,livekit")
        .assert()
        .success()
        .stdout(predicate::str::contains("livekit123"));
}

#[tokio::test]
async fn test_agents_create_with_input_json() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/agents"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "display_name": "Bot",
            "model_type": "MODEL_TYPE_CHAT",
            "metadata": {"chat_endpoint": "https://example.com"}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agent": {
                "id": "new123",
                "display_name": "Bot",
                "model_type": "MODEL_TYPE_CHAT",
                "metadata": {"chat_endpoint": "https://example.com"},
                "create_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("agents")
        .arg("create")
        .arg("--input-json")
        .arg(
            r#"{"display_name":"Bot","model_type":"MODEL_TYPE_CHAT","metadata":{"chat_endpoint":"https://example.com"}}"#,
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("new123"));
}

#[tokio::test]
async fn test_input_json_flags_override_json() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/agents"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "display_name": "Bot",
            "model_type": "MODEL_TYPE_VOICE"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agent": {
                "id": "new123",
                "display_name": "Bot",
                "model_type": "MODEL_TYPE_VOICE",
                "create_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("agents")
        .arg("create")
        .arg("--input-json")
        .arg(r#"{"display_name":"Wrong","model_type":"MODEL_TYPE_CHAT"}"#)
        .arg("--name")
        .arg("Bot")
        .arg("--type")
        .arg("voice")
        .arg("--phone-number")
        .arg("+15551234567")
        .assert()
        .success()
        .stdout(predicate::str::contains("new123"));
}

#[tokio::test]
async fn test_agents_update_with_metadata() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/agents/abc123"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agent": {
                "id": "abc123",
                "display_name": "Updated Agent",
                "model_type": "MODEL_TYPE_CHAT",
                "metadata": {"key": "val"},
                "create_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("agents")
        .arg("update")
        .arg("abc123")
        .arg("--metadata")
        .arg(r#"{"key":"val"}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("abc123"));
}

#[test]
fn test_agents_update_invalid_metadata_json() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg("http://localhost:1")
        .arg("agents")
        .arg("update")
        .arg("abc123")
        .arg("--metadata")
        .arg("not valid json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid JSON for --metadata"));
}

// ── Review Annotations ──────────────────────────────────────────────────

#[tokio::test]
async fn test_review_annotations_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/review-annotations"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "review_annotations": [
                {
                    "id": "ann123",
                    "simulation_output_id": "so123",
                    "metric_id": "met123",
                    "assignee": "reviewer@example.com",
                    "status": "ACTIVE",
                    "completion_status": "PENDING",
                    "priority": "PRIORITY_STANDARD",
                    "create_time": "2025-01-15T10:30:00Z",
                    "update_time": "2025-01-15T10:30:00Z"
                }
            ],
            "next_page_token": null
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("review-annotations")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("ann123"))
        .stdout(predicate::str::contains("reviewer@example.com"));
}

#[tokio::test]
async fn test_review_annotations_get() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/review-annotations/ann123"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "review_annotation": {
                "id": "ann123",
                "simulation_output_id": "so123",
                "metric_id": "met123",
                "assignee": "reviewer@example.com",
                "status": "ACTIVE",
                "completion_status": "PENDING",
                "priority": "PRIORITY_STANDARD",
                "create_time": "2025-01-15T10:30:00Z",
                "update_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("review-annotations")
        .arg("get")
        .arg("ann123")
        .assert()
        .success()
        .stdout(predicate::str::contains("ann123"));
}

#[tokio::test]
async fn test_review_annotations_create() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/review-annotations"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "review_annotation": {
                "id": "ann456",
                "simulation_output_id": "so123",
                "metric_id": "met123",
                "assignee": "reviewer@example.com",
                "status": "ACTIVE",
                "completion_status": "PENDING",
                "priority": "PRIORITY_STANDARD",
                "create_time": "2025-01-15T10:30:00Z",
                "update_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("review-annotations")
        .arg("create")
        .arg("--simulation-output-id")
        .arg("so123")
        .arg("--metric-id")
        .arg("met123")
        .arg("--assignee")
        .arg("reviewer@example.com")
        .assert()
        .success()
        .stdout(predicate::str::contains("ann456"));
}

#[tokio::test]
async fn test_review_annotations_update() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/review-annotations/ann123"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "review_annotation": {
                "id": "ann123",
                "simulation_output_id": "so123",
                "metric_id": "met123",
                "assignee": "reviewer@example.com",
                "status": "ACTIVE",
                "completion_status": "COMPLETED",
                "priority": "PRIORITY_PRIMARY",
                "create_time": "2025-01-15T10:30:00Z",
                "update_time": "2025-01-15T11:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("review-annotations")
        .arg("update")
        .arg("ann123")
        .arg("--priority")
        .arg("primary")
        .arg("--completion-status")
        .arg("completed")
        .assert()
        .success()
        .stdout(predicate::str::contains("ann123"));
}

#[tokio::test]
async fn test_review_annotations_create_with_subvalues() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/review-annotations"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "ground_truth_subvalues_by_timestamp": [
                {
                    "start_offset": 10.5,
                    "end_offset": 12.3,
                    "output_type": "float",
                    "float_value": 1.0
                }
            ]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "review_annotation": {
                "id": "ann789",
                "simulation_output_id": "so123",
                "metric_id": "met123",
                "assignee": "reviewer@example.com",
                "ground_truth_subvalues_by_timestamp": [
                    {
                        "start_offset": 10.5,
                        "end_offset": 12.3,
                        "output_type": "float",
                        "float_value": 1.0
                    }
                ],
                "status": "ACTIVE",
                "completion_status": "COMPLETED",
                "priority": "PRIORITY_STANDARD",
                "create_time": "2025-01-15T10:30:00Z",
                "update_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("review-annotations")
        .arg("create")
        .arg("--simulation-output-id")
        .arg("so123")
        .arg("--metric-id")
        .arg("met123")
        .arg("--assignee")
        .arg("reviewer@example.com")
        .arg("--ground-truth-subvalues")
        .arg(r#"[{"start_offset":10.5,"end_offset":12.3,"output_type":"float","float_value":1.0}]"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("ann789"));
}

#[tokio::test]
async fn test_review_annotations_update_with_subvalues() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/review-annotations/ann123"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "ground_truth_subvalues_by_timestamp": [
                {
                    "start_offset": 5.0,
                    "end_offset": 8.0,
                    "output_type": "string",
                    "string_value": "Neutral",
                    "role": "agent"
                },
                {
                    "start_offset": 12.0,
                    "end_offset": 15.5,
                    "output_type": "string",
                    "string_value": "Positive",
                    "role": "persona"
                }
            ]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "review_annotation": {
                "id": "ann123",
                "simulation_output_id": "so123",
                "metric_id": "met123",
                "assignee": "reviewer@example.com",
                "ground_truth_subvalues_by_timestamp": [
                    {
                        "start_offset": 5.0,
                        "end_offset": 8.0,
                        "output_type": "string",
                        "string_value": "Neutral",
                        "role": "agent"
                    },
                    {
                        "start_offset": 12.0,
                        "end_offset": 15.5,
                        "output_type": "string",
                        "string_value": "Positive",
                        "role": "persona"
                    }
                ],
                "status": "ACTIVE",
                "completion_status": "COMPLETED",
                "priority": "PRIORITY_STANDARD",
                "create_time": "2025-01-15T10:30:00Z",
                "update_time": "2025-01-15T11:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("review-annotations")
        .arg("update")
        .arg("ann123")
        .arg("--ground-truth-subvalues")
        .arg(r#"[{"start_offset":5.0,"end_offset":8.0,"output_type":"string","string_value":"Neutral","role":"agent"},{"start_offset":12.0,"end_offset":15.5,"output_type":"string","string_value":"Positive","role":"persona"}]"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("ann123"));
}

#[tokio::test]
async fn test_review_annotations_create_with_invalid_subvalues_json() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg("http://localhost:1")
        .arg("review-annotations")
        .arg("create")
        .arg("--simulation-output-id")
        .arg("so123")
        .arg("--metric-id")
        .arg("met123")
        .arg("--assignee")
        .arg("reviewer@example.com")
        .arg("--ground-truth-subvalues")
        .arg("not valid json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("expected ident"));
}

#[tokio::test]
async fn test_review_annotations_delete() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/review-annotations/ann123"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("review-annotations")
        .arg("delete")
        .arg("ann123")
        .assert()
        .success()
        .stdout(predicate::str::contains("deleted"));
}

// ── Review Projects ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_review_projects_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/review-projects"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "review_projects": [
                {
                    "id": "proj123",
                    "display_name": "Q1 Review",
                    "assignees": ["alice@example.com"],
                    "linked_simulation_ids": [],
                    "linked_metric_ids": [],
                    "project_type": "PROJECT_COLLABORATIVE",
                    "notifications": true,
                    "create_time": "2025-01-15T10:30:00Z",
                    "update_time": "2025-01-15T10:30:00Z"
                }
            ],
            "next_page_token": null
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("review-projects")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("proj123"))
        .stdout(predicate::str::contains("Q1 Review"));
}

#[tokio::test]
async fn test_review_projects_get() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/review-projects/proj123"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "review_project": {
                "id": "proj123",
                "display_name": "Q1 Review",
                "assignees": ["alice@example.com"],
                "linked_simulation_ids": [],
                "linked_metric_ids": [],
                "project_type": "PROJECT_COLLABORATIVE",
                "notifications": true,
                "create_time": "2025-01-15T10:30:00Z",
                "update_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("review-projects")
        .arg("get")
        .arg("proj123")
        .assert()
        .success()
        .stdout(predicate::str::contains("proj123"));
}

#[tokio::test]
async fn test_review_projects_create() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/review-projects"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "review_project": {
                "id": "proj456",
                "display_name": "New Project",
                "assignees": ["alice@example.com", "bob@example.com"],
                "linked_simulation_ids": ["sim1"],
                "linked_metric_ids": ["met1"],
                "project_type": "PROJECT_COLLABORATIVE",
                "notifications": true,
                "create_time": "2025-01-15T10:30:00Z",
                "update_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("review-projects")
        .arg("create")
        .arg("--name")
        .arg("New Project")
        .arg("--assignees")
        .arg("alice@example.com,bob@example.com")
        .arg("--simulation-ids")
        .arg("sim1")
        .arg("--metric-ids")
        .arg("met1")
        .assert()
        .success()
        .stdout(predicate::str::contains("proj456"));
}

#[tokio::test]
async fn test_review_projects_update() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/review-projects/proj123"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "review_project": {
                "id": "proj123",
                "display_name": "Updated Project",
                "assignees": ["alice@example.com"],
                "linked_simulation_ids": [],
                "linked_metric_ids": [],
                "project_type": "PROJECT_COLLABORATIVE",
                "notifications": false,
                "create_time": "2025-01-15T10:30:00Z",
                "update_time": "2025-01-15T11:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("review-projects")
        .arg("update")
        .arg("proj123")
        .arg("--name")
        .arg("Updated Project")
        .arg("--notifications")
        .arg("false")
        .assert()
        .success()
        .stdout(predicate::str::contains("proj123"));
}

#[tokio::test]
async fn test_review_projects_delete() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/review-projects/proj123"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("review-projects")
        .arg("delete")
        .arg("proj123")
        .assert()
        .success()
        .stdout(predicate::str::contains("deleted"));
}

#[tokio::test]
async fn test_metrics_create_composite_test_case() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/metrics"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "metric_type": "METRIC_COMPOSITE_EVALUATION",
            "criteria_source": "test_case",
            "criteria_path": "expected_behaviors",
            "reporting_method": "all_criteria_met"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metric": {
                "name": "metrics/comp1",
                "id": "comp1",
                "metric_name": "Adversarial Composite",
                "description": "All behaviors met",
                "metric_type": "METRIC_COMPOSITE_EVALUATION",
                "criteria_source": "test_case",
                "criteria_path": "expected_behaviors",
                "reporting_method": "all_criteria_met",
                "create_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("metrics")
        .arg("create")
        .arg("--name")
        .arg("Adversarial Composite")
        .arg("--description")
        .arg("All behaviors met")
        .arg("--type")
        .arg("composite")
        .arg("--criteria-source")
        .arg("test_case")
        .arg("--criteria-path")
        .arg("expected_behaviors")
        .arg("--reporting-method")
        .arg("all_criteria_met")
        .assert()
        .success()
        .stdout(predicate::str::contains("comp1"));
}

fn metric_response_body() -> Value {
    json!({
        "metric": {
            "name": "metrics/met1",
            "id": "met1",
            "metric_name": "Pause analysis",
            "description": "Silence handling",
            "metric_type": "METRIC_PAUSE_ANALYSIS",
            "create_time": "2026-09-02T10:30:00Z"
        }
    })
}

/// Every request field the audit records as published for POST /v1/metrics, so a
/// struct that stops declaring one fails here rather than dropping it in silence.
fn newly_modeled_metric_fields() -> Value {
    json!({
        "max_silence_duration_seconds": 4.5,
        "min_silence_gap_seconds": 0.75,
        "frequency_threshold": 2.0,
        "direction": "above",
        "success_sentiments": ["Happy", "Neutral"],
        "percent_above": 25.0,
        "success_end_reasons": ["AGENT_ENDED_CALL"],
        "observation_name": "long pauses",
        "expected_body": {"status": "ok"},
        "match_path": "payload.status",
        "min_volume_change_for_pitch_misalignment": 3.5,
        "threshold": 4,
        "operator": ">=",
        "sql_query": "SELECT 1",
        "runtime_config": {"model_version": "openai:gpt-4.1-mini-2025-04-14"},
        "tags": ["voice", "latency"]
    })
}

#[tokio::test]
async fn test_metrics_create_forwards_every_modeled_field() {
    let mock_server = MockServer::start().await;
    let capture = BodyCapture::default();

    Mock::given(method("POST"))
        .and(path("/v1/metrics"))
        .and(header("X-API-Key", "test_key"))
        .and(capture.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(metric_response_body()))
        .mount(&mock_server)
        .await;

    let fields = newly_modeled_metric_fields();
    let mut input = fields.as_object().unwrap().clone();
    input.insert("metric_name".into(), json!("Pause analysis"));
    input.insert("description".into(), json!("Silence handling"));
    input.insert("metric_type".into(), json!("METRIC_PAUSE_ANALYSIS"));

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("metrics")
        .arg("create")
        .arg("--input-json")
        .arg(Value::Object(input).to_string())
        .assert()
        .success();

    let body = capture.take();
    for (key, expected) in fields.as_object().unwrap() {
        assert_eq!(&body[key], expected, "field {key} must reach the API");
    }
}

#[tokio::test]
async fn test_metrics_update_forwards_every_modeled_field() {
    let mock_server = MockServer::start().await;
    let capture = BodyCapture::default();

    Mock::given(method("PATCH"))
        .and(path("/v1/metrics/met1"))
        .and(header("X-API-Key", "test_key"))
        .and(capture.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(metric_response_body()))
        .mount(&mock_server)
        .await;

    let fields = newly_modeled_metric_fields();

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("metrics")
        .arg("update")
        .arg("met1")
        .arg("--input-json")
        .arg(fields.to_string())
        .assert()
        .success();

    let body = capture.take();
    for (key, expected) in fields.as_object().unwrap() {
        assert_eq!(&body[key], expected, "field {key} must reach the API");
    }
}

#[tokio::test]
async fn test_metrics_create_forwards_flag_values() {
    let mock_server = MockServer::start().await;
    let capture = BodyCapture::default();

    Mock::given(method("POST"))
        .and(path("/v1/metrics"))
        .and(header("X-API-Key", "test_key"))
        .and(capture.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(metric_response_body()))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("metrics")
        .arg("create")
        .arg("--name")
        .arg("Pause analysis")
        .arg("--description")
        .arg("Silence handling")
        .arg("--type")
        .arg("pause")
        .arg("--direction")
        .arg("above")
        .arg("--threshold")
        .arg("4")
        .arg("--operator")
        .arg(">=")
        .arg("--success-sentiments")
        .arg("Happy,Neutral")
        .arg("--tags")
        .arg("voice,latency")
        .arg("--runtime-config")
        .arg(r#"{"model_version":"openai:gpt-4.1-mini-2025-04-14"}"#)
        .assert()
        .success();

    let body = capture.take();
    assert_eq!(body["direction"], "above");
    assert_eq!(body["threshold"], 4);
    assert_eq!(body["operator"], ">=");
    assert_eq!(body["success_sentiments"], json!(["Happy", "Neutral"]));
    assert_eq!(body["tags"], json!(["voice", "latency"]));
    assert_eq!(
        body["runtime_config"],
        json!({"model_version": "openai:gpt-4.1-mini-2025-04-14"})
    );
}

#[tokio::test]
async fn test_metrics_create_accepts_a_plain_string_expected_body() {
    let mock_server = MockServer::start().await;
    let capture = BodyCapture::default();

    Mock::given(method("POST"))
        .and(path("/v1/metrics"))
        .and(header("X-API-Key", "test_key"))
        .and(capture.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(metric_response_body()))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("metrics")
        .arg("create")
        .arg("--name")
        .arg("Toolcall check")
        .arg("--description")
        .arg("Body match")
        .arg("--type")
        .arg("toolcall")
        .arg("--expected-body")
        .arg("not json at all")
        .assert()
        .success();

    assert_eq!(capture.take()["expected_body"], "not json at all");
}

#[tokio::test]
async fn test_metrics_create_omits_unset_fields() {
    let mock_server = MockServer::start().await;
    let capture = BodyCapture::default();

    Mock::given(method("POST"))
        .and(path("/v1/metrics"))
        .and(header("X-API-Key", "test_key"))
        .and(capture.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(metric_response_body()))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("metrics")
        .arg("create")
        .arg("--name")
        .arg("Simple")
        .arg("--description")
        .arg("No optional fields")
        .arg("--type")
        .arg("llm-binary")
        .arg("--prompt")
        .arg("Did the agent greet the caller?")
        .assert()
        .success();

    let body = capture.take();
    for key in newly_modeled_metric_fields().as_object().unwrap().keys() {
        assert!(body.get(key).is_none(), "unset field {key} must be omitted");
    }
}

#[tokio::test]
async fn test_metrics_update_clears_tags_with_an_empty_list() {
    let mock_server = MockServer::start().await;
    let capture = BodyCapture::default();

    Mock::given(method("PATCH"))
        .and(path("/v1/metrics/met1"))
        .and(header("X-API-Key", "test_key"))
        .and(capture.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(metric_response_body()))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("metrics")
        .arg("update")
        .arg("met1")
        .arg("--input-json")
        .arg(r#"{"tags":[]}"#)
        .assert()
        .success();

    assert_eq!(capture.take()["tags"], json!([]));
}

#[tokio::test]
async fn test_metrics_test_sends_a_batch_and_reports_each_result() {
    let mock_server = MockServer::start().await;
    let capture = BodyCapture::default();

    Mock::given(method("POST"))
        .and(path("/v1/metrics/met1/test"))
        .and(header("X-API-Key", "test_key"))
        .and(capture.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [
                {
                    "simulation_output_id": "aaaaaaaaaaaaaaaaaaaaaa",
                    "status": "QUEUED",
                    "metric_output_ulid": "01ARZ3NDEKTSV4RRFFQ69G5FAV"
                },
                {
                    "simulation_output_id": "bbbbbbbbbbbbbbbbbbbbbb",
                    "status": "NOT_FOUND",
                    "error": "Simulation output not found"
                }
            ],
            "metric_output_ulid": null
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("metrics")
        .arg("test")
        .arg("met1")
        .arg("--simulation-output-ids")
        .arg("aaaaaaaaaaaaaaaaaaaaaa,bbbbbbbbbbbbbbbbbbbbbb")
        .assert()
        .success()
        .stdout(predicate::str::contains("NOT_FOUND"));

    let body = capture.take();
    assert_eq!(
        body["simulation_output_ids"],
        json!(["aaaaaaaaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbbbbbbbb"])
    );
    assert!(body.get("simulation_output_id").is_none());
}

#[tokio::test]
async fn test_metrics_test_still_accepts_the_deprecated_single_id() {
    let mock_server = MockServer::start().await;
    let capture = BodyCapture::default();

    Mock::given(method("POST"))
        .and(path("/v1/metrics/met1/test"))
        .and(header("X-API-Key", "test_key"))
        .and(capture.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [
                {
                    "simulation_output_id": "aaaaaaaaaaaaaaaaaaaaaa",
                    "status": "QUEUED",
                    "metric_output_ulid": "01ARZ3NDEKTSV4RRFFQ69G5FAV"
                }
            ],
            "metric_output_ulid": "01ARZ3NDEKTSV4RRFFQ69G5FAV"
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("metrics")
        .arg("test")
        .arg("met1")
        .arg("--simulation-output-id")
        .arg("aaaaaaaaaaaaaaaaaaaaaa")
        .assert()
        .success()
        .stdout(predicate::str::contains("01ARZ3NDEKTSV4RRFFQ69G5FAV"));

    let body = capture.take();
    assert_eq!(body["simulation_output_id"], "aaaaaaaaaaaaaaaaaaaaaa");
    assert!(body.get("simulation_output_ids").is_none());
}

#[test]
fn test_metrics_test_requires_a_simulation_target() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("metrics")
        .arg("test")
        .arg("met1")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Provide --simulation-output-id or --simulation-output-ids",
        ));
}

#[test]
fn test_metrics_test_rejects_both_simulation_targets() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("metrics")
        .arg("test")
        .arg("met1")
        .arg("--simulation-output-id")
        .arg("aaaaaaaaaaaaaaaaaaaaaa")
        .arg("--simulation-output-ids")
        .arg("bbbbbbbbbbbbbbbbbbbbbb")
        .assert()
        .failure();
}

#[tokio::test]
async fn test_metrics_duplicate() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/metrics/met_source/duplicate"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "metric": {
                "name": "metrics/met_copy",
                "id": "met_copy",
                "metric_name": "Copied metric",
                "description": "A duplicated metric",
                "metric_type": "METRIC_LLM",
                "create_time": "2026-08-12T12:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("metrics")
        .arg("duplicate")
        .arg("met_source")
        .assert()
        .success()
        .stdout(predicate::str::contains("met_copy"));
}

#[test]
fn test_metrics_create_composite_test_case_requires_criteria_path() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("metrics")
        .arg("create")
        .arg("--name")
        .arg("Bad Composite")
        .arg("--description")
        .arg("missing path")
        .arg("--type")
        .arg("composite")
        .arg("--criteria-source")
        .arg("test_case")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--criteria-path is required"));
}

#[test]
fn test_metrics_create_composite_metadata_requires_criteria() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("metrics")
        .arg("create")
        .arg("--name")
        .arg("Bad Composite")
        .arg("--description")
        .arg("missing criteria")
        .arg("--type")
        .arg("composite")
        .arg("--criteria-source")
        .arg("metric_metadata")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--criteria is required"));
}

#[tokio::test]
async fn test_metrics_test_subcommand() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/metrics/met_abc/test"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "simulation_output_id": "simout_def456"
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "metric_output_ulid": "01HXKZ4M5N6P7Q8R9STVWXYZAB"
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("metrics")
        .arg("test")
        .arg("met_abc")
        .arg("--simulation-output-id")
        .arg("simout_def456")
        .assert()
        .success()
        .stdout(predicate::str::contains("01HXKZ4M5N6P7Q8R9STVWXYZAB"));
}

#[tokio::test]
async fn test_metrics_test_subcommand_with_dev_id() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/metrics/met_abc/test"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "simulation_output_id": "simout_def456",
            "dev_id": "debug-trace-001"
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "metric_output_ulid": "01HXKZ4M5N6P7Q8R9STVWXYZAB"
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("metrics")
        .arg("test")
        .arg("met_abc")
        .arg("--simulation-output-id")
        .arg("simout_def456")
        .arg("--dev-id")
        .arg("debug-trace-001")
        .assert()
        .success()
        .stdout(predicate::str::contains("01HXKZ4M5N6P7Q8R9STVWXYZAB"));
}

#[tokio::test]
async fn test_test_cases_create_with_expected_behaviors() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/test-cases"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "test_set_id": "ts123",
            "input_str": "probe",
            "expected_behaviors": ["refuses", "stays on policy"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "test_case": {
                "name": "test-cases/tc1",
                "id": "tc1",
                "test_set_id": "ts123",
                "input_str": "probe",
                "expected_behaviors": ["refuses", "stays on policy"],
                "create_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("test-cases")
        .arg("create")
        .arg("--test-set-id")
        .arg("ts123")
        .arg("--input")
        .arg("probe")
        .arg("--expected-behavior")
        .arg("refuses")
        .arg("--expected-behavior")
        .arg("stays on policy")
        .assert()
        .success()
        .stdout(predicate::str::contains("tc1"));
}

fn full_test_case_fields() -> Value {
    json!({
        "input_str": "What is the weather today?",
        "expected_behaviors": ["states the forecast", "mentions temperature units"],
        "expected_output_str": "Sunny with a high of 75 degrees.",
        "expected_output_json": {"temperature": 75, "condition": "sunny"},
        "description": "Weather query with sunny conditions",
        "input_type": "SCENARIO",
        "simulation_metadata_input": {
            "script_turns": ["Hi, what is the weather?", "Thanks, goodbye."]
        },
        "metric_input": {"expected_entities": ["weather", "temperature"]},
        "user_notes": "Added for regression testing weather queries"
    })
}

fn test_case_response_body(fields: &Value) -> Value {
    json!({
        "test_case": {
            "name": "test-cases/tc1",
            "id": "tc1",
            "test_set_id": "ts123",
            "input_str": fields["input_str"],
            "create_time": "2025-01-15T10:30:00Z"
        }
    })
}

#[tokio::test]
async fn test_test_cases_create_stdin_preserves_all_fields() {
    let mock_server = MockServer::start().await;
    let fields = full_test_case_fields();

    let mut expected_body = fields.clone();
    expected_body["test_set_id"] = json!("ts123");

    Mock::given(method("POST"))
        .and(path("/v1/test-cases"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(test_case_response_body(&fields)))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("--format")
            .arg("json")
            .arg("test-cases")
            .arg("create")
            .arg("--test-set-id")
            .arg("ts123")
            .arg("--stdin")
            .write_stdin(format!("{fields}\n"))
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["created"], 1);
    assert_eq!(value["failed"], 0);
}

#[tokio::test]
async fn test_test_cases_create_stdin_body_matches_input_json_body() {
    let fields = full_test_case_fields();
    let response_body = test_case_response_body(&fields);

    let input_json_server = MockServer::start().await;
    let input_json_capture = BodyCapture::default();
    Mock::given(method("POST"))
        .and(path("/v1/test-cases"))
        .and(header("X-API-Key", "test_key"))
        .and(input_json_capture.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body.clone()))
        .mount(&input_json_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(input_json_server.uri())
        .arg("--format")
        .arg("json")
        .arg("test-cases")
        .arg("create")
        .arg("--test-set-id")
        .arg("ts123")
        .arg("--input-json")
        .arg(fields.to_string())
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let stdin_server = MockServer::start().await;
    let stdin_capture = BodyCapture::default();
    Mock::given(method("POST"))
        .and(path("/v1/test-cases"))
        .and(header("X-API-Key", "test_key"))
        .and(stdin_capture.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&stdin_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(stdin_server.uri())
            .arg("--format")
            .arg("json")
            .arg("test-cases")
            .arg("create")
            .arg("--test-set-id")
            .arg("ts123")
            .arg("--stdin")
            .write_stdin(format!("{fields}\n"))
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );
    assert_eq!(value["created"], 1);
    assert_eq!(value["failed"], 0);

    let input_json_body = input_json_capture.take();
    let stdin_body = stdin_capture.take();
    assert_eq!(stdin_body, input_json_body);
    for (key, expected) in fields.as_object().unwrap() {
        assert_eq!(&stdin_body[key], expected, "field {key} must survive stdin");
    }
    assert_eq!(stdin_body["test_set_id"], "ts123");
}

#[tokio::test]
async fn test_test_cases_create_forwards_top_level_script_turns() {
    let mock_server = MockServer::start().await;
    let capture = BodyCapture::default();
    let script_turns = json!(["Hi.", {"type": "dtmf", "digits": "1"}, {"type": "skip"}]);

    Mock::given(method("POST"))
        .and(path("/v1/test-cases"))
        .and(header("X-API-Key", "test_key"))
        .and(capture.clone())
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(test_case_response_body(&full_test_case_fields())),
        )
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("--format")
        .arg("json")
        .arg("test-cases")
        .arg("create")
        .arg("--test-set-id")
        .arg("ts123")
        .arg("--input-json")
        .arg(
            json!({
                "input_str": "scripted call",
                "input_type": "SCRIPT",
                "script_turns": script_turns,
            })
            .to_string(),
        )
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let body = capture.take();
    assert_eq!(body["script_turns"], script_turns);
    assert_eq!(body["input_type"], "SCRIPT");
}

#[tokio::test]
async fn test_test_cases_create_stdin_forwards_top_level_script_turns() {
    let mock_server = MockServer::start().await;
    let capture = BodyCapture::default();
    let script_turns = json!(["Hi.", {"type": "skip"}]);

    Mock::given(method("POST"))
        .and(path("/v1/test-cases"))
        .and(header("X-API-Key", "test_key"))
        .and(capture.clone())
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(test_case_response_body(&full_test_case_fields())),
        )
        .mount(&mock_server)
        .await;

    let line = json!({
        "input_str": "scripted call",
        "input_type": "SCRIPT",
        "script_turns": script_turns,
    });

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("--format")
        .arg("json")
        .arg("test-cases")
        .arg("create")
        .arg("--test-set-id")
        .arg("ts123")
        .arg("--stdin")
        .write_stdin(format!("{line}\n"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    assert_eq!(capture.take()["script_turns"], script_turns);
}

#[tokio::test]
async fn test_test_cases_create_omits_absent_script_turns() {
    let mock_server = MockServer::start().await;
    let capture = BodyCapture::default();

    Mock::given(method("POST"))
        .and(path("/v1/test-cases"))
        .and(header("X-API-Key", "test_key"))
        .and(capture.clone())
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(test_case_response_body(&full_test_case_fields())),
        )
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("--format")
        .arg("json")
        .arg("test-cases")
        .arg("create")
        .arg("--test-set-id")
        .arg("ts123")
        .arg("--input")
        .arg("plain scenario")
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let body = capture.take();
    assert!(
        body.get("script_turns").is_none(),
        "absent script_turns must not be sent: {body}"
    );
}

#[tokio::test]
async fn test_test_cases_update_distinguishes_absent_and_null_script_turns() {
    let response_body = test_case_response_body(&full_test_case_fields());

    let null_server = MockServer::start().await;
    let null_capture = BodyCapture::default();
    Mock::given(method("PATCH"))
        .and(path("/v1/test-cases/tc1"))
        .and(header("X-API-Key", "test_key"))
        .and(null_capture.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body.clone()))
        .mount(&null_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(null_server.uri())
        .arg("--format")
        .arg("json")
        .arg("test-cases")
        .arg("update")
        .arg("tc1")
        .arg("--input-json")
        .arg(json!({"script_turns": Value::Null}).to_string())
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let null_body = null_capture.take();
    assert!(
        null_body.get("script_turns").is_some_and(Value::is_null),
        "explicit null must clear the turns: {null_body}"
    );

    let absent_server = MockServer::start().await;
    let absent_capture = BodyCapture::default();
    Mock::given(method("PATCH"))
        .and(path("/v1/test-cases/tc1"))
        .and(header("X-API-Key", "test_key"))
        .and(absent_capture.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&absent_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(absent_server.uri())
        .arg("--format")
        .arg("json")
        .arg("test-cases")
        .arg("update")
        .arg("tc1")
        .arg("--description")
        .arg("updated")
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let absent_body = absent_capture.take();
    assert!(
        absent_body.get("script_turns").is_none(),
        "an unrelated update must not clear the turns: {absent_body}"
    );
}

#[tokio::test]
async fn test_test_cases_get_json_output_preserves_fields() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/test-cases/tc1"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "test_case": {
                "name": "test-cases/tc1",
                "id": "tc1",
                "test_set_id": "ts123",
                "input_str": "What is the weather today?",
                "expected_behaviors": ["states the forecast"],
                "expected_output_str": "Sunny with a high of 75 degrees.",
                "expected_output_json": {"temperature": 75, "condition": "sunny"},
                "description": "Weather query with sunny conditions",
                "input_type": "SCRIPT",
                "script_turns": ["Hi", {"type": "dtmf", "digits": "1"}],
                "simulation_metadata_input": {"script_turns": ["Hi"]},
                "metric_input": {"expected_entities": ["weather"]},
                "user_notes": "Added for regression testing weather queries",
                "create_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("--format")
            .arg("json")
            .arg("test-cases")
            .arg("get")
            .arg("tc1")
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["expected_behaviors"], json!(["states the forecast"]));
    assert_eq!(
        value["expected_output_json"],
        json!({"temperature": 75, "condition": "sunny"})
    );
    assert_eq!(
        value["script_turns"],
        json!(["Hi", {"type": "dtmf", "digits": "1"}])
    );
    assert_eq!(
        value["simulation_metadata_input"]["script_turns"],
        json!(["Hi"])
    );
    assert_eq!(
        value["metric_input"]["expected_entities"],
        json!(["weather"])
    );
    assert_eq!(
        value["user_notes"],
        "Added for regression testing weather queries"
    );
}

#[tokio::test]
async fn test_test_sets_create_preserves_parameters() {
    let mock_server = MockServer::start().await;
    let parameters = json!({"name": ["Alice", "Bob"], "issue_type": ["billing", "technical"]});

    Mock::given(method("POST"))
        .and(path("/v1/test-sets"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "display_name": "Parameterized Set",
            "parameters": parameters
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "test_set": {
                "name": "testSets/ts123",
                "id": "ts123",
                "slug": "parameterized-set",
                "display_name": "Parameterized Set",
                "parameters": parameters,
                "create_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("--format")
            .arg("json")
            .arg("test-sets")
            .arg("create")
            .arg("--input-json")
            .arg(json!({"display_name": "Parameterized Set", "parameters": parameters}).to_string())
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["id"], "ts123");
    assert_eq!(value["parameters"]["name"], json!(["Alice", "Bob"]));
}

#[tokio::test]
async fn test_test_sets_update_preserves_parameters() {
    let mock_server = MockServer::start().await;
    let parameters = json!({"name": ["Carol"]});

    Mock::given(method("PATCH"))
        .and(path("/v1/test-sets/ts123"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({"parameters": parameters})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "test_set": {
                "name": "testSets/ts123",
                "id": "ts123",
                "slug": "parameterized-set",
                "display_name": "Parameterized Set",
                "parameters": parameters,
                "create_time": "2025-01-15T10:30:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("--format")
            .arg("json")
            .arg("test-sets")
            .arg("update")
            .arg("ts123")
            .arg("--input-json")
            .arg(json!({"parameters": parameters}).to_string())
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["parameters"]["name"], json!(["Carol"]));
}

#[tokio::test]
async fn test_reports_create_compare_by_test_case() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/reports"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "name": "Adversarial Scorecard",
            "run_ids": ["run1", "run2"],
            "compare_by": "test_case"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "report": {
                "id": "01HXXXXXXXXXXXXXXXXXXXXXXX",
                "name": "Adversarial Scorecard",
                "run_ids": ["run1", "run2"],
                "compare_by": "test_case",
                "permissions": "PRIVATE"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("reports")
        .arg("create")
        .arg("--name")
        .arg("Adversarial Scorecard")
        .arg("--run-ids")
        .arg("run1,run2")
        .arg("--compare-by")
        .arg("test_case")
        .assert()
        .success()
        .stdout(predicate::str::contains("01HXXXXXXXXXXXXXXXXXXXXXXX"));
}

#[tokio::test]
async fn test_reports_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/reports"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "reports": [
                {
                    "id": "01HXXXXXXXXXXXXXXXXXXXXXXX",
                    "name": "Adversarial Scorecard",
                    "run_ids": ["run1"],
                    "compare_by": "test_case",
                    "permissions": "PRIVATE"
                }
            ],
            "next_cursor": null
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("reports")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Adversarial Scorecard"));
}

#[tokio::test]
async fn test_reports_get() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/reports/01HXXXXXXXXXXXXXXXXXXXXXXX"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "report": {
                "id": "01HXXXXXXXXXXXXXXXXXXXXXXX",
                "name": "Adversarial Scorecard",
                "run_ids": ["run1"],
                "compare_by": "test_case",
                "permissions": "PRIVATE"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("reports")
        .arg("get")
        .arg("01HXXXXXXXXXXXXXXXXXXXXXXX")
        .assert()
        .success()
        .stdout(predicate::str::contains("01HXXXXXXXXXXXXXXXXXXXXXXX"));
}

#[tokio::test]
async fn test_reports_delete() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/reports/01HXXXXXXXXXXXXXXXXXXXXXXX"))
        .and(header("X-API-Key", "test_key"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("reports")
        .arg("delete")
        .arg("01HXXXXXXXXXXXXXXXXXXXXXXX")
        .assert()
        .success()
        .stdout(predicate::str::contains("deleted"));
}

#[tokio::test]
async fn test_reports_create_lowercase_permissions_serializes_uppercase() {
    let mock_server = MockServer::start().await;

    // The CLI accepts lowercase `public` to match the other enums, but must
    // serialize the uppercase wire value the v1 API requires.
    Mock::given(method("POST"))
        .and(path("/v1/reports"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "name": "Public Scorecard",
            "run_ids": ["run1"],
            "permissions": "PUBLIC"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "report": {
                "id": "01HXXXXXXXXXXXXXXXXXXXXXXX",
                "name": "Public Scorecard",
                "run_ids": ["run1"],
                "compare_by": "none",
                "permissions": "PUBLIC"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("reports")
        .arg("create")
        .arg("--name")
        .arg("Public Scorecard")
        .arg("--run-ids")
        .arg("run1")
        .arg("--permissions")
        .arg("public")
        .assert()
        .success()
        .stdout(predicate::str::contains("01HXXXXXXXXXXXXXXXXXXXXXXX"));
}

#[test]
fn test_reports_create_metadata_requires_metadata_key() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("reports")
        .arg("create")
        .arg("--name")
        .arg("Bad Report")
        .arg("--run-ids")
        .arg("run1")
        .arg("--compare-by")
        .arg("metadata")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--metadata-key is required"));
}

#[test]
fn test_reports_create_metadata_key_rejected_without_metadata_compare_by() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("reports")
        .arg("create")
        .arg("--name")
        .arg("Bad Report")
        .arg("--run-ids")
        .arg("run1")
        .arg("--compare-by")
        .arg("test_case")
        .arg("--metadata-key")
        .arg("region")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "can only be set when --compare-by is metadata",
        ));
}

#[test]
fn test_reports_create_metadata_rejects_null_metadata_key() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("reports")
        .arg("create")
        .arg("--name")
        .arg("Bad Report")
        .arg("--run-ids")
        .arg("run1")
        .arg("--compare-by")
        .arg("metadata")
        .arg("--input-json")
        .arg(r#"{"metadata_key": null}"#)
        .assert()
        .failure()
        .stderr(predicate::str::contains("--metadata-key is required"));
}

#[tokio::test]
async fn test_reports_create_allows_null_metadata_key_without_metadata_compare_by() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/reports"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "name": "Report",
            "run_ids": ["run1"],
            "compare_by": "run"
        })))
        .and(BodyExcludes("metadata_key"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "report": {
                "id": "01HXXXXXXXXXXXXXXXXXXXXXXX",
                "name": "Report",
                "run_ids": ["run1"],
                "compare_by": "run",
                "permissions": "PRIVATE"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("reports")
        .arg("create")
        .arg("--name")
        .arg("Report")
        .arg("--run-ids")
        .arg("run1")
        .arg("--compare-by")
        .arg("run")
        .arg("--input-json")
        .arg(r#"{"metadata_key": null}"#)
        .assert()
        .success()
        .stderr(predicate::str::contains("--metadata-key can only be set").not());
}

#[tokio::test]
async fn test_reports_rows_forwards_paging_and_filters() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/reports/01HXXXXXXXXXXXXXXXXXXXXXXX/rows"))
        .and(header("X-API-Key", "test_key"))
        .and(query_param("cursor", "500"))
        .and(query_param("limit", "50"))
        .and(query_param("metric_ids", "metric1,metric2"))
        .and(query_param("simulation_output_ids", "sim1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "rows": [
                {
                    "simulation_id": "sim1",
                    "run_id": "run1",
                    "agent_id": "agent1",
                    "persona_id": "persona1",
                    "status": null,
                    "metrics": []
                }
            ],
            "next_page_token": null
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("reports")
        .arg("rows")
        .arg("01HXXXXXXXXXXXXXXXXXXXXXXX")
        .arg("--cursor")
        .arg("500")
        .arg("--limit")
        .arg("50")
        .arg("--metric-ids")
        .arg("metric1,metric2")
        .arg("--simulation-ids")
        .arg("sim1")
        .assert()
        .success()
        .stdout(predicate::str::contains("sim1"));
}

async fn mount_merge_source(
    mock_server: &MockServer,
    report_id: &str,
    name: &str,
    run_ids: Value,
    row_pages: Vec<(Option<&str>, Value, Option<&str>)>,
) {
    Mock::given(method("GET"))
        .and(path(format!("/v1/reports/{report_id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "report": {
                "id": report_id,
                "name": name,
                "run_ids": run_ids,
                "compare_by": "none",
                "permissions": "PRIVATE"
            }
        })))
        .mount(mock_server)
        .await;

    for (cursor, rows, next_page_token) in row_pages {
        let mock = Mock::given(method("GET"))
            .and(path(format!("/v1/reports/{report_id}/rows")))
            .and(query_param("limit", "500"));
        let mock = match cursor {
            Some(cursor) => mock.and(query_param("cursor", cursor)),
            None => mock.and(QueryParamAbsent("cursor")),
        };
        mock.respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "rows": rows,
            "next_page_token": next_page_token
        })))
        .mount(mock_server)
        .await;
    }
}

#[tokio::test]
async fn test_reports_merge_builds_one_group_per_source_report() {
    let mock_server = MockServer::start().await;

    mount_merge_source(
        &mock_server,
        "01HAAAAAAAAAAAAAAAAAAAAAAA",
        "Baseline",
        json!(["run1", "run2"]),
        vec![
            (
                None,
                json!([{"simulation_id": "sim1", "run_id": "run1"}]),
                Some("500"),
            ),
            (
                Some("500"),
                json!([{"simulation_id": "sim2", "run_id": "run2"}]),
                None,
            ),
        ],
    )
    .await;

    // sim2 is in both reports; first-seen attribution keeps it in the Baseline group only.
    mount_merge_source(
        &mock_server,
        "01HBBBBBBBBBBBBBBBBBBBBBBB",
        "Candidate",
        json!(["run2", "run3"]),
        vec![(
            None,
            json!([
                {"simulation_id": "sim2", "run_id": "run2"},
                {"simulation_id": "sim3", "run_id": "run3"}
            ]),
            None,
        )],
    )
    .await;

    Mock::given(method("POST"))
        .and(path("/v1/reports"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "name": "Q3 Scorecard",
            "run_ids": ["run1", "run2", "run3"],
            "compare_by": "custom",
            "view_mode": "grouped",
            "custom_dimension_id": "merged-reports",
            "custom_dimensions": [{
                "id": "merged-reports",
                "name": "Report",
                "hide_unassigned": false,
                "groups": [
                    {
                        "id": "01HAAAAAAAAAAAAAAAAAAAAAAA",
                        "name": "Baseline",
                        "simulation_ids": ["sim1", "sim2"]
                    },
                    {
                        "id": "01HBBBBBBBBBBBBBBBBBBBBBBB",
                        "name": "Candidate",
                        "simulation_ids": ["sim3"]
                    }
                ]
            }]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "report": {
                "id": "01HMERGEDMERGEDMERGEDMERGE",
                "name": "Q3 Scorecard",
                "run_ids": ["run1", "run2", "run3"],
                "compare_by": "custom",
                "custom_dimension_id": "merged-reports",
                "permissions": "PRIVATE"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("reports")
        .arg("merge")
        .arg("--name")
        .arg("Q3 Scorecard")
        .arg("--report-ids")
        .arg("01HAAAAAAAAAAAAAAAAAAAAAAA,01HBBBBBBBBBBBBBBBBBBBBBBB")
        .assert()
        .success()
        .stdout(predicate::str::contains("01HMERGEDMERGEDMERGEDMERGE"));
}

#[test]
fn test_reports_merge_requires_two_reports() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("reports")
        .arg("merge")
        .arg("--name")
        .arg("Q3 Scorecard")
        .arg("--report-ids")
        .arg("01HAAAAAAAAAAAAAAAAAAAAAAA")
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires at least two report IDs"));
}

#[test]
fn test_reports_merge_rejects_duplicate_report_ids() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("reports")
        .arg("merge")
        .arg("--name")
        .arg("Q3 Scorecard")
        .arg("--report-ids")
        .arg("01HAAAAAAAAAAAAAAAAAAAAAAA,01HAAAAAAAAAAAAAAAAAAAAAAA")
        .assert()
        .failure()
        .stderr(predicate::str::contains("ids must be distinct"));
}

#[test]
fn test_reports_create_custom_requires_custom_dimensions() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("reports")
        .arg("create")
        .arg("--name")
        .arg("Bad Report")
        .arg("--run-ids")
        .arg("run1")
        .arg("--compare-by")
        .arg("custom")
        .assert()
        .failure()
        .stderr(predicate::str::contains("custom_dimensions is required"));
}

#[test]
fn test_reports_create_custom_dimensions_rejected_without_custom_compare_by() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("reports")
        .arg("create")
        .arg("--name")
        .arg("Bad Report")
        .arg("--run-ids")
        .arg("run1")
        .arg("--compare-by")
        .arg("agent")
        .arg("--input-json")
        .arg(r#"{"custom_dimensions": [{"id": "d1", "name": "Report", "groups": [], "hide_unassigned": false}]}"#)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "custom_dimensions can only be set when --compare-by is custom",
        ));
}

#[test]
fn test_reports_create_custom_rejects_null_custom_dimensions() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("reports")
        .arg("create")
        .arg("--name")
        .arg("Bad Report")
        .arg("--run-ids")
        .arg("run1")
        .arg("--compare-by")
        .arg("custom")
        .arg("--input-json")
        .arg(r#"{"custom_dimensions": null}"#)
        .assert()
        .failure()
        .stderr(predicate::str::contains("custom_dimensions is required"));
}

#[test]
fn test_reports_create_allows_null_custom_dimensions_without_custom_compare_by() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("reports")
        .arg("create")
        .arg("--name")
        .arg("Report")
        .arg("--run-ids")
        .arg("run1")
        .arg("--compare-by")
        .arg("agent")
        .arg("--input-json")
        .arg(r#"{"custom_dimensions": null}"#)
        .assert()
        .failure()
        .stderr(predicate::str::contains("custom_dimensions can only be set").not());
}

/// Mount a merge source whose rows page out to `total` distinct simulations, 500 per page.
async fn mount_merge_source_with_simulations(
    mock_server: &MockServer,
    report_id: &str,
    name: &str,
    total: usize,
) {
    Mock::given(method("GET"))
        .and(path(format!("/v1/reports/{report_id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "report": {
                "id": report_id,
                "name": name,
                "run_ids": ["run1"],
                "compare_by": "none",
                "permissions": "PRIVATE"
            }
        })))
        .mount(mock_server)
        .await;

    let mut emitted = 0usize;
    while emitted < total {
        let count = (total - emitted).min(500);
        let rows: Vec<Value> = (emitted..emitted + count)
            .map(|index| json!({"simulation_id": format!("{report_id}-sim{index}"), "run_id": "run1"}))
            .collect();
        let mock = Mock::given(method("GET"))
            .and(path(format!("/v1/reports/{report_id}/rows")))
            .and(query_param("limit", "500"));
        let mock = if emitted == 0 {
            mock.and(QueryParamAbsent("cursor"))
        } else {
            mock.and(query_param("cursor", emitted.to_string()))
        };
        emitted += count;
        let next_page_token = (emitted < total).then(|| emitted.to_string());
        mock.respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "rows": rows,
            "next_page_token": next_page_token
        })))
        .mount(mock_server)
        .await;
    }
}

#[tokio::test]
async fn test_reports_merge_rejects_a_source_over_the_simulation_cap() {
    let mock_server = MockServer::start().await;

    // 10,001 crosses the API's 10,000-per-group ceiling on the final page. No POST is
    // mounted, so the asserted message is what distinguishes the guard from a stray 404.
    mount_merge_source_with_simulations(
        &mock_server,
        "01HAAAAAAAAAAAAAAAAAAAAAAA",
        "Baseline",
        10_001,
    )
    .await;
    mount_merge_source_with_simulations(&mock_server, "01HBBBBBBBBBBBBBBBBBBBBBBB", "Candidate", 1)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("reports")
        .arg("merge")
        .arg("--name")
        .arg("Too Big")
        .arg("--report-ids")
        .arg("01HAAAAAAAAAAAAAAAAAAAAAAA,01HBBBBBBBBBBBBBBBBBBBBBBB")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "contributes more than 10000 simulations",
        ));
}

#[tokio::test]
async fn test_reports_merge_allows_a_source_at_the_simulation_cap() {
    let mock_server = MockServer::start().await;

    mount_merge_source_with_simulations(
        &mock_server,
        "01HAAAAAAAAAAAAAAAAAAAAAAA",
        "Baseline",
        10_000,
    )
    .await;
    mount_merge_source_with_simulations(&mock_server, "01HBBBBBBBBBBBBBBBBBBBBBBB", "Candidate", 1)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/reports"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "report": {
                "id": "01HMERGEDMERGEDMERGEDMERGE",
                "name": "At The Cap",
                "run_ids": ["run1"],
                "compare_by": "custom",
                "custom_dimension_id": "merged-reports",
                "permissions": "PRIVATE"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("reports")
        .arg("merge")
        .arg("--name")
        .arg("At The Cap")
        .arg("--report-ids")
        .arg("01HAAAAAAAAAAAAAAAAAAAAAAA,01HBBBBBBBBBBBBBBBBBBBBBBB")
        .assert()
        .success()
        .stdout(predicate::str::contains("01HMERGEDMERGEDMERGEDMERGE"));
}

#[tokio::test]
async fn test_reports_merge_rejects_sources_over_the_run_cap() {
    let mock_server = MockServer::start().await;

    // 2,001 distinct runs across two sources crosses the create request's run_ids ceiling.
    for (report_id, name, range) in [
        ("01HAAAAAAAAAAAAAAAAAAAAAAA", "Baseline", 0..2_000),
        ("01HBBBBBBBBBBBBBBBBBBBBBBB", "Candidate", 2_000..2_001),
    ] {
        let run_ids: Vec<Value> = range.map(|index| json!(format!("run{index}"))).collect();
        Mock::given(method("GET"))
            .and(path(format!("/v1/reports/{report_id}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "report": {
                    "id": report_id,
                    "name": name,
                    "run_ids": run_ids,
                    "compare_by": "none",
                    "permissions": "PRIVATE"
                }
            })))
            .mount(&mock_server)
            .await;
        Mock::given(method("GET"))
            .and(path(format!("/v1/reports/{report_id}/rows")))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "rows": [{"simulation_id": format!("{report_id}-sim"), "run_id": "run0"}],
                "next_page_token": null
            })))
            .mount(&mock_server)
            .await;
    }

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("reports")
        .arg("merge")
        .arg("--name")
        .arg("Too Many Runs")
        .arg("--report-ids")
        .arg("01HAAAAAAAAAAAAAAAAAAAAAAA,01HBBBBBBBBBBBBBBBBBBBBBBB")
        .assert()
        .failure()
        .stderr(predicate::str::contains("span more than 2000 runs"));
}

#[test]
fn test_reports_merge_rejects_more_reports_than_the_group_cap() {
    let report_ids: Vec<String> = (0..501).map(|index| format!("01H{index:023}")).collect();

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("reports")
        .arg("merge")
        .arg("--name")
        .arg("Too Many")
        .arg("--report-ids")
        .arg(report_ids.join(","))
        .assert()
        .failure()
        .stderr(predicate::str::contains("at most 500 groups"));
}

#[tokio::test]
async fn test_reports_create_custom_dimension_defaults_hide_unassigned() {
    let mock_server = MockServer::start().await;

    // hide_unassigned is omitted from the input and custom_dimension_id names d1, so this
    // covers both the serde default and the membership check's accepting path.
    Mock::given(method("POST"))
        .and(path("/v1/reports"))
        .and(body_partial_json(json!({
            "compare_by": "custom",
            "custom_dimension_id": "d1",
            "custom_dimensions": [{
                "id": "d1",
                "name": "Report",
                "hide_unassigned": false,
                "groups": [{"id": "g1", "name": "A", "simulation_ids": ["sim1"]}]
            }]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "report": {
                "id": "01HCUSTOMCUSTOMCUSTOMCUSTO",
                "name": "Custom Report",
                "run_ids": ["run1"],
                "compare_by": "custom",
                "custom_dimension_id": "d1",
                "permissions": "PRIVATE"
            }
        })))
        .mount(&mock_server)
        .await;

    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("--api-url")
        .arg(mock_server.uri())
        .arg("reports")
        .arg("create")
        .arg("--name")
        .arg("Custom Report")
        .arg("--run-ids")
        .arg("run1")
        .arg("--compare-by")
        .arg("custom")
        .arg("--input-json")
        .arg(
            r#"{"custom_dimension_id": "d1", "custom_dimensions": [{"id": "d1", "name": "Report", "groups": [{"id": "g1", "name": "A", "simulation_ids": ["sim1"]}]}]}"#,
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("01HCUSTOMCUSTOMCUSTOMCUSTO"));
}

#[test]
fn test_reports_create_custom_dimension_id_rejected_without_custom_compare_by() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("reports")
        .arg("create")
        .arg("--name")
        .arg("Bad Report")
        .arg("--run-ids")
        .arg("run1")
        .arg("--compare-by")
        .arg("agent")
        .arg("--input-json")
        .arg(r#"{"custom_dimension_id": "d1"}"#)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "custom_dimension_id can only be set when --compare-by is custom",
        ));
}

#[test]
fn test_reports_create_allows_null_custom_dimension_id_without_custom_compare_by() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("reports")
        .arg("create")
        .arg("--name")
        .arg("Report")
        .arg("--run-ids")
        .arg("run1")
        .arg("--compare-by")
        .arg("agent")
        .arg("--input-json")
        .arg(r#"{"custom_dimension_id": null}"#)
        .assert()
        .failure()
        .stderr(predicate::str::contains("custom_dimension_id can only be set").not());
}

#[test]
fn test_reports_create_reports_a_malformed_dimension_as_a_type_error() {
    // The dimension has no id, so the membership check defers and serde names the real fault
    // rather than blaming custom_dimension_id.
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("reports")
        .arg("create")
        .arg("--name")
        .arg("Bad Report")
        .arg("--run-ids")
        .arg("run1")
        .arg("--compare-by")
        .arg("custom")
        .arg("--input-json")
        .arg(
            r#"{"custom_dimension_id": "d1", "custom_dimensions": [{"name": "Report", "groups": []}]}"#,
        )
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("missing field `id`")
                .and(predicate::str::contains("does not match the id").not()),
        );
}

#[test]
fn test_reports_create_custom_dimension_id_must_name_a_supplied_dimension() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("reports")
        .arg("create")
        .arg("--name")
        .arg("Bad Report")
        .arg("--run-ids")
        .arg("run1")
        .arg("--compare-by")
        .arg("custom")
        .arg("--input-json")
        .arg(
            r#"{"custom_dimension_id": "missing", "custom_dimensions": [{"id": "d1", "name": "Report", "groups": [], "hide_unassigned": false}]}"#,
        )
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "custom_dimension_id missing does not match the id of any supplied custom dimension",
        ));
}

#[tokio::test]
async fn test_traces_search_sends_structured_filters_and_preserves_cursor() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/traces/search"))
        .and(header("X-API-Key", "test_key"))
        .and(body_partial_json(json!({
            "limit": 10,
            "filters": {
                "span_name": "llm",
                "status": "ERROR",
                "attribute_filters": [
                    {"key": "tool.error", "operator": "eq", "value": "1"}
                ],
                "sort_by": "slowest"
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [{
                "simulation_output_id": "sim-output-1",
                "run_id": "run-1",
                "latest_matched_timestamp_ms": 1785430800000_i64,
                "first_matched_timestamp_ms": 1785430799000_i64,
                "matched_span_count": 2,
                "total_span_count": 8,
                "error_span_count": 1,
                "ok_span_count": 1,
                "unset_span_count": 0,
                "overall_status": "ERROR",
                "matched_span_names": ["llm"],
                "matched_provider_names": ["openai"],
                "matched_service_names": ["voice-agent"],
                "matched_scope_names": ["agent"]
            }],
            "total_count": 42,
            "next_cursor": "1785430800000::sim-output-1",
            "aggregate_stats": {"error_count": 7, "error_rate": 17}
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--agent")
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("traces")
            .arg("search")
            .arg("--limit")
            .arg("10")
            .arg("--span-name")
            .arg("llm")
            .arg("--status")
            .arg("error")
            .arg("--attribute-filter")
            .arg("tool.error:eq:1")
            .arg("--sort-by")
            .arg("slowest")
            .assert()
            .success()
            .stderr(predicate::str::is_empty()),
    );

    assert_eq!(value["resource"], "traces");
    assert_eq!(value["operation"], "search");
    assert_eq!(value["data"]["total_count"], 42);
    assert_eq!(value["data"]["next_cursor"], "1785430800000::sim-output-1");
    assert_eq!(value["next_actions"][0]["id"], "traces.spans");
    assert_eq!(
        value["next_actions"][0]["argv"],
        json!(["coval", "--agent", "traces", "spans", "sim-output-1"])
    );
}

#[tokio::test]
async fn test_traces_search_merges_stdin_json_with_flag_overrides() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/traces/search"))
        .and(body_partial_json(json!({
            "limit": 10,
            "filters": {
                "provider": "openai",
                "status": "ERROR",
                "duration_ms_min": 250.0
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [],
            "total_count": 0,
            "next_cursor": null,
            "aggregate_stats": {"error_count": 0, "error_rate": 0}
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("--format")
            .arg("json")
            .arg("traces")
            .arg("search")
            .arg("--input-json")
            .arg("-")
            .arg("--limit")
            .arg("10")
            .arg("--status")
            .arg("error")
            .write_stdin(
                r#"{"limit":99,"filters":{"provider":"openai","status":"OK","duration_ms_min":250}}"#,
            )
            .assert()
            .success(),
    );

    assert_eq!(value["items"], json!([]));
    assert_eq!(value["total_count"], 0);
    assert!(value["next_cursor"].is_null());
}

#[tokio::test]
async fn test_traces_summary_uses_exactly_one_target() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/traces/summary"))
        .and(query_param("conversation_id", "conversation-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "target": {"type": "conversation", "id": "conversation-1"},
            "trace_summary": {
                "simulation_output_id": "sim-output-1",
                "total_spans": 12,
                "status_counts": {"ERROR": 1, "OK": 11}
            }
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("--format")
            .arg("json")
            .arg("traces")
            .arg("summary")
            .arg("--conversation-id")
            .arg("conversation-1")
            .assert()
            .success(),
    );

    assert_eq!(value["target"]["type"], "conversation");
    assert_eq!(value["trace_summary"]["total_spans"], 12);
}

#[test]
fn test_traces_summary_rejects_missing_or_ambiguous_target() {
    coval()
        .arg("traces")
        .arg("summary")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--simulation-id <SIMULATION_ID>"));

    coval()
        .arg("traces")
        .arg("summary")
        .arg("--simulation-id")
        .arg("sim-1")
        .arg("--conversation-id")
        .arg("conversation-1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[tokio::test]
async fn test_traces_spans_preserves_raw_span_payload() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/traces/spans"))
        .and(query_param("simulation_output_id", "sim-output-1"))
        .and(query_param("limit", "100"))
        .and(query_param("offset", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "traces": [{
                "trace_id": "trace-1",
                "span_name": "llm",
                "span_attributes": {"gen_ai.request.model": "gpt-4.1"}
            }],
            "total_spans": 9
        })))
        .mount(&mock_server)
        .await;

    let value = stdout_json(
        coval()
            .arg("--api-key")
            .arg("test_key")
            .arg("--api-url")
            .arg(mock_server.uri())
            .arg("--format")
            .arg("json")
            .arg("traces")
            .arg("spans")
            .arg("sim-output-1")
            .arg("--limit")
            .arg("100")
            .arg("--offset")
            .arg("5")
            .assert()
            .success(),
    );

    assert_eq!(value["total_spans"], 9);
    assert_eq!(
        value["traces"][0]["span_attributes"]["gen_ai.request.model"],
        "gpt-4.1"
    );
}

#[test]
fn test_traces_search_rejects_ambiguous_attribute_filter() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("traces")
        .arg("search")
        .arg("--attribute-filter")
        .arg("tool.error:eq")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the eq attribute operator requires a value",
        ));
}

#[test]
fn test_traces_search_rejects_inverted_duration_range() {
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("traces")
        .arg("search")
        .arg("--duration-ms-min")
        .arg("10")
        .arg("--duration-ms-max")
        .arg("5")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "trace search duration minimum (10) must not exceed maximum (5)",
        ));
}

#[test]
fn test_traces_search_rejects_more_than_ten_attribute_filter_flags() {
    let mut command = coval();
    command
        .arg("--api-key")
        .arg("test_key")
        .arg("traces")
        .arg("search");
    for index in 0..11 {
        command
            .arg("--attribute-filter")
            .arg(format!("attribute.{index}:exists"));
    }
    command.assert().failure().stderr(predicate::str::contains(
        "trace search accepts at most 10 attribute filters, got 11",
    ));
}

#[test]
fn test_traces_search_rejects_more_than_ten_json_attribute_filters() {
    let attribute_filters = (0..11)
        .map(|index| json!({"key": format!("attribute.{index}"), "operator": "exists"}))
        .collect::<Vec<_>>();
    coval()
        .arg("--api-key")
        .arg("test_key")
        .arg("traces")
        .arg("search")
        .arg("--input-json")
        .arg(json!({"filters": {"attribute_filters": attribute_filters}}).to_string())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "trace search accepts at most 10 attribute filters, got 11",
        ));
}

#[tokio::test]
async fn test_uploaded_conversation_commands_use_canonical_routes() {
    let mock_server = MockServer::start().await;
    let conversation = json!({
        "name": "conversations/uploaded123",
        "conversation_id": "uploaded123",
        "status": "COMPLETED",
        "create_time": "2026-09-01T12:00:00Z",
        "has_audio": true,
        "external_conversation_id": "customer-call-123"
    });

    Mock::given(method("GET"))
        .and(path("/v1/conversations/uploaded"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "uploaded_conversations": [conversation.clone()],
            "next_page_token": null
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/conversations/uploaded/uploaded123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "uploaded_conversation": conversation.clone()
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/conversations/uploaded:submit"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "uploaded_conversation": conversation.clone()
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v1/conversations/uploaded/uploaded123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "uploaded_conversation": conversation.clone()
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v1/conversations/uploaded/uploaded123"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/conversations/uploaded/uploaded123/audio"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "audio_url": "https://storage.example.com/uploaded.wav",
            "conversation_id": "uploaded123",
            "url_expires_in_seconds": 3600
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/conversations/uploaded/uploaded123/metrics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metrics": [{"metric_output_id": "output123", "metric_id": "metric123"}],
            "next_page_token": null
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/conversations/uploaded/uploaded123/metrics/output123",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metric": {"metric_output_id": "output123", "metric_id": "metric123"}
        })))
        .mount(&mock_server)
        .await;

    let listed = stdout_json(
        coval_with_api(&mock_server)
            .args(["--format", "json", "uploaded-conversations", "list"])
            .assert()
            .success(),
    );
    assert_eq!(listed[0]["conversation_id"], "uploaded123");
    assert!(listed.get("uploaded_conversations").is_none());

    let fetched = stdout_json(
        coval_with_api(&mock_server)
            .args([
                "--format",
                "json",
                "uploaded-conversations",
                "get",
                "uploaded123",
            ])
            .assert()
            .success(),
    );
    assert_eq!(fetched["external_conversation_id"], "customer-call-123");

    coval_with_api(&mock_server)
        .args([
            "uploaded-conversations",
            "submit",
            "--input-json",
            r#"{"transcript": []}"#,
        ])
        .assert()
        .success();
    coval_with_api(&mock_server)
        .args([
            "uploaded-conversations",
            "patch",
            "uploaded123",
            "--audio-url",
            "https://storage.example.com/new.wav",
        ])
        .assert()
        .success();
    coval_with_api(&mock_server)
        .args(["uploaded-conversations", "audio", "uploaded123"])
        .assert()
        .success()
        .stdout(predicate::str::contains("uploaded.wav"));
    coval_with_api(&mock_server)
        .args(["uploaded-conversations", "metrics", "uploaded123"])
        .assert()
        .success()
        .stdout(predicate::str::contains("output123"));
    coval_with_api(&mock_server)
        .args([
            "uploaded-conversations",
            "metric-detail",
            "uploaded123",
            "output123",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("metric123"));
    coval_with_api(&mock_server)
        .args(["uploaded-conversations", "delete", "uploaded123"])
        .assert()
        .success();
}

#[tokio::test]
async fn test_simulated_conversation_commands_use_canonical_routes() {
    let mock_server = MockServer::start().await;
    let simulation = json!({
        "name": "simulated-conversations/simulated123",
        "simulation_id": "simulated123",
        "run_id": "run123",
        "status": "COMPLETED",
        "create_time": "2026-09-01T12:00:00Z",
        "has_audio": true
    });

    Mock::given(method("GET"))
        .and(path("/v1/conversations/simulated"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "simulated_conversations": [simulation.clone()],
            "next_page_token": null
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/conversations/simulated/simulated123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "simulated_conversation": simulation.clone()
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v1/conversations/simulated/simulated123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "simulated_conversation": simulation.clone()
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/conversations/simulated/simulated123/resimulate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "simulated_conversation": simulation.clone()
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v1/conversations/simulated/simulated123"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/conversations/simulated/simulated123/audio"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "audio_url": "https://storage.example.com/simulated.wav",
            "simulation_id": "simulated123",
            "url_expires_in_seconds": 3600
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/conversations/simulated/simulated123/metrics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metrics": [{"metric_output_id": "output123", "metric_id": "metric123"}]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/conversations/simulated/simulated123/metrics/output123",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metric": {"metric_output_id": "output123", "metric_id": "metric123"}
        })))
        .mount(&mock_server)
        .await;

    let listed = stdout_json(
        coval_with_api(&mock_server)
            .args([
                "--format",
                "json",
                "simulated-conversations",
                "list",
                "--run-id",
                "run123",
            ])
            .assert()
            .success(),
    );
    assert_eq!(listed[0]["simulation_id"], "simulated123");
    assert!(listed.get("simulated_conversations").is_none());

    let fetched = stdout_json(
        coval_with_api(&mock_server)
            .args([
                "--format",
                "json",
                "simulated-conversations",
                "get",
                "simulated123",
            ])
            .assert()
            .success(),
    );
    assert_eq!(fetched["run_id"], "run123");

    coval_with_api(&mock_server)
        .args([
            "simulated-conversations",
            "update",
            "simulated123",
            "--notes",
            "reviewed",
        ])
        .assert()
        .success();
    coval_with_api(&mock_server)
        .args(["simulated-conversations", "resimulate", "simulated123"])
        .assert()
        .success();
    coval_with_api(&mock_server)
        .args(["simulated-conversations", "audio", "simulated123"])
        .assert()
        .success()
        .stdout(predicate::str::contains("simulated.wav"));
    coval_with_api(&mock_server)
        .args(["simulated-conversations", "metrics", "simulated123"])
        .assert()
        .success()
        .stdout(predicate::str::contains("output123"));
    coval_with_api(&mock_server)
        .args([
            "simulated-conversations",
            "metric-detail",
            "simulated123",
            "output123",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("metric123"));
    coval_with_api(&mock_server)
        .args(["simulated-conversations", "delete", "simulated123"])
        .assert()
        .success();
}
