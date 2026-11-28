#!/usr/bin/env python3
"""Detects modified subsystems in pull requests and CI runs.

Categorizes modified files against the base reference (e.g. `origin/dev`) into:
- crates: Rust crates in `crates/` or workspace `Cargo.*`
- libraries: Standard library packages in `libraries/`
- vscode: VS Code extension files in `editors/vscode/`
- docs: Documentation in `docs/`, `*.md`
- benchmarks_needed: Whether `crates/` was modified requiring benchmark validation
- docs_only: Whether ONLY documentation / guidelines / meta files were modified

Outputs JSON summary and GitHub Actions step outputs.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
from typing import List, Optional


def get_changed_files_git(base_ref: str) -> List[str]:
    """Retrieves list of changed files via `git diff --name-only`."""
    try:
        output = subprocess.check_output(
            ["git", "diff", "--name-only", f"{base_ref}...HEAD"],
            text=True,
            stderr=subprocess.DEVNULL,
        )
        return [line.strip() for line in output.splitlines() if line.strip()]
    except Exception:
        # Fallback if three-dot diff fails
        try:
            output = subprocess.check_output(
                ["git", "diff", "--name-only", base_ref],
                text=True,
                stderr=subprocess.DEVNULL,
            )
            return [line.strip() for line in output.splitlines() if line.strip()]
        except Exception as error_msg:
            print(f"[detect_subsystems] Git diff failed: {error_msg}", file=sys.stderr)
            return []


def analyze_changed_subsystems(changed_files: List[str]) -> dict:
    """Classifies changed files into subsystem categories."""
    if not changed_files:
        # If no files detected (or schedule/manual dispatch), default to building everything
        return {
            "has_crates": True,
            "has_libraries": True,
            "has_vscode": True,
            "has_docs": True,
            "benchmarks_needed": True,
            "docs_only": False,
            "changed_files_count": 0,
        }

    has_crates = any(
        f.startswith("crates/") or f in ("Cargo.toml", "Cargo.lock") or f.startswith("ffi/")
        for f in changed_files
    )
    has_libraries = any(f.startswith("libraries/") for f in changed_files)
    has_vscode = any(f.startswith("editors/vscode/") for f in changed_files)
    has_docs = any(
        f.startswith("docs/") or f.endswith(".md") or f.startswith(".github/ISSUE_TEMPLATE/")
        for f in changed_files
    )

    # Any other changes (scripts, workflows, etc.)
    has_other_code = any(
        f.startswith("scripts/") or f.startswith(".github/")
        for f in changed_files
    )

    # Docs only is true if docs changed AND no crates, libraries, vscode, or build scripts changed
    is_docs_only = has_docs and not (has_crates or has_libraries or has_vscode or has_other_code)

    # Benchmarks are strictly required ONLY when crates/ or Cargo dependencies change
    benchmarks_needed = has_crates

    return {
        "has_crates": has_crates or has_other_code,
        "has_libraries": has_libraries or has_other_code,
        "has_vscode": has_vscode or has_other_code,
        "has_docs": has_docs,
        "benchmarks_needed": benchmarks_needed,
        "docs_only": is_docs_only,
        "changed_files_count": len(changed_files),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Detect changed Solix subsystems for CI.")
    parser.add_argument("--base", default="origin/dev", help="Base ref to diff against")
    parser.add_argument("--files-json", help="Path to JSON file with changed file list")
    parser.add_argument("--output-json", help="Optional path to write output JSON")
    args = parser.parse_args()

    changed_files: List[str] = []
    if args.files_json and Path(args.files_json).is_file():
        try:
            content = json.loads(Path(args.files_json).read_text(encoding="utf-8"))
            if isinstance(content, list):
                for item in content:
                    if isinstance(item, str):
                        changed_files.append(item)
                    elif isinstance(item, dict) and "filename" in item:
                        changed_files.append(item["filename"])
        except Exception as read_err:
            print(f"[detect_subsystems] Failed to parse {args.files_json}: {read_err}", file=sys.stderr)

    if not changed_files:
        changed_files = get_changed_files_git(args.base)

    summary = analyze_changed_subsystems(changed_files)

    if args.output_json:
        Path(args.output_json).write_text(json.dumps(summary, indent=2), encoding="utf-8")

    # Output to GitHub Actions step output if available
    github_output_path = os.environ.get("GITHUB_OUTPUT")
    if github_output_path:
        with open(github_output_path, "a", encoding="utf-8") as out_file:
            for key, val in summary.items():
                out_file.write(f"{key}={'true' if val is True else ('false' if val is False else val)}\n")

    print("[detect_subsystems] Subsystem Change Detection:")
    for key, val in summary.items():
        print(f"  {key}: {val}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
