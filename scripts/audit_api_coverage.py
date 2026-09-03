"""Compare first-class CLI operations and request fields with Coval's OpenAPI catalog.

The audit covers two layers. Route coverage asks whether every published
operation reaches a first-class CLI command. Request-field coverage asks whether
every published request-body property is declared on the Rust request struct the
CLI actually serializes, because serde silently drops undeclared fields.

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
REQUEST_BODY_METHODS = frozenset({"patch", "post", "put"})
REQUEST_BODY_MEDIA_TYPE = "application/json"
FETCH_ATTEMPTS = 3
ROOT = Path(__file__).resolve().parents[1]
CLIENT_PATH = ROOT / "src" / "client" / "mod.rs"
COMMANDS_PATH = ROOT / "src" / "commands"
MODELS_PATH = ROOT / "src" / "client" / "models"
MANIFEST_PATH = ROOT / "api-coverage.toml"

# A `serde_json::Value` request body forwards caller JSON verbatim, so it cannot
# drop a published field the way a hand-written struct can.
PASSTHROUGH_BODY = "serde_json::Value"


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


def _published_specs(catalog_url: str, allowed_origins: frozenset[str]) -> list[dict]:
    """Fetch the catalog index and every per-resource spec exactly once."""

    catalog = json.loads(_fetch(catalog_url, allowed_origins))
    return [
        yaml.safe_load(_fetch(entry["url"], allowed_origins))
        for entry in catalog["specs"]
    ]


def _published_operations(specs: list[dict]) -> dict[str, str]:
    operations: dict[str, str] = {}
    for spec in specs:
        for path, path_item in (spec.get("paths") or {}).items():
            for method in HTTP_METHODS & set(path_item):
                canonical = _canonical_operation(method, path)
                display = f"{method.upper()} {path}"
                previous = operations.get(canonical)
                if previous is not None and previous != display:
                    raise RuntimeError(
                        f"published operations {previous!r} and {display!r} "
                        f"share canonical key {canonical!r}"
                    )
                operations[canonical] = display
    return operations


def _schema_properties(
    schema: dict, schemas: dict, seen: tuple[str, ...] = ()
) -> set[str]:
    """Return the top-level JSON property names a request-body schema accepts."""

    reference = schema.get("$ref")
    if reference is not None:
        name = reference.rsplit("/", 1)[-1]
        if name in seen:
            return set()
        target = schemas.get(name)
        if target is None:
            raise RuntimeError(f"request body references unknown schema {name!r}")
        return _schema_properties(target, schemas, (*seen, name))

    properties = set(schema.get("properties") or {})
    # A composed schema accepts every branch's properties, because one flat Rust
    # struct has to be able to serialize whichever branch the caller picked.
    for keyword in ("allOf", "anyOf", "oneOf"):
        for subschema in schema.get(keyword) or []:
            properties |= _schema_properties(subschema, schemas, seen)
    return properties


def _published_request_fields(specs: list[dict]) -> dict[str, set[str]]:
    """Map each published body operation to its JSON request-body properties."""

    request_fields: dict[str, set[str]] = {}
    for spec in specs:
        schemas = (spec.get("components") or {}).get("schemas") or {}
        for path, path_item in (spec.get("paths") or {}).items():
            for method in REQUEST_BODY_METHODS & set(path_item):
                operation = path_item[method]
                if not isinstance(operation, dict):
                    continue
                content = (operation.get("requestBody") or {}).get("content") or {}
                media = content.get(REQUEST_BODY_MEDIA_TYPE)
                if media is None:
                    continue
                canonical = _canonical_operation(method, path)
                properties = _schema_properties(media.get("schema") or {}, schemas)
                request_fields.setdefault(canonical, set()).update(properties)
    return request_fields


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
            request_bodies = _request_body_types(client_method, block)
            for path in paths:
                canonical = _canonical_operation(method, path)
                operation = operations.setdefault(
                    canonical,
                    {
                        "operation": f"{method} {path.removeprefix('/v1')}",
                        "client_methods": set(),
                        "request_bodies": set(),
                    },
                )
                operation["client_methods"].add(client_method)
                operation["request_bodies"].update(request_bodies)
    return operations


def _request_body_types(client_method: str, block: str) -> set[str]:
    """Return the Rust types a client function serializes as its request body."""

    types: set[str] = set()
    for match in re.finditer(
        r"\.(?:post|patch|put)\(\s*url\s*,\s*&(?P<binding>[A-Za-z0-9_]+)\s*\)", block
    ):
        binding = match.group("binding")
        if re.search(rf"\b{binding}\s*:\s*serde_json::Value\b", block):
            types.add(PASSTHROUGH_BODY)
            continue
        declaration = re.search(
            rf"\b{binding}\s*:\s*models::(?P<name>[A-Za-z0-9_]+)", block
        ) or re.search(
            rf"\blet\s+{binding}\s*=\s*models::(?P<name>[A-Za-z0-9_]+)", block
        )
        if declaration is None:
            raise RuntimeError(
                "could not resolve the request-body type bound to "
                f"{binding!r} in client function {client_method}"
            )
        types.add(declaration.group("name"))
    return types


_STRUCT_PATTERN = re.compile(
    r"(?ms)^pub struct (?P<name>[A-Za-z0-9_]+)\s*\{(?P<body>.*?)^\}"
)
_FIELD_PATTERN = re.compile(
    r"(?P<attributes>(?:[ \t]*#\[[^\]]*\]\n)*)[ \t]*pub (?P<field>[a-z_][A-Za-z0-9_]*)\s*:"
)


def _model_request_fields() -> dict[str, set[str]]:
    """Map each model struct to the JSON field names serde will serialize."""

    structs: dict[str, set[str]] = {}
    for path in sorted(MODELS_PATH.glob("*.rs")):
        source = path.read_text()
        for struct in _STRUCT_PATTERN.finditer(source):
            fields: set[str] = set()
            for field in _FIELD_PATTERN.finditer(struct.group("body")):
                attributes = field.group("attributes")
                # `flatten` merges an untyped bag, and a skipped field is never
                # written, so neither proves the CLI models a published name.
                if "flatten" in attributes or re.search(
                    r"\bskip(?:_serializing)?\s*[,)]", attributes
                ):
                    continue
                rename = re.search(r'rename\s*=\s*"(?P<name>[^"]+)"', attributes)
                fields.add(rename.group("name") if rename else field.group("field"))
            name = struct.group("name")
            if name in structs and structs[name] != fields:
                raise RuntimeError(f"model struct {name!r} is defined more than once")
            structs[name] = fields
    return structs


def _cli_request_fields(
    command_operations: dict[str, dict], model_fields: dict[str, set[str]]
) -> dict[str, set[str] | None]:
    """Map each command-backed body operation to the fields the CLI can send.

    ``None`` means the operation forwards caller JSON verbatim, so no published
    field can be silently dropped.
    """

    request_fields: dict[str, set[str] | None] = {}
    for canonical, operation in command_operations.items():
        bodies = operation["request_bodies"]
        if not bodies:
            continue
        if PASSTHROUGH_BODY in bodies:
            request_fields[canonical] = None
            continue
        fields: set[str] = set()
        for name in bodies:
            if name not in model_fields:
                raise RuntimeError(
                    f"request-body struct {name!r} used by {canonical} "
                    f"was not found in {MODELS_PATH}"
                )
            fields |= model_fields[name]
        request_fields[canonical] = fields
    return request_fields


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


def _manifest_field_entries(entries: list[dict], section: str) -> dict[tuple, dict]:
    """Validate and index manifest exceptions keyed by operation and field."""

    exceptions: dict[tuple, dict] = {}
    for entry in entries:
        operation = entry.get("operation", "")
        field = entry.get("field", "")
        reason = entry.get("reason", "")
        if (
            not isinstance(operation, str)
            or not operation
            or not isinstance(field, str)
            or not field.strip()
            or not isinstance(reason, str)
            or not reason.strip()
        ):
            raise ValueError(
                f"{section} entries require non-empty operation, field, and reason fields"
            )
        method, separator, path = operation.partition(" ")
        if (
            not separator
            or method.lower() not in REQUEST_BODY_METHODS
            or not path.startswith("/")
        ):
            raise ValueError(f"invalid operation in {section}: {operation}")
        key = (_canonical_operation(method, path), field)
        if key in exceptions:
            raise ValueError(f"duplicate entry in {section}: {operation} {field}")
        exceptions[key] = entry
    return exceptions


def _field_reconciliation(
    published_fields: dict[str, set[str]],
    cli_fields: dict[str, set[str] | None],
    known_field_gaps: dict[tuple, dict],
    allowed_extra_fields: dict[tuple, dict],
) -> dict:
    """Diff published request-body properties against the CLI's serde fields.

    Only operations that already reach a first-class command are compared; a
    published operation the CLI does not implement is a route gap, and reporting
    its fields as well would double-count the same missing work.
    """

    compared = sorted(set(published_fields) & set(cli_fields))
    actual_gaps: set[tuple] = set()
    actual_extras: set[tuple] = set()
    modeled_field_count = 0
    for canonical in compared:
        published = published_fields[canonical]
        modeled = cli_fields[canonical]
        if modeled is None:
            modeled_field_count += len(published)
            continue
        actual_gaps |= {(canonical, field) for field in published - modeled}
        actual_extras |= {(canonical, field) for field in modeled - published}
        modeled_field_count += len(published & modeled)

    return {
        "compared_operations": compared,
        "compared_field_count": sum(len(published_fields[key]) for key in compared),
        "modeled_field_count": modeled_field_count,
        "actual_gaps": actual_gaps,
        "actual_extras": actual_extras,
        "new_gaps": actual_gaps - set(known_field_gaps),
        "stale_gaps": set(known_field_gaps) - actual_gaps,
        "unexpected_extras": actual_extras - set(allowed_extra_fields),
        "stale_allowed_extras": set(allowed_extra_fields) - actual_extras,
    }


def _format_field(key: tuple) -> str:
    canonical, field = key
    return f"{canonical} {field}"


def _snapshot_mismatches(
    snapshot: dict,
    *,
    catalog_url: str,
    published_operation_count: int,
    supported_operation_count: int,
    compared_field_count: int,
    modeled_field_count: int,
) -> list[str]:
    expected = {
        "catalog_url": catalog_url,
        "published_operations": published_operation_count,
        "cli_supported_operations": supported_operation_count,
        "published_request_fields": compared_field_count,
        "cli_modeled_request_fields": modeled_field_count,
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
    specs = _published_specs(catalog_url, allowed_origins)
    published = _published_operations(specs)
    published_fields = _published_request_fields(specs)
    client = _client_operations()
    commands, unmapped_command_methods = _command_operations(client)
    cli_fields = _cli_request_fields(commands, _model_request_fields())
    known_gaps = _manifest_operations(manifest.get("known_gap", []), "known_gap")
    allowed_extras = _manifest_operations(
        manifest.get("allowed_extra", []), "allowed_extra"
    )
    planned = _manifest_operations(
        manifest.get("planned_operation", []), "planned_operation"
    )
    known_field_gaps = _manifest_field_entries(
        manifest.get("known_field_gap", []), "known_field_gap"
    )
    allowed_extra_fields = _manifest_field_entries(
        manifest.get("allowed_extra_field", []), "allowed_extra_field"
    )
    conflicting_field_entries = set(known_field_gaps) & set(allowed_extra_fields)
    if conflicting_field_entries:
        raise ValueError(
            "manifest request fields may appear in only one section: "
            + ", ".join(sorted(map(_format_field, conflicting_field_entries)))
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
    fields = _field_reconciliation(
        published_fields, cli_fields, known_field_gaps, allowed_extra_fields
    )
    snapshot_mismatches = _snapshot_mismatches(
        manifest.get("snapshot", {}),
        catalog_url=catalog_url,
        published_operation_count=len(published),
        supported_operation_count=len(published_keys & command_keys),
        compared_field_count=fields["compared_field_count"],
        modeled_field_count=fields["modeled_field_count"],
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
        "compared_request_field_count": fields["compared_field_count"],
        "modeled_request_field_count": fields["modeled_field_count"],
        "known_field_gap_count": len(fields["actual_gaps"] & set(known_field_gaps)),
        "new_field_gaps": sorted(map(_format_field, fields["new_gaps"])),
        "stale_field_gaps": sorted(map(_format_field, fields["stale_gaps"])),
        "unexpected_cli_request_fields": sorted(
            map(_format_field, fields["unexpected_extras"])
        ),
        "stale_allowed_extra_fields": sorted(
            map(_format_field, fields["stale_allowed_extras"])
        ),
        "all_current_field_gaps": sorted(map(_format_field, fields["actual_gaps"])),
    }
    passed = not (
        new_gaps
        or stale_gaps
        or unexpected_extras
        or stale_allowed_extras
        or stale_planned
        or unmapped_command_methods
        or snapshot_mismatches
        or fields["new_gaps"]
        or fields["stale_gaps"]
        or fields["unexpected_extras"]
        or fields["stale_allowed_extras"]
    )
    return report, passed


def render_markdown_report(report: dict, passed: bool) -> str:
    """Render a deterministic, reviewable API-parity report."""

    lines = [
        "# CLI API Coverage Report",
        "",
        "<!-- Generated by scripts/audit_api_coverage.py; do not edit by hand. -->",
        "",
        "This report compares the published Coval OpenAPI catalog with first-class",
        "CLI commands, and each covered operation's published request-body",
        "properties with the serde fields on the Rust struct the CLI sends. It is",
        "intentionally timestamp-free so the weekly workflow opens or updates a PR",
        "only when coverage actually changes.",
        "",
        "## Summary",
        "",
        "| Metric | Value |",
        "| --- | ---: |",
        f"| Reconciliation status | {'PASS' if passed else 'ACTION REQUIRED'} |",
        f"| Published operations | {report['published_operation_count']} |",
        f"| First-class CLI operations | {report['supported_operation_count']} |",
        f"| Reviewed gaps | {report['known_gap_count']} |",
        f"| Client operations | {report['client_operation_count']} |",
        f"| Published request fields on covered operations | "
        f"{report['compared_request_field_count']} |",
        f"| Request fields modeled by the CLI | "
        f"{report['modeled_request_field_count']} |",
        f"| Reviewed request-field gaps | {report['known_field_gap_count']} |",
        "",
        f"Catalog: {report['catalog_url']}",
        "",
    ]

    sections = (
        ("New published operations without CLI commands", "new_gaps"),
        ("Reviewed gaps no longer present", "stale_gaps"),
        ("CLI operations absent from published OpenAPI", "unexpected_cli_operations"),
        ("Allowed extras no longer present", "stale_allowed_extras"),
        ("Planned operations no longer present", "stale_planned_operations"),
        ("Client methods not mapped to operations", "unmapped_command_client_methods"),
        ("Coverage snapshot mismatches", "snapshot_mismatches"),
        ("Client-only operations", "client_only_operations"),
        ("All current published gaps", "all_current_gaps"),
        ("New published request fields the CLI drops", "new_field_gaps"),
        ("Reviewed request-field gaps no longer present", "stale_field_gaps"),
        (
            "CLI request fields absent from published OpenAPI",
            "unexpected_cli_request_fields",
        ),
        (
            "Allowed extra request fields no longer present",
            "stale_allowed_extra_fields",
        ),
        ("All current request-field gaps", "all_current_field_gaps"),
    )
    for title, key in sections:
        lines.extend((f"## {title}", ""))
        values = report[key]
        lines.extend(f"- `{value}`" for value in values)
        if not values:
            lines.append("- None.")
        lines.append("")

    return "\n".join(lines)


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
    parser.add_argument(
        "--write-markdown",
        type=Path,
        help="Write a deterministic Markdown report to this path",
    )
    parser.add_argument(
        "--allow-drift",
        action="store_true",
        help=(
            "Exit successfully after writing a report even when parity needs "
            "reconciliation; requires --write-markdown"
        ),
    )
    args = parser.parse_args()
    if args.allow_drift and args.write_markdown is None:
        parser.error("--allow-drift requires --write-markdown")

    configured_origins = args.allowed_origin or sorted(DEFAULT_ALLOWED_ORIGINS)
    allowed_origins = frozenset(
        _normalize_allowed_origin(origin) for origin in configured_origins
    )
    report, passed = audit(args.catalog_url, allowed_origins)
    if args.write_markdown is not None:
        args.write_markdown.write_text(render_markdown_report(report, passed))
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        status = "PASS" if passed else "FAIL"
        print(
            f"{status}: {report['supported_operation_count']}/{report['published_operation_count']} "
            f"published operations have first-class CLI command coverage; "
            f"{report['known_gap_count']} reviewed gaps remain."
        )
        print(
            f"      {report['modeled_request_field_count']}/{report['compared_request_field_count']} "
            f"published request fields on those operations are modeled by the CLI; "
            f"{report['known_field_gap_count']} reviewed field gaps remain."
        )
        for key in (
            "new_gaps",
            "stale_gaps",
            "unexpected_cli_operations",
            "stale_allowed_extras",
            "stale_planned_operations",
            "unmapped_command_client_methods",
            "snapshot_mismatches",
            "new_field_gaps",
            "stale_field_gaps",
            "unexpected_cli_request_fields",
            "stale_allowed_extra_fields",
        ):
            values = report[key]
            if values:
                print(f"{key}:")
                for value in values:
                    print(f"  - {value}")
    return 0 if passed or args.allow_drift else 1


if __name__ == "__main__":
    sys.exit(main())
