#!/usr/bin/env python3
"""Comprehensive Governance, Release-Gate, Security & Documentation Test Suite."""

from __future__ import annotations

import json
from pathlib import Path
import platform
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
for _subdir in ("ci", "docs", "release", "bench"):
    sys.path.insert(0, str(SCRIPTS_DIR / _subdir))

import assemble_dev_release_notes  # noqa: E402
import build_docs_site  # noqa: E402
import bump_solix_projects  # noqa: E402
import doc_sync_check  # noqa: E402
import merge_bundled_packages  # noqa: E402
import pr_risk_audit  # noqa: E402
import prepare_release_train  # noqa: E402
import re  # noqa: E402
import render_coverage_summary  # noqa: E402
import solix_bench  # noqa: E402
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

    def test_cli_invocation_step_summary_flag(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            fake_summary_path = tmp / "step_summary.md"
            import os
            original_environment_summary = os.environ.get("GITHUB_STEP_SUMMARY")
            try:
                os.environ["GITHUB_STEP_SUMMARY"] = str(fake_summary_path)
                exit_code_without_flag = pr_risk_audit.main(
                    ["--files", ".github/workflows/release.yml", "Cargo.toml"]
                )
                self.assertEqual(exit_code_without_flag, 0)
                self.assertFalse(fake_summary_path.exists())

                exit_code_with_flag = pr_risk_audit.main(
                    ["--files", ".github/workflows/release.yml", "Cargo.toml", "--step-summary"]
                )
                self.assertEqual(exit_code_with_flag, 0)
                self.assertTrue(fake_summary_path.exists())
                self.assertIn("Sensitive Paths Modified", fake_summary_path.read_text(encoding="utf-8"))
            finally:
                if original_environment_summary is not None:
                    os.environ["GITHUB_STEP_SUMMARY"] = original_environment_summary
                else:
                    os.environ.pop("GITHUB_STEP_SUMMARY", None)


class CoverageSummaryRendererTests(unittest.TestCase):
    """Tests `scripts/ci/render_coverage_summary.py` parsing and markdown rendering."""

    def test_render_coverage_summary_parses_subsystems_and_renders_markdown(self) -> None:
        sample_output = (
            "Filename                                                                        Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover\n"
            "-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------\n"
            "crates/solix-common/src/bytecode/binary.rs                                          761                73    90.41%          32                 1    96.88%         571                18    96.85%           0                 0         -\n"
            "crates/solix-runtime/src/vm/fast.rs                                                4780              1543    67.72%         165                49    70.30%        4089              1063    74.00%           0                 0         -\n"
            "-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------\n"
            "TOTAL                                                                              5541              1616    70.84%         197                50    74.62%        4660              1081    76.80%           0                 0         -\n"
        )
        parsed = render_coverage_summary.parse_coverage_summary(sample_output)
        self.assertIsNotNone(parsed["total"])
        self.assertEqual(parsed["total"]["total_lines"], 4660)
        self.assertEqual(parsed["total"]["missed_lines"], 1081)
        self.assertEqual(parsed["total"]["line_coverage_percentage"], 76.80)

        markdown = render_coverage_summary.render_markdown_coverage_report(parsed, sample_output)
        self.assertIn("## Workspace Code Coverage Report", markdown)
        self.assertIn("Overall Workspace Line Coverage", markdown)
        self.assertIn("76.80%", markdown)
        self.assertIn("crates/solix-common", markdown)
        self.assertIn("crates/solix-runtime", markdown)
        self.assertIn("Subsystem Coverage Breakdown", markdown)

    def test_render_coverage_summary_pr_mode_and_changed_files_delta(self) -> None:
        sample_current = (
            "Filename                                                                        Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover\n"
            "-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------\n"
            "crates/solix-runtime/src/vm/fast.rs                                                4780              1543    67.72%         165                49    70.30%        4000              1000    75.00%           0                 0         -\n"
            "-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------\n"
            "TOTAL                                                                              4780              1543    67.72%         165                49    70.30%        4000              1000    75.00%           0                 0         -\n"
        )
        sample_base = (
            "Filename                                                                        Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover\n"
            "-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------\n"
            "crates/solix-runtime/src/vm/fast.rs                                                4780              1543    67.72%         165                49    70.30%        4000              1200    70.00%           0                 0         -\n"
            "-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------\n"
            "TOTAL                                                                              4780              1543    67.72%         165                49    70.30%        4000              1200    70.00%           0                 0         -\n"
        )
        parsed_current = render_coverage_summary.parse_coverage_summary(sample_current)
        parsed_base = render_coverage_summary.parse_coverage_summary(sample_base)

        markdown = render_coverage_summary.render_markdown_coverage_report(
            parsed_current,
            sample_current,
            base_summary=parsed_base,
            changed_files_list=["crates/solix-runtime/src/vm/fast.rs"],
            is_pr_mode=True,
        )
        self.assertIn("Modified Subsystems Coverage & Delta", markdown)
        self.assertIn("crates/solix-runtime", markdown)
        self.assertIn("IMPROVED", markdown)
        self.assertIn("+5.00%", markdown)

    def test_bump_solix_projects_discovers_packages(self) -> None:
        discovered_projects = bump_solix_projects.discover_solix_projects(REPO_ROOT)
        project_names = [project.project_name for project in discovered_projects]
        self.assertIn("solix", project_names)
        self.assertIn("raylib", project_names)
        self.assertIn("breakout", project_names)

    def test_assemble_dev_release_notes_renders_comprehensive_sections(self) -> None:
        notes_content = assemble_dev_release_notes.assemble_dev_release_notes(
            repository_name="Jehudme/solix-language",
            commit_short_hash="abc1234",
            latest_stable_tag="v1.9.0",
            changelog_preview="- Test commit description",
        )
        self.assertIn("Solix Dev (Nightly) Build — `abc1234`", notes_content)
        self.assertIn("Performance & Cross-Language Benchmarks", notes_content)
        self.assertIn("Cumulative Test Coverage", notes_content)
        self.assertIn("Sensitive Paths Modified Since v1.9.0", notes_content)
        self.assertIn("Commits on `dev` since v1.9.0", notes_content)
        self.assertIn("- Test commit description", notes_content)


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
            self.assertTrue((out_dir / "guide" / "build_system.html").is_file())
            self.assertTrue((out_dir / "spec" / "disassembly-format-slxasm.html").is_file())
            self.assertTrue((out_dir / "spec" / "tools" / "cli" / "wiki.html").is_file())
            self.assertFalse((out_dir / "features").exists())
            self.assertFalse((out_dir / "overview").exists())
            self.assertFalse((out_dir / "internals").exists())
            self.assertFalse((out_dir / "guidelines").exists())

            app_js = (out_dir / "assets" / "app.js").read_text(encoding="utf-8")
            self.assertIn("nav-subgroup", app_js)
            self.assertIn("renderTreeBranch", app_js)

            index_html = (out_dir / "index.html").read_text(encoding="utf-8")
            self.assertIn("v9.9.9-test", index_html)
            self.assertIn('href="guide/index.html"', index_html)
            self.assertNotIn('href="guide/README.md"', index_html)
            guide_index_html = (out_dir / "guide" / "index.html").read_text(encoding="utf-8")
            self.assertIn('href="01_getting_started.html"', guide_index_html)
            self.assertNotIn('href="01_getting_started.md"', guide_index_html)

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
            out_dir.mkdir(parents=True)
            stale_page = out_dir / "Stale--Obsolete-Page.md"
            stale_page.write_text("# Old Stale Page\n", encoding="utf-8")
            result = sync_github_wiki.stage_github_wiki(
                REPO_ROOT,
                out_dir,
                version_override="1.5.0",
            )
            self.assertGreater(result["pages"], 10)
            self.assertFalse(stale_page.exists())
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
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            linux_dir = temporary_root / "bundled-packages-x86_64-unknown-linux-gnu" / "demo-release@1.0.0"
            mac_dir = temporary_root / "bundled-packages-aarch64-apple-darwin" / "demo-release@1.0.0"
            linux_dir.mkdir(parents=True)
            mac_dir.mkdir(parents=True)
            output_directory = temporary_root / "universal"

            base_manifest = "[configuration]\nname = \"demo\"\nversion = \"1.0.0\"\n"
            with zipfile.ZipFile(linux_dir / "demo.slxarc", "w") as zip_handle:
                zip_handle.writestr("manifest.toml", base_manifest)
                zip_handle.writestr("bin/x86_64-unknown-linux-gnu/libdemo.so", b"linux-bytes")
            with zipfile.ZipFile(mac_dir / "demo.slxarc", "w") as zip_handle:
                zip_handle.writestr("manifest.toml", base_manifest)
                zip_handle.writestr("bin/aarch64-apple-darwin/libdemo.dylib", b"mac-bytes")

            merge_process = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPTS_DIR / "release" / "merge_bundled_packages.py"),
                    "--input-dirs",
                    str(linux_dir.parent),
                    str(mac_dir.parent),
                    "--output-dir",
                    str(output_directory),
                ],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(merge_process.returncode, 0, merge_process.stderr)
            merged_archive = output_directory / "demo-release@1.0.0" / "demo.slxarc"
            self.assertTrue(merged_archive.is_file())
            with zipfile.ZipFile(merged_archive, "r") as zip_handle:
                names = set(zip_handle.namelist())
                self.assertIn("bin/x86_64-unknown-linux-gnu/libdemo.so", names)
                self.assertIn("bin/aarch64-apple-darwin/libdemo.dylib", names)
                merged_manifest = zip_handle.read("manifest.toml").decode("utf-8")
                self.assertIn("[native_library_checksums]", merged_manifest)
                self.assertIn(
                    merge_bundled_packages.compute_sha256_bytes(b"linux-bytes"),
                    merged_manifest,
                )

            debug_dir = temporary_root / "bundled-packages-x86_64-unknown-linux-gnu" / "demo-debug@1.0.0"
            debug_dir.mkdir(parents=True)
            with zipfile.ZipFile(debug_dir / "demo.slxarc", "w") as zip_handle:
                zip_handle.writestr("manifest.toml", base_manifest)
            out_release_only = temporary_root / "universal_release_only"
            release_process = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPTS_DIR / "release" / "merge_bundled_packages.py"),
                    "--input-dirs",
                    str(linux_dir.parent),
                    str(mac_dir.parent),
                    "--output-dir",
                    str(out_release_only),
                    "--profiles",
                    "release",
                ],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(release_process.returncode, 0, release_process.stderr)
            self.assertFalse((out_release_only / "demo-debug@1.0.0").exists())
            self.assertTrue((out_release_only / "demo-release@1.0.0").exists())

    def test_merge_bundled_packages_supports_manifest_yaml(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            linux_dir = temporary_root / "bundled-packages-x86_64-unknown-linux-gnu" / "yamldemo-release@2.1.0"
            mac_dir = temporary_root / "bundled-packages-aarch64-apple-darwin" / "yamldemo-release@2.1.0"
            linux_dir.mkdir(parents=True)
            mac_dir.mkdir(parents=True)
            output_directory = temporary_root / "universal_yaml"

            base_manifest = (
                "configuration:\n"
                "  name: yamldemo\n"
                "  version: \"2.1.0\"\n"
                "profiles:\n"
                "  release:\n"
                "    static: true\n"
            )
            with zipfile.ZipFile(linux_dir / "yamldemo.slxarc", "w") as zip_handle:
                zip_handle.writestr("manifest.yaml", base_manifest)
                zip_handle.writestr("bin/x86_64-unknown-linux-gnu/libyamldemo.so", b"yaml-linux")
            with zipfile.ZipFile(mac_dir / "yamldemo.slxarc", "w") as zip_handle:
                zip_handle.writestr("manifest.yaml", base_manifest)
                zip_handle.writestr("bin/aarch64-apple-darwin/libyamldemo.dylib", b"yaml-mac")

            merge_process = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPTS_DIR / "release" / "merge_bundled_packages.py"),
                    "--input-dirs",
                    str(linux_dir.parent),
                    str(mac_dir.parent),
                    "--output-dir",
                    str(output_directory),
                ],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(merge_process.returncode, 0, merge_process.stderr)
            merged_archive = output_directory / "yamldemo-release@2.1.0" / "yamldemo.slxarc"
            self.assertTrue(merged_archive.is_file())
            with zipfile.ZipFile(merged_archive, "r") as zip_handle:
                names = set(zip_handle.namelist())
                self.assertIn("manifest.yaml", names)
                self.assertIn("bin/x86_64-unknown-linux-gnu/libyamldemo.so", names)
                self.assertIn("bin/aarch64-apple-darwin/libyamldemo.dylib", names)
                merged_manifest = zip_handle.read("manifest.yaml").decode("utf-8")
                self.assertIn("native_library_checksums:", merged_manifest)
                self.assertIn("supported_platforms:", merged_manifest)
                self.assertIn(
                    merge_bundled_packages.compute_sha256_bytes(b"yaml-linux"),
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
        local_action_re = re.compile(r"^\./\.github/actions/[A-Za-z0-9_.-]+$")
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
                    if local_action_re.match(action_spec):
                        action_manifest = REPO_ROOT / action_spec.removeprefix("./") / "action.yml"
                        self.assertTrue(
                            action_manifest.is_file(),
                            f"Missing local composite action manifest for {action_spec}",
                        )
                        continue
                    self.assertRegex(
                        action_spec,
                        sha_pin_re,
                        f"Unpinned GitHub Action in {workflow_file.name}: {action_spec}",
                    )

        actions_dir = REPO_ROOT / ".github" / "actions"
        if actions_dir.is_dir():
            for action_file in sorted(actions_dir.glob("*/action.yml")):
                action_text = action_file.read_text(encoding="utf-8")
                if yaml is not None:
                    parsed_action = yaml.safe_load(action_text)
                    self.assertIsInstance(parsed_action, dict, f"{action_file} invalid YAML")
                for raw_line in action_text.splitlines():
                    stripped = raw_line.strip()
                    if stripped.startswith(("uses:", "- uses:")):
                        action_spec = stripped.split("uses:", 1)[1].split("#", 1)[0].strip().strip("\"'")
                        self.assertRegex(
                            action_spec,
                            sha_pin_re,
                            f"Unpinned GitHub Action in {action_file}: {action_spec}",
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
            self.assertIn("ffi/rust/CHANGELOG.md", prepare_release_train.CHANGELOG_FILES)
            self.assertIn("ffi/c/CHANGELOG.md", prepare_release_train.CHANGELOG_FILES)
            self.assertIn("CHANGELOG.md", prepare_release_train.CHANGELOG_FILES)

            ffi_c_dir = tmp / "ffi" / "c" / "include"
            ffi_c_dir.mkdir(parents=True, exist_ok=True)
            (tmp / "ffi" / "c" / "CMakeLists.txt").write_text(
                "cmake_minimum_required(VERSION 3.20)\nproject(solix_ffi VERSION 1.4.0 LANGUAGES C CXX)\n",
                encoding="utf-8",
            )
            (ffi_c_dir / "solix_ffi.h").write_text(
                '#define SOLIX_FFI_VERSION_MAJOR 1\n'
                '#define SOLIX_FFI_VERSION_MINOR 4\n'
                '#define SOLIX_FFI_VERSION_PATCH 0\n'
                '#define SOLIX_FFI_VERSION "1.4.0"\n',
                encoding="utf-8",
            )
            prepare_release_train.update_c_ffi_version(tmp, "1.5.0")
            self.assertIn("VERSION 1.5.0", (tmp / "ffi" / "c" / "CMakeLists.txt").read_text(encoding="utf-8"))
            updated_header = (ffi_c_dir / "solix_ffi.h").read_text(encoding="utf-8")
            self.assertIn("#define SOLIX_FFI_VERSION_MINOR 5", updated_header)
            self.assertIn('#define SOLIX_FFI_VERSION "1.5.0"', updated_header)

    def test_vscode_extension_version_synchronized_during_release(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root_temp = Path(temporary_directory)
            vscode_temp_dir = repo_root_temp / "editors" / "vscode"
            vscode_temp_dir.mkdir(parents=True, exist_ok=True)
            package_json_temp = vscode_temp_dir / "package.json"
            package_lock_temp = vscode_temp_dir / "package-lock.json"

            package_json_temp.write_text(
                json.dumps({"name": "solix", "version": "1.3.1"}, indent=4) + "\n",
                encoding="utf-8",
            )
            package_lock_temp.write_text(
                json.dumps(
                    {
                        "name": "solix",
                        "version": "1.3.1",
                        "packages": {"": {"name": "solix", "version": "1.3.1"}},
                    },
                    indent=4,
                )
                + "\n",
                encoding="utf-8",
            )

            prepare_release_train.update_vscode_extension_version(repo_root_temp, "2.0.1")

            updated_package_data = json.loads(package_json_temp.read_text(encoding="utf-8"))
            self.assertEqual(updated_package_data["version"], "2.0.1")

            updated_lock_data = json.loads(package_lock_temp.read_text(encoding="utf-8"))
            self.assertEqual(updated_lock_data["version"], "2.0.1")
            self.assertEqual(updated_lock_data["packages"][""]["version"], "2.0.1")

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
        for script_rel_path in (
            "ci/doc_sync_check.py",
            "ci/verify_docs.py",
            "ci/verify_doc_links.py",
            "ci/pr_risk_audit.py",
            "release/prepare_release_train.py",
            "release/build_default_libraries.py",
            "release/merge_bundled_packages.py",
            "docs/build_docs_site.py",
            "docs/sync_github_wiki.py",
            "bench/solix_bench.py",
        ):
            content = (SCRIPTS_DIR / script_rel_path).read_text(encoding="utf-8")
            for emoji in forbidden_emojis:
                self.assertNotIn(emoji, content, f"{script_rel_path} should remain emoji-free")

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

    def test_cli_lifecycle_matrix_and_maintenance_mandate_registered(self) -> None:
        lifecycle_script = SCRIPTS_DIR / "tests" / "test_cli_lifecycle_matrix.py"
        self.assertTrue(lifecycle_script.is_file(), "test_cli_lifecycle_matrix.py must exist")

        ci_workflow_content = (REPO_ROOT / ".github" / "workflows" / "ci.yml").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            "test_cli_lifecycle_matrix.py",
            ci_workflow_content,
            "CI workflow must run test_cli_lifecycle_matrix.py",
        )

        test_guideline_content = (
            REPO_ROOT / "docs" / "guidelines" / "authoring-test.md"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "CLI Command & Option Lifecycle Matrix Maintenance Mandate",
            test_guideline_content,
        )

        subcommands = verify_docs.extract_active_subcommands(
            REPO_ROOT / "crates" / "solix-toolchain" / "src" / "cli.rs"
        )
        script_content = lifecycle_script.read_text(encoding="utf-8")
        for subcommand in subcommands:
            self.assertIn(
                f'"{subcommand}"',
                script_content,
                f"test_cli_lifecycle_matrix.py must execute subcommand '{subcommand}'",
            )

    def test_verify_docs_error_cli_ffi_and_hygiene_gates(self) -> None:
        # Real repo must pass all automated parity and hygiene gates cleanly.
        self.assertEqual(verify_docs.check_cli_flag_parity(REPO_ROOT / "crates" / "solix-toolchain" / "src" / "cli.rs", REPO_ROOT / "docs"), [])
        self.assertEqual(verify_docs.check_error_code_parity(REPO_ROOT, REPO_ROOT / "docs"), [])
        self.assertEqual(verify_docs.check_diagnostic_message_hygiene(REPO_ROOT), [])
        self.assertEqual(verify_docs.check_ffi_symbol_parity(REPO_ROOT, REPO_ROOT / "docs"), [])

        # Synthetic negative cases must be rejected cleanly.
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            (root / "crates" / "solix-compilation" / "src" / "diagnostics").mkdir(parents=True)
            (root / "crates" / "solix-common" / "src" / "diagnostics").mkdir(parents=True)
            (root / "crates" / "solix-toolchain" / "src").mkdir(parents=True)
            (root / "crates" / "solix-runtime" / "src" / "native").mkdir(parents=True)
            (root / "ffi" / "c" / "include").mkdir(parents=True)
            (root / "docs" / "wiki" / "errors").mkdir(parents=True)
            (root / "docs" / "wiki" / "commands").mkdir(parents=True)
            (root / "docs" / "wiki" / "ffi").mkdir(parents=True)
            (root / "docs" / "spec" / "tools" / "cli").mkdir(parents=True)

            # 1. Missing error code, stale doc error code, duplicate formal code, and {:?} / manifest.toml leak
            (root / "crates" / "solix-compilation" / "src" / "diagnostics" / "binding.rs").write_text(
                '#[error("Bad: {candidates:?}")]\n#[diagnostic(code(solix::b), help("Check manifest.toml"))]\nSelf::Missing => "E0299"\nSelf::Dup => "E0299"\n',
                encoding="utf-8",
            )
            (root / "docs" / "wiki" / "errors" / "binder-errors.md").write_text(
                "### `E0002`: Removed Code\n",
                encoding="utf-8",
            )
            err_parity = verify_docs.check_error_code_parity(root, root / "docs")
            self.assertTrue(any("Duplicate formal diagnostic code 'E0299'" in msg for msg in err_parity))
            self.assertTrue(any("Active diagnostic code 'E0299'" in msg for msg in err_parity))
            self.assertTrue(any("Stale diagnostic code 'E0002'" in msg for msg in err_parity))

            hygiene_errs = verify_docs.check_diagnostic_message_hygiene(root)
            self.assertTrue(any("Debug formatting" in msg for msg in hygiene_errs))
            self.assertTrue(any("manifest.toml" in msg for msg in hygiene_errs))

            # 2. Undocumented CLI flag
            (root / "crates" / "solix-toolchain" / "src" / "cli.rs").write_text(
                "pub struct RunArgs {\n    #[arg(long)]\n    pub cycle_threshold: usize,\n}\n",
                encoding="utf-8",
            )
            (root / "docs" / "spec" / "tools" / "cli" / "run.md").write_text("# empty\n", encoding="utf-8")
            (root / "docs" / "wiki" / "commands" / "run.md").write_text("# empty\n", encoding="utf-8")
            cli_errs = verify_docs.check_cli_flag_parity(
                root / "crates" / "solix-toolchain" / "src" / "cli.rs", root / "docs"
            )
            self.assertEqual(len(cli_errs), 2)
            self.assertTrue(all("--cycle-threshold" in msg for msg in cli_errs))

            # 3. Missing FFI symbol & stale term pattern
            (root / "crates" / "solix-runtime" / "src" / "native" / "ffi.rs").write_text(
                'pub unsafe extern "C" fn solix_vm_get_field_i32() {}\n', encoding="utf-8"
            )
            (root / "docs" / "wiki" / "ffi" / "c-cpp-sdk.md").write_text(
                "Uses `solix_vm_get_field` on Solix v2.0\n", encoding="utf-8"
            )
            ffi_errs = verify_docs.check_ffi_symbol_parity(root, root / "docs")
            self.assertTrue(any("solix_vm_get_field_i32" in msg for msg in ffi_errs))
            term_errs = verify_docs.check_deprecated_terms(
                [root / "docs" / "wiki" / "ffi" / "c-cpp-sdk.md"]
            )
            self.assertTrue(any("solix_vm_get_field" in msg for msg in term_errs))
            self.assertTrue(any("Solix v2.0" in msg for msg in term_errs))

        # New Trigger Matrix rules detect unsynced diagnostic/extension changes
        diag_violations = doc_sync_check.check_trigger_matrix(
            ["crates/solix-compilation/src/diagnostics/binding.rs"]
        )
        self.assertTrue(any("Diagnostics & Error Codes" in msg for msg in diag_violations))
        vscode_violations = doc_sync_check.check_trigger_matrix(
            ["editors/vscode/src/extension.ts"]
        )
        self.assertTrue(any("VS Code Extension" in msg for msg in vscode_violations))


class SolixBenchCliTests(unittest.TestCase):
    """Tests multi-baseline comparison and cross-language reporting in `scripts/bench/solix_bench.py`."""

    def test_multi_baseline_compare_and_cross_language_markdown(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            b1 = tmp / "v1_4_0.json"
            b2 = tmp / "v1_5_0.json"
            b3 = tmp / "v1_6_0.json"
            md_out = tmp / "multi.md"
            json_out = tmp / "multi.json"

            b1.write_text(
                json.dumps(
                    {
                        "metadata": {"label": "v1.4.0"},
                        "workloads": {
                            "01_fib": {"solix_fast": {"median_ms": 200.0, "rss_kb": 12000}}
                        },
                    }
                ),
                encoding="utf-8",
            )
            b2.write_text(
                json.dumps(
                    {
                        "metadata": {"label": "v1.5.0"},
                        "workloads": {
                            "01_fib": {"solix_fast": {"median_ms": 150.0, "rss_kb": 11000}}
                        },
                    }
                ),
                encoding="utf-8",
            )
            b3.write_text(
                json.dumps(
                    {
                        "metadata": {"label": "v1.6.0"},
                        "workloads": {
                            "01_fib": {
                                "solix_fast": {"median_ms": 100.0, "rss_kb": 10000},
                                "python3": {"median_ms": 400.0, "rss_kb": 20000},
                                "lua": {"median_ms": 120.0, "rss_kb": 4000},
                                "java": {"median_ms": 80.0, "rss_kb": 35000},
                            }
                        },
                    }
                ),
                encoding="utf-8",
            )

            proc = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPTS_DIR / "bench" / "solix_bench.py"),
                    "compare",
                    str(b1),
                    str(b2),
                    str(b3),
                    "--cross-language",
                    "--markdown-output",
                    str(md_out),
                    "--json-output",
                    str(json_out),
                ],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(proc.returncode, 0, msg=proc.stderr)
            self.assertIn("v1.4.0", proc.stdout)
            self.assertIn("2.00x", proc.stdout)

            rendered_md = md_out.read_text(encoding="utf-8")
            self.assertIn("### Benchmark Environment", rendered_md)
            self.assertIn("### Multi-Baseline Benchmark Comparison", rendered_md)
            self.assertIn("### Cross-Language Comparison", rendered_md)
            self.assertIn(
                "| Workload | Solix VM (ms) | Python 3 (ms) | Lua (ms) | LuaJIT (ms) | C -O3 (ms) | Java (ms) |",
                rendered_md,
            )
            self.assertIn("80.00", rendered_md)
            self.assertNotIn("vs Python", rendered_md)

            parsed_json = json.loads(json_out.read_text(encoding="utf-8"))
            self.assertEqual(parsed_json["labels"], ["v1.4.0", "v1.5.0", "v1.6.0"])
            self.assertEqual(parsed_json["summary"]["geomean_vs_first"], [1.0, 1.33, 2.0])

    def test_solix_bench_telemetry_and_scaling_flags(self) -> None:
        """Tests that `solix_bench run` captures hardware telemetry and supports iteration scaling."""
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            mock_solix = temp_path / ("solix.cmd" if platform.system() == "Windows" else "solix")
            mock_solix_py = temp_path / "mock_solix.py"
            mock_solix_py.write_text(
                "import sys, pathlib\n"
                "arguments = sys.argv[1:]\n"
                "if arguments and arguments[0] == 'compile':\n"
                "    for index, token in enumerate(arguments):\n"
                "        if token == '-o' and index + 1 < len(arguments):\n"
                "            pathlib.Path(arguments[index + 1]).write_bytes(b'SLXB_MOCK')\n"
                "    print('Compiled mock')\n"
                "    sys.exit(0)\n"
                "print('RESULT: 317811')\n"
                "sys.exit(0)\n",
                encoding="utf-8",
            )
            if platform.system() == "Windows":
                mock_solix.write_text(
                    f"@echo off\r\n\"{sys.executable}\" \"{mock_solix_py}\" %*\r\n",
                    encoding="utf-8",
                )
            else:
                mock_solix.write_text(
                    f"#!/bin/sh\nexec \"{sys.executable}\" \"{mock_solix_py}\" \"$@\"\n",
                    encoding="utf-8",
                )
                mock_solix.chmod(0o755)

            bench_proc = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPTS_DIR / "bench" / "solix_bench.py"),
                    "run",
                    "--solix-bin",
                    str(mock_solix),
                    "--workload",
                    "01_fib",
                    "--solix-only",
                    "--scale-iterations",
                    "2",
                    "--warmup",
                    "1",
                    "--iterations",
                    "2",
                    "--json",
                ],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(bench_proc.returncode, 0, msg=bench_proc.stderr)
            benchmark_payload = json.loads(bench_proc.stdout)
            self.assertIn("metadata", benchmark_payload)
            self.assertIn("telemetry", benchmark_payload["metadata"])
            self.assertIn("cpu_model", benchmark_payload["metadata"]["telemetry"])
            self.assertIn("cpu_cores", benchmark_payload["metadata"]["telemetry"])
            self.assertIn("total_ram_mb", benchmark_payload["metadata"]["telemetry"])
            self.assertEqual(benchmark_payload["metadata"]["scale_iterations"], 2)
            self.assertTrue(benchmark_payload["metadata"]["precompiled"])
            self.assertIn("01_fib_recursive", benchmark_payload["workloads"])
            solix_stats = benchmark_payload["workloads"]["01_fib_recursive"]["solix_fast"]
            self.assertIn("compile_ms", solix_stats)
            self.assertIn("cold_start_ms", solix_stats)
            self.assertGreater(solix_stats["compile_ms"], 0.0)

    def test_solix_bench_java_engine_resolution_and_compilation(self) -> None:
        """Tests that solix_bench resolves javac/java and compiles Java workloads."""
        benchmark_runner = solix_bench.BenchmarkRunner(REPO_ROOT)
        resolved_javac = benchmark_runner.resolve_engine("javac")
        resolved_java = benchmark_runner.resolve_engine("java")
        if not resolved_javac or not resolved_java:
            self.skipTest("javac or java not available on host system")

        fibonacci_workload_dir = REPO_ROOT / "benchmarks" / "workloads" / "01_fib_recursive"
        java_files = list(fibonacci_workload_dir.glob("*.java"))
        self.assertTrue(java_files, "Expected Fib.java to exist in 01_fib_recursive")

        compiled_target = benchmark_runner.compile_java_workload(java_files[0])
        self.assertIsNotNone(compiled_target)
        output_directory, main_class_name = compiled_target
        self.assertEqual(main_class_name, "Fib")
        self.assertTrue((output_directory / "Fib.class").is_file())

        exit_code, standard_output, _, _ = benchmark_runner.execute_command(
            [resolved_java, "-cp", str(output_directory), main_class_name]
        )
        self.assertEqual(exit_code, 0)
        self.assertIn("RESULT: 2178309", standard_output)

    def test_solix_bench_lua_engine_resolution_and_luac_compilation(self) -> None:
        """Tests that solix_bench resolves luac and precompiles Lua workloads to .luac."""
        benchmark_runner = solix_bench.BenchmarkRunner(REPO_ROOT)
        resolved_luac = benchmark_runner.resolve_engine("luac")
        resolved_lua = benchmark_runner.resolve_engine("lua")
        if not resolved_luac or not resolved_lua:
            self.skipTest("luac or lua not available on host system")

        fibonacci_workload_dir = REPO_ROOT / "benchmarks" / "workloads" / "01_fib_recursive"
        lua_files = list(fibonacci_workload_dir.glob("*.lua"))
        self.assertTrue(lua_files, "Expected fib.lua to exist in 01_fib_recursive")

        compiled_luac = benchmark_runner.compile_lua_workload(lua_files[0])
        self.assertIsNotNone(compiled_luac)
        self.assertTrue(compiled_luac.is_file())
        self.assertEqual(compiled_luac.suffix, ".luac")

        exit_code, standard_output, _, _ = benchmark_runner.execute_command(
            [resolved_lua, str(compiled_luac)]
        )
        self.assertEqual(exit_code, 0)
        self.assertIn("RESULT: 2178309", standard_output)

    def test_execute_command_measures_independent_child_memory(self) -> None:
        benchmark_runner = solix_bench.BenchmarkRunner(REPO_ROOT)
        exit_code, standard_output, duration_milliseconds, peak_rss_kilobytes = (
            benchmark_runner.execute_command(
                ["python3", "-c", "import sys; sys.stdout.write('memory_isolation_test')"]
            )
        )
        self.assertEqual(exit_code, 0)
        self.assertEqual(standard_output, "memory_isolation_test")
        self.assertGreater(duration_milliseconds, 0.0)
        self.assertGreaterEqual(peak_rss_kilobytes, 0)

    def test_render_markdown_comparison_table_includes_column_glossary(self) -> None:
        sample_comparison = {
            "workloads": [
                {
                    "workload": "01_fib_recursive",
                    "previous_ms": 10.0,
                    "current_ms": 9.5,
                    "previous_compile_ms": 4.5,
                    "current_compile_ms": 4.2,
                    "compile_diff_ms": -0.3,
                    "previous_rss_kb": 10240,
                    "current_rss_kb": 9012,
                    "speedup_pct": 5.0,
                    "speedup_ratio": 1.05,
                    "memory_diff_mb": -1.2,
                    "badge": "FASTER",
                }
            ],
            "summary": {
                "geometric_mean_speedup": 1.05,
                "total_memory_diff_mb": -1.2,
                "faster_count": 1,
                "stable_count": 0,
                "regressed_count": 0,
            },
        }
        markdown = solix_bench.render_markdown_comparison_table(
            sample_comparison,
            previous_tag_label="v1.9.0",
            current_tag_label="PR #182",
        )
        self.assertIn("### Benchmark Environment", markdown)
        self.assertIn(
            "| Workload | Prev VM (ms) | Curr VM (ms) | VM Speedup | Prev Compile (ms) | Curr Compile (ms) | Compile Diff | Prev RSS (KB) | Curr RSS (KB) | RSS Diff | Status |",
            markdown,
        )
        self.assertIn("Column Glossary", markdown)
        self.assertIn("**Workload:**", markdown)
        self.assertIn("**Prev VM (ms) / Curr VM (ms):**", markdown)
        self.assertIn("**VM Speedup:**", markdown)
        self.assertIn("**Compile Diff:**", markdown)
        self.assertIn("**RSS Diff:**", markdown)

    def test_ci_workflow_removes_sensitive_label_and_includes_pr_benchmark(self) -> None:
        ci_text = (REPO_ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
        self.assertIn("--remove-label", ci_text)
        self.assertIn("pr-benchmark:", ci_text)
        self.assertIn("<!-- solix-pr-benchmark -->", ci_text)
        self.assertIn("<!-- solix-pr-coverage -->", ci_text)
        self.assertIn("Publish benchmark report to Step Summary", ci_text)
        self.assertIn(
            "pr-benchmark:\n    name: PR Performance Benchmark (vs dev)\n    if: github.event_name == 'pull_request' && github.base_ref == 'dev'\n",
            ci_text,
        )
        self.assertNotIn("  push:\n    branches:\n      - dev", ci_text)
        self.assertIn("key: solix-dev-bench-bin-", ci_text)
        self.assertIn("CARGO_TARGET_DIR=", ci_text)
        self.assertIn("./.github/actions/solix-setup", ci_text)
        self.assertNotIn("EmbarkStudios/cargo-deny-action", ci_text)
        self.assertIn("cargo-deny-0.19.0", ci_text)
        self.assertIn("codeql:", ci_text)
        self.assertIn("staged-offline-docs", ci_text)
        self.assertFalse((REPO_ROOT / ".github" / "workflows" / "codeql.yml").exists())

        setup_action_text = (
            REPO_ROOT / ".github" / "actions" / "solix-setup" / "action.yml"
        ).read_text(encoding="utf-8")
        self.assertIn("bundled-libs-key-prefix", setup_action_text)

        dev_release_text = (REPO_ROOT / ".github" / "workflows" / "dev-release.yml").read_text(
            encoding="utf-8"
        )
        self.assertIn("solix-bundled-dev-all", dev_release_text)
        self.assertIn("key: solix-ci-llvm-cov-0.9.1", dev_release_text)

    def test_pr_discussion_sync_workflow_handles_artifacts_and_comments(self) -> None:
        ci_text = (REPO_ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
        self.assertIn("name: pr-risk-audit", ci_text)
        self.assertIn("name: pr-benchmark-report", ci_text)
        self.assertIn("name: pr-coverage-report", ci_text)

        sync_workflow_path = REPO_ROOT / ".github" / "workflows" / "pr-discussion-sync.yml"
        self.assertTrue(sync_workflow_path.is_file())
        sync_text = sync_workflow_path.read_text(encoding="utf-8")
        self.assertIn("workflow_run:", sync_text)
        self.assertIn("Solix Continuous Integration", sync_text)
        self.assertIn("gh run download", sync_text)
        self.assertIn("<!-- solix-pr-risk-audit -->", sync_text)
        self.assertIn("<!-- solix-pr-benchmark -->", sync_text)
        self.assertIn("<!-- solix-pr-coverage -->", sync_text)

    def test_release_workflow_uses_single_runner_universal_docker_builds(self) -> None:
        release_text = (REPO_ROOT / ".github" / "workflows" / "release.yml").read_text(
            encoding="utf-8"
        )
        self.assertIn("Build Universal Bundled Libraries (Docker Containerized)", release_text)
        self.assertNotIn("merge-universal-bundled-packages:", release_text)
        self.assertNotIn("merge_bundled_packages.py", release_text)
        self.assertNotIn("runner: macos-latest", release_text)
        self.assertIn("name: bundled-packages-universal", release_text)
        for required_triple in (
            "x86_64-unknown-linux-gnu",
            "aarch64-unknown-linux-gnu",
            "x86_64-pc-windows-gnu",
            "x86_64-apple-darwin",
            "aarch64-apple-darwin",
        ):
            self.assertIn(required_triple, release_text)

        for crate_manifest in (
            "crates/solix-common/Cargo.toml",
            "crates/solix-compilation/Cargo.toml",
            "crates/solix-runtime/Cargo.toml",
        ):
            manifest_text = (REPO_ROOT / crate_manifest).read_text(encoding="utf-8")
            self.assertIn("version.workspace = true", manifest_text, crate_manifest)

        ffi_manifest_text = (REPO_ROOT / "ffi/rust/Cargo.toml").read_text(encoding="utf-8")
        self.assertNotIn(".workspace = true", ffi_manifest_text)
        sys.path.insert(0, str(SCRIPTS_DIR / "release"))
        try:
            import prepare_release_train  # noqa: E402

            with tempfile.TemporaryDirectory() as tmp_dir:
                sandbox = Path(tmp_dir)
                (sandbox / "ffi" / "rust").mkdir(parents=True)
                (sandbox / "ffi" / "rust" / "Cargo.toml").write_text(
                    ffi_manifest_text, encoding="utf-8"
                )
                prepare_release_train.update_rust_ffi_version(sandbox, "9.8.7")
                bumped = (sandbox / "ffi" / "rust" / "Cargo.toml").read_text(encoding="utf-8")
                self.assertIn('version = "9.8.7"', bumped)
        finally:
            if str(SCRIPTS_DIR / "release") in sys.path:
                sys.path.remove(str(SCRIPTS_DIR / "release"))

        self.assertIn("Verified release version parity:", release_text)

        for cross_toml_rel in (
            "libraries/raylib/natives/raylib/Cross.toml",
            "libraries/solixlib/natives/solixlib_native/Cross.toml",
        ):
            cross_text = (REPO_ROOT / cross_toml_rel).read_text(encoding="utf-8")
            self.assertIn('image = "solix/apple-darwin-cross:15.5-ubuntu"', cross_text)
            self.assertNotIn("ghcr.io/cross-rs/x86_64-apple-darwin:edge", cross_text)
            self.assertNotIn("ghcr.io/cross-rs/aarch64-apple-darwin:edge", cross_text)

        sys.path.insert(0, str(SCRIPTS_DIR / "release"))
        try:
            import build_default_libraries  # noqa: E402

            self.assertTrue(build_default_libraries.is_cross_target_triple("linux,windows"))
            self.assertTrue(build_default_libraries.is_cross_target_triple("macos"))
            self.assertTrue(build_default_libraries.is_cross_target_triple("all"))
            self.assertFalse(build_default_libraries.is_cross_target_triple("host"))
            self.assertFalse(build_default_libraries.is_cross_target_triple(None))
        finally:
            if str(SCRIPTS_DIR / "release") in sys.path:
                sys.path.remove(str(SCRIPTS_DIR / "release"))


if __name__ == "__main__":
    unittest.main()
