use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn coval() -> Command {
    #[allow(deprecated)]
    Command::cargo_bin("coval").unwrap()
}

#[test]
fn test_help() {
    coval()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Coval AI evaluation CLI"));
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
