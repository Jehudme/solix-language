#!/usr/bin/env python3
"""Non-blocking Sensitive-Path Risk Auditor for Solix Pull Requests (GL-0014).

Detects modifications to CI workflows, engineering guidelines, build/supply-chain
files, and runtime security/FFI boundaries. Writes GitHub annotations, step summaries,
JSON reports, and sticky PR advisory comments without failing valid PRs (always exits 0).
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
import os
from pathlib import Path
import re
import subprocess
import sys


MARKER_COMMENT = "<!-- solix-pr-risk-audit -->"


@dataclass(frozen=True)
class RiskTier:
    """Defines a sensitive-path governance category and reviewer checklist."""

    tier_id: str
    label: str
    title: str
    severity: str
    description: str
    patterns: tuple[re.Pattern[str], ...]
    checklist: tuple[str, ...]


RISK_TIERS: tuple[RiskTier, ...] = (
    RiskTier(
        tier_id="ci-workflow",
        label="risk:ci-workflow",
        title="CI Workflows, Actions & Repository Automation",
        severity="HIGH",
        description="May execute privileged jobs or access CI tokens (`RELEASE_TOKEN`, `CARGO_REGISTRY_TOKEN`, `GITHUB_TOKEN`).",
        patterns=(
            re.compile(r"^\.github/"),
        ),
        checklist=(
            "Verify NO untrusted PR head checkout occurs inside `pull_request_target`.",
            "Confirm top-level `permissions:` remain least-privilege (`contents: read` by default).",
            "Ensure all third-party Actions remain pinned to full 40-char commit SHAs.",
            "Audit shell `run:` blocks for `${{ github.event.* }}` script injection vectors.",
        ),
    ),
    RiskTier(
        tier_id="guidelines",
        label="risk:guidelines",
        title="Engineering Guidelines & Pre-Merge Guardrails",
        severity="HIGH",
        description="Alters normative engineering rules (`GL-*`), contributor docs, or automated CI/doc enforcement scripts.",
        patterns=(
            re.compile(r"^docs/internals/guidelines/"),
            re.compile(r"^CONTRIBUTING\.md$"),
            re.compile(r"^scripts/(doc_sync_check|verify_docs|verify_doc_links|pr_risk_audit|check_release_pr_source)\.py$"),
        ),
        checklist=(
            "Confirm `GL-0011` (`#![forbid(unsafe_code)]`, zero single-letter variables) and test gates are not silently weakened.",
            "Verify `dev -> master` two-wave branch rules and CODEOWNERS protections remain intact.",
            "Check whether `TRIGGER_MATRIX` or reviewer guidance updates are consistent across `CONTRIBUTING.md` and `GL-0014`.",
        ),
    ),
    RiskTier(
        tier_id="supply-chain",
        label="risk:supply-chain",
        title="Supply Chain, Build Scripts & Packaged Assets",
        severity="ELEVATED",
        description="Executes build-time code (`build.rs`, `.py`) or changes crate dependencies and binary release assets.",
        patterns=(
            re.compile(r"^Cargo\.(toml|lock)$"),
            re.compile(r"^deny\.toml$"),
            re.compile(r"^release-plz\.toml$"),
            re.compile(r"^cliff\.toml$"),
            re.compile(r"^scripts/"),
            re.compile(r"(^|/)build\.rs$"),
            re.compile(r"^libraries/[^/]+/scripts/"),
            re.compile(r"^crates/solix-toolchain/assets/"),
        ),
        checklist=(
            "Review newly added or bumped crates in `Cargo.toml`/`Cargo.lock` (`cargo-deny` & `cargo-audit` clean).",
            "Inspect `build.rs` and `.py` build scripts for arbitrary network fetches or unsafe path traversal.",
            "Confirm staged assets in `crates/solix-toolchain/assets/` exclude secret tokens and unintended binaries.",
        ),
    ),
    RiskTier(
        tier_id="runtime-security",
        label="risk:runtime-security",
        title="Runtime Security, Archive Extraction & FFI Boundaries",
        severity="HIGH",
        description="Touches `.slxarc` extraction path safety, SHA-256 checksum enforcement, native plugins, or `solix-ffi`.",
        patterns=(
            re.compile(r"^crates/solix-common/src/(project|hash)\.rs$"),
            re.compile(r"^crates/solix-runtime/src/native/"),
            re.compile(r"^crates/solix-ffi/"),
        ),
        checklist=(
            "Verify `.slxarc` archive extraction rejects `..`, absolute paths, and backslash traversal (`Zip-Slip`).",
            "Confirm native shared library SHA-256 verification cannot be skipped in release mode.",
            "Audit every `unsafe` block inside `crates/solix-ffi/` for null-pointer checks and ABI lifetime safety.",
        ),
    ),
)


def normalize_rel_path(raw_path: str) -> str:
    """Normalizes repository-relative POSIX path strings."""
    cleaned = raw_path.strip().replace("\\", "/")
    while cleaned.startswith("./"):
        cleaned = cleaned[2:]
    return cleaned


def collect_changed_files_from_git(base_ref: str) -> list[str]:
    """Collects modified file paths relative to `base_ref` using git diff with safe fallbacks."""
    candidate_base = "HEAD~1" if set(base_ref) == {"0"} else base_ref
    try:
        diff_proc = subprocess.run(
            ["git", "diff", "--name-only", f"{candidate_base}...HEAD"],
            capture_output=True,
            text=True,
            check=True,
        )
        files = [normalize_rel_path(line) for line in diff_proc.stdout.splitlines() if line.strip()]
        if files:
            return files
        direct_proc = subprocess.run(
            ["git", "diff", "--name-only", candidate_base, "HEAD"],
            capture_output=True,
            text=True,
            check=True,
        )
        return [normalize_rel_path(line) for line in direct_proc.stdout.splitlines() if line.strip()]
    except subprocess.CalledProcessError:
        try:
            tree_proc = subprocess.run(
                ["git", "diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"],
                capture_output=True,
                text=True,
                check=True,
            )
            return [normalize_rel_path(line) for line in tree_proc.stdout.splitlines() if line.strip()]
        except subprocess.CalledProcessError:
            return []


def audit_paths(changed_paths: list[str]) -> dict[str, object]:
    """Classifies changed file paths across governance risk tiers."""
    normalized = sorted({normalize_rel_path(item) for item in changed_paths if item.strip()})
    findings: list[dict[str, object]] = []
    labels: list[str] = []

    for tier in RISK_TIERS:
        matched_files = [
            file_path
            for file_path in normalized
            if any(pattern.search(file_path) for pattern in tier.patterns)
        ]
        if matched_files:
            labels.append(tier.label)
            findings.append(
                {
                    "tier_id": tier.tier_id,
                    "label": tier.label,
                    "title": tier.title,
                    "severity": tier.severity,
                    "description": tier.description,
                    "files": matched_files,
                    "checklist": list(tier.checklist),
                }
            )

    return {
        "has_sensitive_changes": bool(findings),
        "total_changed_files": len(normalized),
        "labels": labels,
        "findings": findings,
    }


def render_markdown_report(report: dict[str, object]) -> str:
    """Renders a GitHub-flavored Markdown advisory for PR comments and job summaries."""
    findings = report["findings"]
    if not findings:
        return (
            f"{MARKER_COMMENT}\n"
            "## Solix Sensitive-Path Risk Audit\n\n"
            f"Scanned **{report['total_changed_files']}** changed file(s). "
            "No CI, guideline, supply-chain, or runtime security paths were modified.\n"
        )

    lines: list[str] = [
        MARKER_COMMENT,
        "## Solix Sensitive-Path Reviewer Advisory (Non-Blocking)",
        "",
        "> [!CAUTION]",
        "> This PR modifies sensitive repository infrastructure, governance rules, build scripts, or security-critical runtime boundaries. **This notice does not automatically fail or block valid contributions** — it highlights areas requiring careful human maintainer (`@Jehudme`) inspection.",
        "",
        "| Risk Tier | Label | Severity | Modified Sensitive Files |",
        "| :--- | :--- | :---: | :--- |",
    ]

    for finding in findings:
        file_list = "<br>".join(f"`{path}`" for path in finding["files"])
        lines.append(
            f"| **{finding['title']}** | `{finding['label']}` | **{finding['severity']}** | {file_list} |"
        )

    lines.append("")
    lines.append("### Maintainer Review Checklist")
    lines.append("")
    for finding in findings:
        lines.append(f"**{finding['title']} (`{finding['label']}`)** — {finding['description']}")
        for item in finding["checklist"]:
            lines.append(f"- [ ] {item}")
        lines.append("")

    return "\n".join(lines).rstrip() + "\n"


def emit_github_annotations(report: dict[str, object]) -> None:
    """Emits non-blocking GitHub Actions warning annotations for each sensitive file."""
    for finding in report["findings"]:
        label = finding["label"]
        title = finding["title"]
        for file_path in finding["files"]:
            print(
                f"::warning file={file_path},title={title} ({label})::"
                f"Sensitive file modified ({finding['severity']} review tier). "
                "Verify maintainer CODEOWNER approval and security checklist."
            )


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    """Parses CLI flags for PR risk audit execution."""
    parser = argparse.ArgumentParser(
        description="Non-blocking sensitive-path risk auditor for Solix pull requests."
    )
    parser.add_argument(
        "--base",
        type=str,
        default=None,
        help="Git base branch/SHA to diff against HEAD",
    )
    parser.add_argument(
        "--files",
        nargs="*",
        default=None,
        help="Explicit list of changed repository-relative file paths",
    )
    parser.add_argument(
        "--files-json",
        type=Path,
        default=None,
        help="JSON file containing a list of changed file paths (or objects with `filename`)",
    )
    parser.add_argument(
        "--json-out",
        type=Path,
        default=None,
        help="Optional output path to write structured JSON audit results",
    )
    parser.add_argument(
        "--markdown-out",
        type=Path,
        default=None,
        help="Optional output path to write rendered Markdown advisory",
    )
    parser.add_argument(
        "--emit-annotations",
        action="store_true",
        help="Print `::warning` GitHub Actions workflow annotations",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    """Runs sensitive-path risk audit and always returns 0 (non-blocking)."""
    args = parse_args(argv)
    changed_paths: list[str] = []

    if args.files_json is not None and args.files_json.is_file():
        payload = json.loads(args.files_json.read_text(encoding="utf-8"))
        if isinstance(payload, list):
            for entry in payload:
                if isinstance(entry, str):
                    changed_paths.append(entry)
                elif isinstance(entry, dict) and isinstance(entry.get("filename"), str):
                    changed_paths.append(entry["filename"])
    elif args.files is not None:
        changed_paths.extend(args.files)
    elif args.base is not None:
        changed_paths.extend(collect_changed_files_from_git(args.base))

    report = audit_paths(changed_paths)
    markdown_report = render_markdown_report(report)

    if args.json_out is not None:
        args.json_out.parent.mkdir(parents=True, exist_ok=True)
        args.json_out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    if args.markdown_out is not None:
        args.markdown_out.parent.mkdir(parents=True, exist_ok=True)
        args.markdown_out.write_text(markdown_report, encoding="utf-8")

    step_summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if step_summary_path:
        with Path(step_summary_path).open("a", encoding="utf-8") as summary_handle:
            summary_handle.write("\n" + markdown_report)

    if args.emit_annotations and report["has_sensitive_changes"]:
        emit_github_annotations(report)

    print(
        f"[pr_risk_audit] Audited {report['total_changed_files']} file(s); "
        f"risk labels: {', '.join(report['labels']) if report['labels'] else 'none'}."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
