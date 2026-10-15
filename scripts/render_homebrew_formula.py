#!/usr/bin/env python3
"""Render the Homebrew formula and tap PR body for one validated CLI release."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

if __package__:
    from .release_version import VERSION_RE
else:
    from release_version import VERSION_RE

SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
RELEASES_URL = "https://github.com/coval-ai/cli/releases"
PLATFORM_ASSETS = {
    "macos_arm64": "coval-macos-arm64.tar.gz",
    "macos_x64": "coval-macos-x64.tar.gz",
    "linux_arm64": "coval-linux-arm64.tar.gz",
    "linux_x64": "coval-linux-x64.tar.gz",
}


def validate_release(version: str, checksums: dict[str, str]) -> None:
    if VERSION_RE.fullmatch(version) is None:
        raise ValueError(f"invalid release version: {version!r}")

    expected_platforms = set(PLATFORM_ASSETS)
    if set(checksums) != expected_platforms:
        missing = sorted(expected_platforms - set(checksums))
        unexpected = sorted(set(checksums) - expected_platforms)
        raise ValueError(
            f"invalid checksum platforms; missing={missing}, unexpected={unexpected}"
        )
    for platform, checksum in checksums.items():
        if SHA256_RE.fullmatch(checksum) is None:
            raise ValueError(f"invalid SHA-256 for {platform}: {checksum!r}")


def render_formula(version: str, checksums: dict[str, str]) -> str:
    validate_release(version, checksums)

    return f'''class Coval < Formula
  desc "CLI for Coval AI agent evaluation platform"
  homepage "https://coval.dev"
  version "{version}"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/coval-ai/cli/releases/download/v{version}/coval-macos-arm64.tar.gz"
      sha256 "{checksums["macos_arm64"]}"
    else
      url "https://github.com/coval-ai/cli/releases/download/v{version}/coval-macos-x64.tar.gz"
      sha256 "{checksums["macos_x64"]}"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/coval-ai/cli/releases/download/v{version}/coval-linux-arm64.tar.gz"
      sha256 "{checksums["linux_arm64"]}"
    else
      url "https://github.com/coval-ai/cli/releases/download/v{version}/coval-linux-x64.tar.gz"
      sha256 "{checksums["linux_x64"]}"
    end
  end

  def install
    bin.install "coval"
  end

  test do
    system "#{{bin}}/coval", "--version"
  end
end
'''


def render_pull_request_body(
    version: str, checksums: dict[str, str], run_url: str
) -> str:
    validate_release(version, checksums)

    download_url = f"{RELEASES_URL}/download/v{version}"
    rows = "\n".join(
        f"| [`{asset}`]({download_url}/{asset}) | `{checksums[platform]}` |"
        for platform, asset in PLATFORM_ASSETS.items()
    )
    return f"""Updates `Formula/coval.rb` to Coval CLI {version}.

Release: {RELEASES_URL}/tag/v{version}
Checksums: {download_url}/SHA256SUMS

| Asset | SHA-256 |
| --- | --- |
{rows}

Opened by the CLI release workflow: {run_url}
"""


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("--macos-arm64", required=True)
    parser.add_argument("--macos-x64", required=True)
    parser.add_argument("--linux-arm64", required=True)
    parser.add_argument("--linux-x64", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--pull-request-body", type=Path)
    parser.add_argument("--run-url")
    args = parser.parse_args()
    if args.pull_request_body and not args.run_url:
        parser.error("--pull-request-body requires --run-url")

    checksums = {
        "macos_arm64": args.macos_arm64,
        "macos_x64": args.macos_x64,
        "linux_arm64": args.linux_arm64,
        "linux_x64": args.linux_x64,
    }
    args.output.write_text(render_formula(args.version, checksums))
    if args.pull_request_body:
        args.pull_request_body.write_text(
            render_pull_request_body(args.version, checksums, args.run_url)
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
