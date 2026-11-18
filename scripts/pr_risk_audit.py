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


SENSITIVE_LABEL = "sensitive"

RISK_TIERS: tuple[RiskTier, ...] = (
    RiskTier(
        tier_id="ci-workflow",
        label=SENSITIVE_LABEL,
        title="CI Workflows & Automation",
        severity="HIGH",
        description="Modifies CI workflows or repository automation.",
        patterns=(
            re.compile(r"^\.github/"),
        ),
        checklist=(
            "Verify least-privilege permissions, 40-char SHA action pinning, and safe event triggers.",
        ),
    ),
    RiskTier(
        tier_id="guidelines",
        label=SENSITIVE_LABEL,
        title="Engineering Guidelines & Governance Scripts",
        severity="HIGH",
        description="Modifies repository guidelines, contributor policy, or doc/release automation scripts.",
        patterns=(
            re.compile(r"^docs/guidelines/"),
            re.compile(r"^CONTRIBUTING\.md$"),
            re.compile(r"^scripts/(doc_sync_check|verify_docs|verify_doc_links|pr_risk_audit|prepare_release_train|build_docs_site|sync_github_wiki)\.py$"),
        ),
        checklist=(
            "Confirm governance and test gates remain consistent.",
        ),
    ),
    RiskTier(
        tier_id="supply-chain",
        label=SENSITIVE_LABEL,
        title="Supply Chain, Dependencies & Build Scripts",
        severity="ELEVATED",
        description="Modifies dependencies, build scripts, or packaged release assets.",
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
            "Review dependency updates and build-script changes.",
        ),
    ),
    RiskTier(
        tier_id="runtime-security",
        label=SENSITIVE_LABEL,
        title="Runtime Security, Archives & FFI",
        severity="HIGH",
        description="Touches archive extraction safety, SHA-256 verification, native plugins, or FFI.",
        patterns=(
            re.compile(r"^crates/solix-common/src/(project|hash)\.rs$"),
            re.compile(r"^crates/solix-runtime/src/native/"),
            re.compile(r"^crates/solix-ffi/"),
        ),
        checklist=(
            "Verify path safety, checksum verification, and FFI bounds.",
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

    for tier in RISK_TIERS:
        matched_files = [
            file_path
            for file_path in normalized
            if any(pattern.search(file_path) for pattern in tier.patterns)
        ]
        if matched_files:
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

    labels = sorted({finding["label"] for finding in findings})
    return {
        "has_sensitive_changes": bool(findings),
        "total_changed_files": len(normalized),
        "labels": labels,
        "findings": findings,
    }


def render_markdown_report(report: dict[str, object]) -> str:
    """Renders a concise Markdown notice for PR comments and job summaries."""
    findings = report["findings"]
    if not findings:
        return (
            f"{MARKER_COMMENT}\n"
            f"No sensitive paths modified across {report['total_changed_files']} file(s).\n"
        )

    lines: list[str] = [
        MARKER_COMMENT,
        "## Sensitive Paths Modified",
        "",
        "| Area | Files |",
        "| :--- | :--- |",
    ]

    for finding in findings:
        file_list = ", ".join(f"`{path}`" for path in finding["files"])
        lines.append(f"| **{finding['title']}** | {file_list} |")

    lines.append("")
    return "\n".join(lines)


def emit_github_annotations(report: dict[str, object]) -> None:
    """Emits non-blocking GitHub Actions warning annotations for each sensitive file."""
    for finding in report["findings"]:
        title = finding["title"]
        for file_path in finding["files"]:
            print(
                f"::warning file={file_path},title=Sensitive Path ({title})::"
                "Modified sensitive path; confirm maintainer review."
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
