"""Tests for release metadata and Homebrew formula generation."""

import tempfile
import unittest
from pathlib import Path

from scripts import bump_version
from scripts import release_version
from scripts import render_homebrew_formula


class ReleaseVersionTests(unittest.TestCase):
    def _root(
        self, cargo_version: str, lock_version: str
    ) -> tempfile.TemporaryDirectory:
        directory = tempfile.TemporaryDirectory()
        root = Path(directory.name)
        (root / "Cargo.toml").write_text(
            f'[package]\nname = "coval"\nversion = "{cargo_version}"\n'
        )
        (root / "Cargo.lock").write_text(
            f'[[package]]\nname = "coval"\nversion = "{lock_version}"\n'
        )
        return directory

    def test_returns_matching_release_metadata(self):
        with self._root("0.6.0", "0.6.0") as directory:
            metadata = release_version.release_metadata(Path(directory), "v0.6.0")

        self.assertEqual({"version": "0.6.0", "tag": "v0.6.0"}, metadata)

    def test_rejects_manifest_mismatch(self):
        with self._root("0.6.0", "0.5.0") as directory:
            with self.assertRaisesRegex(ValueError, "Cargo.lock"):
                release_version.release_metadata(Path(directory))

    def test_rejects_tag_mismatch(self):
        with self._root("0.6.0", "0.6.0") as directory:
            with self.assertRaisesRegex(ValueError, "does not match"):
                release_version.release_metadata(Path(directory), "v0.5.0")


class BumpVersionTests(unittest.TestCase):
    def test_bumps_minor_version_in_both_manifests(self):
        with ReleaseVersionTests()._root("0.5.0", "0.5.0") as directory:
            root = Path(directory)
            updated = bump_version.bump_version("minor", root)

            self.assertEqual("0.6.0", updated)
            self.assertEqual("0.6.0", release_version.release_version(root))

    def test_bumps_patch_version(self):
        self.assertEqual("0.5.1", bump_version.next_version("0.5.0", "patch"))


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
