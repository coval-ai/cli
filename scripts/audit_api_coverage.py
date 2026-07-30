"""Compare first-class CLI operations with Coval's published OpenAPI catalog.

The audit is intentionally live: the public catalog is the source of truth, while
``api-coverage.toml`` records reviewed gaps and temporary pre-deploy operations.
Run it from the repository root:

    python3 scripts/audit_api_coverage.py

PyYAML is required because the public specs are YAML:

    python3 -m pip install PyYAML
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener

import tomllib
import yaml

CATALOG_URL = "https://api.coval.dev/v1/openapi"
DEFAULT_ALLOWED_ORIGINS = frozenset({"https://api.coval.dev"})
HTTP_METHODS = frozenset({"delete", "get", "patch", "post", "put"})
FETCH_ATTEMPTS = 3
ROOT = Path(__file__).resolve().parents[1]
CLIENT_PATH = ROOT / "src" / "client" / "mod.rs"
COMMANDS_PATH = ROOT / "src" / "commands"
MANIFEST_PATH = ROOT / "api-coverage.toml"


def _https_origin(url: str) -> str:
    if any(character.isspace() for character in url):
        raise ValueError(f"URL must not contain whitespace: {url!r}")
    parsed = urlsplit(url)
    if parsed.scheme.lower() != "https" or not parsed.hostname:
        raise ValueError(f"URL must use HTTPS and include a host: {url!r}")
    if parsed.username is not None or parsed.password is not None:
        raise ValueError(f"URL must not include credentials: {url!r}")
    try:
        port = parsed.port
    except ValueError as exception:
        raise ValueError(f"URL has an invalid port: {url!r}") from exception
    host = parsed.hostname.encode("idna").decode("ascii").lower().rstrip(".")
    return f"https://{host}" if port in (None, 443) else f"https://{host}:{port}"


def _normalize_allowed_origin(origin: str) -> str:
    parsed = urlsplit(origin)
    if parsed.path not in ("", "/") or parsed.query or parsed.fragment:
        raise ValueError(
            f"allowed origin must not include a path, query, or fragment: {origin!r}"
        )
    return _https_origin(origin)


def _validate_fetch_url(url: str, allowed_origins: frozenset[str]) -> None:
    parsed = urlsplit(url)
    if parsed.fragment:
        raise ValueError(f"fetch URL must not include a fragment: {url!r}")
    origin = _https_origin(url)
    if origin not in allowed_origins:
        raise ValueError(f"URL origin {origin!r} is not allowed")


class _AllowedOriginRedirectHandler(HTTPRedirectHandler):
    def __init__(self, allowed_origins: frozenset[str]):
        super().__init__()
        self._allowed_origins = allowed_origins

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        _validate_fetch_url(newurl, self._allowed_origins)
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def _fetch(url: str, allowed_origins: frozenset[str]) -> bytes:
    _validate_fetch_url(url, allowed_origins)
    # The URL is HTTPS and origin-allowlisted above; redirects are checked by the custom handler.
    request = Request(  # noqa: S310
        url, headers={"User-Agent": "coval-cli-api-coverage-audit"}
    )
    opener = build_opener(_AllowedOriginRedirectHandler(allowed_origins))
    for attempt in range(FETCH_ATTEMPTS):
        try:
            with opener.open(request, timeout=30) as response:
                _validate_fetch_url(response.geturl(), allowed_origins)
                return response.read()
        except HTTPError as error:
            if error.code < 500 or attempt == FETCH_ATTEMPTS - 1:
                raise
        except (TimeoutError, URLError):
            if attempt == FETCH_ATTEMPTS - 1:
                raise
        time.sleep(2**attempt)
    raise AssertionError("fetch retry loop exited unexpectedly")


def _canonical_operation(method: str, path: str) -> str:
    path = path.removeprefix("/v1")
    path = re.sub(r"\{[^}]*\}", "{id}", path)
    return f"{method.upper()} {path}"


def _published_operations(
    catalog_url: str, allowed_origins: frozenset[str]
) -> dict[str, str]:
    catalog = json.loads(_fetch(catalog_url, allowed_origins))
    operations: dict[str, str] = {}
    for entry in catalog["specs"]:
        spec = yaml.safe_load(_fetch(entry["url"], allowed_origins))
        for path, path_item in (spec.get("paths") or {}).items():
            for method in HTTP_METHODS & set(path_item):
                canonical = _canonical_operation(method, path)
                operations[canonical] = f"{method.upper()} {path}"
    return operations


def _rust_function_blocks(source: str) -> list[tuple[str, str]]:
    starts = list(re.finditer(r"(?m)^    pub async fn (?P<name>[A-Za-z0-9_]+)", source))
    blocks: list[tuple[str, str]] = []
    for index, match in enumerate(starts):
        end = starts[index + 1].start() if index + 1 < len(starts) else len(source)
        blocks.append((match.group("name"), source[match.start() : end]))
    return blocks


def _snake_case_client_name(name: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", name.removesuffix("Client")).lower()


def _client_operations() -> dict[str, dict]:
    source = CLIENT_PATH.read_text()
    implementations = list(
        re.finditer(r"(?m)^impl (?P<name>[A-Za-z0-9_]+Client)<'_> \{", source)
    )
    operations: dict[str, dict] = {}

    for index, implementation in enumerate(implementations):
        end = (
            implementations[index + 1].start()
            if index + 1 < len(implementations)
            else len(source)
        )
        accessor = _snake_case_client_name(implementation.group("name"))
        implementation_source = source[implementation.end() : end]

        for function_name, block in _rust_function_blocks(implementation_source):
            paths = re.findall(r'"(/v1/[^"]+)"', block)
            if not paths:
                continue

            method_calls = set(
                re.findall(r"\.(get|post|post_empty|patch|delete)\(", block)
            )
            methods = {
                "POST" if method in {"post", "post_empty"} else method.upper()
                for method in method_calls
            }
            if len(methods) != 1:
                raise RuntimeError(
                    "could not infer exactly one HTTP method for client function "
                    f"{accessor}.{function_name}: {sorted(methods)}"
                )

            method = methods.pop()
            client_method = f"{accessor}.{function_name}"
            for path in paths:
                canonical = _canonical_operation(method, path)
                operation = operations.setdefault(
                    canonical,
                    {
                        "operation": f"{method} {path.removeprefix('/v1')}",
                        "client_methods": set(),
                    },
                )
                operation["client_methods"].add(client_method)
    return operations


def _command_client_methods(allowed_accessors: set[str]) -> dict[str, set[str]]:
    """Return resource-client methods referenced by first-class command modules."""

    pattern = re.compile(
        r"\bclient\s*\.\s*(?P<accessor>[A-Za-z0-9_]+)\s*"
        r"\([^)]*\)\s*\.\s*(?P<method>[A-Za-z0-9_]+)\s*\(",
        re.MULTILINE,
    )
    methods: dict[str, set[str]] = {}
    for path in sorted(COMMANDS_PATH.glob("*.rs")):
        for match in pattern.finditer(path.read_text()):
            if match.group("accessor") not in allowed_accessors:
                continue
            method = f"{match.group('accessor')}.{match.group('method')}"
            methods.setdefault(method, set()).add(path.name)
    return methods


def _command_operations(
    client_operations: dict[str, dict],
) -> tuple[dict[str, dict], list[str]]:
    known_client_methods = {
        method
        for operation in client_operations.values()
        for method in operation["client_methods"]
    }
    allowed_accessors = {method.partition(".")[0] for method in known_client_methods}
    command_methods = _command_client_methods(allowed_accessors)
    operations = {
        canonical: operation
        for canonical, operation in client_operations.items()
        if operation["client_methods"] & set(command_methods)
    }
    unmapped_methods = sorted(set(command_methods) - known_client_methods)
    return operations, unmapped_methods


def _manifest_operations(entries: list[dict], section: str) -> dict[str, dict]:
    operations: dict[str, dict] = {}
    for entry in entries:
        operation = entry.get("operation", "")
        reason = entry.get("reason", "")
        if (
            not isinstance(operation, str)
            or not operation
            or not isinstance(reason, str)
            or not reason.strip()
        ):
            raise ValueError(
                f"{section} entries require non-empty operation and reason fields"
            )
        method, separator, path = operation.partition(" ")
        if (
            not separator
            or method.lower() not in HTTP_METHODS
            or not path.startswith("/")
        ):
            raise ValueError(f"invalid operation in {section}: {operation}")
        canonical = _canonical_operation(method, path)
        if canonical in operations:
            raise ValueError(f"duplicate operation in {section}: {operation}")
        operations[canonical] = entry
    return operations


def _snapshot_mismatches(
    snapshot: dict,
    *,
    catalog_url: str,
    published_operation_count: int,
    supported_operation_count: int,
) -> list[str]:
    expected = {
        "catalog_url": catalog_url,
        "published_operations": published_operation_count,
        "cli_supported_operations": supported_operation_count,
    }
    return [
        f"{field}: recorded {snapshot.get(field)!r}, current {value!r}"
        for field, value in expected.items()
        if snapshot.get(field) != value
    ]


def audit(
    catalog_url: str,
    allowed_origins: frozenset[str] = DEFAULT_ALLOWED_ORIGINS,
) -> tuple[dict, bool]:
    manifest = tomllib.loads(MANIFEST_PATH.read_text())
    published = _published_operations(catalog_url, allowed_origins)
    client = _client_operations()
    commands, unmapped_command_methods = _command_operations(client)
    known_gaps = _manifest_operations(manifest.get("known_gap", []), "known_gap")
    allowed_extras = _manifest_operations(
        manifest.get("allowed_extra", []), "allowed_extra"
    )
    planned = _manifest_operations(
        manifest.get("planned_operation", []), "planned_operation"
    )
    overlapping_manifest_entries = (
        (set(known_gaps) & set(allowed_extras))
        | (set(known_gaps) & set(planned))
        | (set(allowed_extras) & set(planned))
    )
    if overlapping_manifest_entries:
        raise ValueError(
            "manifest operations may appear in only one section: "
            + ", ".join(sorted(overlapping_manifest_entries))
        )

    published_keys = set(published)
    client_keys = set(client)
    command_keys = set(commands)
    actual_gaps = published_keys - command_keys
    actual_extras = command_keys - published_keys

    new_gaps = actual_gaps - set(known_gaps)
    stale_gaps = set(known_gaps) - actual_gaps
    unexpected_extras = actual_extras - set(allowed_extras) - set(planned)
    stale_allowed_extras = set(allowed_extras) - actual_extras
    stale_planned = set(planned) - actual_extras
    client_only = client_keys - command_keys
    snapshot_mismatches = _snapshot_mismatches(
        manifest.get("snapshot", {}),
        catalog_url=catalog_url,
        published_operation_count=len(published),
        supported_operation_count=len(published_keys & command_keys),
    )

    report = {
        "catalog_url": catalog_url,
        "published_operation_count": len(published),
        "client_operation_count": len(client),
        "command_operation_count": len(commands),
        "supported_operation_count": len(published_keys & command_keys),
        "known_gap_count": len(actual_gaps & set(known_gaps)),
        "new_gaps": [published[item] for item in sorted(new_gaps)],
        "stale_gaps": [known_gaps[item]["operation"] for item in sorted(stale_gaps)],
        "unexpected_cli_operations": [
            client[item]["operation"] for item in sorted(unexpected_extras)
        ],
        "stale_allowed_extras": [
            allowed_extras[item]["operation"] for item in sorted(stale_allowed_extras)
        ],
        "stale_planned_operations": [
            planned[item]["operation"] for item in sorted(stale_planned)
        ],
        "client_only_operations": [
            client[item]["operation"] for item in sorted(client_only)
        ],
        "unmapped_command_client_methods": unmapped_command_methods,
        "snapshot_mismatches": snapshot_mismatches,
        "all_current_gaps": [published[item] for item in sorted(actual_gaps)],
    }
    passed = not (
        new_gaps
        or stale_gaps
        or unexpected_extras
        or stale_allowed_extras
        or stale_planned
        or unmapped_command_methods
        or snapshot_mismatches
    )
    return report, passed


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalog-url", default=CATALOG_URL)
    parser.add_argument(
        "--allowed-origin",
        action="append",
        help="HTTPS origin allowed for catalog/spec fetches; repeat for multiple origins",
    )
    parser.add_argument(
        "--json", action="store_true", help="Emit the full machine-readable report"
    )
    args = parser.parse_args()

    configured_origins = args.allowed_origin or sorted(DEFAULT_ALLOWED_ORIGINS)
    allowed_origins = frozenset(
        _normalize_allowed_origin(origin) for origin in configured_origins
    )
    report, passed = audit(args.catalog_url, allowed_origins)
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        status = "PASS" if passed else "FAIL"
        print(
            f"{status}: {report['supported_operation_count']}/{report['published_operation_count']} "
            f"published operations have first-class CLI command coverage; "
            f"{report['known_gap_count']} reviewed gaps remain."
        )
        for key in (
            "new_gaps",
            "stale_gaps",
            "unexpected_cli_operations",
            "stale_allowed_extras",
            "stale_planned_operations",
            "unmapped_command_client_methods",
            "snapshot_mismatches",
        ):
            values = report[key]
            if values:
                print(f"{key}:")
                for value in values:
                    print(f"  - {value}")
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
