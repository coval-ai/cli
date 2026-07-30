"""Tests for the live API-coverage audit's network boundary."""

import unittest
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


if __name__ == "__main__":
    unittest.main()
