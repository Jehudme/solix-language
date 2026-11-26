#!/usr/bin/env python3
"""Unit tests for `solix_scaffold.py` bundled standard library version resolution."""

from __future__ import annotations

import json
import os
from pathlib import Path
import sys
import tempfile
import unittest

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(REPO_ROOT / "crates" / "solix-toolchain" / "assets" / "scripts"))

import solix_scaffold  # noqa: E402


class TestSolixScaffold(unittest.TestCase):
    """Validates decoupled bundled `solix` version resolution in `solix_scaffold.py`."""

    def test_parse_bundled_package_version_prefers_release_and_rejects_invalid(self) -> None:
        sample_manifest = json.dumps(
            {
                "schema_version": 1,
                "packages": [
                    {
                        "name": "solix",
                        "version": "2.2.0-debug",
                        "profile": "debug",
                    },
                    {
                        "name": "solix",
                        "version": "2.2.0",
                        "profile": "release",
                    },
                ],
            }
        )
        self.assertEqual(
            solix_scaffold.parse_bundled_package_version(sample_manifest, "solix"),
            "2.2.0",
        )
        self.assertIsNone(solix_scaffold.parse_bundled_package_version("{bad json", "solix"))
        self.assertIsNone(
            solix_scaffold.parse_bundled_package_version(sample_manifest, "nonexistent")
        )

    def test_generate_project_pins_active_bundled_solix_version(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            bundled_dir = temp_root / "bundled"
            bundled_dir.mkdir(parents=True, exist_ok=True)
            (bundled_dir / "bundled_manifest.json").write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "packages": [
                            {
                                "name": "solix",
                                "version": "9.4.1",
                                "profile": "release",
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )

            project_dir = temp_root / "sample_app"
            previous_env = os.environ.get("SOLIX_BUNDLED_PACKAGES_DIR")
            try:
                os.environ["SOLIX_BUNDLED_PACKAGES_DIR"] = str(bundled_dir)
                solix_scaffold.generate_project(
                    target_path=project_dir,
                    no_native=True,
                )
            finally:
                if previous_env is None:
                    os.environ.pop("SOLIX_BUNDLED_PACKAGES_DIR", None)
                else:
                    os.environ["SOLIX_BUNDLED_PACKAGES_DIR"] = previous_env

            manifest_content = (project_dir / "manifest.yaml").read_text(encoding="utf-8")
            self.assertIn('version: "9.4.1"', manifest_content)
            self.assertNotIn('version: "2.0.1"', manifest_content)

    def test_generate_project_no_std_and_explicit_override(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            no_std_dir = temp_root / "freestanding"
            solix_scaffold.generate_project(
                target_path=no_std_dir,
                no_native=True,
                no_std=True,
            )
            no_std_manifest = (no_std_dir / "manifest.yaml").read_text(encoding="utf-8")
            self.assertNotIn("solix:", no_std_manifest)

            override_dir = temp_root / "pinned"
            solix_scaffold.generate_project(
                target_path=override_dir,
                no_native=True,
                dependencies=["solix@4.5.6"],
            )
            override_manifest = (override_dir / "manifest.yaml").read_text(encoding="utf-8")
            self.assertIn('version: "4.5.6"', override_manifest)


if __name__ == "__main__":
    unittest.main()
