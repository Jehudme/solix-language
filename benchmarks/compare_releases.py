#!/usr/bin/env python3
"""
Solix Release Benchmark Comparison and Regression Detection Utility.

Compares benchmark execution results between the current build and the previous
release baseline, calculates speedup ratios, execution delta percentages, and
memory RSS differences, generates GitHub Flavored Markdown comparison tables
for release notes, and enforces performance regression quality gates in CI/CD.
"""

from __future__ import annotations

import argparse
import json
import math
import os
from pathlib import Path
import subprocess
import sys
from typing import Any, Dict, List, Optional, Tuple

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")


def load_benchmark_data(source_target: str) -> Dict[str, Any]:
    """Loads benchmark JSON data from a filesystem path or raw JSON string."""
    candidate_path = Path(source_target)
    if candidate_path.exists() and candidate_path.is_file():
        with open(candidate_path, "r", encoding="utf-8") as file_handle:
            return json.load(file_handle)

    # Attempt parsing as raw JSON string
    try:
        return json.loads(source_target)
    except json.JSONDecodeError as decode_error:
        raise ValueError(
            f"Failed to load benchmark data from '{source_target}': "
            f"not an existing file and invalid JSON ({decode_error})"
        )


def find_local_baseline_fallback(baselines_directory: Path) -> Optional[Path]:
    """Locates the latest baseline JSON file in the baselines directory."""
    if not baselines_directory.exists() or not baselines_directory.is_dir():
        return None

    candidate_files = [
        file_path
        for file_path in baselines_directory.glob("*.json")
        if file_path.name.endswith("_baseline.json")
        or file_path.name.startswith("ms")
        or file_path.name == "release_baseline.json"
    ]

    if not candidate_files:
        return None

    # Sort by filename descending (e.g. ms018 > ms017 > ms016) and timestamp
    candidate_files.sort(
        key=lambda file_path: (file_path.stat().st_mtime, file_path.name),
        reverse=True,
    )
    return candidate_files[0]


def fetch_previous_release_baseline_from_github(
    target_tag: Optional[str] = None,
    exclude_tag: Optional[str] = None,
) -> Optional[Dict[str, Any]]:
    """Attempts to fetch previous release baseline JSON using GitHub CLI (gh)."""
    try:
        resolved_tag = target_tag
        if not resolved_tag:
            # Query the list of releases to find the most recent published release
            list_command = [
                "gh",
                "release",
                "list",
                "--limit",
                "10",
                "--json",
                "tagName,isPrerelease,isDraft",
            ]
            list_process = subprocess.run(
                list_command,
                capture_output=True,
                text=True,
                check=False,
            )
            if list_process.returncode == 0 and list_process.stdout.strip():
                releases_list = json.loads(list_process.stdout)
                for release_entry in releases_list:
                    tag_candidate = release_entry.get("tagName")
                    if release_entry.get("isDraft", False):
                        continue
                    if exclude_tag and tag_candidate == exclude_tag:
                        continue
                    if tag_candidate:
                        resolved_tag = tag_candidate
                        break

        if not resolved_tag:
            return None

        # Download release_baseline.json asset via gh release download
        download_command = [
            "gh",
            "release",
            "download",
            resolved_tag,
            "--pattern",
            "release_baseline.json",
            "--output",
            "-",
        ]
        download_process = subprocess.run(
            download_command,
            capture_output=True,
            text=True,
            check=False,
        )
        if download_process.returncode == 0 and download_process.stdout.strip():
            return json.loads(download_process.stdout)

    except Exception as network_error:
        # Graceful fallback when gh is unavailable or network/auth fails
        sys.stderr.write(
            f"[WARNING] GitHub CLI baseline retrieval skipped: {network_error}\n"
        )

    return None


def calculate_geometric_mean(numbers_list: List[float]) -> float:
    """Calculates the geometric mean of a list of positive float ratios."""
    positive_numbers = [number for number in numbers_list if number > 0.0]
    if not positive_numbers:
        return 1.0
    log_sum = sum(math.log(number) for number in positive_numbers)
    return math.exp(log_sum / len(positive_numbers))


def compare_benchmarks(
    current_data: Dict[str, Any],
    previous_data: Optional[Dict[str, Any]],
    regression_threshold_pct: float = 5.0,
    preferred_engine: str = "solix_fast",
) -> Dict[str, Any]:
    """Compares benchmark metrics between current and previous datasets."""
    current_workloads: Dict[str, Any] = current_data.get("workloads", {})
    previous_workloads: Dict[str, Any] = (
        previous_data.get("workloads", {}) if previous_data else {}
    )

    all_workload_keys = sorted(
        set(current_workloads.keys()).union(set(previous_workloads.keys()))
    )

    comparison_rows: List[Dict[str, Any]] = []
    speedup_ratios_list: List[float] = []
    total_memory_diff_kb: int = 0
    faster_workloads_count: int = 0
    stable_workloads_count: int = 0
    regressed_workloads_count: int = 0

    for workload_name in all_workload_keys:
        current_entry = current_workloads.get(workload_name, {}).get(
            preferred_engine, {}
        )
        previous_entry = previous_workloads.get(workload_name, {}).get(
            preferred_engine, {}
        )

        current_median_ms: float = float(current_entry.get("median_ms", 0.0))
        previous_median_ms: float = float(previous_entry.get("median_ms", 0.0))
        current_rss_kb: int = int(current_entry.get("rss_kb", 0))
        previous_rss_kb: int = int(previous_entry.get("rss_kb", 0))

        row_metric: Dict[str, Any] = {
            "workload": workload_name,
            "current_ms": current_median_ms,
            "previous_ms": previous_median_ms,
            "current_rss_kb": current_rss_kb,
            "previous_rss_kb": previous_rss_kb,
            "speedup_ratio": 1.0,
            "speedup_pct": 0.0,
            "memory_diff_mb": 0.0,
            "status": "UNKNOWN",
            "badge": "➖",
            "is_regression": False,
        }

        if previous_median_ms <= 0.0 and current_median_ms > 0.0:
            row_metric["status"] = "NEW"
            row_metric["badge"] = "🆕 NEW"
            row_metric["memory_diff_mb"] = round(current_rss_kb / 1024.0, 2)
        elif current_median_ms <= 0.0 and previous_median_ms > 0.0:
            row_metric["status"] = "REMOVED"
            row_metric["badge"] = "🗑️ REMOVED"
            row_metric["memory_diff_mb"] = round(-previous_rss_kb / 1024.0, 2)
        elif current_median_ms > 0.0 and previous_median_ms > 0.0:
            speedup_ratio = previous_median_ms / current_median_ms
            speedup_ratios_list.append(speedup_ratio)

            # Execution time difference percentage
            # positive speedup_pct means current executed faster than previous
            speedup_percentage = (
                (previous_median_ms - current_median_ms) / previous_median_ms
            ) * 100.0

            memory_diff_kb = current_rss_kb - previous_rss_kb
            total_memory_diff_kb += memory_diff_kb
            memory_diff_mb = round(memory_diff_kb / 1024.0, 2)

            row_metric["speedup_ratio"] = round(speedup_ratio, 2)
            row_metric["speedup_pct"] = round(speedup_percentage, 1)
            row_metric["memory_diff_mb"] = memory_diff_mb

            # Threshold classification
            if speedup_percentage > regression_threshold_pct:
                row_metric["status"] = "FASTER"
                row_metric["badge"] = "🚀 FASTER"
                faster_workloads_count += 1
            elif speedup_percentage < -regression_threshold_pct:
                row_metric["status"] = "REGRESSION"
                row_metric["badge"] = "⚠️ SLOWER"
                row_metric["is_regression"] = True
                regressed_workloads_count += 1
            else:
                row_metric["status"] = "STABLE"
                row_metric["badge"] = "⚖️ STABLE"
                stable_workloads_count += 1

        comparison_rows.append(row_metric)

    geometric_mean_speedup = calculate_geometric_mean(speedup_ratios_list)
    overall_has_regression = regressed_workloads_count > 0

    return {
        "workloads": comparison_rows,
        "summary": {
            "total_evaluated": len(comparison_rows),
            "faster_count": faster_workloads_count,
            "stable_count": stable_workloads_count,
            "regressed_count": regressed_workloads_count,
            "has_regression": overall_has_regression,
            "geometric_mean_speedup": round(geometric_mean_speedup, 2),
            "total_memory_diff_mb": round(total_memory_diff_kb / 1024.0, 2),
            "regression_threshold_pct": regression_threshold_pct,
        },
    }


def render_markdown_comparison_table(
    comparison_results: Dict[str, Any],
    previous_tag_label: str = "Previous Release",
    current_tag_label: str = "Current Release",
) -> str:
    """Renders formatted GitHub Flavored Markdown table for release notes."""
    workload_rows: List[Dict[str, Any]] = comparison_results.get("workloads", [])
    summary_data: Dict[str, Any] = comparison_results.get("summary", {})

    has_previous_data = any(row.get("previous_ms", 0.0) > 0.0 for row in workload_rows)

    markdown_lines: List[str] = []

    if has_previous_data:
        markdown_lines.append(
            f"### Performance Benchmarks vs {previous_tag_label} (`{current_tag_label}`)\n"
        )
        markdown_lines.append(
            "| Workload | Previous (ms) | Current (ms) | Speedup | Memory Diff | Status |"
        )
        markdown_lines.append(
            "| :--- | :--- | :--- | :--- | :--- | :--- |"
        )

        for row in workload_rows:
            workload_name = row["workload"]
            previous_ms = row["previous_ms"]
            current_ms = row["current_ms"]
            speedup_pct = row["speedup_pct"]
            speedup_ratio = row["speedup_ratio"]
            memory_diff_mb = row["memory_diff_mb"]
            badge = row["badge"]

            if previous_ms > 0.0 and current_ms > 0.0:
                speedup_sign = "+" if speedup_pct >= 0.0 else ""
                speedup_formatted = (
                    f"{speedup_sign}{speedup_pct:.1f}% ({speedup_ratio:.2f}x)"
                )
                memory_sign = "+" if memory_diff_mb >= 0.0 else ""
                memory_formatted = f"{memory_sign}{memory_diff_mb:.1f} MB"
                markdown_lines.append(
                    f"| `{workload_name}` | {previous_ms:.2f} ms | {current_ms:.2f} ms | "
                    f"{speedup_formatted} | {memory_formatted} | {badge} |"
                )
            elif current_ms > 0.0:
                markdown_lines.append(
                    f"| `{workload_name}` | - | {current_ms:.2f} ms | - | - | {badge} |"
                )
            else:
                markdown_lines.append(
                    f"| `{workload_name}` | {previous_ms:.2f} ms | - | - | - | {badge} |"
                )

        # Append summary block
        geomean = summary_data.get("geometric_mean_speedup", 1.0)
        total_mem = summary_data.get("total_memory_diff_mb", 0.0)
        faster = summary_data.get("faster_count", 0)
        stable = summary_data.get("stable_count", 0)
        regressed = summary_data.get("regressed_count", 0)

        mem_sign = "+" if total_mem >= 0.0 else ""
        markdown_lines.append("")
        markdown_lines.append(
            f"> [!NOTE]\n"
            f"> **Benchmark Summary:** Geometric mean speedup: **{geomean:.2f}x** "
            f"across {len(workload_rows)} suites. "
            f"Total memory delta: **{mem_sign}{total_mem:.1f} MB**. "
            f"Distribution: {faster} faster, {stable} stable, {regressed} regressed."
        )
    else:
        # Initial baseline recording format
        markdown_lines.append(f"### Performance Baseline ({current_tag_label})\n")
        markdown_lines.append(
            "*Initial performance baseline recorded. Comparative delta tracking will activate in subsequent releases.*\n"
        )
        markdown_lines.append(
            "| Workload | Median Execution Time (ms) | Peak RSS (KB) | Status |"
        )
        markdown_lines.append("| :--- | :--- | :--- | :--- |")
        for row in workload_rows:
            workload_name = row["workload"]
            current_ms = row["current_ms"]
            current_rss_kb = row["current_rss_kb"]
            markdown_lines.append(
                f"| `{workload_name}` | {current_ms:.2f} ms | {current_rss_kb} KB | 📌 BASELINE |"
            )

    return "\n".join(markdown_lines) + "\n"


def inject_into_release_notes(
    notes_file_path: Path,
    markdown_content: str,
) -> None:
    """Injects or appends markdown comparison content into an existing release notes file."""
    existing_content = ""
    if notes_file_path.exists():
        with open(notes_file_path, "r", encoding="utf-8") as notes_file:
            existing_content = notes_file.read()

    separator = "\n\n" if existing_content and not existing_content.endswith("\n\n") else ""
    updated_content = existing_content + separator + markdown_content

    with open(notes_file_path, "w", encoding="utf-8") as notes_file:
        notes_file.write(updated_content)


def parse_arguments() -> argparse.Namespace:
    """Configures and parses command-line arguments."""
    parser = argparse.ArgumentParser(
        description="Solix Dynamic Release Benchmark Comparison & CI Gate"
    )
    parser.add_argument(
        "--current",
        type=str,
        required=True,
        help="Path or raw JSON string for current benchmark baseline",
    )
    parser.add_argument(
        "--previous",
        type=str,
        default=None,
        help="Path or raw JSON string for previous benchmark baseline",
    )
    parser.add_argument(
        "--download-previous",
        action="store_true",
        help="Attempt downloading previous release baseline asset via GitHub CLI",
    )
    parser.add_argument(
        "--no-fallback",
        action="store_true",
        help="Do not fallback to local baselines directory if previous baseline is not found",
    )
    parser.add_argument(
        "--threshold-pct",
        type=float,
        default=5.0,
        help="Performance regression tolerance percentage threshold (default: 5.0)",
    )
    parser.add_argument(
        "--current-tag",
        type=str,
        default="Current Build",
        help="Label or tag for current build version (e.g. v2.0.0)",
    )
    parser.add_argument(
        "--previous-tag",
        type=str,
        default="Previous Release",
        help="Label or tag for previous release version (e.g. v1.2.0)",
    )
    parser.add_argument(
        "--markdown-output",
        type=str,
        default=None,
        help="Path to write generated GitHub Flavored Markdown comparison table",
    )
    parser.add_argument(
        "--json-output",
        type=str,
        default=None,
        help="Path to write structured JSON comparison results",
    )
    parser.add_argument(
        "--inject-notes",
        type=str,
        default=None,
        help="Path to release notes file where markdown table should be appended",
    )
    parser.add_argument(
        "--fail-on-regression",
        action="store_true",
        help="Exit with non-zero status code if performance regresses beyond threshold",
    )
    parser.add_argument(
        "--quiet",
        action="store_true",
        help="Suppress human-readable console stdout output",
    )
    return parser.parse_args()


def main() -> int:
    """Main execution entrypoint."""
    parsed_args = parse_arguments()

    current_benchmark_data = load_benchmark_data(parsed_args.current)

    previous_benchmark_data: Optional[Dict[str, Any]] = None

    if parsed_args.previous:
        previous_benchmark_data = load_benchmark_data(parsed_args.previous)
    elif parsed_args.download_previous:
        previous_benchmark_data = fetch_previous_release_baseline_from_github(
            target_tag=parsed_args.previous_tag
            if parsed_args.previous_tag != "Previous Release"
            else None,
            exclude_tag=parsed_args.current_tag
            if parsed_args.current_tag != "Current Build"
            else None,
        )

    # If previous benchmark data is still not found, search local baselines folder
    if previous_benchmark_data is None and not parsed_args.no_fallback:
        script_root = Path(__file__).resolve().parent
        baselines_directory = script_root / "baselines"
        local_fallback_file = find_local_baseline_fallback(baselines_directory)
        if local_fallback_file:
            try:
                previous_benchmark_data = load_benchmark_data(str(local_fallback_file))
                if not parsed_args.quiet:
                    print(
                        f"[INFO] Using local fallback baseline: {local_fallback_file.name}"
                    )
            except Exception:
                pass

    comparison_results = compare_benchmarks(
        current_data=current_benchmark_data,
        previous_data=previous_benchmark_data,
        regression_threshold_pct=parsed_args.threshold_pct,
    )

    markdown_table = render_markdown_comparison_table(
        comparison_results=comparison_results,
        previous_tag_label=parsed_args.previous_tag,
        current_tag_label=parsed_args.current_tag,
    )

    if not parsed_args.quiet:
        print(markdown_table)

    if parsed_args.markdown_output:
        markdown_target = Path(parsed_args.markdown_output)
        markdown_target.parent.mkdir(parents=True, exist_ok=True)
        with open(markdown_target, "w", encoding="utf-8") as markdown_file:
            markdown_file.write(markdown_table)

    if parsed_args.json_output:
        json_target = Path(parsed_args.json_output)
        json_target.parent.mkdir(parents=True, exist_ok=True)
        with open(json_target, "w", encoding="utf-8") as json_file:
            json.dump(comparison_results, json_file, indent=2)

    if parsed_args.inject_notes:
        inject_into_release_notes(
            notes_file_path=Path(parsed_args.inject_notes),
            markdown_content=markdown_table,
        )

    if (
        parsed_args.fail_on_regression
        and comparison_results.get("summary", {}).get("has_regression", False)
    ):
        regressed_count = comparison_results.get("summary", {}).get(
            "regressed_count", 0
        )
        sys.stderr.write(
            f"\n❌ Quality Gate Failed: {regressed_count} workload(s) regressed beyond "
            f"{parsed_args.threshold_pct}% threshold!\n"
        )
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
