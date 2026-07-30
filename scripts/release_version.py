#!/usr/bin/env python3
"""Validate CLI release manifests and emit the matching tag."""

from __future__ import annotations

import argparse
import json
import re
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VERSION_RE = re.compile(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$")


def release_version(root: Path = ROOT) -> str:
    cargo = tomllib.loads((root / "Cargo.toml").read_text())
    name = cargo["package"]["name"]
    version = cargo["package"]["version"]
    if VERSION_RE.fullmatch(version) is None:
        raise ValueError(f"Cargo.toml has a non-release version: {version!r}")

    lock = tomllib.loads((root / "Cargo.lock").read_text())
    workspace_packages = [
        package
        for package in lock["package"]
        if package["name"] == name and "source" not in package
    ]
    if len(workspace_packages) != 1:
        raise ValueError(
            f"expected one workspace package named {name!r} in Cargo.lock, "
            f"found {len(workspace_packages)}"
        )
    locked_version = workspace_packages[0]["version"]
    if locked_version != version:
        raise ValueError(
            f"Cargo.lock has {name} {locked_version}, but Cargo.toml has {version}"
        )
    return version


def release_metadata(
    root: Path = ROOT, expected_tag: str | None = None
) -> dict[str, str]:
    version = release_version(root)
    tag = f"v{version}"
    if expected_tag is not None and expected_tag != tag:
        raise ValueError(
            f"release tag {expected_tag!r} does not match Cargo version tag {tag!r}"
        )
    return {"version": version, "tag": tag}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--expected-tag")
    parser.add_argument("--github-output", type=Path)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    metadata = release_metadata(args.root, args.expected_tag)
    if args.github_output is not None:
        with args.github_output.open("a") as output:
            for key, value in metadata.items():
                output.write(f"{key}={value}\n")
    if args.json:
        print(json.dumps(metadata, sort_keys=True))
    else:
        print(metadata["tag"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
