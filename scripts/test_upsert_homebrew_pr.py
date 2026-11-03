import json
import subprocess
import unittest
from pathlib import Path
from unittest.mock import patch

import yaml

from scripts import upsert_homebrew_pr as publisher

REPO = "coval-ai/homebrew-tap"
BRANCH = "release/coval-v0.10.0"
PR = {"number": 10, "html_url": f"https://github.com/{REPO}/pull/10"}


def response(status, body=None, headers=None):
    header = f"HTTP/2.0 {status} Status\n" + "\n".join(
        f"{key}: {value}" for key, value in (headers or {}).items()
    )
    return subprocess.CompletedProcess(
        [], 0 if status < 400 else 1, header.rstrip() + "\n\n" + json.dumps(body), ""
    )


class HomebrewPrTests(unittest.TestCase):
    def setUp(self):
        self.elapsed = 0
        self.responses = []
        self.calls = []
        self.sleeps = []
        self.patches = [
            patch.object(publisher.subprocess, "run", side_effect=self.run_gh),
            patch.object(
                publisher.time, "time", side_effect=lambda: 1000 + self.elapsed
            ),
            patch.object(publisher.time, "monotonic", side_effect=lambda: self.elapsed),
            patch.object(publisher.time, "sleep", side_effect=self.sleep),
        ]
        for item in self.patches:
            item.start()
            self.addCleanup(item.stop)

    def run_gh(self, args, **kwargs):
        self.calls.append(args)
        self.assertEqual(kwargs, {"capture_output": True, "text": True, "timeout": 30})
        result = self.responses.pop(0)
        if isinstance(result, Exception):
            raise result
        return result() if callable(result) else result

    def sleep(self, seconds):
        self.sleeps.append(seconds)
        self.elapsed += seconds

    def upsert(self):
        return publisher.upsert(REPO, BRANCH, "chore: update coval", Path("body.md"))

    def methods(self):
        return [args[4] for args in self.calls]

    def test_create_uses_exact_branch_base_and_inherited_credentials(self):
        self.responses = [response(200, []), response(201, PR)]
        self.assertEqual(self.upsert(), PR["html_url"])
        self.assertIn("head=coval-ai%3Arelease%2Fcoval-v0.10.0", self.calls[0][5])
        self.assertIn("base=main&state=open", self.calls[0][5])
        self.assertIn(f"head={BRANCH}", self.calls[1])
        self.assertIn("base=main", self.calls[1])
        self.assertEqual(self.methods(), ["GET", "POST"])
        self.assertEqual(self.sleeps, [])

    def test_existing_pr_is_updated_without_creating(self):
        self.responses = [response(200, [PR]), response(200, PR)]
        self.assertEqual(self.upsert(), PR["html_url"])
        self.assertEqual(self.methods(), ["GET", "PATCH"])

    def test_primary_quota_waits_until_reset_then_creates(self):
        self.responses = [
            response(
                403, {}, {"X-RateLimit-Remaining": "0", "X-RateLimit-Reset": "1010"}
            ),
            response(200, []),
            response(201, PR),
        ]
        self.assertEqual(self.upsert(), PR["html_url"])
        self.assertEqual(self.sleeps, [11])

    def test_update_429_relooks_up_before_retrying(self):
        self.responses = [
            response(200, [PR]),
            response(429, {}, {"Retry-After": "10"}),
            response(200, [PR]),
            response(200, PR),
        ]
        self.assertEqual(self.upsert(), PR["html_url"])
        self.assertEqual(self.methods(), ["GET", "PATCH", "GET", "PATCH"])
        self.assertEqual(self.sleeps, [11])

    def test_create_quota_reconciles_pr_before_any_second_post(self):
        self.responses = [
            response(200, []),
            response(
                403, {}, {"X-RateLimit-Remaining": "0", "X-RateLimit-Reset": "1001"}
            ),
            response(200, [PR]),
            response(200, PR),
        ]
        self.assertEqual(self.upsert(), PR["html_url"])
        self.assertEqual(self.methods(), ["GET", "POST", "GET", "PATCH"])

    def test_accepted_create_lost_response_is_reconciled_once(self):
        for failure in [subprocess.TimeoutExpired("gh", 30), response(502, {})]:
            with self.subTest(failure=failure):
                self.calls = []
                self.responses = [response(200, []), failure, response(200, [PR])]
                self.assertEqual(self.upsert(), PR["html_url"])
                self.assertEqual(self.methods(), ["GET", "POST", "GET"])
        self.assertEqual(self.sleeps, [])

    def test_ambiguous_create_reconciliation_quota_waits_then_only_reads(self):
        self.responses = [
            response(200, []),
            subprocess.TimeoutExpired("gh", 30),
            response(
                403, {}, {"X-RateLimit-Remaining": "0", "X-RateLimit-Reset": "1010"}
            ),
            response(200, [PR]),
        ]
        self.assertEqual(self.upsert(), PR["html_url"])
        self.assertEqual(self.methods(), ["GET", "POST", "GET", "GET"])
        self.assertEqual(self.sleeps, [11])

    def test_reconciliation_quota_does_not_retry_rejected_create(self):
        self.responses = [
            response(200, []),
            response(422, {}),
            response(429, {}, {"Retry-After": "1"}),
            response(200, []),
        ]
        with self.assertRaises(publisher.ApiError) as error:
            self.upsert()
        self.assertEqual(error.exception.status, 422)
        self.assertEqual(self.methods(), ["GET", "POST", "GET", "GET"])
        self.assertEqual(self.sleeps, [2])

    def test_raw_crlf_and_mixed_case_headers(self):
        self.responses = [
            subprocess.CompletedProcess(
                [], 0, "HTTP/2.0 200 OK\r\nContent-Type: application/json\r\n\r\n[]", ""
            )
        ]
        self.assertEqual(publisher.api("GET", "repos/test"), [])
        self.responses = [
            subprocess.CompletedProcess(
                [],
                1,
                "HTTP/2.0 403 Forbidden\r\nx-RaTeLiMiT-ReMaInInG: 0\r\nX-RATELIMIT-RESET: 1010\r\n\r\n{}",
                "",
            )
        ]
        with self.assertRaises(publisher.ApiError) as error:
            publisher.api("GET", "repos/test")
        self.assertEqual(error.exception.quota_delay(1000), 11)

    def test_concurrent_422_reuses_matching_pr(self):
        self.responses = [response(200, []), response(422, {}), response(200, [PR])]
        self.assertEqual(self.upsert(), PR["html_url"])
        self.assertEqual(self.methods(), ["GET", "POST", "GET"])

    def test_validation_422_and_unaccepted_502_do_not_retry_create(self):
        for status in [422, 502]:
            with self.subTest(status=status):
                self.calls = []
                self.responses = [
                    response(200, []),
                    response(status, {}),
                    response(200, []),
                ]
                with self.assertRaises(publisher.ApiError) as error:
                    self.upsert()
                self.assertEqual(error.exception.status, status)
                self.assertEqual(self.methods(), ["GET", "POST", "GET"])
        self.assertEqual(self.sleeps, [])

    def test_nonquota_403_fails_immediately_even_on_create(self):
        for headers in [
            {},
            {"X-RateLimit-Remaining": "1", "X-RateLimit-Reset": "1010"},
        ]:
            for creating in [False, True]:
                with self.subTest(headers=headers, creating=creating):
                    self.calls = []
                    self.responses = ([response(200, [])] if creating else []) + [
                        response(403, {}, headers)
                    ]
                    with self.assertRaises(publisher.ApiError):
                        self.upsert()
                    self.assertEqual(
                        self.methods(), ["GET", "POST"] if creating else ["GET"]
                    )
        self.assertEqual(self.sleeps, [])

    def test_invalid_or_past_timing_does_not_retry(self):
        cases = [(403, {"x-ratelimit-remaining": "0"})]
        cases += [
            (403, {"x-ratelimit-remaining": "0", "x-ratelimit-reset": x})
            for x in ["bad", "nan", "inf", "999", "1000"]
        ]
        cases += [(429, {"retry-after": x}) for x in ["bad", "nan", "inf", "0", "-1"]]
        for status, headers in cases:
            with self.subTest(status=status, headers=headers):
                self.responses = [response(status, {}, headers)]
                with self.assertRaises(publisher.ApiError):
                    self.upsert()
        self.assertEqual(self.sleeps, [])

    def test_retry_after_http_date(self):
        error = publisher.ApiError(
            429, {"retry-after": "Thu, 01 Jan 1970 00:17:00 GMT"}
        )
        self.assertEqual(error.quota_delay(1000), 21)

    def test_oversized_wait_is_rejected(self):
        self.responses = [response(429, {}, {"Retry-After": "3606"})]
        with self.assertRaises(publisher.ApiError):
            self.upsert()
        self.assertEqual(self.sleeps, [])

    def test_attempt_limit(self):
        self.responses = [response(429, {}, {"Retry-After": "1"}) for _ in range(3)]
        with self.assertRaises(publisher.ApiError):
            self.upsert()
        self.assertEqual(self.methods(), ["GET"] * 3)
        self.assertEqual(self.sleeps, [2, 2])

    def test_monotonic_budget_includes_elapsed_request_time(self):
        def slow_quota():
            self.elapsed += 3
            return response(429, {}, {"Retry-After": "1"})

        self.responses = [response(429, {}, {"Retry-After": "1"}), slow_quota]
        with patch.object(publisher, "MAX_WAIT_SECONDS", 5):
            with self.assertRaises(publisher.ApiError):
                self.upsert()
        self.assertEqual(self.sleeps, [2])

    def test_missing_headers_and_invalid_success_body_fail_closed(self):
        for result in [
            subprocess.CompletedProcess([], 1, "", "not printed"),
            response(200, {}),
        ]:
            self.responses = [result]
            with self.assertRaises((publisher.ApiError, RuntimeError)):
                self.upsert()
        self.assertEqual(self.sleeps, [])

    def test_workflow_limits_only_homebrew_job_and_preserves_token_boundary(self):
        workflow = yaml.load(
            (Path(__file__).parents[1] / ".github/workflows/release.yml").read_text(),
            Loader=yaml.BaseLoader,
        )
        jobs = workflow["jobs"]
        self.assertEqual(jobs["update-homebrew"]["timeout-minutes"], "70")
        for name, job in jobs.items():
            if name != "update-homebrew":
                self.assertNotIn("timeout-minutes", job)
        steps = jobs["update-homebrew"]["steps"]
        command = next(step["run"] for step in steps if step.get("id") == "tap")
        self.assertIn('export GH_TOKEN="$HOMEBREW_TAP_TOKEN"', command)
        self.assertIn("python scripts/upsert_homebrew_pr.py", command)
        self.assertEqual(steps[0]["with"], {"persist-credentials": "false"})
