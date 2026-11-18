#!/usr/bin/env python3
"""Comprehensive Governance, Release-Gate, Security & Documentation Test Suite."""

from __future__ import annotations

import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
import zipfile

try:
    import yaml  # type: ignore[import-not-found]
except ImportError:
    yaml = None


SCRIPTS_DIR = Path(__file__).resolve().parent.parent
REPO_ROOT = SCRIPTS_DIR.parent
sys.path.insert(0, str(SCRIPTS_DIR))

import build_docs_site  # noqa: E402
import doc_sync_check  # noqa: E402
import merge_bundled_packages  # noqa: E402
import pr_risk_audit  # noqa: E402
import prepare_release_train  # noqa: E402
import re  # noqa: E402
import sync_github_wiki  # noqa: E402
import verify_docs  # noqa: E402

sys.path.insert(0, str(REPO_ROOT / "crates" / "solix-toolchain" / "assets" / "scripts"))
import solix_build  # noqa: E402


class PrRiskAuditTests(unittest.TestCase):
    """Tests non-blocking sensitive-path risk classification (MS063-T04 / T06)."""

    def test_clean_feature_files_produce_no_risk_alerts_and_exit_zero(self) -> None:
        report = pr_risk_audit.audit_paths(
            ["crates/solix-compilation/src/lib.rs", "tests/sample.slx"]
        )
        self.assertFalse(report["has_sensitive_changes"])
        self.assertEqual(report["labels"], [])

    def test_all_four_risk_tiers_are_classified_accurately(self) -> None:
        report = pr_risk_audit.audit_paths(
            [
                ".github/workflows/ci.yml",
                "docs/guidelines/pr-and-release-governance.md",
                "crates/solix-common/build.rs",
                "crates/solix-runtime/src/native/bridge.rs",
            ]
        )
        self.assertTrue(report["has_sensitive_changes"])
        self.assertEqual(report["labels"], ["sensitive"])
        markdown = pr_risk_audit.render_markdown_report(report)
        self.assertIn(pr_risk_audit.MARKER_COMMENT, markdown)
        self.assertIn("## Sensitive Paths Modified", markdown)
        self.assertNotIn("Maintainer Review Checklist", markdown)
        self.assertNotIn("[!CAUTION]", markdown)
        self.assertIn("ci.yml", markdown)

    def test_cli_invocation_always_exits_zero_even_with_sensitive_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            json_out = tmp / "report.json"
            md_out = tmp / "advisory.md"
            exit_code = pr_risk_audit.main(
                [
                    "--files",
                    ".github/workflows/release.yml",
                    "Cargo.toml",
                    "--json-out",
                    str(json_out),
                    "--markdown-out",
                    str(md_out),
                ]
            )
            self.assertEqual(exit_code, 0)
            payload = json.loads(json_out.read_text(encoding="utf-8"))
            self.assertEqual(payload["labels"], ["sensitive"])
            self.assertIn("Sensitive Paths Modified", md_out.read_text(encoding="utf-8"))


class StaticDocsAndWikiPortalTests(unittest.TestCase):
    """Tests `scripts/build_docs_site.py` and `scripts/sync_github_wiki.py`."""

    def test_build_docs_site_generates_offline_html_and_excludes_internals(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            out_dir = tmp / "site"
            summary = build_docs_site.build_site(
                REPO_ROOT,
                out_dir,
                version_override="9.9.9-test",
            )
            self.assertGreater(summary["page_count"], 50)
            self.assertTrue((out_dir / "index.html").is_file())
            self.assertTrue((out_dir / "guide" / "01_getting_started.html").is_file())
            self.assertTrue((out_dir / "spec" / "tools" / "cli" / "wiki.html").is_file())
            self.assertFalse((out_dir / "internals").exists())
            self.assertFalse((out_dir / "guidelines").exists())

            index_html = (out_dir / "index.html").read_text(encoding="utf-8")
            self.assertIn("v9.9.9-test", index_html)
            self.assertIn("guide/01_getting_started.html", index_html)
            self.assertNotIn("guide/01_getting_started.md", index_html)

            search_js = (out_dir / "assets" / "search_index.js").read_text(encoding="utf-8")
            self.assertTrue(search_js.startswith("window.SOLIX_DOCS_INDEX = "))
            raw_json = search_js[len("window.SOLIX_DOCS_INDEX = ") :].rstrip().rstrip(";")
            indexed_entries = json.loads(raw_json)
            self.assertTrue(
                all(not entry["path"].startswith(("internals", "guidelines")) for entry in indexed_entries),
                "docs/guidelines/ files must never be indexed into solix wiki search_index.js",
            )

            node_bin = shutil.which("node")
            if node_bin:
                proc = subprocess.run(
                    [node_bin, "--check", str(out_dir / "assets" / "app.js")],
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertEqual(proc.returncode, 0, proc.stderr)

    def test_sync_github_wiki_stages_versioned_banner_without_touching_source_docs(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            out_dir = Path(tmpdir) / "wiki_checkout"
            result = sync_github_wiki.stage_github_wiki(
                REPO_ROOT,
                out_dir,
                version_override="1.5.0",
            )
            self.assertGreater(result["pages"], 10)
            home_text = (out_dir / "Home.md").read_text(encoding="utf-8")
            self.assertIn("Solix v1.5.0", home_text)
            self.assertIn("solix wiki", home_text)
            self.assertTrue((out_dir / "_Sidebar.md").is_file())
            self.assertTrue((REPO_ROOT / "docs" / "README.md").is_file())


class SecurityAndPackagingRegressionTests(unittest.TestCase):
    """Tests build-script Zip-Slip guards, archive merging, doc verification, and templates."""

    def test_solix_build_rejects_zip_slip_in_dependency_native_libs(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            store_dir = tmp / "store"
            pkg_dir = store_dir / "evil@1.0.0"
            pkg_dir.mkdir(parents=True)
            evil_archive = pkg_dir / "evil.slxarc"
            with zipfile.ZipFile(evil_archive, "w") as zip_handle:
                zip_handle.writestr(
                    "manifest.toml",
                    "[configuration]\nname = \"evil\"\nversion = \"1.0.0\"\n",
                )
                zip_handle.writestr("lib/../../../escaped.so", b"pwned")

            stage_lib = tmp / "project" / "stage_lib"
            stage_lib.mkdir(parents=True)
            old_pkg_dir = None
            import os

            old_pkg_dir = os.environ.get("SOLIX_PKG_DIR")
            os.environ["SOLIX_PKG_DIR"] = str(store_dir)
            try:
                manifest = {
                    "configuration": {"name": "consumer", "version": "0.1.0"},
                    "profiles": {
                        "release": {
                            "link": "static",
                            "dependencies": {"evil": {"version": "1.0.0"}},
                        }
                    },
                }
                with self.assertRaises(solix_build.BuildError) as ctx:
                    solix_build.stage_dependency_native_libraries(
                        manifest, "release", stage_lib
                    )
                self.assertIn("Unsafe dependency archive path", str(ctx.exception))
                self.assertFalse((tmp / "escaped.so").exists())
            finally:
                if old_pkg_dir is None:
                    os.environ.pop("SOLIX_PKG_DIR", None)
                else:
                    os.environ["SOLIX_PKG_DIR"] = old_pkg_dir

    def test_merge_bundled_packages_combines_triples_and_recomputes_checksums(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            linux_dir = tmp / "bundled-packages-x86_64-unknown-linux-gnu" / "demo-release@1.0.0"
            mac_dir = tmp / "bundled-packages-aarch64-apple-darwin" / "demo-release@1.0.0"
            linux_dir.mkdir(parents=True)
            mac_dir.mkdir(parents=True)
            out_dir = tmp / "universal"

            base_manifest = "[configuration]\nname = \"demo\"\nversion = \"1.0.0\"\n"
            with zipfile.ZipFile(linux_dir / "demo.slxarc", "w") as zip_handle:
                zip_handle.writestr("manifest.toml", base_manifest)
                zip_handle.writestr("lib/x86_64-unknown-linux-gnu/libdemo.so", b"linux-bytes")
            with zipfile.ZipFile(mac_dir / "demo.slxarc", "w") as zip_handle:
                zip_handle.writestr("manifest.toml", base_manifest)
                zip_handle.writestr("lib/aarch64-apple-darwin/libdemo.dylib", b"mac-bytes")

            proc = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPTS_DIR / "merge_bundled_packages.py"),
                    "--input-dirs",
                    str(linux_dir.parent),
                    str(mac_dir.parent),
                    "--output-dir",
                    str(out_dir),
                ],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(proc.returncode, 0, proc.stderr)
            merged_archive = out_dir / "demo-release@1.0.0" / "demo.slxarc"
            self.assertTrue(merged_archive.is_file())
            with zipfile.ZipFile(merged_archive, "r") as zip_handle:
                names = set(zip_handle.namelist())
                self.assertIn("lib/x86_64-unknown-linux-gnu/libdemo.so", names)
                self.assertIn("lib/aarch64-apple-darwin/libdemo.dylib", names)
                merged_manifest = zip_handle.read("manifest.toml").decode("utf-8")
                self.assertIn("[native_library_checksums]", merged_manifest)
                self.assertIn(
                    merge_bundled_packages.compute_sha256_bytes(b"linux-bytes"),
                    merged_manifest,
                )

    def test_issue_templates_and_workflows_parse_as_valid_yaml(self) -> None:
        template_dir = REPO_ROOT / ".github" / "ISSUE_TEMPLATE"
        for yml_file in sorted(template_dir.glob("*.yml")):
            raw_text = yml_file.read_text(encoding="utf-8")
            self.assertNotIn("\t", raw_text, f"{yml_file.name} must not contain tabs")
            if yaml is not None:
                parsed = yaml.safe_load(raw_text)
                self.assertIsInstance(parsed, dict, f"{yml_file} must parse as YAML mapping")
            else:
                self.assertTrue(raw_text.strip(), f"{yml_file.name} must not be empty")

        self.assertTrue((REPO_ROOT / ".github" / "CODEOWNERS").is_file())
        self.assertTrue((REPO_ROOT / ".github" / "dependabot.yml").is_file())
        dependabot_raw = (REPO_ROOT / ".github" / "dependabot.yml").read_text(encoding="utf-8")
        if yaml is not None:
            dependabot = yaml.safe_load(dependabot_raw)
            for update in dependabot.get("updates", []):
                self.assertEqual(update.get("target-branch"), "dev")
        else:
            self.assertIn('target-branch: "dev"', dependabot_raw)
            self.assertNotIn('target-branch: "master"', dependabot_raw)

        sha_pin_re = re.compile(r"^[^@]+@[0-9a-f]{40}$")
        for workflow_file in sorted((REPO_ROOT / ".github" / "workflows").glob("*.yml")):
            wf_text = workflow_file.read_text(encoding="utf-8")
            if yaml is not None:
                parsed_wf = yaml.safe_load(wf_text)
                self.assertIsInstance(parsed_wf, dict, f"{workflow_file} invalid YAML")
                self.assertIn(
                    "permissions",
                    parsed_wf,
                    f"{workflow_file} MUST declare top-level permissions",
                )
                self.assertEqual(parsed_wf["permissions"].get("contents"), "read")
            else:
                self.assertRegex(
                    wf_text,
                    r"(?m)^permissions:\s*\n\s+contents:\s*read\s*$",
                    f"{workflow_file.name} MUST declare top-level permissions: contents: read",
                )
            for raw_line in wf_text.splitlines():
                stripped = raw_line.strip()
                if stripped.startswith(("uses:", "- uses:")):
                    action_spec = stripped.split("uses:", 1)[1].split("#", 1)[0].strip().strip("\"'")
                    self.assertRegex(
                        action_spec,
                        sha_pin_re,
                        f"Unpinned GitHub Action in {workflow_file.name}: {action_spec}",
                    )


class ReleaseTrainVersionCalculationTests(unittest.TestCase):
    """Tests Conventional Commit SemVer inference, label overrides, and Cargo.toml/CHANGELOG updates."""

    def test_conventional_commit_bump_and_label_overrides(self) -> None:
        raw_patch = "sha1\x1ffix(toolchain): handle solix wiki edge case\x1f\x1e"
        raw_feat = "sha2\x1ffeat(toolchain): add offline solix wiki command\x1f\x1e"
        raw_break = "sha3\x1ffeat(runtime)!: change bytecode header\x1fBREAKING CHANGE: recompile\x1e"

        patch_commits = prepare_release_train.parse_commits(raw_patch)
        self.assertEqual(
            prepare_release_train.determine_bump_kind(patch_commits)[0],
            "patch",
        )

        feat_commits = prepare_release_train.parse_commits(raw_patch + raw_feat)
        self.assertEqual(
            prepare_release_train.determine_bump_kind(feat_commits)[0],
            "minor",
        )
        self.assertEqual(
            prepare_release_train.determine_bump_kind(
                feat_commits, pr_labels=["release:major"]
            )[0],
            "major",
        )
        self.assertEqual(
            prepare_release_train.determine_bump_kind(
                feat_commits, override="2.3.4"
            )[0],
            "2.3.4",
        )

        break_commits = prepare_release_train.parse_commits(raw_break)
        self.assertEqual(
            prepare_release_train.determine_bump_kind(break_commits)[0],
            "major",
        )

    def test_update_cargo_toml_and_prepend_changelog(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            cargo_toml = tmp / "Cargo.toml"
            cargo_toml.write_text(
                '[workspace]\nmembers = ["crates/solix-common"]\n\n'
                '[workspace.package]\nversion = "1.4.0"\nedition = "2024"\n\n'
                '[workspace.dependencies]\nsolix-common = { path = "crates/solix-common", version = "1.4.0" }\n',
                encoding="utf-8",
            )
            prepare_release_train.update_cargo_toml_version(cargo_toml, "1.5.0")
            updated_toml = cargo_toml.read_text(encoding="utf-8")
            self.assertIn('version = "1.5.0"', updated_toml)
            self.assertNotIn("1.4.0", updated_toml)

            changelog = tmp / "CHANGELOG.md"
            changelog.write_text(
                "# Changelog\n\n"
                "All notable changes to the Solix Programming Language will be documented in this file.\n\n"
                "## [1.4.0] - 2026-11-17\n\n### Features\n\n- Old item\n",
                encoding="utf-8",
            )
            commits = prepare_release_train.parse_commits(
                "sha1\x1ffeat(toolchain): add offline solix wiki\x1f\x1e"
            )
            section = prepare_release_train.render_changelog_section(
                "1.5.0", commits, date_str="2026-11-18"
            )
            prepare_release_train.prepend_changelog_section(changelog, "1.5.0", section)
            updated_log = changelog.read_text(encoding="utf-8")
            self.assertIn("## [1.5.0] - 2026-11-18", updated_log)
            self.assertIn("## [1.4.0] - 2026-11-17", updated_log)
            self.assertLess(
                updated_log.index("## [1.5.0]"),
                updated_log.index("## [1.4.0]"),
            )

            root_log = tmp / "ROOT_CHANGELOG.md"
            root_log.write_text(
                "# Changelog\n\nAll notable changes...\n\n---\n\n## [1.4.0] - 2026-11-17\n\n- Old\n",
                encoding="utf-8",
            )
            prepare_release_train.prepend_changelog_section(root_log, "1.5.0", section)
            updated_root = root_log.read_text(encoding="utf-8")
            self.assertLess(updated_root.index("---"), updated_root.index("## [1.5.0]"))
            self.assertLess(updated_root.index("## [1.5.0]"), updated_root.index("## [1.4.0]"))
            self.assertIn("crates/solix-ffi/CHANGELOG.md", prepare_release_train.CHANGELOG_FILES)
            self.assertIn("CHANGELOG.md", prepare_release_train.CHANGELOG_FILES)

    def test_doc_sync_ignores_lsp_cargo_toml_and_scripts_are_emoji_free(self) -> None:
        self.assertEqual(doc_sync_check.check_trigger_matrix(["crates/solix-lsp/Cargo.toml"]), [])
        lsp_src_violations = doc_sync_check.check_trigger_matrix(["crates/solix-lsp/src/server.rs"])
        self.assertEqual(len(lsp_src_violations), 1)
        self.assertEqual(
            doc_sync_check.check_trigger_matrix(
                ["crates/solix-lsp/src/server.rs", "docs/wiki/commands/lsp.md"]
            ),
            [],
        )

        forbidden_emojis = ("❌", "✅", "🎉", "🚀", "⚠", "🔴", "🟢", "📉")
        for script_name in (
            "doc_sync_check.py",
            "verify_docs.py",
            "verify_doc_links.py",
            "pr_risk_audit.py",
            "prepare_release_train.py",
        ):
            content = (SCRIPTS_DIR / script_name).read_text(encoding="utf-8")
            for emoji in forbidden_emojis:
                self.assertNotIn(emoji, content, f"{script_name} should remain emoji-free")

    def test_cli_subcommands_are_documented_and_all_docs_pass_gl0011(self) -> None:
        subcommands = verify_docs.extract_active_subcommands(
            REPO_ROOT / "crates" / "solix-toolchain" / "src" / "cli.rs"
        )
        self.assertIn("wiki", subcommands)
        missing_errors = verify_docs.check_subcommands_documented(
            subcommands, REPO_ROOT / "docs", REPO_ROOT
        )
        self.assertEqual(missing_errors, [])
        all_markdown = sorted((REPO_ROOT / "docs").rglob("*.md"))
        gl0011_errors = doc_sync_check.check_single_letter_variables(all_markdown)
        self.assertEqual(gl0011_errors, [])


if __name__ == "__main__":
    unittest.main()
