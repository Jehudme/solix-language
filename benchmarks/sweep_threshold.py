#!/usr/bin/env python3
"""
Solix Inlining Threshold Empirical Sweep Experiment (Task 6.1)
Sweeps inline_threshold N in {0, 2, 4, 6, 8, 12, 16} across all 16 benchmark workloads.
Measures:
  1. Compiled .slxb binary size (bytes).
  2. Execution time (median ms over multiple iterations).
"""

import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent
SOLIX_BIN = ROOT_DIR / "target" / "release" / "solix"
WORKLOADS_DIR = ROOT_DIR / "benchmarks" / "workloads"
THRESHOLDS = [0, 2, 4, 6, 8, 12, 16]
ITERATIONS = 3


def get_workloads():
    dirs = [d for d in WORKLOADS_DIR.iterdir() if d.is_dir() and not d.name.startswith(".")]
    return sorted(dirs, key=lambda d: d.name)


def run_cmd(cmd):
    start = time.perf_counter()
    proc = subprocess.run(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        cwd=str(ROOT_DIR),
    )
    duration_ms = (time.perf_counter() - start) * 1000.0
    return proc.returncode, proc.stdout.strip(), proc.stderr.strip(), duration_ms


def main():
    if not SOLIX_BIN.exists():
        print(f"Error: {SOLIX_BIN} not found. Please build release first.")
        sys.exit(1)

    workloads = get_workloads()
    print(f"Starting threshold sweep across {len(workloads)} workloads and {len(THRESHOLDS)} thresholds...")

    results = {}
    temp_dir = Path(tempfile.gettempdir()) / "solix_sweep"
    temp_dir.mkdir(parents=True, exist_ok=True)

    for w_dir in workloads:
        w_name = w_dir.name
        results[w_name] = {}
        slx_files = sorted(w_dir.glob("*.slx"))
        if len(slx_files) > 1:
            slx_files = sorted(slx_files, key=lambda p: (0 if "lib" in p.name else 1, p.name))

        # Check if multi-file linking
        is_multi = len(slx_files) > 1

        for thresh in THRESHOLDS:
            # Measure execution directly using solix run with --inline-threshold
            run_args = [str(SOLIX_BIN), "run"] + [str(p) for p in slx_files] + ["--inline-threshold", str(thresh)]

            times = []
            for _ in range(ITERATIONS):
                r_code, r_out, _, r_ms = run_cmd(run_args)
                if r_code == 0:
                    times.append(r_ms)

            median_ms = sorted(times)[len(times) // 2] if times else 0.0

            # Measure compiled binary size
            bin_path = temp_dir / f"{w_name}_t{thresh}.slxb"
            if is_multi:
                lib_file = slx_files[0]
                main_file = slx_files[1]
                lib_bin = temp_dir / f"{w_name}_lib_t{thresh}.slxb"
                run_cmd([
                    str(SOLIX_BIN), "compile", str(lib_file),
                    "--inline-threshold", str(thresh),
                    "-o", str(lib_bin)
                ])
                run_cmd([
                    str(SOLIX_BIN), "compile", str(main_file),
                    "--libs", str(lib_bin),
                    "--inline-threshold", str(thresh),
                    "-o", str(bin_path)
                ])
            else:
                run_cmd([
                    str(SOLIX_BIN), "compile", str(slx_files[0]),
                    "--inline-threshold", str(thresh),
                    "-o", str(bin_path)
                ])

            bin_size = bin_path.stat().st_size if bin_path.exists() else 0

            results[w_name][thresh] = {
                "bin_size": bin_size,
                "median_ms": round(median_ms, 2),
            }
            print(f"  [{w_name:<22}] N={thresh:>2}: {median_ms:>7.2f} ms (size: {bin_size:>5} B)")

    # Print summary Markdown table
    print("\n\n### Empirical Inlining Threshold Sweep Results Table (Execution Time in ms)\n")
    header = "| Workload | N=0 (No Inlining) | N=2 | N=4 | N=6 | N=8 (Default) | N=12 | N=16 | Delta (N=8 vs N=0) |"
    sep = "| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |"
    print(header)
    print(sep)

    for w_name, t_data in results.items():
        row = [f"`{w_name}`"]
        t0_ms = t_data.get(0, {}).get("median_ms", 0.0)
        t8_ms = t_data.get(8, {}).get("median_ms", 0.0)
        for t in THRESHOLDS:
            ms = t_data.get(t, {}).get("median_ms", 0.0)
            if t == 8:
                row.append(f"**{ms:.2f}**")
            else:
                row.append(f"{ms:.2f}")
        delta = f"{((t0_ms - t8_ms) / t0_ms * 100):+.1f}%" if t0_ms > 0 else "N/A"
        row.append(f"**{delta}**")
        print("| " + " | ".join(row) + " |")

    # Save to JSON
    out_json = ROOT_DIR / "benchmarks" / "baselines" / "inlining_threshold_sweep.json"
    out_json.parent.mkdir(parents=True, exist_ok=True)
    with open(out_json, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)
    print(f"\n[INFO] Saved inlining sweep dataset to '{out_json}'")


if __name__ == "__main__":
    main()
