#!/usr/bin/env python3
"""Assembles telemetry reports into comprehensive GitHub Dev Pre-Release notes.

Combines cross-language benchmarks, performance comparison vs latest official release,
cumulative code coverage, sensitive-path risk audits, and commit changelogs.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import sys
from typing import Optional


def read_report_section(report_file_path: Optional[Path], fallback_message: str) -> str:
    """Reads markdown section from file or returns fallback notification."""
    if report_file_path and report_file_path.is_file():
        try:
            file_content = report_file_path.read_text(encoding="utf-8").strip()
            if file_content:
                return file_content
        except Exception as file_read_error:
            return f"{fallback_message} (Read error: {file_read_error})"
    return fallback_message


def build_environment_banner_fallback() -> str:
    """Renders a fallback Benchmark Environment table when the benchmark report lacks one."""
    bench_scripts_dir = Path(__file__).resolve().parent.parent / "bench"
    if str(bench_scripts_dir) not in sys.path:
        sys.path.insert(0, str(bench_scripts_dir))
    try:
        import solix_bench  # type: ignore

        return solix_bench.render_benchmark_environment_markdown().rstrip()
    except Exception:
        return ""


def assemble_dev_release_notes(
    repository_name: str,
    commit_short_hash: str,
    latest_stable_tag: str,
    benchmark_report_path: Optional[Path] = None,
    coverage_report_path: Optional[Path] = None,
    sensitive_audit_path: Optional[Path] = None,
    changelog_preview: str = "",
) -> str:
    """Combines individual subsystem telemetry files into full release notes markdown."""
    benchmark_markdown = read_report_section(
        benchmark_report_path, "Benchmark telemetry report not generated."
    )
    if "### Benchmark Environment" not in benchmark_markdown:
        env_banner = build_environment_banner_fallback()
        if env_banner:
            benchmark_markdown = f"{env_banner}\n\n{benchmark_markdown}"
    coverage_markdown = read_report_section(
        coverage_report_path, "Coverage telemetry report not generated."
    )
    sensitive_markdown = read_report_section(
        sensitive_audit_path, "No sensitive paths modified."
    )

    stable_reference_label = latest_stable_tag.strip() if latest_stable_tag.strip() else "Initial Release"
    formatted_changelog = changelog_preview.strip() if changelog_preview.strip() else "- Development iterations on dev trunk"

    return f"""# Solix Dev (Nightly) Build — `{commit_short_hash}`

> [!NOTE]
> This is an automated rolling development build tracking the latest verified state of the `dev` branch.
> It contains cutting-edge compiler and runtime enhancements prior to official release.

### One-Line Dev Install
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/{repository_name}/releases/download/dev/solix-toolchain-installer.sh | sh
```

---

## Performance & Cross-Language Benchmarks
{benchmark_markdown}

---

## Cumulative Test Coverage
{coverage_markdown}

---

## Sensitive Paths Modified Since {stable_reference_label}
{sensitive_markdown}

---

## Commits on `dev` since {stable_reference_label}
{formatted_changelog}
"""


def parse_cli_arguments(argument_list: Optional[list[str]] = None) -> argparse.Namespace:
    """Parses CLI flags."""
    parser = argparse.ArgumentParser(description="Assemble dev pre-release markdown release notes.")
    parser.add_argument("--repo-name", type=str, default="Jehudme/solix-language", help="GitHub repository name")
    parser.add_argument("--short-sha", type=str, required=True, help="Short commit SHA")
    parser.add_argument("--latest-stable-tag", type=str, default="", help="Latest stable release tag")
    parser.add_argument("--benchmark-report", type=Path, default=None, help="Path to benchmark markdown report")
    parser.add_argument("--coverage-report", type=Path, default=None, help="Path to coverage markdown report")
    parser.add_argument("--sensitive-audit", type=Path, default=None, help="Path to sensitive audit markdown")
    parser.add_argument("--changelog-preview", type=str, default="", help="Changelog markdown summary")
    parser.add_argument("--output", type=Path, required=True, help="Destination path for assembled release notes")
    return parser.parse_args(argument_list)


def main(argument_list: Optional[list[str]] = None) -> int:
    """CLI entrypoint."""
    parsed_arguments = parse_cli_arguments(argument_list)
    assembled_markdown = assemble_dev_release_notes(
        repository_name=parsed_arguments.repo_name,
        commit_short_hash=parsed_arguments.short_sha,
        latest_stable_tag=parsed_arguments.latest_stable_tag,
        benchmark_report_path=parsed_arguments.benchmark_report,
        coverage_report_path=parsed_arguments.coverage_report,
        sensitive_audit_path=parsed_arguments.sensitive_audit,
        changelog_preview=parsed_arguments.changelog_preview,
    )
    parsed_arguments.output.parent.mkdir(parents=True, exist_ok=True)
    parsed_arguments.output.write_text(assembled_markdown, encoding="utf-8")
    print(f"[SUCCESS] Assembled dev release notes into {parsed_arguments.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
