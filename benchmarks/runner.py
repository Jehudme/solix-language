#!/usr/bin/env python3
"""
Solix Multi-Dimensional Benchmarking & Telemetry Suite
Universal Cross-Platform Runner (Linux, macOS, Windows)

Evaluates:
  1. Functional verification gate across all comparative engines (--verify).
  2. Solix internal execution modes: fast mode vs. --safe checked mode (--mode parity).
  3. Cross-language comparative metrics against Lua, LuaJIT, Python, and C.
  4. Cross-platform memory RSS tracking and automated baseline diffing.
"""

import argparse
import ctypes
import json
import os
import platform
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple

OS_NAME = platform.system()  # 'Linux', 'Darwin', 'Windows'

# Try importing psutil if available
try:
    import psutil
    HAS_PSUTIL = True
except ImportError:
    HAS_PSUTIL = False

# Try importing resource on POSIX
try:
    import resource
    HAS_RESOURCE = True
except ImportError:
    HAS_RESOURCE = False


def get_peak_rss_kb(pid: Optional[int] = None) -> int:
    """
    Returns peak resident set size (RSS) in kilobytes across Linux, macOS, and Windows.
    """
    if HAS_PSUTIL and pid is not None:
        try:
            proc = psutil.Process(pid)
            return proc.memory_info().rss // 1024
        except Exception:
            pass

    if OS_NAME == "Linux":
        if pid is not None:
            try:
                status_path = f"/proc/{pid}/status"
                if os.path.exists(status_path):
                    with open(status_path, "r", encoding="utf-8") as f:
                        for line in f:
                            if line.startswith("VmHWM:"):
                                parts = line.split()
                                if len(parts) >= 2:
                                    return int(parts[1])
                            elif line.startswith("VmRSS:"):
                                parts = line.split()
                                if len(parts) >= 2:
                                    return int(parts[1])
            except Exception:
                pass
        if HAS_RESOURCE:
            ru = resource.getrusage(resource.RUSAGE_CHILDREN)
            return ru.ru_maxrss

    elif OS_NAME == "Darwin":
        if HAS_RESOURCE:
            ru = resource.getrusage(resource.RUSAGE_CHILDREN)
            # On macOS (Darwin), ru_maxrss is reported in bytes, not kilobytes
            return ru.ru_maxrss // 1024

    elif OS_NAME == "Windows":
        try:
            # Win32 PROCESS_MEMORY_COUNTERS_EX query via ctypes
            class PROCESS_MEMORY_COUNTERS_EX(ctypes.Structure):
                _fields_ = [
                    ("cb", ctypes.c_ulong),
                    ("PageFaultCount", ctypes.c_ulong),
                    ("PeakWorkingSetSize", ctypes.c_size_t),
                    ("WorkingSetSize", ctypes.c_size_t),
                    ("QuotaPeakPagedPoolUsage", ctypes.c_size_t),
                    ("QuotaPagedPoolUsage", ctypes.c_size_t),
                    ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t),
                    ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
                    ("PagefileUsage", ctypes.c_size_t),
                    ("PeakPagefileUsage", ctypes.c_size_t),
                    ("PrivateUsage", ctypes.c_size_t),
                ]
            if pid is not None:
                handle = ctypes.windll.kernel32.OpenProcess(0x0400 | 0x0010, False, pid)
                if handle:
                    try:
                        counters = PROCESS_MEMORY_COUNTERS_EX()
                        counters.cb = ctypes.sizeof(PROCESS_MEMORY_COUNTERS_EX)
                        if ctypes.windll.psapi.GetProcessMemoryInfo(handle, ctypes.byref(counters), counters.cb):
                            return counters.PeakWorkingSetSize // 1024
                    finally:
                        ctypes.windll.kernel32.CloseHandle(handle)
        except Exception:
            pass

    return 0


class BenchmarkRunner:
    def __init__(self, root_dir: Path, config_file: Optional[Path] = None):
        self.root_dir = root_dir
        self.workloads_dir = root_dir / "benchmarks" / "workloads"
        self.baselines_dir = root_dir / "benchmarks" / "baselines"
        self.config = self._load_config(config_file or (root_dir / "benchmarks" / "config.json"))
        self.c_bins_cache: Dict[str, Path] = {}

    def _load_config(self, config_path: Path) -> dict:
        if config_path.exists():
            with open(config_path, "r", encoding="utf-8") as f:
                return json.load(f)
        return {
            "engines": {
                "solix": "./target/release/solix",
                "python3": "python3",
                "lua": "lua",
                "luajit": "luajit",
                "gcc": "gcc",
            },
            "defaults": {"warmup": 2, "iterations": 5},
        }

    def resolve_engine(self, engine_name: str) -> Optional[str]:
        if engine_name == "solix":
            candidate_binary = os.environ.get("SOLIX_BIN")
            if candidate_binary and Path(candidate_binary).exists():
                return str(Path(candidate_binary).resolve())
        engines = self.config.get("engines", {})
        cmd = engines.get(engine_name, engine_name)
        if cmd.startswith("./") or cmd.startswith("../"):
            abs_cmd = (self.root_dir / cmd).resolve()
            if abs_cmd.exists() and os.access(abs_cmd, os.X_OK):
                return str(abs_cmd)
            debug_fallback = self.root_dir / "target" / "debug" / "solix"
            if debug_fallback.exists() and os.access(debug_fallback, os.X_OK):
                return str(debug_fallback)
        resolved = shutil.which(cmd)
        return resolved

    def get_workload_dirs(self, filter_pattern: Optional[str] = None) -> List[Path]:
        if not self.workloads_dir.exists():
            return []
        dirs = [d for d in self.workloads_dir.iterdir() if d.is_dir() and not d.name.startswith(".")]
        dirs.sort(key=lambda d: d.name)
        if filter_pattern:
            dirs = [d for d in dirs if filter_pattern in d.name]
        return dirs

    def compile_c_workload(self, c_file: Path) -> Optional[Path]:
        if str(c_file) in self.c_bins_cache:
            return self.c_bins_cache[str(c_file)]
        gcc = self.resolve_engine("gcc")
        if not gcc:
            return None
        temp_build_dir = Path(tempfile.gettempdir()) / "solix_bench_build"
        temp_build_dir.mkdir(parents=True, exist_ok=True)
        out_bin = temp_build_dir / (c_file.parent.name + "_" + c_file.stem + ("" if OS_NAME != "Windows" else ".exe"))
        try:
            cmd = [gcc, "-O3", str(c_file), "-o", str(out_bin), "-lm"]
            res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, check=False)
            if res.returncode == 0 and out_bin.exists():
                self.c_bins_cache[str(c_file)] = out_bin
                return out_bin
        except Exception:
            pass
        return None

    def execute_command(self, cmd: List[str]) -> Tuple[int, str, float, int]:
        """
        Executes a command and returns (exit_code, stdout, duration_ms, peak_rss_kb).
        """
        start_time = time.perf_counter()
        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            cwd=str(self.root_dir),
        )
        stdout, stderr = process.communicate()
        duration_ms = (time.perf_counter() - start_time) * 1000.0
        rss_kb = get_peak_rss_kb(process.pid)
        return process.returncode, stdout.strip(), duration_ms, rss_kb

    def extract_result_checksum(self, output: str) -> Optional[str]:
        for line in output.splitlines():
            line = line.strip()
            if line.startswith("RESULT:"):
                return line[len("RESULT:"):].strip()
        return None

    def run_workload_verification(self, workload_dir: Path) -> Tuple[bool, str, Dict[str, Optional[str]]]:
        """
        Runs the workload across all available engines and verifies identical checksum outputs.
        """
        workload_name = workload_dir.name
        results = {}

        # 1. Solix execution
        solix_bin = self.resolve_engine("solix")
        if solix_bin:
            slx_files = sorted(workload_dir.glob("*.slx"))
            # For 16_multi_binary_link ensure lib.slx comes before main.slx if present
            if len(slx_files) > 1:
                slx_files = sorted(slx_files, key=lambda p: (0 if "lib" in p.name else 1, p.name))
            cmd = [solix_bin, "run"] + [str(p) for p in slx_files]
            code, out, _, _ = self.execute_command(cmd)
            results["solix"] = self.extract_result_checksum(out) if code == 0 else None

        # 2. Python execution
        py_bin = self.resolve_engine("python3")
        py_files = list(workload_dir.glob("*.py"))
        if py_bin and py_files:
            cmd = [py_bin, str(py_files[0])]
            code, out, _, _ = self.execute_command(cmd)
            results["python"] = self.extract_result_checksum(out) if code == 0 else None

        # 3. Lua execution
        lua_bin = self.resolve_engine("lua")
        lua_files = list(workload_dir.glob("*.lua"))
        if lua_bin and lua_files:
            cmd = [lua_bin, str(lua_files[0])]
            code, out, _, _ = self.execute_command(cmd)
            results["lua"] = self.extract_result_checksum(out) if code == 0 else None

        # 4. LuaJIT execution
        luajit_bin = self.resolve_engine("luajit")
        if luajit_bin and lua_files:
            cmd = [luajit_bin, str(lua_files[0])]
            code, out, _, _ = self.execute_command(cmd)
            results["luajit"] = self.extract_result_checksum(out) if code == 0 else None

        # 5. C execution
        c_files = list(workload_dir.glob("*.c"))
        if c_files:
            c_bin = self.compile_c_workload(c_files[0])
            if c_bin:
                code, out, _, _ = self.execute_command([str(c_bin)])
                results["c"] = self.extract_result_checksum(out) if code == 0 else None

        # Verification check
        if results.get("solix") is None:
            return False, "Solix execution failed or produced no checksum", results

        target_checksum = results["solix"]
        all_match = all(v == target_checksum for v in results.values() if v is not None)
        return all_match, target_checksum, results

    def benchmark_workload_engine(
        self,
        cmd: List[str],
        warmup_runs: int = 2,
        timed_runs: int = 5,
    ) -> Dict[str, float]:
        """
        Runs warmup and timed iterations for a single engine command, computing statistics.
        """
        # Warmup
        for _ in range(warmup_runs):
            self.execute_command(cmd)

        durations = []
        peak_rss_list = []
        for _ in range(timed_runs):
            code, out, duration_ms, rss_kb = self.execute_command(cmd)
            if code == 0:
                durations.append(duration_ms)
                if rss_kb > 0:
                    peak_rss_list.append(rss_kb)

        if not durations:
            return {"median_ms": 0.0, "min_ms": 0.0, "max_ms": 0.0, "stdev_ms": 0.0, "rss_kb": 0}

        med_ms = statistics.median(durations)
        min_ms = min(durations)
        max_ms = max(durations)
        stdev_ms = statistics.stdev(durations) if len(durations) > 1 else 0.0
        rss_kb = int(statistics.median(peak_rss_list)) if peak_rss_list else 0

        return {
            "median_ms": round(med_ms, 2),
            "min_ms": round(min_ms, 2),
            "max_ms": round(max_ms, 2),
            "stdev_ms": round(stdev_ms, 2),
            "rss_kb": rss_kb,
        }

    def run_all_benchmarks(
        self,
        mode: str = "fast",
        warmup: int = 2,
        iterations: int = 5,
        filter_pattern: Optional[str] = None,
    ) -> dict:
        """
        Runs benchmarks across all workloads and active engines.
        """
        workload_dirs = self.get_workload_dirs(filter_pattern)
        solix_bin = self.resolve_engine("solix")
        py_bin = self.resolve_engine("python3")
        lua_bin = self.resolve_engine("lua")
        luajit_bin = self.resolve_engine("luajit")

        results = {
            "metadata": {
                "os": OS_NAME,
                "mode": mode,
                "warmup": warmup,
                "iterations": iterations,
                "timestamp": time.time(),
            },
            "workloads": {},
        }

        for wdir in workload_dirs:
            wname = wdir.name
            results["workloads"][wname] = {}
            slx_files = sorted(wdir.glob("*.slx"))
            if len(slx_files) > 1:
                slx_files = sorted(slx_files, key=lambda p: (0 if "lib" in p.name else 1, p.name))

            # Solix Fast Mode
            if solix_bin and slx_files:
                fast_cmd = [solix_bin, "run"] + [str(p) for p in slx_files]
                results["workloads"][wname]["solix_fast"] = self.benchmark_workload_engine(
                    fast_cmd, warmup, iterations
                )

            # Solix Safe Mode
            if solix_bin and slx_files and mode in ("safe", "parity"):
                safe_cmd = [solix_bin, "run"] + [str(p) for p in slx_files] + ["--safe"]
                results["workloads"][wname]["solix_safe"] = self.benchmark_workload_engine(
                    safe_cmd, warmup, iterations
                )

            # Comparative: Python
            py_files = list(wdir.glob("*.py"))
            if py_bin and py_files and mode != "parity":
                cmd = [py_bin, str(py_files[0])]
                results["workloads"][wname]["python3"] = self.benchmark_workload_engine(
                    cmd, warmup, iterations
                )

            # Comparative: Lua
            lua_files = list(wdir.glob("*.lua"))
            if lua_bin and lua_files and mode != "parity":
                cmd = [lua_bin, str(lua_files[0])]
                results["workloads"][wname]["lua"] = self.benchmark_workload_engine(
                    cmd, warmup, iterations
                )

            # Comparative: LuaJIT
            if luajit_bin and lua_files and mode != "parity":
                cmd = [luajit_bin, str(lua_files[0])]
                results["workloads"][wname]["luajit"] = self.benchmark_workload_engine(
                    cmd, warmup, iterations
                )

            # Comparative: C
            c_files = list(wdir.glob("*.c"))
            if c_files and mode != "parity":
                c_bin = self.compile_c_workload(c_files[0])
                if c_bin:
                    results["workloads"][wname]["c"] = self.benchmark_workload_engine(
                        [str(c_bin)], warmup, iterations
                    )

        return results


def print_markdown_table(benchmark_data: dict):
    workloads = benchmark_data.get("workloads", {})
    mode = benchmark_data.get("metadata", {}).get("mode", "fast")

    if mode == "parity":
        print("| Workload | Solix Fast (ms) | Solix Safe (ms) | Overhead % | Fast RSS (KB) | Safe RSS (KB) |")
        print("| :--- | :---: | :---: | :---: | :---: | :---: |")
        for wname, edata in sorted(workloads.items()):
            fast_ms = edata.get("solix_fast", {}).get("median_ms", 0.0)
            safe_ms = edata.get("solix_safe", {}).get("median_ms", 0.0)
            fast_rss = edata.get("solix_fast", {}).get("rss_kb", 0)
            safe_rss = edata.get("solix_safe", {}).get("rss_kb", 0)
            overhead = f"{((safe_ms - fast_ms) / fast_ms * 100.0):+.1f}%" if fast_ms > 0 else "N/A"
            print(f"| `{wname}` | {fast_ms:.2f} | {safe_ms:.2f} | {overhead} | {fast_rss} | {safe_rss} |")
    else:
        print("| Workload | Solix (ms) | Python (ms) | Lua (ms) | LuaJIT (ms) | C (ms) | Speedup vs Python | RSS (KB) |")
        print("| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |")
        for wname, edata in sorted(workloads.items()):
            solix_ms = edata.get("solix_fast", {}).get("median_ms", 0.0)
            py_ms = edata.get("python3", {}).get("median_ms", 0.0)
            lua_ms = edata.get("lua", {}).get("median_ms", 0.0)
            luajit_ms = edata.get("luajit", {}).get("median_ms", 0.0)
            c_ms = edata.get("c", {}).get("median_ms", 0.0)
            rss_kb = edata.get("solix_fast", {}).get("rss_kb", 0)

            speedup_str = f"{(py_ms / solix_ms):.2f}x" if solix_ms > 0 and py_ms > 0 else "N/A"
            lua_str = f"{lua_ms:.2f}" if lua_ms > 0 else "-"
            luajit_str = f"{luajit_ms:.2f}" if luajit_ms > 0 else "-"
            c_str = f"{c_ms:.2f}" if c_ms > 0 else "-"
            py_str = f"{py_ms:.2f}" if py_ms > 0 else "-"

            print(f"| `{wname}` | **{solix_ms:.2f}** | {py_str} | {lua_str} | {luajit_str} | {c_str} | {speedup_str} | {rss_kb} |")


def compare_with_baseline(current_data: dict, baseline_path: Path):
    if not baseline_path.exists():
        print(f"Error: Baseline file '{baseline_path}' not found.")
        return

    with open(baseline_path, "r", encoding="utf-8") as f:
        baseline = json.load(f)

    curr_workloads = current_data.get("workloads", {})
    base_workloads = baseline.get("workloads", {})

    print(f"\n>>> Comparing with baseline '{baseline_path.name}':")
    for wname, curr_edata in sorted(curr_workloads.items()):
        base_edata = base_workloads.get(wname, {})
        curr_fast = curr_edata.get("solix_fast", {}).get("median_ms", 0.0)
        base_fast = base_edata.get("solix_fast", {}).get("median_ms", 0.0)
        if base_fast > 0 and curr_fast > 0:
            delta_pct = ((curr_fast - base_fast) / base_fast) * 100.0
            speedup = base_fast / curr_fast
            status = "[PASSED]" if delta_pct <= 2.0 else "[REGRESSION]"
            print(f"  {wname:<25}: {base_fast:8.2f} ms -> {curr_fast:8.2f} ms ({delta_pct:+6.1f}%, {speedup:.2f}x) {status}")
        else:
            print(f"  {wname:<25}: (new or missing data)")


def main():
    parser = argparse.ArgumentParser(description="Solix Universal Cross-Platform Benchmark Suite")
    parser.add_argument("--verify", action="store_true", help="Run correctness verification gate across all engines")
    parser.add_argument("--mode", choices=["fast", "safe", "parity"], default="fast", help="Execution mode")
    parser.add_argument("--warmup", type=int, default=2, help="Number of warmup iterations")
    parser.add_argument("--iterations", type=int, default=5, help="Number of timed benchmark iterations")
    parser.add_argument("--workload", type=str, default=None, help="Filter to specific workload name")
    parser.add_argument("--markdown", action="store_true", help="Output benchmark results table as Markdown")
    parser.add_argument("--save-baseline", type=str, default=None, help="Save benchmark results to JSON baseline file")
    parser.add_argument("--compare", type=str, default=None, help="Compare current run against baseline JSON")

    args = parser.parse_args()
    root_dir = Path(__file__).resolve().parent.parent
    runner = BenchmarkRunner(root_dir)

    if args.verify:
        print(f"=== Running Solix Benchmark Correctness Verification Gate ({OS_NAME}) ===")
        workload_dirs = runner.get_workload_dirs(args.workload)
        all_passed = True
        for wdir in workload_dirs:
            passed, checksum, results = runner.run_workload_verification(wdir)
            status_str = "PASS" if passed else "FAIL"
            print(f"  [{status_str}] {wdir.name:<25} Checksum: {checksum} (Engines: {list(results.keys())})")
            if not passed:
                all_passed = False
                for eng, csum in results.items():
                    print(f"       -> {eng}: {csum}")
        if not all_passed:
            print("\n❌ Verification gate failed: output checksum mismatch detected!")
            sys.exit(1)
        else:
            print("\n✅ Verification gate passed: all workloads produced identical results across engines.")
            return

    # Benchmark Execution
    benchmark_data = runner.run_all_benchmarks(
        mode=args.mode,
        warmup=args.warmup,
        iterations=args.iterations,
        filter_pattern=args.workload,
    )

    if args.markdown:
        print_markdown_table(benchmark_data)
    else:
        print(json.dumps(benchmark_data, indent=2))

    if args.save_baseline:
        out_path = Path(args.save_baseline)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        with open(out_path, "w", encoding="utf-8") as f:
            json.dump(benchmark_data, f, indent=2)
        print(f"\n[INFO] Saved benchmark baseline to '{out_path}'")

    if args.compare:
        compare_with_baseline(benchmark_data, Path(args.compare))


if __name__ == "__main__":
    main()
