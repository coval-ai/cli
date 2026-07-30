"""Compare CLI HTTP operations with Coval's published OpenAPI catalog.

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
from pathlib import Path
from urllib.request import Request, urlopen

import tomllib
import yaml

CATALOG_URL = "https://api.coval.dev/v1/openapi"
HTTP_METHODS = frozenset({"delete", "get", "patch", "post", "put"})
ROOT = Path(__file__).resolve().parents[1]
CLIENT_PATH = ROOT / "src" / "client" / "mod.rs"
MANIFEST_PATH = ROOT / "api-coverage.toml"


def _fetch(url: str) -> bytes:
    request = Request(url, headers={"User-Agent": "coval-cli-api-coverage-audit"})
    with urlopen(request, timeout=30) as response:
        return response.read()


def _canonical_operation(method: str, path: str) -> str:
    path = path.removeprefix("/v1")
    path = re.sub(r"\{[^}]*\}", "{id}", path)
    return f"{method.upper()} {path}"


def _published_operations(catalog_url: str) -> dict[str, str]:
    catalog = json.loads(_fetch(catalog_url))
    operations: dict[str, str] = {}
    for entry in catalog["specs"]:
        spec = yaml.safe_load(_fetch(entry["url"]))
        for path, path_item in (spec.get("paths") or {}).items():
            for method in HTTP_METHODS & set(path_item):
                canonical = _canonical_operation(method, path)
                operations[canonical] = f"{method.upper()} {path}"
    return operations


def _rust_function_blocks(source: str) -> list[str]:
    starts = list(re.finditer(r"(?m)^    pub async fn ", source))
    blocks: list[str] = []
    for index, match in enumerate(starts):
        end = starts[index + 1].start() if index + 1 < len(starts) else len(source)
        blocks.append(source[match.start() : end])
    return blocks


def _client_operations() -> dict[str, str]:
    source = CLIENT_PATH.read_text()
    operations: dict[str, str] = {}
    for block in _rust_function_blocks(source):
        paths = re.findall(r'"(/v1/[^"]+)"', block)
        if not paths:
            continue

        methods = []
        if re.search(r"\.(?:post|post_empty)\(", block):
            methods.append("POST")
        if re.search(r"\.patch\(", block):
            methods.append("PATCH")
        if re.search(r"\.delete\(", block):
            methods.append("DELETE")
        if re.search(r"\.get\(", block):
            methods.append("GET")

        if len(methods) != 1:
            function_name = re.search(r"pub async fn ([a-zA-Z0-9_]+)", block)
            name = function_name.group(1) if function_name else "<unknown>"
            raise RuntimeError(f"could not infer exactly one HTTP method for client function {name}: {methods}")

        for path in paths:
            canonical = _canonical_operation(methods[0], path)
            operations[canonical] = f"{methods[0]} {path.removeprefix('/v1')}"
    return operations


def _manifest_operations(entries: list[dict], section: str) -> dict[str, dict]:
    operations: dict[str, dict] = {}
    for entry in entries:
        operation = entry.get("operation", "")
        reason = entry.get("reason", "")
        if not operation or not reason.strip():
            raise ValueError(f"{section} entries require non-empty operation and reason fields")
        method, separator, path = operation.partition(" ")
        if not separator or method.lower() not in HTTP_METHODS or not path.startswith("/"):
            raise ValueError(f"invalid operation in {section}: {operation}")
        canonical = _canonical_operation(method, path)
        if canonical in operations:
            raise ValueError(f"duplicate operation in {section}: {operation}")
        operations[canonical] = entry
    return operations


def audit(catalog_url: str) -> tuple[dict, bool]:
    manifest = tomllib.loads(MANIFEST_PATH.read_text())
    published = _published_operations(catalog_url)
    client = _client_operations()
    known_gaps = _manifest_operations(manifest.get("known_gap", []), "known_gap")
    allowed_extras = _manifest_operations(manifest.get("allowed_extra", []), "allowed_extra")
    planned = _manifest_operations(manifest.get("planned_operation", []), "planned_operation")
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
    actual_gaps = published_keys - client_keys
    actual_extras = client_keys - published_keys

    new_gaps = actual_gaps - set(known_gaps)
    stale_gaps = set(known_gaps) - actual_gaps
    unexpected_extras = actual_extras - set(allowed_extras) - set(planned)
    stale_allowed_extras = set(allowed_extras) - actual_extras
    stale_planned = set(planned) - actual_extras

    report = {
        "catalog_url": catalog_url,
        "published_operation_count": len(published),
        "cli_operation_count": len(client),
        "supported_operation_count": len(published_keys & client_keys),
        "known_gap_count": len(actual_gaps & set(known_gaps)),
        "new_gaps": [published[item] for item in sorted(new_gaps)],
        "stale_gaps": [known_gaps[item]["operation"] for item in sorted(stale_gaps)],
        "unexpected_cli_operations": [client[item] for item in sorted(unexpected_extras)],
        "stale_allowed_extras": [
            allowed_extras[item]["operation"] for item in sorted(stale_allowed_extras)
        ],
        "stale_planned_operations": [planned[item]["operation"] for item in sorted(stale_planned)],
        "all_current_gaps": [published[item] for item in sorted(actual_gaps)],
    }
    passed = not (
        new_gaps
        or stale_gaps
        or unexpected_extras
        or stale_allowed_extras
        or stale_planned
    )
    return report, passed


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalog-url", default=CATALOG_URL)
    parser.add_argument("--json", action="store_true", help="Emit the full machine-readable report")
    args = parser.parse_args()

    report, passed = audit(args.catalog_url)
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        status = "PASS" if passed else "FAIL"
        print(
            f"{status}: {report['supported_operation_count']}/{report['published_operation_count']} "
            f"published operations have CLI HTTP coverage; "
            f"{report['known_gap_count']} reviewed gaps remain."
        )
        for key in (
            "new_gaps",
            "stale_gaps",
            "unexpected_cli_operations",
            "stale_allowed_extras",
            "stale_planned_operations",
        ):
            values = report[key]
            if values:
                print(f"{key}:")
                for value in values:
                    print(f"  - {value}")
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
