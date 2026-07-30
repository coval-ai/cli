#!/usr/bin/env python3
"""Render the Homebrew formula for one validated Coval CLI release."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

if __package__:
    from .release_version import VERSION_RE
else:
    from release_version import VERSION_RE

SHA256_RE = re.compile(r"^[0-9a-f]{64}$")


def render_formula(version: str, checksums: dict[str, str]) -> str:
    if VERSION_RE.fullmatch(version) is None:
        raise ValueError(f"invalid release version: {version!r}")

    expected_platforms = {"macos_arm64", "macos_x64", "linux_arm64", "linux_x64"}
    if set(checksums) != expected_platforms:
        missing = sorted(expected_platforms - set(checksums))
        unexpected = sorted(set(checksums) - expected_platforms)
        raise ValueError(
            f"invalid checksum platforms; missing={missing}, unexpected={unexpected}"
        )
    for platform, checksum in checksums.items():
        if SHA256_RE.fullmatch(checksum) is None:
            raise ValueError(f"invalid SHA-256 for {platform}: {checksum!r}")

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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("--macos-arm64", required=True)
    parser.add_argument("--macos-x64", required=True)
    parser.add_argument("--linux-arm64", required=True)
    parser.add_argument("--linux-x64", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    formula = render_formula(
        args.version,
        {
            "macos_arm64": args.macos_arm64,
            "macos_x64": args.macos_x64,
            "linux_arm64": args.linux_arm64,
            "linux_x64": args.linux_x64,
        },
    )
    args.output.write_text(formula)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
