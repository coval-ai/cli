"""Tests for the live API-coverage audit."""

import tempfile
import unittest
from pathlib import Path
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


if __name__ == "__main__":
    unittest.main()
