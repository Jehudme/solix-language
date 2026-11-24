#!/usr/bin/env python3
"""Coverage Summary Formatter for GitHub Actions CI and Step Summary.

Parses cargo-llvm-cov summary text output and renders structured, readable
GitHub Flavored Markdown reports with per-subsystem aggregation, delta comparison against
base branches, and pull request discussion comments (GL-0014).
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import sys
from typing import Any, Dict, List, Optional, Set, Tuple


MARKER_COMMENT_TAG = "<!-- solix-pr-coverage -->"


def classify_subsystem(file_path: str) -> str:
    """Maps a repository file path to its containing subsystem or crate."""
    normalized_path = file_path.strip()
    if normalized_path.startswith("crates/solix-common/"):
        return "crates/solix-common"
    if normalized_path.startswith("crates/solix-compilation/"):
        return "crates/solix-compilation"
    if normalized_path.startswith("crates/solix-runtime/"):
        return "crates/solix-runtime"
    if normalized_path.startswith("crates/solix-toolchain/"):
        return "crates/solix-toolchain"
    if normalized_path.startswith("crates/solix-lsp/"):
        return "crates/solix-lsp"
    if normalized_path.startswith("crates/solix-dap/"):
        return "crates/solix-dap"
    if normalized_path.startswith("crates/solix-tests/"):
        return "crates/solix-tests"
    if normalized_path.startswith("ffi/rust/"):
        return "ffi/rust"
    if normalized_path.startswith("ffi/"):
        return "ffi"
    return "workspace-other"


def parse_percentage_value(percentage_string: str) -> float:
    """Parses a percentage string like '76.02%' into a float value."""
    cleaned_string = percentage_string.strip().rstrip("%")
    try:
        return float(cleaned_string)
    except ValueError:
        return 0.0


def parse_coverage_summary(raw_summary_text: str) -> Dict[str, Any]:
    """Parses raw text output from `cargo llvm-cov --summary-only`."""
    subsystem_totals: Dict[str, Dict[str, int]] = {}
    overall_total_metrics: Optional[Dict[str, Any]] = None

    for raw_line in raw_summary_text.splitlines():
        line_content = raw_line.strip()
        if not line_content or line_content.startswith("---") or line_content.startswith("Filename"):
            continue

        tokens = line_content.split()
        if len(tokens) < 13:
            continue

        item_name = " ".join(tokens[:-12])
        try:
            total_functions = int(tokens[-9])
            missed_functions = int(tokens[-8])
            total_lines = int(tokens[-6])
            missed_lines = int(tokens[-5])
        except ValueError:
            continue

        if item_name == "TOTAL":
            line_coverage_percentage = parse_percentage_value(tokens[-4])
            function_coverage_percentage = parse_percentage_value(tokens[-7])
            overall_total_metrics = {
                "total_lines": total_lines,
                "missed_lines": missed_lines,
                "executed_lines": total_lines - missed_lines,
                "line_coverage_percentage": line_coverage_percentage,
                "total_functions": total_functions,
                "missed_functions": missed_functions,
                "executed_functions": total_functions - missed_functions,
                "function_coverage_percentage": function_coverage_percentage,
            }
            continue

        subsystem_name = classify_subsystem(item_name)
        if subsystem_name not in subsystem_totals:
            subsystem_totals[subsystem_name] = {
                "total_lines": 0,
                "missed_lines": 0,
                "total_functions": 0,
                "missed_functions": 0,
            }

        subsystem_totals[subsystem_name]["total_lines"] += total_lines
        subsystem_totals[subsystem_name]["missed_lines"] += missed_lines
        subsystem_totals[subsystem_name]["total_functions"] += total_functions
        subsystem_totals[subsystem_name]["missed_functions"] += missed_functions

    return {
        "subsystems": subsystem_totals,
        "total": overall_total_metrics,
    }


def compute_subsystem_percentages(
    subsystem_metrics: Dict[str, int],
) -> Tuple[float, float]:
    """Calculates (line_coverage_percentage, function_coverage_percentage)."""
    total_lines = subsystem_metrics.get("total_lines", 0)
    missed_lines = subsystem_metrics.get("missed_lines", 0)
    covered_lines = total_lines - missed_lines
    line_percentage = (covered_lines / total_lines * 100.0) if total_lines > 0 else 100.0

    total_functions = subsystem_metrics.get("total_functions", 0)
    missed_functions = subsystem_metrics.get("missed_functions", 0)
    covered_functions = total_functions - missed_functions
    function_percentage = (
        (covered_functions / total_functions * 100.0) if total_functions > 0 else 100.0
    )

    return line_percentage, function_percentage


def extract_changed_subsystems(changed_file_paths: List[str]) -> Set[str]:
    """Maps a collection of changed file paths to unique subsystem names."""
    identified_subsystems: Set[str] = set()
    for file_path in changed_file_paths:
        subsystem_identifier = classify_subsystem(file_path)
        if subsystem_identifier != "workspace-other":
            identified_subsystems.add(subsystem_identifier)
    return identified_subsystems


def render_markdown_coverage_report(
    parsed_summary: Dict[str, Any],
    raw_summary_text: str,
    minimum_coverage_floor: float = 55.0,
    base_summary: Optional[Dict[str, Any]] = None,
    changed_files_list: Optional[List[str]] = None,
    is_pr_mode: bool = False,
) -> str:
    """Formats parsed coverage statistics into structured GitHub Flavored Markdown."""
    overall_total = parsed_summary.get("total")
    current_subsystems: Dict[str, Dict[str, int]] = parsed_summary.get("subsystems", {})
    base_subsystems: Dict[str, Dict[str, int]] = (
        base_summary.get("subsystems", {}) if base_summary else {}
    )
    base_total = base_summary.get("total") if base_summary else None

    markdown_lines: List[str] = [MARKER_COMMENT_TAG, "## Workspace Code Coverage Report", ""]

    if overall_total is None:
        markdown_lines.append("> [!WARNING]")
        markdown_lines.append("> Unable to parse total coverage summary metrics from llvm-cov output.")
        markdown_lines.append("")
        markdown_lines.append("```text")
        markdown_lines.append(raw_summary_text.strip())
        markdown_lines.append("```")
        return "\n".join(markdown_lines) + "\n"

    total_lines = overall_total["total_lines"]
    executed_lines = overall_total["executed_lines"]
    missed_lines = overall_total["missed_lines"]
    line_coverage_percentage = overall_total["line_coverage_percentage"]
    total_functions = overall_total["total_functions"]
    executed_functions = overall_total["executed_functions"]
    function_coverage_percentage = overall_total["function_coverage_percentage"]

    gate_status = "PASSED" if line_coverage_percentage >= minimum_coverage_floor else "FAILED"
    alert_level = "NOTE" if gate_status == "PASSED" else "CAUTION"

    total_delta_annotation = ""
    if base_total and "line_coverage_percentage" in base_total:
        base_line_percentage = base_total["line_coverage_percentage"]
        workspace_delta = line_coverage_percentage - base_line_percentage
        sign_prefix = "+" if workspace_delta >= 0.0 else ""
        total_delta_annotation = f" ({sign_prefix}{workspace_delta:.2f}% vs dev)"

    markdown_lines.append(f"> [!{alert_level}]")
    markdown_lines.append(
        f"> **Overall Workspace Line Coverage:** **{line_coverage_percentage:.2f}%**"
        f"{total_delta_annotation} "
        f"(Gate: `>= {minimum_coverage_floor:.1f}%` required — **{gate_status}**)"
    )
    markdown_lines.append(
        f"> • **Lines Covered:** {executed_lines:,} / {total_lines:,} ({missed_lines:,} missed)"
    )
    markdown_lines.append(
        f"> • **Functions Executed:** {executed_functions:,} / {total_functions:,} ({function_coverage_percentage:.2f}%)"
    )
    markdown_lines.append("")

    changed_subsystems: Set[str] = set()
    if changed_files_list:
        changed_subsystems = extract_changed_subsystems(changed_files_list)

    if is_pr_mode and changed_files_list:
        markdown_lines.append("### Modified Subsystems Coverage & Delta")
        markdown_lines.append("")

        if not changed_subsystems:
            markdown_lines.append("> *No core Rust crate subsystems were modified in this pull request.*")
            markdown_lines.append("")
        else:
            markdown_lines.append(
                "| Modified Subsystem / Crate | dev Base Coverage | PR Coverage | Delta | Status |"
            )
            markdown_lines.append("| :--- | :---: | :---: | :---: | :---: |")

            for subsystem_name in sorted(changed_subsystems):
                current_metrics = current_subsystems.get(subsystem_name, {})
                current_line_pct, _ = compute_subsystem_percentages(current_metrics)

                base_metrics = base_subsystems.get(subsystem_name, {})
                base_line_pct, _ = compute_subsystem_percentages(base_metrics)

                subsystem_delta = current_line_pct - base_line_pct
                delta_sign = "+" if subsystem_delta >= 0.0 else ""

                if subsystem_delta > 0.05:
                    status_indicator = "IMPROVED"
                elif subsystem_delta < -0.05:
                    status_indicator = "REGRESSED"
                else:
                    status_indicator = "UNCHANGED"

                base_display = f"{base_line_pct:.2f}%" if base_metrics else "N/A"
                markdown_lines.append(
                    f"| `{subsystem_name}` | {base_display} | {current_line_pct:.2f}% | "
                    f"**{delta_sign}{subsystem_delta:.2f}%** | {status_indicator} |"
                )

            markdown_lines.append("")

    markdown_lines.append("### Subsystem Coverage Breakdown")
    markdown_lines.append("")
    markdown_lines.append(
        "| Subsystem / Crate | Total Lines | Covered Lines | Missed Lines | Line Coverage | Function Coverage | Status |"
    )
    markdown_lines.append("| :--- | :---: | :---: | :---: | :---: | :---: | :---: |")

    for subsystem_name, metrics in sorted(current_subsystems.items()):
        subsystem_total_lines = metrics["total_lines"]
        subsystem_missed_lines = metrics["missed_lines"]
        subsystem_covered_lines = subsystem_total_lines - subsystem_missed_lines
        subsystem_line_percentage, subsystem_function_percentage = compute_subsystem_percentages(metrics)

        subsystem_status = "PASS" if subsystem_line_percentage >= minimum_coverage_floor else "WARN"
        markdown_lines.append(
            f"| `{subsystem_name}` | {subsystem_total_lines:,} | {subsystem_covered_lines:,} | "
            f"{subsystem_missed_lines:,} | {subsystem_line_percentage:.2f}% | "
            f"{subsystem_function_percentage:.2f}% | {subsystem_status} |"
        )

    markdown_lines.append(
        f"| **TOTAL (Workspace)** | **{total_lines:,}** | **{executed_lines:,}** | "
        f"**{missed_lines:,}** | **{line_coverage_percentage:.2f}%** | "
        f"**{function_coverage_percentage:.2f}%** | **{gate_status}** |"
    )
    markdown_lines.append("")
    markdown_lines.append("<details>")
    markdown_lines.append("<summary>Click to view complete file-by-file raw coverage output</summary>")
    markdown_lines.append("")
    markdown_lines.append("```text")
    markdown_lines.append(raw_summary_text.strip())
    markdown_lines.append("```")
    markdown_lines.append("</details>")
    markdown_lines.append("")

    return "\n".join(markdown_lines)


def parse_arguments(arguments_list: Optional[List[str]] = None) -> argparse.Namespace:
    """Parses command line arguments."""
    parser = argparse.ArgumentParser(
        description="Renders structured Markdown coverage reports from llvm-cov summary text."
    )
    parser.add_argument(
        "--input",
        type=Path,
        required=True,
        help="Path to raw coverage summary text file produced by cargo llvm-cov",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Optional destination path to write rendered Markdown coverage report",
    )
    parser.add_argument(
        "--json-output",
        type=Path,
        default=None,
        help="Optional destination path to write parsed JSON coverage telemetry",
    )
    parser.add_argument(
        "--base-input",
        type=Path,
        default=None,
        help="Optional path to base branch coverage summary text or JSON file for delta calculation",
    )
    parser.add_argument(
        "--changed-files",
        type=str,
        default="",
        help="Optional comma- or newline-separated list of modified repository file paths",
    )
    parser.add_argument(
        "--pr-mode",
        action="store_true",
        help="Enable pull request mode focusing on modified modules and delta breakdown",
    )
    parser.add_argument(
        "--step-summary",
        action="store_true",
        help="Append rendered report to GITHUB_STEP_SUMMARY environment variable if present",
    )
    parser.add_argument(
        "--minimum-coverage-floor",
        type=float,
        default=55.0,
        help="Minimum coverage line threshold percentage (default: 55.0)",
    )
    return parser.parse_args(arguments_list)


def main(arguments_list: Optional[List[str]] = None) -> int:
    """Main execution function."""
    parsed_arguments = parse_arguments(arguments_list)
    input_file_path: Path = parsed_arguments.input

    if not input_file_path.is_file():
        print(f"[ERROR] Input coverage summary file does not exist: {input_file_path}", file=sys.stderr)
        return 1

    raw_summary_text = input_file_path.read_text(encoding="utf-8")
    parsed_summary = parse_coverage_summary(raw_summary_text)

    base_summary: Optional[Dict[str, Any]] = None
    if parsed_arguments.base_input and parsed_arguments.base_input.is_file():
        try:
            base_content = parsed_arguments.base_input.read_text(encoding="utf-8")
            if base_content.strip().startswith("{"):
                base_summary = json.loads(base_content)
            else:
                base_summary = parse_coverage_summary(base_content)
        except Exception as error_reading_base:
            print(f"[WARNING] Could not parse base coverage file: {error_reading_base}", file=sys.stderr)

    changed_files_list: List[str] = []
    if parsed_arguments.changed_files:
        changed_files_list = [
            path_item.strip()
            for path_item in re.split(r"[,\n\r]+", parsed_arguments.changed_files)
            if path_item.strip()
        ]

    rendered_markdown = render_markdown_coverage_report(
        parsed_summary,
        raw_summary_text,
        minimum_coverage_floor=parsed_arguments.minimum_coverage_floor,
        base_summary=base_summary,
        changed_files_list=changed_files_list,
        is_pr_mode=parsed_arguments.pr_mode or bool(changed_files_list),
    )

    if parsed_arguments.output is not None:
        parsed_arguments.output.parent.mkdir(parents=True, exist_ok=True)
        parsed_arguments.output.write_text(rendered_markdown, encoding="utf-8")

    if parsed_arguments.json_output is not None:
        parsed_arguments.json_output.parent.mkdir(parents=True, exist_ok=True)
        parsed_arguments.json_output.write_text(
            json.dumps(parsed_summary, indent=2) + "\n",
            encoding="utf-8",
        )

    if parsed_arguments.step_summary:
        step_summary_environment_path = os.environ.get("GITHUB_STEP_SUMMARY")
        if step_summary_environment_path:
            with Path(step_summary_environment_path).open("a", encoding="utf-8") as summary_file_handle:
                summary_file_handle.write("\n" + rendered_markdown + "\n")

    print(
        f"[SUCCESS] Parsed coverage report. Total lines: "
        f"{parsed_summary.get('total', {}).get('total_lines', 'unknown')}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
