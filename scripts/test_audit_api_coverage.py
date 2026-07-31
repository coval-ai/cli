"""Tests for the live API-coverage audit."""

import io
import tempfile
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
    @patch("scripts.audit_api_coverage._fetch")
    def test_rejects_distinct_operations_with_same_canonical_key(self, fetch):
        fetch.side_effect = [
            b'{"specs":[{"url":"https://api.coval.dev/a"},'
            b'{"url":"https://api.coval.dev/b"}]}',
            b"paths:\n  /v1/agents/{agent_id}:\n    get: {}\n",
            b"paths:\n  /v1/agents/{id}:\n    get: {}\n",
        ]

        with self.assertRaisesRegex(RuntimeError, "share canonical key"):
            audit_api_coverage._published_operations(
                audit_api_coverage.CATALOG_URL,
                audit_api_coverage.DEFAULT_ALLOWED_ORIGINS,
            )

    @patch("scripts.audit_api_coverage._fetch")
    def test_deduplicates_identical_operation_across_specs(self, fetch):
        fetch.side_effect = [
            b'{"specs":[{"url":"https://api.coval.dev/a"},'
            b'{"url":"https://api.coval.dev/b"}]}',
            b"paths:\n  /v1/agents/{agent_id}:\n    get: {}\n",
            b"paths:\n  /v1/agents/{agent_id}:\n    get: {}\n",
        ]

        operations = audit_api_coverage._published_operations(
            audit_api_coverage.CATALOG_URL,
            audit_api_coverage.DEFAULT_ALLOWED_ORIGINS,
        )

        self.assertEqual(
            {"GET /agents/{id}": "GET /v1/agents/{agent_id}"},
            operations,
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


class AuditAggregationTests(unittest.TestCase):
    def _audit(
        self,
        manifest: str,
        published: dict[str, str],
        commands: dict[str, dict],
    ) -> tuple[dict, bool]:
        with tempfile.TemporaryDirectory() as directory:
            manifest_path = Path(directory) / "api-coverage.toml"
            manifest_path.write_text(manifest)
            with (
                patch.object(audit_api_coverage, "MANIFEST_PATH", manifest_path),
                patch.object(
                    audit_api_coverage,
                    "_published_operations",
                    return_value=published,
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
            ):
                return audit_api_coverage.audit(audit_api_coverage.CATALOG_URL)

    def test_accepts_reviewed_gap_and_allowed_extra(self):
        manifest = f"""
[snapshot]
catalog_url = "{audit_api_coverage.CATALOG_URL}"
published_operations = 2
cli_supported_operations = 1

[[known_gap]]
operation = "GET /beta"
reason = "Reviewed"

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

    def test_rejects_operation_in_multiple_manifest_sections(self):
        manifest = """
[snapshot]
catalog_url = "https://api.coval.dev/v1/openapi"
published_operations = 0
cli_supported_operations = 0

[[known_gap]]
operation = "GET /agents"
reason = "Reviewed"

[[allowed_extra]]
operation = "GET /agents"
reason = "Pre-deploy"
"""
        with self.assertRaisesRegex(ValueError, "only one section"):
            self._audit(manifest, {}, {})


class SnapshotTests(unittest.TestCase):
    def test_reports_stale_snapshot_fields(self):
        mismatches = audit_api_coverage._snapshot_mismatches(
            {
                "catalog_url": audit_api_coverage.CATALOG_URL,
                "published_operations": 10,
                "cli_supported_operations": 8,
            },
            catalog_url=audit_api_coverage.CATALOG_URL,
            published_operation_count=11,
            supported_operation_count=8,
        )

        self.assertEqual(
            ["published_operations: recorded 10, current 11"],
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
        }

    def test_renders_deterministic_actionable_report(self):
        rendered = audit_api_coverage.render_markdown_report(self.report, False)

        self.assertIn("| Reconciliation status | ACTION REQUIRED |", rendered)
        self.assertIn("- `GET /v1/beta`", rendered)
        self.assertIn(
            "- `published_operations: recorded 2, current 3`",
            rendered,
        )
        self.assertNotIn("Generated at", rendered)

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

    def test_allow_drift_requires_markdown_output(self):
        with (
            patch("sys.argv", ["audit_api_coverage.py", "--allow-drift"]),
            redirect_stderr(io.StringIO()),
            self.assertRaises(SystemExit) as exception,
        ):
            audit_api_coverage.main()

        self.assertEqual(2, exception.exception.code)


if __name__ == "__main__":
    unittest.main()
