import itertools
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]


class CiGateTests(unittest.TestCase):
    def setUp(self):
        self.workflow = yaml.load(
            (ROOT / ".github/workflows/ci.yml").read_text(), Loader=yaml.BaseLoader
        )
        self.gate = self.workflow["jobs"]["switchboard_gate"]

    def test_pr_and_queue_gate_require_all_existing_checks_and_platforms(self):
        events = self.workflow["on"]
        self.assertIn("pull_request", events)
        self.assertEqual(["checks_requested"], events["merge_group"]["types"])
        self.assertEqual("Switchboard Gate", self.gate["name"])
        self.assertEqual("always()", self.gate["if"])
        self.assertEqual(["check", "build"], self.gate["needs"])
        build = self.workflow["jobs"]["build"]
        self.assertEqual("check", build["needs"])
        self.assertNotIn("continue-on-error", build)
        targets = [entry["target"] for entry in build["strategy"]["matrix"]["include"]]
        self.assertCountEqual(
            [
                "x86_64-unknown-linux-gnu",
                "aarch64-unknown-linux-gnu",
                "x86_64-apple-darwin",
                "aarch64-apple-darwin",
                "x86_64-pc-windows-msvc",
            ],
            targets,
        )
        self.assertEqual(
            {
                "CHECK_RESULT": "${{ needs.check.result }}",
                "BUILD_RESULT": "${{ needs.build.result }}",
            },
            self.gate["steps"][0]["env"],
        )

    def test_actual_gate_script_fails_closed_for_every_non_success_result(self):
        statuses = ["success", "failure", "cancelled", "skipped", None, "", "unknown"]
        script = self.gate["steps"][0]["run"]
        with tempfile.TemporaryDirectory() as directory:
            for check, build in itertools.product(statuses, repeat=2):
                with self.subTest(check=check, build=build):
                    env = dict(os.environ)
                    env["GITHUB_STEP_SUMMARY"] = str(Path(directory) / "summary.md")
                    for name, value in [
                        ("CHECK_RESULT", check),
                        ("BUILD_RESULT", build),
                    ]:
                        env.pop(name, None)
                        if value is not None:
                            env[name] = value
                    result = subprocess.run(
                        ["bash", "-c", script], env=env, capture_output=True, text=True
                    )
                    self.assertEqual(
                        check == "success" and build == "success",
                        result.returncode == 0,
                        result.stdout + result.stderr,
                    )
