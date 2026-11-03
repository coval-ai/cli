#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import math
import subprocess
import sys
import time
from email.utils import parsedate_to_datetime
from pathlib import Path
from urllib.parse import urlencode

MAX_ATTEMPTS = 3
MAX_WAIT_SECONDS = 3605


class ApiError(RuntimeError):
    def __init__(self, status: int | None, headers: dict[str, str]):
        super().__init__(f"Homebrew PR API failed (HTTP {status or 'unknown'})")
        self.status = status
        self.headers = headers

    def quota_delay(self, now: float) -> float | None:
        try:
            if self.status == 403 and self.headers.get("x-ratelimit-remaining") == "0":
                delay = float(self.headers["x-ratelimit-reset"]) - now
            elif self.status == 429:
                value = self.headers["retry-after"]
                try:
                    delay = float(value)
                except ValueError:
                    delay = parsedate_to_datetime(value).timestamp() - now
            else:
                return None
            return delay + 1 if math.isfinite(delay) and delay > 0 else None
        except (KeyError, ValueError, TypeError, OverflowError):
            return None


def api(method: str, endpoint: str, fields: tuple[str, ...] = ()):
    try:
        result = subprocess.run(
            ["gh", "api", "--include", "--method", method, endpoint, *fields],
            capture_output=True,
            text=True,
            timeout=30,
        )
    except subprocess.TimeoutExpired:
        raise ApiError(None, {}) from None
    head, separator, body = result.stdout.replace("\r\n", "\n").partition("\n\n")
    lines = head.splitlines()
    try:
        status = int(lines[0].split()[1])
    except (IndexError, ValueError):
        raise ApiError(None, {}) from None
    headers = {
        key.lower(): value.strip()
        for line in lines[1:]
        if ":" in line
        for key, value in [line.split(":", 1)]
    }
    if result.returncode or status >= 400:
        raise ApiError(status, headers)
    if not separator:
        raise ApiError(None, {})
    try:
        return json.loads(body)
    except ValueError:
        raise ApiError(None, {}) from None


def find_pr(repo: str, branch: str):
    query = urlencode(
        {"head": f"{repo.split('/')[0]}:{branch}", "base": "main", "state": "open"}
    )
    pulls = api("GET", f"repos/{repo}/pulls?{query}")
    if not isinstance(pulls, list) or len(pulls) > 1:
        raise RuntimeError("Homebrew PR lookup returned an unexpected result")
    return pulls[0] if pulls else None


def pr_url(pr, repo: str) -> str:
    number = pr.get("number") if isinstance(pr, dict) else None
    expected = f"https://github.com/{repo}/pull/{number}"
    if not isinstance(number, int) or number < 1 or pr.get("html_url") != expected:
        raise RuntimeError("Homebrew PR response is missing its canonical URL")
    return expected


def upsert(repo: str, branch: str, title: str, body_file: Path) -> str:
    deadline = time.monotonic() + MAX_WAIT_SECONDS
    fields = ("-f", f"title={title}", "-F", f"body=@{body_file}")
    unconfirmed_create_error = None
    for attempt in range(MAX_ATTEMPTS):
        creating = False
        try:
            existing = find_pr(repo, branch)
            if unconfirmed_create_error is not None:
                if existing:
                    return pr_url(existing, repo)
                raise unconfirmed_create_error
            if existing:
                pr_url(existing, repo)
                result = api(
                    "PATCH", f"repos/{repo}/pulls/{existing['number']}", fields
                )
            else:
                creating = True
                result = api(
                    "POST",
                    f"repos/{repo}/pulls",
                    (*fields, "-f", f"head={branch}", "-f", "base=main"),
                )
            return pr_url(result, repo)
        except ApiError as error:
            if creating and (error.status in (None, 422) or error.status >= 500):
                try:
                    existing = find_pr(repo, branch)
                    if existing:
                        return pr_url(existing, repo)
                except ApiError as lookup_error:
                    unconfirmed_create_error = error
                    error = lookup_error
            delay = error.quota_delay(time.time())
            if (
                delay is None
                or attempt + 1 == MAX_ATTEMPTS
                or time.monotonic() + delay > deadline
            ):
                raise
            print(
                f"Homebrew PR quota exhausted; retrying in {math.ceil(delay)} seconds",
                file=sys.stderr,
            )
            time.sleep(delay)
    raise RuntimeError("Homebrew PR attempts exhausted")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--branch", required=True)
    parser.add_argument("--title", required=True)
    parser.add_argument("--body-file", required=True, type=Path)
    args = parser.parse_args()
    try:
        print(upsert(args.repo, args.branch, args.title, args.body_file))
    except (ApiError, RuntimeError) as error:
        print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
