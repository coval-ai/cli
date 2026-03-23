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
                    "status": "COMPLETED",
                    "value": 0.95,
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
                    ]
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
