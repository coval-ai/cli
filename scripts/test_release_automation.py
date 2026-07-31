"""Tests for release metadata and Homebrew formula generation."""

import io
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch

from scripts import bump_version
from scripts import release_version
from scripts import render_homebrew_formula


def temporary_release_root(
    cargo_version: str,
    lock_version: str,
    *,
    package_name: str = "coval",
    cargo_prefix: str = "",
) -> tempfile.TemporaryDirectory:
    directory = tempfile.TemporaryDirectory()
    root = Path(directory.name)
    (root / "Cargo.toml").write_text(
        f'{cargo_prefix}[package]\nname = "{package_name}"\n'
        f'version = "{cargo_version}"\n'
    )
    (root / "Cargo.lock").write_text(
        f'[[package]]\nname = "{package_name}"\nversion = "{lock_version}"\n'
    )
    return directory


class ReleaseVersionTests(unittest.TestCase):
    def test_returns_matching_release_metadata(self):
        with temporary_release_root("0.6.0", "0.6.0") as directory:
            metadata = release_version.release_metadata(Path(directory), "v0.6.0")

        self.assertEqual({"version": "0.6.0", "tag": "v0.6.0"}, metadata)

    def test_rejects_manifest_mismatch(self):
        with temporary_release_root("0.6.0", "0.5.0") as directory:
            with self.assertRaisesRegex(ValueError, "Cargo.lock"):
                release_version.release_metadata(Path(directory))

    def test_rejects_tag_mismatch(self):
        with temporary_release_root("0.6.0", "0.6.0") as directory:
            with self.assertRaisesRegex(ValueError, "does not match"):
                release_version.release_metadata(Path(directory), "v0.5.0")

    def test_identifies_non_release_version(self):
        with temporary_release_root("0.6.0-rc.1", "0.6.0-rc.1") as directory:
            with self.assertRaises(release_version.NonReleaseVersionError):
                release_version.release_metadata(Path(directory))

    def test_automatic_release_can_skip_non_release_version(self):
        with temporary_release_root("0.6.0-rc.1", "0.6.0-rc.1") as directory:
            output_path = Path(directory) / "github-output"
            argv = [
                "release_version.py",
                "--root",
                directory,
                "--allow-non-release",
                "--github-output",
                str(output_path),
            ]
            with patch("sys.argv", argv), redirect_stdout(io.StringIO()):
                result = release_version.main()

            self.assertEqual(0, result)
            self.assertEqual("release_candidate=false\n", output_path.read_text())


class BumpVersionTests(unittest.TestCase):
    def test_bumps_minor_version_in_both_manifests(self):
        with temporary_release_root("0.5.0", "0.5.0") as directory:
            root = Path(directory)
            updated = bump_version.bump_version("minor", root)

            self.assertEqual("0.6.0", updated)
            self.assertEqual("0.6.0", release_version.release_version(root))

    def test_bumps_patch_version(self):
        self.assertEqual("0.5.1", bump_version.next_version("0.5.0", "patch"))

    def test_only_bumps_renamed_workspace_package(self):
        prefix = '[dependencies]\nhelper = "0.5.0"\n\n'
        with temporary_release_root(
            "0.5.0",
            "0.5.0",
            package_name="renamed-cli",
            cargo_prefix=prefix,
        ) as directory:
            root = Path(directory)
            bump_version.bump_version("minor", root)

            cargo = (root / "Cargo.toml").read_text()
            self.assertIn('helper = "0.5.0"', cargo)
            self.assertEqual("0.6.0", release_version.release_version(root))


class HomebrewFormulaTests(unittest.TestCase):
    def setUp(self):
        self.checksums = {
            "macos_arm64": "a" * 64,
            "macos_x64": "b" * 64,
            "linux_arm64": "c" * 64,
            "linux_x64": "d" * 64,
        }

    def test_renders_release_urls_and_ruby_interpolation(self):
        formula = render_homebrew_formula.render_formula("0.6.0", self.checksums)

        self.assertIn("releases/download/v0.6.0/coval-macos-arm64.tar.gz", formula)
        self.assertIn('system "#{bin}/coval", "--version"', formula)

    def test_rejects_invalid_checksum(self):
        self.checksums["linux_x64"] = "not-a-checksum"

        with self.assertRaisesRegex(ValueError, "invalid SHA-256"):
            render_homebrew_formula.render_formula("0.6.0", self.checksums)


if __name__ == "__main__":
    unittest.main()
