#!/usr/bin/env python3
"""Bump the CLI's stable Cargo version in both release manifests."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

if __package__:
    from .release_version import ROOT
    from .release_version import release_version
else:
    from release_version import ROOT
    from release_version import release_version


def next_version(version: str, part: str) -> str:
    major, minor, patch = (int(value) for value in version.split("."))
    if part == "minor":
        return f"{major}.{minor + 1}.0"
    if part == "patch":
        return f"{major}.{minor}.{patch + 1}"
    raise ValueError(f"unsupported version part: {part!r}")


def _replace_once(contents: str, pattern: str, replacement: str, path: Path) -> str:
    updated, count = re.subn(
        pattern, replacement, contents, count=1, flags=re.MULTILINE
    )
    if count != 1:
        raise ValueError(f"could not find one release version in {path}")
    return updated


def bump_version(part: str, root: Path = ROOT) -> str:
    current = release_version(root)
    updated = next_version(current, part)
    cargo_path = root / "Cargo.toml"
    lock_path = root / "Cargo.lock"

    cargo = _replace_once(
        cargo_path.read_text(),
        rf'^version = "{re.escape(current)}"$',
        f'version = "{updated}"',
        cargo_path,
    )
    lock = _replace_once(
        lock_path.read_text(),
        (
            rf'(^\[\[package\]\]\nname = "coval"\nversion = ")'
            rf"{re.escape(current)}" + r'("$)'
        ),
        rf"\g<1>{updated}\g<2>",
        lock_path,
    )

    cargo_path.write_text(cargo)
    lock_path.write_text(lock)
    return updated


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("part", choices=("minor", "patch"))
    parser.add_argument("--root", type=Path, default=ROOT)
    args = parser.parse_args()

    print(bump_version(args.part, args.root))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
