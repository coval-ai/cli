"""Tests for the live API-coverage audit."""

import datetime
import io
import tempfile
import tomllib
import unittest
from contextlib import redirect_stderr
from contextlib import redirect_stdout
from pathlib import Path
from urllib.error import HTTPError
from urllib.error import URLError
from unittest.mock import Mock
from unittest.mock import patch

from scripts import audit_api_coverage


class FetchSafetyTests(unittest.TestCase):
    def test_rejects_non_https_url(self):
        with self.assertRaisesRegex(ValueError, "must use HTTPS"):
            audit_api_coverage._validate_fetch_url(
                "file:///etc/passwd",
                audit_api_coverage.DEFAULT_ALLOWED_ORIGINS,
            )

    def test_rejects_disallowed_origin(self):
        with self.assertRaisesRegex(ValueError, "is not allowed"):
            audit_api_coverage._validate_fetch_url(
                "https://example.com/openapi",
                audit_api_coverage.DEFAULT_ALLOWED_ORIGINS,
            )

    def test_rejects_cross_origin_redirect(self):
        handler = audit_api_coverage._AllowedOriginRedirectHandler(
            audit_api_coverage.DEFAULT_ALLOWED_ORIGINS
        )
        with self.assertRaisesRegex(ValueError, "is not allowed"):
            handler.redirect_request(
                Mock(),
                Mock(),
                302,
                "Found",
                {},
                "https://example.com/spec",
            )

    @patch("scripts.audit_api_coverage.build_opener")
    def test_validates_final_response_url(self, build_opener):
        response = Mock()
        response.__enter__ = Mock(return_value=response)
        response.__exit__ = Mock(return_value=False)
        response.geturl.return_value = "https://example.com/redirected"
        build_opener.return_value.open.return_value = response

        with self.assertRaisesRegex(ValueError, "is not allowed"):
            audit_api_coverage._fetch(
                audit_api_coverage.CATALOG_URL,
                audit_api_coverage.DEFAULT_ALLOWED_ORIGINS,
            )

    @patch("scripts.audit_api_coverage.time.sleep")
    @patch("scripts.audit_api_coverage.build_opener")
    def test_retries_transient_network_failure(self, build_opener, sleep):
        response = Mock()
        response.__enter__ = Mock(return_value=response)
        response.__exit__ = Mock(return_value=False)
        response.geturl.return_value = audit_api_coverage.CATALOG_URL
        response.read.return_value = b"ok"
        build_opener.return_value.open.side_effect = [
            URLError("temporary"),
            response,
        ]

        result = audit_api_coverage._fetch(
            audit_api_coverage.CATALOG_URL,
            audit_api_coverage.DEFAULT_ALLOWED_ORIGINS,
        )

        self.assertEqual(b"ok", result)
        sleep.assert_called_once_with(1)

    @patch("scripts.audit_api_coverage.time.sleep")
    @patch("scripts.audit_api_coverage.build_opener")
    def test_retries_server_http_error(self, build_opener, sleep):
        server_error = HTTPError(
            audit_api_coverage.CATALOG_URL,
            503,
            "Service Unavailable",
            {},
            None,
        )
        response = Mock()
        response.__enter__ = Mock(return_value=response)
        response.__exit__ = Mock(return_value=False)
        response.geturl.return_value = audit_api_coverage.CATALOG_URL
        response.read.return_value = b"ok"
        build_opener.return_value.open.side_effect = [
            server_error,
            response,
        ]

        try:
            result = audit_api_coverage._fetch(
                audit_api_coverage.CATALOG_URL,
                audit_api_coverage.DEFAULT_ALLOWED_ORIGINS,
            )
        finally:
            server_error.close()

        self.assertEqual(b"ok", result)
        sleep.assert_called_once_with(1)

    @patch("scripts.audit_api_coverage.time.sleep")
    @patch("scripts.audit_api_coverage.build_opener")
    def test_does_not_retry_client_http_error(self, build_opener, sleep):
        client_error = HTTPError(
            audit_api_coverage.CATALOG_URL,
            404,
            "Not Found",
            {},
            None,
        )
        build_opener.return_value.open.side_effect = client_error

        try:
            with self.assertRaises(HTTPError):
                audit_api_coverage._fetch(
                    audit_api_coverage.CATALOG_URL,
                    audit_api_coverage.DEFAULT_ALLOWED_ORIGINS,
                )
        finally:
            client_error.close()

        build_opener.return_value.open.assert_called_once()
        sleep.assert_not_called()


class PublishedOperationsTests(unittest.TestCase):
    @staticmethod
    def _specs(*documents: bytes) -> list[dict]:
        with patch("scripts.audit_api_coverage._fetch") as fetch:
            fetch.side_effect = [
                (
                    b'{"specs":['
                    + b",".join(
                        b'{"url":"https://api.coval.dev/%d"}' % index
                        for index in range(len(documents))
                    )
                    + b"]}"
                ),
                *documents,
            ]
            return audit_api_coverage._published_specs(
                audit_api_coverage.CATALOG_URL,
                audit_api_coverage.DEFAULT_ALLOWED_ORIGINS,
            )

    def test_rejects_distinct_operations_with_same_canonical_key(self):
        specs = self._specs(
            b"paths:\n  /v1/agents/{agent_id}:\n    get: {}\n",
            b"paths:\n  /v1/agents/{id}:\n    get: {}\n",
        )

        with self.assertRaisesRegex(RuntimeError, "share canonical key"):
            audit_api_coverage._published_operations(specs)

    def test_deduplicates_identical_operation_across_specs(self):
        specs = self._specs(
            b"paths:\n  /v1/agents/{agent_id}:\n    get: {}\n",
            b"paths:\n  /v1/agents/{agent_id}:\n    get: {}\n",
        )

        self.assertEqual(
            {"GET /agents/{id}": "GET /v1/agents/{agent_id}"},
            audit_api_coverage._published_operations(specs),
        )

    def test_fetches_the_catalog_and_each_spec_once(self):
        with patch("scripts.audit_api_coverage._fetch") as fetch:
            fetch.side_effect = [
                b'{"specs":[{"url":"https://api.coval.dev/a"}]}',
                b"paths: {}\n",
            ]
            audit_api_coverage._published_specs(
                audit_api_coverage.CATALOG_URL,
                audit_api_coverage.DEFAULT_ALLOWED_ORIGINS,
            )

        self.assertEqual(2, fetch.call_count)


class PublishedRequestFieldTests(unittest.TestCase):
    SCHEMAS = {
        "CreateThing": {
            "type": "object",
            "properties": {"display_name": {}, "tags": {}},
        },
        "Composed": {
            "allOf": [
                {"$ref": "#/components/schemas/CreateThing"},
                {"type": "object", "properties": {"extra": {}}},
            ]
        },
        "SelfReferential": {
            "type": "object",
            "properties": {"child": {}},
            "anyOf": [{"$ref": "#/components/schemas/SelfReferential"}],
        },
    }

    def test_resolves_a_reference_to_its_properties(self):
        self.assertEqual(
            {"display_name", "tags"},
            audit_api_coverage._schema_properties(
                {"$ref": "#/components/schemas/CreateThing"}, self.SCHEMAS
            ),
        )

    def test_unions_every_branch_of_a_composed_schema(self):
        self.assertEqual(
            {"display_name", "tags", "extra"},
            audit_api_coverage._schema_properties(
                {"$ref": "#/components/schemas/Composed"}, self.SCHEMAS
            ),
        )

    def test_tolerates_a_self_referential_schema(self):
        self.assertEqual(
            {"child"},
            audit_api_coverage._schema_properties(
                {"$ref": "#/components/schemas/SelfReferential"}, self.SCHEMAS
            ),
        )

    def test_rejects_an_unresolvable_reference(self):
        with self.assertRaisesRegex(RuntimeError, "unknown schema"):
            audit_api_coverage._schema_properties(
                {"$ref": "#/components/schemas/Missing"}, self.SCHEMAS
            )

    def test_collects_only_json_request_bodies_on_body_methods(self):
        spec = {
            "components": {"schemas": self.SCHEMAS},
            "paths": {
                "/things": {
                    "get": {"responses": {}},
                    "post": {
                        "requestBody": {
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/CreateThing"
                                    }
                                }
                            }
                        }
                    },
                },
                "/things/{thing_id}/archive": {
                    "post": {"responses": {}},
                },
                "/traces": {
                    "post": {
                        "requestBody": {
                            "content": {"application/x-protobuf": {"schema": {}}}
                        }
                    }
                },
            },
        }

        self.assertEqual(
            {"POST /things": {"display_name", "tags"}},
            audit_api_coverage._published_request_fields([spec]),
        )

    def test_merges_the_same_operation_across_specs(self):
        def spec(field):
            return {
                "components": {"schemas": {}},
                "paths": {
                    "/things": {
                        "post": {
                            "requestBody": {
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "type": "object",
                                            "properties": {field: {}},
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            }

        self.assertEqual(
            {"POST /things": {"first", "second"}},
            audit_api_coverage._published_request_fields(
                [spec("first"), spec("second")]
            ),
        )


class CommandCoverageTests(unittest.TestCase):
    def test_only_counts_client_operations_referenced_by_commands(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            client_path = root / "client.rs"
            commands_path = root / "commands"
            commands_path.mkdir()
            client_path.write_text(
                """
impl AgentsClient<'_> {
    pub async fn list(&self) {
        let url = self.0.url("/v1/agents");
        self.0.get(url).await
    }

    pub async fn get(&self, id: &str) {
        let url = self.0.url(&format!("/v1/agents/{id}"));
        self.0.get(url).await
    }
}
"""
            )
            (commands_path / "agents.rs").write_text(
                """
let agents = client
    .agents()
    .list()
    .await?;
let response = client.get(url).send().await?;
"""
            )

            with (
                patch.object(audit_api_coverage, "CLIENT_PATH", client_path),
                patch.object(audit_api_coverage, "COMMANDS_PATH", commands_path),
            ):
                client_operations = audit_api_coverage._client_operations()
                command_operations, unmapped = audit_api_coverage._command_operations(
                    client_operations
                )

            self.assertEqual(2, len(client_operations))
            self.assertEqual(
                ["GET /agents"],
                [operation["operation"] for operation in command_operations.values()],
            )
            self.assertEqual([], unmapped)

    def test_any_exposed_client_method_covers_a_shared_operation(self):
        client_operations = {
            "GET /conversations": {
                "operation": "GET /conversations",
                "client_methods": {
                    "conversations.list",
                    "conversations.list_with_metric_outputs",
                },
            }
        }
        with patch.object(
            audit_api_coverage,
            "_command_client_methods",
            return_value={"conversations.list": {"conversations.rs"}},
        ):
            command_operations, unmapped = audit_api_coverage._command_operations(
                client_operations
            )

        self.assertEqual(client_operations, command_operations)
        self.assertEqual([], unmapped)


class RequestBodyTypeTests(unittest.TestCase):
    def test_reads_the_type_from_the_function_signature(self):
        block = """
    pub async fn create(&self, req: models::CreateAgentRequest) -> Result<T, E> {
        let url = self.0.url("/v1/agents");
        self.0.post(url, &req).await
    }
"""
        self.assertEqual(
            {"CreateAgentRequest"},
            audit_api_coverage._request_body_types("agents.create", block),
        )

    def test_reads_the_type_from_a_local_binding(self):
        block = """
    pub async fn test_evaluate(&self, run_id: &str) -> Result<T, E> {
        let url = self.0.url("/v1/monitors/x/test-evaluate");
        let req = models::TestEvaluateRequest {
            run_id: run_id.to_string(),
        };
        self.0.post(url, &req).await
    }
"""
        self.assertEqual(
            {"TestEvaluateRequest"},
            audit_api_coverage._request_body_types("monitors.test_evaluate", block),
        )

    def test_treats_a_json_value_body_as_passthrough(self):
        block = """
    pub async fn create_baseline(&self, req: serde_json::Value) -> Result<T, E> {
        let url = self.0.url("/v1/metrics/x/baselines");
        self.0.post(url, &req).await
    }
"""
        self.assertEqual(
            {audit_api_coverage.PASSTHROUGH_BODY},
            audit_api_coverage._request_body_types("metrics.create_baseline", block),
        )

    def test_ignores_a_call_that_sends_no_body(self):
        block = """
    pub async fn duplicate(&self, id: &str) -> Result<T, E> {
        let url = self.0.url("/v1/metrics/x/duplicate");
        self.0.post_empty(url).await
    }
"""
        self.assertEqual(
            set(), audit_api_coverage._request_body_types("metrics.duplicate", block)
        )

    def test_rejects_an_unresolvable_body_binding(self):
        block = """
    pub async fn create(&self, req: SomeLocalType) -> Result<T, E> {
        let url = self.0.url("/v1/agents");
        self.0.post(url, &req).await
    }
"""
        with self.assertRaisesRegex(RuntimeError, "could not resolve"):
            audit_api_coverage._request_body_types("agents.create", block)


class RequestBodyAuditRegressionTests(unittest.TestCase):
    def test_rejects_an_unrecognized_json_body_call(self):
        block = """
    pub async fn create(&self, request: models::CreateThingRequest) -> Result<T, E> {
        self.0.post(self.0.url("/v1/things"), &request).await
    }
"""
        with self.assertRaisesRegex(
            RuntimeError, "could not resolve every request-body"
        ):
            audit_api_coverage._request_body_types("things.create", block)

    def test_audits_all_python_metric_draft_request_fields(self):
        operations, _ = audit_api_coverage._command_operations(
            audit_api_coverage._client_operations()
        )
        fields = audit_api_coverage._cli_request_fields(
            operations, audit_api_coverage._model_request_fields()
        )
        expected = {
            "POST /python-metric-drafts": {
                "display_name",
                "source",
                "output_type",
                "target_metric_id",
                "target_version_id",
                "description",
                "unit",
                "requirements",
            },
            "POST /python-metric-drafts/{id}/previews": {"simulation_output_ids"},
            "POST /python-metric-drafts/{id}/publish": {"preview_ids", "fingerprint"},
        }
        for operation, expected_fields in expected.items():
            with self.subTest(operation=operation):
                self.assertEqual(expected_fields, fields.get(operation))


class ModelRequestFieldTests(unittest.TestCase):
    def _fields(self, source: str) -> dict[str, set[str]]:
        with tempfile.TemporaryDirectory() as directory:
            models = Path(directory)
            (models / "agent.rs").write_text(source)
            with patch.object(audit_api_coverage, "MODELS_PATH", models):
                return audit_api_coverage._model_request_fields()

    def test_reads_plain_and_renamed_fields(self):
        fields = self._fields(
            """
#[derive(Debug, Serialize)]
pub struct CreateWidgetRequest {
    pub title: String,
    #[serde(rename = "grid_x", skip_serializing_if = "Option::is_none")]
    pub x: Option<u32>,
}
"""
        )

        self.assertEqual({"CreateWidgetRequest": {"title", "grid_x"}}, fields)

    def test_ignores_flattened_and_skipped_fields(self):
        fields = self._fields(
            """
pub struct Agent {
    pub id: String,
    #[serde(skip)]
    pub cached: bool,
    #[serde(skip_serializing)]
    pub read_only: bool,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
"""
        )

        self.assertEqual({"Agent": {"id"}}, fields)

    def test_reads_a_field_behind_a_multi_line_attribute(self):
        fields = self._fields(
            """
pub struct UpdateTestCaseRequest {
    #[serde(
        default,
        deserialize_with = "explicit_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub script_turns: Option<Option<Vec<serde_json::Value>>>,
}
"""
        )

        self.assertEqual({"UpdateTestCaseRequest": {"script_turns"}}, fields)


class CliRequestFieldTests(unittest.TestCase):
    MODELS = {
        "CreateThingRequest": {"display_name"},
        "SubmitThingRequest": {"display_name", "audio"},
    }

    def test_unions_every_struct_bound_to_one_operation(self):
        operations = {
            "POST /things": {
                "request_bodies": {"CreateThingRequest", "SubmitThingRequest"}
            }
        }

        self.assertEqual(
            {"POST /things": {"display_name", "audio"}},
            audit_api_coverage._cli_request_fields(operations, self.MODELS),
        )

    def test_marks_a_passthrough_operation_as_unbounded(self):
        operations = {
            "POST /things": {
                "request_bodies": {audit_api_coverage.PASSTHROUGH_BODY},
            }
        }

        self.assertEqual(
            {"POST /things": None},
            audit_api_coverage._cli_request_fields(operations, self.MODELS),
        )

    def test_skips_an_operation_with_no_request_body(self):
        operations = {"GET /things": {"request_bodies": set()}}

        self.assertEqual(
            {}, audit_api_coverage._cli_request_fields(operations, self.MODELS)
        )

    def test_rejects_a_body_struct_that_is_not_defined(self):
        operations = {"POST /things": {"request_bodies": {"MissingRequest"}}}

        with self.assertRaisesRegex(RuntimeError, "was not found"):
            audit_api_coverage._cli_request_fields(operations, self.MODELS)


class ManifestFieldTests(unittest.TestCase):
    def test_rejects_a_missing_field_name(self):
        with self.assertRaisesRegex(ValueError, "non-empty"):
            audit_api_coverage._manifest_field_entries(
                [{"operation": "POST /agents", "reason": "Reviewed"}],
                "known_field_gap",
            )

    def test_rejects_a_method_that_carries_no_request_body(self):
        with self.assertRaisesRegex(ValueError, "invalid operation"):
            audit_api_coverage._manifest_field_entries(
                [{"operation": "GET /agents", "field": "tags", "reason": "Reviewed"}],
                "known_field_gap",
            )

    def test_rejects_a_duplicate_entry(self):
        with self.assertRaisesRegex(ValueError, "duplicate entry"):
            audit_api_coverage._manifest_field_entries(
                [
                    {"operation": "POST /agents", "field": "tags", "reason": "First"},
                    {"operation": "POST /agents", "field": "tags", "reason": "Second"},
                ],
                "known_field_gap",
            )

    def test_canonicalizes_the_operation_path(self):
        entries = audit_api_coverage._manifest_field_entries(
            [
                {
                    "operation": "PATCH /agents/{agent_id}",
                    "field": "tags",
                    "reason": "Reviewed",
                }
            ],
            "known_field_gap",
        )

        self.assertEqual([("PATCH /agents/{id}", "tags")], list(entries))


class FieldReconciliationTests(unittest.TestCase):
    def test_classifies_gaps_extras_and_stale_exceptions(self):
        result = audit_api_coverage._field_reconciliation(
            published_fields={"POST /things": {"name", "tags"}},
            cli_fields={"POST /things": {"name", "legacy"}},
            known_field_gaps={("POST /things", "retired"): {}},
            allowed_extra_fields={("POST /things", "legacy"): {}},
        )

        self.assertEqual({("POST /things", "tags")}, result["new_gaps"])
        self.assertEqual({("POST /things", "retired")}, result["stale_gaps"])
        self.assertEqual(set(), result["unexpected_extras"])
        self.assertEqual(set(), result["stale_allowed_extras"])
        self.assertEqual(2, result["compared_field_count"])
        self.assertEqual(1, result["modeled_field_count"])

    def test_counts_a_passthrough_operation_as_fully_modeled(self):
        result = audit_api_coverage._field_reconciliation(
            published_fields={"POST /things": {"name", "tags"}},
            cli_fields={"POST /things": None},
            known_field_gaps={},
            allowed_extra_fields={},
        )

        self.assertEqual(set(), result["actual_gaps"])
        self.assertEqual(2, result["modeled_field_count"])

    def test_ignores_an_operation_the_cli_does_not_implement(self):
        result = audit_api_coverage._field_reconciliation(
            published_fields={"POST /things": {"name"}, "POST /others": {"name"}},
            cli_fields={"POST /things": {"name"}},
            known_field_gaps={},
            allowed_extra_fields={},
        )

        self.assertEqual(["POST /things"], result["compared_operations"])
        self.assertEqual(set(), result["actual_gaps"])


class ManifestTests(unittest.TestCase):
    def test_rejects_missing_reason(self):
        with self.assertRaisesRegex(ValueError, "non-empty"):
            audit_api_coverage._manifest_operations(
                [{"operation": "GET /agents"}],
                "known_gap",
            )

    def test_rejects_invalid_http_method(self):
        with self.assertRaisesRegex(ValueError, "invalid operation"):
            audit_api_coverage._manifest_operations(
                [{"operation": "HEAD /agents", "reason": "Not supported"}],
                "known_gap",
            )

    def test_rejects_duplicate_operation(self):
        with self.assertRaisesRegex(ValueError, "duplicate operation"):
            audit_api_coverage._manifest_operations(
                [
                    {"operation": "GET /agents", "reason": "First"},
                    {"operation": "GET /agents", "reason": "Second"},
                ],
                "known_gap",
            )


AS_OF = datetime.date(2026, 11, 6)


class AuditAggregationTests(unittest.TestCase):
    def _audit(
        self,
        manifest: str,
        published: dict[str, str],
        commands: dict[str, dict],
        published_fields: dict[str, set[str]] | None = None,
        cli_fields: dict[str, set[str] | None] | None = None,
        published_parameters: dict[str, set[tuple[str, str]]] | None = None,
        client_headers: set[tuple[str, str]] | None = None,
        as_of: datetime.date = AS_OF,
    ) -> tuple[dict, bool]:
        with tempfile.TemporaryDirectory() as directory:
            manifest_path = Path(directory) / "api-coverage.toml"
            manifest_path.write_text(manifest)
            with (
                patch.object(audit_api_coverage, "MANIFEST_PATH", manifest_path),
                patch.object(audit_api_coverage, "_published_specs", return_value=[]),
                patch.object(
                    audit_api_coverage,
                    "_published_parameters",
                    return_value=(
                        published_parameters or {},
                        {("header", "x-coval-workspace-id"): "X-Coval-Workspace-Id"},
                    ),
                ),
                patch.object(
                    audit_api_coverage,
                    "_client_wide_headers",
                    return_value=client_headers or set(),
                ),
                patch.object(
                    audit_api_coverage,
                    "_published_operations",
                    return_value=published,
                ),
                patch.object(
                    audit_api_coverage,
                    "_published_request_fields",
                    return_value=published_fields or {},
                ),
                patch.object(
                    audit_api_coverage,
                    "_client_operations",
                    return_value=commands,
                ),
                patch.object(
                    audit_api_coverage,
                    "_command_operations",
                    return_value=(commands, []),
                ),
                patch.object(
                    audit_api_coverage, "_model_request_fields", return_value={}
                ),
                patch.object(
                    audit_api_coverage,
                    "_cli_request_fields",
                    return_value=cli_fields or {},
                ),
            ):
                return audit_api_coverage.audit(
                    audit_api_coverage.CATALOG_URL, as_of=as_of
                )

    def test_accepts_reviewed_gap_and_allowed_extra(self):
        manifest = f"""
[snapshot]
catalog_url = "{audit_api_coverage.CATALOG_URL}"
published_operations = 2
cli_supported_operations = 1
published_request_fields = 0
cli_modeled_request_fields = 0
published_parameters = 0
cli_supported_parameters = 0

[[known_gap]]
operation = "GET /beta"
reason = "Reviewed"
reviewed_at = "2026-11-01"

[[allowed_extra]]
operation = "POST /gamma"
reason = "Pre-deploy"
"""
        published = {
            "GET /alpha": "GET /v1/alpha",
            "GET /beta": "GET /v1/beta",
        }
        commands = {
            "GET /alpha": {"operation": "GET /alpha"},
            "POST /gamma": {"operation": "POST /gamma"},
        }

        report, passed = self._audit(manifest, published, commands)

        self.assertTrue(passed)
        self.assertEqual(1, report["known_gap_count"])
        self.assertEqual([], report["new_gaps"])
        self.assertEqual([], report["unexpected_cli_operations"])

    def test_classifies_new_gap_and_unexpected_extra_as_failure(self):
        manifest = f"""
[snapshot]
catalog_url = "{audit_api_coverage.CATALOG_URL}"
published_operations = 2
cli_supported_operations = 1
published_request_fields = 0
cli_modeled_request_fields = 0
published_parameters = 0
cli_supported_parameters = 0
"""
        published = {
            "GET /alpha": "GET /v1/alpha",
            "GET /beta": "GET /v1/beta",
        }
        commands = {
            "GET /alpha": {"operation": "GET /alpha"},
            "POST /gamma": {"operation": "POST /gamma"},
        }

        report, passed = self._audit(manifest, published, commands)

        self.assertFalse(passed)
        self.assertEqual(["GET /v1/beta"], report["new_gaps"])
        self.assertEqual(["POST /gamma"], report["unexpected_cli_operations"])

    def test_classifies_an_unreviewed_dropped_field_as_failure(self):
        manifest = f"""
[snapshot]
catalog_url = "{audit_api_coverage.CATALOG_URL}"
published_operations = 1
cli_supported_operations = 1
published_request_fields = 2
cli_modeled_request_fields = 1
published_parameters = 0
cli_supported_parameters = 0
"""
        published = {"POST /things": "POST /v1/things"}
        commands = {"POST /things": {"operation": "POST /things"}}

        report, passed = self._audit(
            manifest,
            published,
            commands,
            published_fields={"POST /things": {"name", "tags"}},
            cli_fields={"POST /things": {"name"}},
        )

        self.assertFalse(passed)
        self.assertEqual(["POST /things tags"], report["new_field_gaps"])
        self.assertEqual(["POST /things tags"], report["all_current_field_gaps"])

    def test_accepts_a_reviewed_field_gap_and_allowed_extra_field(self):
        manifest = f"""
[snapshot]
catalog_url = "{audit_api_coverage.CATALOG_URL}"
published_operations = 1
cli_supported_operations = 1
published_request_fields = 2
cli_modeled_request_fields = 1
published_parameters = 0
cli_supported_parameters = 0

[[known_field_gap]]
operation = "POST /things"
field = "tags"
reason = "Reviewed"
reviewed_at = "2026-11-01"

[[allowed_extra_field]]
operation = "POST /things"
field = "legacy"
reason = "Served but undocumented"
"""
        published = {"POST /things": "POST /v1/things"}
        commands = {"POST /things": {"operation": "POST /things"}}

        report, passed = self._audit(
            manifest,
            published,
            commands,
            published_fields={"POST /things": {"name", "tags"}},
            cli_fields={"POST /things": {"name", "legacy"}},
        )

        self.assertTrue(passed)
        self.assertEqual(1, report["known_field_gap_count"])
        self.assertEqual([], report["new_field_gaps"])
        self.assertEqual([], report["unexpected_cli_request_fields"])

    def test_rejects_request_field_in_multiple_manifest_sections(self):
        manifest = """
[snapshot]
catalog_url = "https://api.coval.dev/v1/openapi"
published_operations = 0
cli_supported_operations = 0
published_request_fields = 0
cli_modeled_request_fields = 0
published_parameters = 0
cli_supported_parameters = 0

[[known_field_gap]]
operation = "POST /agents"
field = "tags"
reason = "Reviewed"
reviewed_at = "2026-11-01"

[[allowed_extra_field]]
operation = "POST /agents"
field = "tags"
reason = "Served but undocumented"
"""
        with self.assertRaisesRegex(ValueError, "only one section"):
            self._audit(manifest, {}, {})

    def test_rejects_operation_in_multiple_manifest_sections(self):
        manifest = """
[snapshot]
catalog_url = "https://api.coval.dev/v1/openapi"
published_operations = 0
cli_supported_operations = 0
published_request_fields = 0
cli_modeled_request_fields = 0
published_parameters = 0
cli_supported_parameters = 0

[[known_gap]]
operation = "GET /agents"
reason = "Reviewed"
reviewed_at = "2026-11-01"

[[allowed_extra]]
operation = "GET /agents"
reason = "Pre-deploy"
"""
        with self.assertRaisesRegex(ValueError, "only one section"):
            self._audit(manifest, {}, {})

    def _snapshot(self, **counts) -> str:
        values = {
            "published_operations": 1,
            "cli_supported_operations": 1,
            "published_request_fields": 0,
            "cli_modeled_request_fields": 0,
            "published_parameters": 0,
            "cli_supported_parameters": 0,
            **counts,
        }
        lines = [f'catalog_url = "{audit_api_coverage.CATALOG_URL}"']
        lines += [f"{key} = {value}" for key, value in values.items()]
        return "[snapshot]\n" + "\n".join(lines) + "\n"

    def test_requires_reviewed_at_on_every_reviewed_gap(self):
        manifest = (
            self._snapshot()
            + """
[[known_gap]]
operation = "GET /beta"
reason = "Reviewed"
"""
        )
        with self.assertRaisesRegex(ValueError, "requires reviewed_at"):
            self._audit(manifest, {"GET /beta": "GET /v1/beta"}, {})

    def test_reports_a_reviewed_gap_past_its_deadline_as_overdue(self):
        manifest = (
            self._snapshot(published_operations=2)
            + """
[gap_review]
max_age_days = 90

[[known_gap]]
operation = "GET /beta"
reason = "Reviewed"
reviewed_at = "2026-07-30"
"""
        )
        published = {"GET /alpha": "GET /v1/alpha", "GET /beta": "GET /v1/beta"}
        commands = {"GET /alpha": {"operation": "GET /alpha"}}

        before, passed_before = self._audit(
            manifest, published, commands, as_of=datetime.date(2026, 10, 28)
        )
        after, passed_after = self._audit(
            manifest, published, commands, as_of=datetime.date(2026, 10, 29)
        )

        self.assertTrue(passed_before)
        self.assertEqual([], before["overdue_reviewed_gaps"])
        self.assertEqual("2026-10-28", before["earliest_review_deadline"])
        self.assertFalse(passed_after)
        self.assertEqual(
            ["known_gap GET /beta (reviewed 2026-07-30)"],
            after["overdue_reviewed_gaps"],
        )
        self.assertEqual(
            {
                key: value
                for key, value in before.items()
                if key != "overdue_reviewed_gaps"
            },
            {
                key: value
                for key, value in after.items()
                if key != "overdue_reviewed_gaps"
            },
        )

    def test_does_not_age_a_stale_reviewed_gap(self):
        manifest = (
            self._snapshot()
            + """
[[known_gap]]
operation = "GET /beta"
reason = "Reviewed"
reviewed_at = "2025-01-01"
"""
        )
        commands = {"GET /alpha": {"operation": "GET /alpha"}}

        report, passed = self._audit(
            manifest, {"GET /alpha": "GET /v1/alpha"}, commands
        )

        self.assertFalse(passed)
        self.assertEqual(["GET /beta"], report["stale_gaps"])
        self.assertEqual([], report["overdue_reviewed_gaps"])
        self.assertIsNone(report["earliest_review_deadline"])

    def test_ages_reviewed_field_and_parameter_gaps(self):
        manifest = (
            self._snapshot(
                published_request_fields=1,
                published_parameters=1,
            )
            + """
[gap_review]
max_age_days = 30

[[known_field_gap]]
operation = "POST /things"
field = "tags"
reason = "Reviewed"
reviewed_at = "2026-09-01"

[[known_parameter_gap]]
operation = "*"
in = "header"
name = "X-Coval-Workspace-Id"
reason = "Reviewed"
reviewed_at = "2026-10-01"
"""
        )
        commands = {"POST /things": {"operation": "POST /things"}}

        report, passed = self._audit(
            manifest,
            {"POST /things": "POST /v1/things"},
            commands,
            published_fields={"POST /things": {"tags"}},
            cli_fields={"POST /things": set()},
            published_parameters={"POST /things": {("header", "x-coval-workspace-id")}},
        )

        self.assertFalse(passed)
        self.assertEqual(
            [
                "known_field_gap POST /things tags (reviewed 2026-09-01)",
                "known_parameter_gap * header X-Coval-Workspace-Id "
                "(reviewed 2026-10-01)",
            ],
            report["overdue_reviewed_gaps"],
        )

    def test_fails_on_a_published_header_the_cli_cannot_send(self):
        manifest = self._snapshot(published_parameters=1)
        commands = {"GET /agents": {"operation": "GET /agents"}}

        report, passed = self._audit(
            manifest,
            {"GET /agents": "GET /v1/agents"},
            commands,
            published_parameters={"GET /agents": {("header", "x-coval-workspace-id")}},
            client_headers={("header", "x-api-key")},
        )

        self.assertFalse(passed)
        self.assertEqual(
            ["GET /agents header X-Coval-Workspace-Id"],
            report["new_parameter_gaps"],
        )

    def test_accepts_a_client_wide_header_as_sendable(self):
        manifest = self._snapshot(published_parameters=1, cli_supported_parameters=1)
        commands = {"GET /agents": {"operation": "GET /agents"}}

        report, passed = self._audit(
            manifest,
            {"GET /agents": "GET /v1/agents"},
            commands,
            published_parameters={"GET /agents": {("header", "x-coval-workspace-id")}},
            client_headers={("header", "x-coval-workspace-id")},
        )

        self.assertTrue(passed)
        self.assertEqual([], report["all_current_parameter_gaps"])

    def test_reports_a_wildcard_parameter_gap_the_client_now_sends_as_stale(self):
        manifest = (
            self._snapshot(published_parameters=1, cli_supported_parameters=1)
            + """
[[known_parameter_gap]]
operation = "*"
in = "header"
name = "X-Coval-Workspace-Id"
reason = "Removed once the client sends it"
reviewed_at = "2026-11-06"
"""
        )
        commands = {"GET /agents": {"operation": "GET /agents"}}

        report, passed = self._audit(
            manifest,
            {"GET /agents": "GET /v1/agents"},
            commands,
            published_parameters={"GET /agents": {("header", "x-coval-workspace-id")}},
            client_headers={("header", "x-coval-workspace-id")},
        )

        self.assertFalse(passed)
        self.assertEqual(
            ["* header X-Coval-Workspace-Id"], report["stale_parameter_gaps"]
        )

    def test_rejects_a_parameter_in_multiple_manifest_sections(self):
        manifest = (
            self._snapshot()
            + """
[[known_parameter_gap]]
operation = "GET /agents"
in = "query"
name = "tag_filters"
reason = "Reviewed"
reviewed_at = "2026-11-06"

[[allowed_extra_parameter]]
operation = "GET /agents"
in = "query"
name = "tag_filters"
reason = "Served but undocumented"
"""
        )
        with self.assertRaisesRegex(ValueError, "only one section"):
            self._audit(manifest, {}, {})


class SnapshotTests(unittest.TestCase):
    def test_reports_stale_snapshot_fields(self):
        mismatches = audit_api_coverage._snapshot_mismatches(
            {
                "catalog_url": audit_api_coverage.CATALOG_URL,
                "published_operations": 10,
                "cli_supported_operations": 8,
                "published_request_fields": 40,
                "cli_modeled_request_fields": 40,
                "published_parameters": 20,
                "cli_supported_parameters": 12,
            },
            catalog_url=audit_api_coverage.CATALOG_URL,
            published_operation_count=11,
            supported_operation_count=8,
            compared_field_count=40,
            modeled_field_count=40,
            compared_parameter_count=20,
            supported_parameter_count=12,
        )

        self.assertEqual(
            ["published_operations: recorded 10, current 11"],
            mismatches,
        )

    def test_reports_an_unrecorded_request_field_count(self):
        mismatches = audit_api_coverage._snapshot_mismatches(
            {
                "catalog_url": audit_api_coverage.CATALOG_URL,
                "published_operations": 10,
                "cli_supported_operations": 8,
            },
            catalog_url=audit_api_coverage.CATALOG_URL,
            published_operation_count=10,
            supported_operation_count=8,
            compared_field_count=40,
            modeled_field_count=39,
            compared_parameter_count=20,
            supported_parameter_count=12,
        )

        self.assertEqual(
            [
                "published_request_fields: recorded None, current 40",
                "cli_modeled_request_fields: recorded None, current 39",
                "published_parameters: recorded None, current 20",
                "cli_supported_parameters: recorded None, current 12",
            ],
            mismatches,
        )


class MarkdownReportTests(unittest.TestCase):
    def setUp(self):
        self.report = {
            "catalog_url": audit_api_coverage.CATALOG_URL,
            "published_operation_count": 3,
            "client_operation_count": 2,
            "command_operation_count": 2,
            "supported_operation_count": 2,
            "known_gap_count": 0,
            "new_gaps": ["GET /v1/beta"],
            "stale_gaps": [],
            "unexpected_cli_operations": [],
            "stale_allowed_extras": [],
            "stale_planned_operations": [],
            "client_only_operations": [],
            "unmapped_command_client_methods": [],
            "snapshot_mismatches": ["published_operations: recorded 2, current 3"],
            "all_current_gaps": ["GET /v1/beta"],
            "compared_request_field_count": 4,
            "modeled_request_field_count": 3,
            "known_field_gap_count": 0,
            "new_field_gaps": ["POST /things tags"],
            "stale_field_gaps": [],
            "unexpected_cli_request_fields": [],
            "stale_allowed_extra_fields": [],
            "all_current_field_gaps": ["POST /things tags"],
            "compared_parameter_count": 5,
            "supported_parameter_count": 3,
            "known_parameter_gap_count": 1,
            "new_parameter_gaps": ["GET /things query tag_filters"],
            "stale_parameter_gaps": [],
            "unexpected_cli_parameters": [],
            "stale_allowed_extra_parameters": [],
            "all_current_parameter_gaps": [
                "* header X-Coval-Workspace-Id (1 operation)",
                "GET /things query tag_filters",
            ],
            "gap_review_max_age_days": 120,
            "earliest_review_deadline": "2026-11-27",
            "overdue_reviewed_gaps": ["known_gap GET /beta (reviewed 2026-07-30)"],
        }

    def test_renders_deterministic_actionable_report(self):
        rendered = audit_api_coverage.render_markdown_report(self.report, False)

        self.assertIn("| Reconciliation status | ACTION REQUIRED |", rendered)
        self.assertIn("- `GET /v1/beta`", rendered)
        self.assertIn(
            "- `published_operations: recorded 2, current 3`",
            rendered,
        )
        self.assertIn("| Request fields modeled by the CLI | 3 |", rendered)
        self.assertIn("## New published request fields the CLI drops", rendered)
        self.assertIn("- `POST /things tags`", rendered)
        self.assertIn(
            "| Published header and query parameters on covered operations | 5 |",
            rendered,
        )
        self.assertIn("| Reviewed gaps overdue for re-review | 1 |", rendered)
        self.assertIn("| Earliest re-review deadline | 2026-11-27 |", rendered)
        self.assertIn("## Reviewed gaps overdue for re-review", rendered)
        self.assertIn("- `known_gap GET /beta (reviewed 2026-07-30)`", rendered)
        self.assertIn("## New published parameters the CLI cannot send", rendered)
        self.assertIn("- `* header X-Coval-Workspace-Id (1 operation)`", rendered)
        self.assertIn("due for re-review 120 days", rendered)
        self.assertNotIn("Generated at", rendered)
        self.assertEqual(
            rendered, audit_api_coverage.render_markdown_report(self.report, False)
        )

    def test_reports_no_deadline_without_reviewed_gaps(self):
        self.report["earliest_review_deadline"] = None
        self.report["overdue_reviewed_gaps"] = []

        rendered = audit_api_coverage.render_markdown_report(self.report, True)

        self.assertIn("| Reconciliation status | PASS |", rendered)
        self.assertIn("| Earliest re-review deadline | None |", rendered)

    @patch("scripts.audit_api_coverage.audit")
    def test_allow_drift_writes_report_and_returns_success(self, audit):
        audit.return_value = (self.report, False)
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "coverage.md"
            argv = [
                "audit_api_coverage.py",
                "--write-markdown",
                str(output),
                "--allow-drift",
            ]
            with (
                patch("sys.argv", argv),
                redirect_stdout(io.StringIO()),
            ):
                result = audit_api_coverage.main()

            self.assertEqual(0, result)
            self.assertIn("ACTION REQUIRED", output.read_text())

    @patch("scripts.audit_api_coverage.audit")
    def test_as_of_ages_reviewed_gaps_against_the_given_date(self, audit):
        audit.return_value = (self.report, True)
        argv = ["audit_api_coverage.py", "--as-of", "2026-12-01"]
        with patch("sys.argv", argv), redirect_stdout(io.StringIO()):
            audit_api_coverage.main()

        self.assertEqual(datetime.date(2026, 12, 1), audit.call_args.args[2])

    def test_allow_drift_requires_markdown_output(self):
        with (
            patch("sys.argv", ["audit_api_coverage.py", "--allow-drift"]),
            redirect_stderr(io.StringIO()),
            self.assertRaises(SystemExit) as exception,
        ):
            audit_api_coverage.main()

        self.assertEqual(2, exception.exception.code)


class PublishedParameterTests(unittest.TestCase):
    def test_collects_header_and_query_parameters_including_path_level_ones(self):
        spec = {
            "paths": {
                "/v1/agents/{agent_id}": {
                    "parameters": [
                        {"name": "agent_id", "in": "path"},
                        {"$ref": "#/components/parameters/WorkspaceId"},
                    ],
                    "get": {
                        "parameters": [
                            {"name": "view", "in": "query"},
                            {"name": "session", "in": "cookie"},
                        ]
                    },
                    "delete": {},
                }
            },
            "components": {
                "parameters": {
                    "WorkspaceId": {"name": "X-Coval-Workspace-Id", "in": "header"}
                }
            },
        }

        parameters, names = audit_api_coverage._published_parameters([spec])

        workspace = ("header", "x-coval-workspace-id")
        self.assertEqual(
            {
                "GET /agents/{id}": {workspace, ("query", "view")},
                "DELETE /agents/{id}": {workspace},
            },
            parameters,
        )
        self.assertEqual("X-Coval-Workspace-Id", names[workspace])

    def test_rejects_an_unknown_parameter_reference(self):
        spec = {
            "paths": {
                "/v1/agents": {
                    "get": {"parameters": [{"$ref": "#/components/parameters/Nope"}]}
                }
            }
        }
        with self.assertRaisesRegex(RuntimeError, "unknown component"):
            audit_api_coverage._published_parameters([spec])

    def test_rejects_a_reference_outside_components_parameters(self):
        spec = {
            "paths": {
                "/v1/agents": {
                    "get": {"parameters": [{"$ref": "other.yaml#/Workspace"}]}
                }
            }
        }
        with self.assertRaisesRegex(RuntimeError, "unsupported parameter reference"):
            audit_api_coverage._published_parameters([spec])

    def test_matches_header_names_without_regard_to_case(self):
        self.assertEqual(
            ("header", "if-match"),
            audit_api_coverage._parameter_key("header", "If-Match"),
        )
        self.assertEqual(
            ("query", "pageSize"),
            audit_api_coverage._parameter_key("query", "pageSize"),
        )


class FunctionParameterTests(unittest.TestCase):
    HELPERS = {"ListParams": {"filter", "page_size"}}

    def _parameters(self, block: str, constants: dict[str, str] | None = None):
        return audit_api_coverage._function_parameters(
            "things.list", block, constants or {}, self.HELPERS
        )

    def test_separates_named_parameters_from_shared_helper_keys(self):
        block = """
    pub async fn list(&self, params: models::ListParams, status: Option<String>) {
        let mut url = self.0.url("/v1/things");
        params.apply_to(&mut url);
        if let Some(status) = status {
            url.query_pairs_mut().append_pair("status", &status);
        }
        self.0.get(url).await
    }
"""
        named, shared = self._parameters(block)

        self.assertEqual({("query", "status")}, named)
        self.assertEqual({("query", "filter"), ("query", "page_size")}, shared)

    def test_resolves_a_header_name_constant(self):
        block = """
    pub async fn update(&self, etag: &str) {
        let url = self.0.url("/v1/things/x");
        self.0.http.patch(url).header(IF_MATCH_HEADER, etag);
    }
"""
        named, _ = self._parameters(block, {"IF_MATCH_HEADER": "If-Match"})

        self.assertEqual({("header", "if-match")}, named)

    def test_rejects_a_computed_parameter_name(self):
        block = """
    pub async fn list(&self, key: &str) {
        let mut url = self.0.url("/v1/things");
        url.query_pairs_mut().append_pair(key, "1");
    }
"""
        with self.assertRaisesRegex(RuntimeError, "could not resolve the parameter"):
            self._parameters(block)

    def test_rejects_an_unresolvable_query_helper(self):
        block = """
    pub async fn list(&self, params: LocalParams) {
        let mut url = self.0.url("/v1/things");
        params.apply_to(&mut url);
    }
"""
        with self.assertRaisesRegex(RuntimeError, "query helper"):
            self._parameters(block)

    def test_rejects_query_construction_it_cannot_read(self):
        block = """
    pub async fn list(&self) {
        let mut url = self.0.url("/v1/things");
        url.set_query(Some("view=full"));
    }
"""
        with self.assertRaisesRegex(RuntimeError, "query_pairs_mut"):
            self._parameters(block)


class ModelQueryHelperTests(unittest.TestCase):
    def test_reads_the_keys_an_apply_to_helper_appends(self):
        with tempfile.TemporaryDirectory() as directory:
            models = Path(directory)
            (models / "common.rs").write_text(
                """
const ORDER_KEY: &str = "order_by";

pub struct ListParams {
    pub filter: Option<String>,
}

impl ListParams {
    pub fn apply_to(&self, url: &mut Url) {
        let mut pairs = url.query_pairs_mut();
        if let Some(ref filter) = self.filter {
            pairs.append_pair("filter", filter);
        }
        pairs.append_pair(ORDER_KEY, "name");
    }
}

impl Other {
    pub fn describe(&self) {}
}
"""
            )
            with patch.object(audit_api_coverage, "MODELS_PATH", models):
                helpers = audit_api_coverage._model_query_helpers()

        self.assertEqual({"ListParams": {"filter", "order_by"}}, helpers)

    def test_reads_the_real_list_params_helper(self):
        helpers = audit_api_coverage._model_query_helpers()

        self.assertEqual(
            {"filter", "page_size", "page_token", "order_by"}, helpers["ListParams"]
        )


class ClientWideHeaderTests(unittest.TestCase):
    def _headers(self, source: str) -> set[tuple[str, str]]:
        with tempfile.TemporaryDirectory() as directory:
            client_path = Path(directory) / "mod.rs"
            client_path.write_text(source)
            with patch.object(audit_api_coverage, "CLIENT_PATH", client_path):
                return audit_api_coverage._client_wide_headers()

    def test_reads_headers_set_inline_on_each_request(self):
        source = """
impl CovalClient {
    pub async fn get(&self, url: Url) {
        self.http.get(url).header("X-API-Key", &self.api_key).send().await
    }
}

impl AgentsClient<'_> {
    pub async fn update(&self) {
        self.0.http.patch(url).header("If-Match", etag);
    }
}
"""
        self.assertEqual({("header", "x-api-key")}, self._headers(source))

    def test_reads_an_authorize_helper_that_adds_the_workspace_header(self):
        source = """
pub const WORKSPACE_HEADER: &str = "X-Coval-Workspace-Id";

impl CovalClient {
    fn authorize(&self, request: RequestBuilder) -> RequestBuilder {
        let request = request.header("X-API-Key", &self.api_key);
        match &self.workspace_id {
            Some(workspace_id) => request.header(WORKSPACE_HEADER, workspace_id),
            None => request,
        }
    }
}
"""
        self.assertEqual(
            {("header", "x-api-key"), ("header", "x-coval-workspace-id")},
            self._headers(source),
        )

    def test_rejects_a_client_without_a_base_implementation(self):
        with self.assertRaisesRegex(RuntimeError, "impl CovalClient"):
            self._headers("impl AgentsClient<'_> {\n}\n")

    def test_reads_the_real_client(self):
        self.assertIn(
            ("header", "x-api-key"), audit_api_coverage._client_wide_headers()
        )


class ManifestParameterTests(unittest.TestCase):
    @staticmethod
    def _entry(**overrides) -> dict:
        return {
            "operation": "GET /agents",
            "in": "query",
            "name": "tag_filters",
            "reason": "Reviewed",
            **overrides,
        }

    def test_rejects_a_missing_location(self):
        with self.assertRaisesRegex(ValueError, "non-empty"):
            audit_api_coverage._manifest_parameter_entries(
                [self._entry(**{"in": ""})], "known_parameter_gap"
            )

    def test_rejects_a_path_parameter(self):
        with self.assertRaisesRegex(ValueError, "invalid parameter location"):
            audit_api_coverage._manifest_parameter_entries(
                [self._entry(**{"in": "path"})], "known_parameter_gap"
            )

    def test_rejects_an_invalid_operation(self):
        with self.assertRaisesRegex(ValueError, "invalid operation"):
            audit_api_coverage._manifest_parameter_entries(
                [self._entry(operation="HEAD /agents")], "known_parameter_gap"
            )

    def test_rejects_a_header_listed_twice_in_different_case(self):
        with self.assertRaisesRegex(ValueError, "duplicate entry"):
            audit_api_coverage._manifest_parameter_entries(
                [
                    self._entry(**{"in": "header", "name": "If-Match"}),
                    self._entry(**{"in": "header", "name": "if-match"}),
                ],
                "known_parameter_gap",
            )

    def test_canonicalizes_operations_and_accepts_any_operation(self):
        entries = audit_api_coverage._manifest_parameter_entries(
            [
                self._entry(operation="GET /agents/{agent_id}", name="view"),
                self._entry(
                    operation="*", **{"in": "header", "name": "X-Coval-Workspace-Id"}
                ),
            ],
            "known_parameter_gap",
        )

        self.assertEqual(
            [
                ("GET /agents/{id}", "query", "view"),
                ("*", "header", "x-coval-workspace-id"),
            ],
            list(entries),
        )


class ParameterReconciliationTests(unittest.TestCase):
    WORKSPACE = ("header", "x-coval-workspace-id")

    def test_classifies_gaps_extras_and_stale_exceptions(self):
        result = audit_api_coverage._parameter_reconciliation(
            published_parameters={
                "GET /things": {self.WORKSPACE, ("query", "view"), ("query", "filter")},
                "GET /others": {("query", "view")},
            },
            cli_parameters={
                "GET /things": {
                    "named": {("query", "legacy"), ("query", "view")},
                    "shared": {("query", "filter"), ("query", "order_by")},
                }
            },
            client_headers={("header", "x-api-key")},
            known_parameter_gaps={
                ("*", "header", "x-coval-workspace-id"): {},
                ("GET /things", "query", "retired"): {},
            },
            allowed_extra_parameters={},
        )

        self.assertEqual({("GET /things", *self.WORKSPACE)}, result["actual_gaps"])
        self.assertEqual(set(), result["new_gaps"])
        self.assertEqual(
            {("GET /things", *self.WORKSPACE): ("*", *self.WORKSPACE)},
            result["gap_matches"],
        )
        self.assertEqual({("GET /things", "query", "retired")}, result["stale_gaps"])
        self.assertEqual(
            {("GET /things", "query", "legacy")}, result["unexpected_extras"]
        )
        self.assertEqual(3, result["compared_parameter_count"])
        self.assertEqual(2, result["supported_parameter_count"])

    def test_prefers_an_exact_entry_over_a_wildcard(self):
        gap = ("GET /things", *self.WORKSPACE)
        entries = {("*", *self.WORKSPACE): {}, gap: {}}

        self.assertEqual(
            gap, audit_api_coverage._matching_parameter_entry(gap, entries)
        )

    def test_accepts_a_reviewed_extra_and_reports_a_stale_one(self):
        result = audit_api_coverage._parameter_reconciliation(
            published_parameters={"GET /api-keys": set()},
            cli_parameters={
                "GET /api-keys": {"named": {("query", "environment")}, "shared": set()}
            },
            client_headers=set(),
            known_parameter_gaps={},
            allowed_extra_parameters={
                ("GET /api-keys", "query", "environment"): {},
                ("GET /api-keys", "query", "retired"): {},
            },
        )

        self.assertEqual(set(), result["unexpected_extras"])
        self.assertEqual(
            {("GET /api-keys", "query", "retired")}, result["stale_allowed_extras"]
        )

    def test_folds_wildcard_gaps_into_one_report_line(self):
        names = {self.WORKSPACE: "X-Coval-Workspace-Id"}
        known = {
            ("*", *self.WORKSPACE): {
                "operation": "*",
                "in": "header",
                "name": "X-Coval-Workspace-Id",
            },
            ("GET /b", "query", "view"): {},
        }
        result = audit_api_coverage._parameter_reconciliation(
            published_parameters={
                "GET /a": {self.WORKSPACE},
                "GET /b": {self.WORKSPACE, ("query", "view")},
            },
            cli_parameters={
                "GET /a": {"named": set(), "shared": set()},
                "GET /b": {"named": set(), "shared": set()},
            },
            client_headers=set(),
            known_parameter_gaps=known,
            allowed_extra_parameters={},
        )

        self.assertEqual(
            [
                "* header X-Coval-Workspace-Id (2 operations)",
                "GET /b query view",
            ],
            audit_api_coverage._current_parameter_gaps(result, known, names),
        )


class ReviewDateTests(unittest.TestCase):
    def _dates(self, value) -> dict:
        return audit_api_coverage._review_dates(
            {"GET /agents": {"reviewed_at": value}}, "known_gap", AS_OF
        )

    def test_accepts_a_quoted_or_native_toml_date(self):
        expected = {"GET /agents": datetime.date(2026, 8, 24)}
        self.assertEqual(expected, self._dates("2026-08-24"))
        self.assertEqual(expected, self._dates(datetime.date(2026, 8, 24)))

    def test_rejects_a_missing_or_malformed_date(self):
        for value in (None, "", "2026-8-24", "20260824", "2026-02-30", 20260824):
            with self.subTest(value=value):
                with self.assertRaisesRegex(ValueError, "requires reviewed_at"):
                    self._dates(value)

    def test_rejects_a_date_and_time(self):
        with self.assertRaisesRegex(ValueError, "requires reviewed_at"):
            self._dates(datetime.datetime(2026, 8, 24, 12, 0))

    def test_rejects_a_date_more_than_a_day_ahead(self):
        self.assertEqual(
            {"GET /agents": datetime.date(2026, 11, 7)}, self._dates("2026-11-07")
        )
        with self.assertRaisesRegex(ValueError, "in the future"):
            self._dates("2026-11-08")

    def test_validates_the_maximum_age(self):
        self.assertEqual(
            audit_api_coverage.DEFAULT_GAP_REVIEW_MAX_AGE_DAYS,
            audit_api_coverage._gap_review_max_age({}),
        )
        self.assertEqual(
            30,
            audit_api_coverage._gap_review_max_age(
                {"gap_review": {"max_age_days": 30}}
            ),
        )
        for value in (0, -1, "90", True):
            with self.subTest(value=value):
                with self.assertRaisesRegex(ValueError, "positive integer"):
                    audit_api_coverage._gap_review_max_age(
                        {"gap_review": {"max_age_days": value}}
                    )

    def test_marks_a_gap_overdue_only_after_its_deadline(self):
        reviews = [
            ("known_gap GET /a", datetime.date(2026, 7, 30)),
            ("known_gap GET /b", datetime.date(2026, 8, 24)),
        ]

        on_deadline = audit_api_coverage._overdue_reviews(
            reviews, datetime.date(2026, 10, 28), 90
        )
        after_deadline = audit_api_coverage._overdue_reviews(
            reviews, datetime.date(2026, 10, 29), 90
        )

        self.assertEqual(([], "2026-10-28"), on_deadline)
        self.assertEqual(
            (["known_gap GET /a (reviewed 2026-07-30)"], "2026-10-28"), after_deadline
        )

    def test_the_repository_manifest_dates_every_reviewed_gap(self):
        manifest = tomllib.loads(audit_api_coverage.MANIFEST_PATH.read_text())
        for section in audit_api_coverage.REVIEWED_GAP_SECTIONS:
            with self.subTest(section=section):
                for entry in manifest.get(section, []):
                    self.assertRegex(
                        str(entry.get("reviewed_at")), r"^\d{4}-\d{2}-\d{2}$"
                    )


if __name__ == "__main__":
    unittest.main()
