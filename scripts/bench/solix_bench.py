#!/usr/bin/env python3
"""Solix Universal Multi-Dimensional Benchmarking & Telemetry CLI.

Subcommands:
  verify   Run cross-engine output checksum parity gate across workloads.
  run      Execute Solix and comparative language benchmarks with JSON/table/Markdown output.
  compare  Compare two or more benchmark JSON baselines (CI regression gate + multi-baseline tables).
"""

from __future__ import annotations

import argparse
import ctypes
import json
import math
import os
from pathlib import Path
import platform
import shutil
import statistics
import subprocess
import sys
import tempfile
import threading
import time
from typing import Any, Dict, List, Optional, Tuple


if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

OS_NAME = platform.system()

try:
    import psutil

    HAS_PSUTIL = True
except ImportError:
    HAS_PSUTIL = False

try:
    import resource

    HAS_RESOURCE = True
except ImportError:
    HAS_RESOURCE = False


def find_repo_root(start_path: Optional[Path] = None) -> Path:
    """Locates the Solix repository root containing Cargo.toml and benchmarks/."""
    candidate = (start_path or Path(__file__)).resolve()
    for parent_dir in (candidate, *candidate.parents):
        if (parent_dir / "Cargo.toml").is_file() and (
            parent_dir / "benchmarks"
        ).is_dir():
            return parent_dir
    return Path(__file__).resolve().parent.parent.parent


def get_peak_rss_kb(pid: Optional[int] = None) -> int:
    """Returns peak resident set size (RSS) in kilobytes across Linux, macOS, and Windows."""
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
                    with open(status_path, "r", encoding="utf-8") as status_file:
                        for raw_line in status_file:
                            if raw_line.startswith("VmHWM:") or raw_line.startswith(
                                "VmRSS:"
                            ):
                                parts = raw_line.split()
                                if len(parts) >= 2:
                                    return int(parts[1])
            except Exception:
                pass
        if HAS_RESOURCE:
            usage = resource.getrusage(resource.RUSAGE_CHILDREN)
            return usage.ru_maxrss

    elif OS_NAME == "Darwin":
        if HAS_RESOURCE:
            usage = resource.getrusage(resource.RUSAGE_CHILDREN)
            return usage.ru_maxrss // 1024

    elif OS_NAME == "Windows":
        try:
            class ProcessMemoryCountersEx(ctypes.Structure):
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
                handle = ctypes.windll.kernel32.OpenProcess(
                    0x0400 | 0x0010, False, pid
                )
                if handle:
                    try:
                        counters = ProcessMemoryCountersEx()
                        counters.cb = ctypes.sizeof(ProcessMemoryCountersEx)
                        if ctypes.windll.psapi.GetProcessMemoryInfo(
                            handle, ctypes.byref(counters), counters.cb
                        ):
                            return counters.PeakWorkingSetSize // 1024
                    finally:
                        ctypes.windll.kernel32.CloseHandle(handle)
        except Exception:
            pass

    return 0


def get_hardware_telemetry() -> Dict[str, Any]:
    """Extracts host machine hardware telemetry (CPU model, cores, total RAM, OS kernel)."""
    detected_cpu = "Unknown CPU"
    if OS_NAME == "Linux":
        try:
            cpu_info_path = Path("/proc/cpuinfo")
            if cpu_info_path.exists():
                for raw_line in cpu_info_path.read_text(
                    encoding="utf-8", errors="replace"
                ).splitlines():
                    if raw_line.startswith("model name"):
                        line_parts = raw_line.split(":", 1)
                        if len(line_parts) == 2:
                            detected_cpu = line_parts[1].strip()
                            break
        except Exception:
            pass
    elif OS_NAME == "Darwin":
        try:
            sysctl_proc = subprocess.run(
                ["sysctl", "-n", "machdep.cpu.brand_string"],
                capture_output=True,
                text=True,
                check=False,
            )
            if sysctl_proc.returncode == 0 and sysctl_proc.stdout.strip():
                detected_cpu = sysctl_proc.stdout.strip()
        except Exception:
            pass
    elif OS_NAME == "Windows":
        try:
            wmic_proc = subprocess.run(
                ["wmic", "cpu", "get", "name"],
                capture_output=True,
                text=True,
                check=False,
            )
            if wmic_proc.returncode == 0:
                cpu_lines = [
                    entry.strip()
                    for entry in wmic_proc.stdout.splitlines()
                    if entry.strip() and entry.strip() != "Name"
                ]
                if cpu_lines:
                    detected_cpu = cpu_lines[0]
        except Exception:
            pass

    if detected_cpu == "Unknown CPU" and platform.processor():
        detected_cpu = platform.processor()

    cpu_cores_count = os.cpu_count() or 1

    total_ram_megabytes = 0
    if HAS_PSUTIL:
        try:
            total_ram_megabytes = int(psutil.virtual_memory().total // (1024 * 1024))
        except Exception:
            pass

    if total_ram_megabytes == 0 and OS_NAME == "Linux":
        try:
            mem_info_path = Path("/proc/meminfo")
            if mem_info_path.exists():
                for raw_line in mem_info_path.read_text(
                    encoding="utf-8", errors="replace"
                ).splitlines():
                    if raw_line.startswith("MemTotal:"):
                        line_parts = raw_line.split()
                        if len(line_parts) >= 2:
                            total_ram_megabytes = int(line_parts[1]) // 1024
                            break
        except Exception:
            pass
    elif total_ram_megabytes == 0 and OS_NAME == "Darwin":
        try:
            sysctl_mem = subprocess.run(
                ["sysctl", "-n", "hw.memsize"],
                capture_output=True,
                text=True,
                check=False,
            )
            if sysctl_mem.returncode == 0 and sysctl_mem.stdout.strip():
                total_ram_megabytes = int(
                    int(sysctl_mem.stdout.strip()) // (1024 * 1024)
                )
        except Exception:
            pass
    elif total_ram_megabytes == 0 and OS_NAME == "Windows":
        try:
            wmic_mem = subprocess.run(
                ["wmic", "computersystem", "get", "TotalPhysicalMemory"],
                capture_output=True,
                text=True,
                check=False,
            )
            if wmic_mem.returncode == 0:
                mem_lines = [
                    entry.strip()
                    for entry in wmic_mem.stdout.splitlines()
                    if entry.strip() and entry.strip() != "TotalPhysicalMemory"
                ]
                if mem_lines:
                    total_ram_megabytes = int(
                        int(mem_lines[0]) // (1024 * 1024)
                    )
        except Exception:
            pass

    return {
        "cpu_model": detected_cpu,
        "cpu_cores": cpu_cores_count,
        "total_ram_mb": total_ram_megabytes,
        "os_name": OS_NAME,
        "os_kernel": platform.platform(),
        "arch": platform.machine(),
    }


class BenchmarkRunner:
    """Executes benchmark verification and timed multi-engine suites."""

    def __init__(
        self,
        root_dir: Path,
        config_file: Optional[Path] = None,
        solix_bin_override: Optional[str] = None,
        pin_cpu: bool = False,
    ) -> None:
        self.root_dir = root_dir
        self.workloads_dir = root_dir / "benchmarks" / "workloads"
        self.baselines_dir = root_dir / "benchmarks" / "baselines"
        self.config = self._load_config(
            config_file or (root_dir / "benchmarks" / "config.json")
        )
        self.solix_bin_override = solix_bin_override
        self.taskset_bin = (
            shutil.which("taskset") if (pin_cpu and OS_NAME == "Linux") else None
        )
        self.c_bins_cache: Dict[str, Path] = {}
        self.java_bins_cache: Dict[str, Tuple[Path, str]] = {}
        self.solix_bins_cache: Dict[str, Tuple[Path, float]] = {}

    def _load_config(self, config_path: Path) -> Dict[str, Any]:
        if config_path.exists():
            with open(config_path, "r", encoding="utf-8") as cfg_file:
                return json.load(cfg_file)
        return {
            "engines": {
                "solix": "./target/release/solix",
                "python3": "python3",
                "lua": "lua",
                "luajit": "luajit",
                "gcc": "gcc",
                "javac": "javac",
                "java": "java",
            },
            "defaults": {"warmup": 3, "iterations": 11},
        }

    def resolve_engine(self, engine_name: str) -> Optional[str]:
        """Resolves executable path for `solix`, `python3`, `lua`, `luajit`, `gcc`, `javac`, or `java`."""
        if engine_name == "solix":
            if self.solix_bin_override and Path(self.solix_bin_override).exists():
                return str(Path(self.solix_bin_override).resolve())
            candidate_binary = os.environ.get("SOLIX_BIN")
            if candidate_binary and Path(candidate_binary).exists():
                return str(Path(candidate_binary).resolve())

        engines = self.config.get("engines", {})
        cmd = engines.get(engine_name, engine_name)
        exe_suffix = ".exe" if OS_NAME == "Windows" else ""

        if cmd.startswith("./") or cmd.startswith("../"):
            abs_cmd = (self.root_dir / f"{cmd}{exe_suffix}").resolve()
            if abs_cmd.exists() and (
                OS_NAME == "Windows" or os.access(abs_cmd, os.X_OK)
            ):
                return str(abs_cmd)
            debug_fallback = (
                self.root_dir / "target" / "debug" / f"solix{exe_suffix}"
            )
            if debug_fallback.exists() and (
                OS_NAME == "Windows" or os.access(debug_fallback, os.X_OK)
            ):
                return str(debug_fallback)

        if engine_name == "python3":
            resolved_py = shutil.which(cmd)
            if resolved_py and "WindowsApps" not in resolved_py:
                return resolved_py
            return sys.executable

        return shutil.which(cmd)

    def get_workload_dirs(
        self, filter_pattern: Optional[str] = None
    ) -> List[Path]:
        """Lists sorted workload directories matching optional substring filter."""
        if not self.workloads_dir.exists():
            return []
        dirs = [
            entry
            for entry in self.workloads_dir.iterdir()
            if entry.is_dir() and not entry.name.startswith(".")
        ]
        dirs.sort(key=lambda entry: entry.name)
        if filter_pattern:
            dirs = [entry for entry in dirs if filter_pattern in entry.name]
        return dirs

    def compile_c_workload(self, c_file: Path) -> Optional[Path]:
        """Compiles a C comparative workload with `-O3 -lm` into temp cache."""
        if str(c_file) in self.c_bins_cache:
            return self.c_bins_cache[str(c_file)]
        gcc_bin = self.resolve_engine("gcc")
        if not gcc_bin:
            return None
        temp_build_dir = Path(tempfile.gettempdir()) / "solix_bench_build"
        temp_build_dir.mkdir(parents=True, exist_ok=True)
        suffix = "" if OS_NAME != "Windows" else ".exe"
        out_bin = temp_build_dir / f"{c_file.parent.name}_{c_file.stem}{suffix}"
        try:
            cmd = [gcc_bin, "-O3", str(c_file), "-o", str(out_bin), "-lm"]
            res = subprocess.run(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                encoding="utf-8",
                errors="replace",
                check=False,
            )
            if res.returncode == 0 and out_bin.exists():
                self.c_bins_cache[str(c_file)] = out_bin
                return out_bin
        except Exception:
            pass
        return None

    def compile_java_workload(
        self, java_file: Path
    ) -> Optional[Tuple[Path, str]]:
        """Compiles a Java comparative workload into temp cache, returning (class_dir, main_class)."""
        if str(java_file) in self.java_bins_cache:
            return self.java_bins_cache[str(java_file)]
        javac_bin = self.resolve_engine("javac")
        java_bin = self.resolve_engine("java")
        if not javac_bin or not java_bin:
            return None
        temp_build_dir = (
            Path(tempfile.gettempdir())
            / "solix_bench_build"
            / f"java_{java_file.parent.name}_{java_file.stem}"
        )
        temp_build_dir.mkdir(parents=True, exist_ok=True)
        try:
            compile_cmd = [javac_bin, "-d", str(temp_build_dir), str(java_file)]
            compile_res = subprocess.run(
                compile_cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                encoding="utf-8",
                errors="replace",
                check=False,
            )
            if compile_res.returncode == 0:
                main_class = java_file.stem
                self.java_bins_cache[str(java_file)] = (temp_build_dir, main_class)
                return temp_build_dir, main_class
        except Exception:
            pass
        return None

    def execute_command(self, cmd: List[str]) -> Tuple[int, str, float, int]:
        """Executes a command and returns `(exit_code, stdout, duration_ms, peak_rss_kb)`."""
        actual_cmd = (
            [self.taskset_bin, "-c", "0"] + cmd if self.taskset_bin else cmd
        )
        start_time = time.perf_counter()
        process = subprocess.Popen(
            actual_cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            encoding="utf-8",
            errors="replace",
            cwd=str(self.root_dir),
        )
        measured_peak_rss_kilobytes = 0
        stop_sampling_event = threading.Event()

        def sample_memory_telemetry() -> None:
            nonlocal measured_peak_rss_kilobytes
            if OS_NAME != "Linux" or process.pid is None:
                return
            status_file_path = Path(f"/proc/{process.pid}/status")
            while not stop_sampling_event.is_set():
                try:
                    if status_file_path.is_file():
                        for status_line in status_file_path.read_text(
                            encoding="utf-8", errors="replace"
                        ).splitlines():
                            if status_line.startswith(("VmHWM:", "VmRSS:")):
                                status_parts = status_line.split()
                                if len(status_parts) >= 2:
                                    status_value = int(status_parts[1])
                                    if status_value > measured_peak_rss_kilobytes:
                                        measured_peak_rss_kilobytes = status_value
                except Exception:
                    break
                time.sleep(0.001)

        sampler_thread = threading.Thread(
            target=sample_memory_telemetry, daemon=True
        )
        sampler_thread.start()

        stdout, _stderr = process.communicate()
        duration_ms = (time.perf_counter() - start_time) * 1000.0
        stop_sampling_event.set()
        sampler_thread.join(timeout=0.01)
        rss_kb = (
            measured_peak_rss_kilobytes
            if measured_peak_rss_kilobytes > 0
            else get_peak_rss_kb(process.pid)
        )
        return process.returncode, stdout.strip(), duration_ms, rss_kb

    def extract_result_checksum(self, output: str) -> Optional[str]:
        """Extracts trailing `RESULT:<value>` verification checksum line."""
        for raw_line in output.splitlines():
            stripped = raw_line.strip()
            if stripped.startswith("RESULT:"):
                return stripped[len("RESULT:") :].strip()
        return None

    def _ordered_solix_files(self, workload_dir: Path) -> List[Path]:
        slx_files = sorted(workload_dir.glob("*.slx"))
        if len(slx_files) > 1:
            slx_files = sorted(
                slx_files,
                key=lambda item: (0 if "lib" in item.name else 1, item.name),
            )
        return slx_files

    def compile_solix_workload(
        self, slx_files: List[Path], profile: str = "release"
    ) -> Tuple[Optional[Path], float]:
        """Compiles Solix source files into an executable .slxb binary before timed VM execution."""
        if not slx_files:
            return None, 0.0
        cache_key = f"{slx_files[0].parent.name}_{profile}"
        if cache_key in self.solix_bins_cache:
            return self.solix_bins_cache[cache_key]

        solix_bin = self.resolve_engine("solix")
        if not solix_bin:
            return None, 0.0

        temp_build_dir = Path(tempfile.gettempdir()) / "solix_bench_build"
        temp_build_dir.mkdir(parents=True, exist_ok=True)
        out_binary = temp_build_dir / f"{cache_key}.slxb"

        cmd = (
            [solix_bin, "compile"]
            + [str(path) for path in slx_files]
            + ["-o", str(out_binary), "--profile", profile]
        )
        # Warm up filesystem caches once, then sample 3 passes and use the median
        # to eliminate cold-start subprocess variance in CI telemetry.
        compile_code, _compile_out, _compile_dur, _compile_rss = self.execute_command(cmd)
        if compile_code != 0 or not out_binary.exists():
            return None, 0.0

        compile_samples_ms: List[float] = []
        for _ in range(3):
            start_time = time.perf_counter()
            sample_code, _sample_out, _sample_dur, _sample_rss = self.execute_command(cmd)
            elapsed_ms = (time.perf_counter() - start_time) * 1000.0
            if sample_code != 0:
                return None, 0.0
            compile_samples_ms.append(elapsed_ms)

        compile_ms = statistics.median(compile_samples_ms)
        if out_binary.exists():
            cached_result = (out_binary, round(compile_ms, 2))
            self.solix_bins_cache[cache_key] = cached_result
            return cached_result
        return None, 0.0

    def run_workload_verification(
        self, workload_dir: Path
    ) -> Tuple[bool, str, Dict[str, Optional[str]]]:
        """Runs a workload across all available engines and checks checksum parity."""
        results: Dict[str, Optional[str]] = {}

        solix_bin = self.resolve_engine("solix")
        if solix_bin:
            slx_files = self._ordered_solix_files(workload_dir)
            if slx_files:
                compiled_bin, _ = self.compile_solix_workload(
                    slx_files, profile="release"
                )
                if compiled_bin:
                    cmd = [solix_bin, "run", str(compiled_bin)]
                else:
                    cmd = [solix_bin, "run"] + [str(path) for path in slx_files]
                code, out, _, _ = self.execute_command(cmd)
                results["solix"] = (
                    self.extract_result_checksum(out) if code == 0 else None
                )

        py_bin = self.resolve_engine("python3")
        py_files = sorted(workload_dir.glob("*.py"))
        if py_bin and py_files:
            code, out, _, _ = self.execute_command([py_bin, str(py_files[0])])
            results["python"] = (
                self.extract_result_checksum(out) if code == 0 else None
            )

        lua_bin = self.resolve_engine("lua")
        lua_files = sorted(workload_dir.glob("*.lua"))
        if lua_bin and lua_files:
            code, out, _, _ = self.execute_command([lua_bin, str(lua_files[0])])
            results["lua"] = (
                self.extract_result_checksum(out) if code == 0 else None
            )

        luajit_bin = self.resolve_engine("luajit")
        if luajit_bin and lua_files:
            code, out, _, _ = self.execute_command([luajit_bin, str(lua_files[0])])
            results["luajit"] = (
                self.extract_result_checksum(out) if code == 0 else None
            )

        c_files = sorted(workload_dir.glob("*.c"))
        if c_files:
            c_bin = self.compile_c_workload(c_files[0])
            if c_bin:
                code, out, _, _ = self.execute_command([str(c_bin)])
                results["c"] = (
                    self.extract_result_checksum(out) if code == 0 else None
                )

        java_bin = self.resolve_engine("java")
        java_files = sorted(workload_dir.glob("*.java"))
        if java_bin and java_files:
            java_target = self.compile_java_workload(java_files[0])
            if java_target:
                class_dir, main_class = java_target
                code, out, _, _ = self.execute_command(
                    [java_bin, "-cp", str(class_dir), main_class]
                )
                results["java"] = (
                    self.extract_result_checksum(out) if code == 0 else None
                )

        target_checksum = results.get("solix")
        if target_checksum is None:
            return False, "Solix execution failed or produced no checksum", results

        all_match = all(
            value == target_checksum
            for value in results.values()
            if value is not None
        )
        return all_match, target_checksum, results

    def benchmark_workload_engine(
        self,
        cmd: List[str],
        warmup_runs: int = 2,
        timed_runs: int = 5,
        scale_iterations: int = 1,
        benchmark_duration_ms: Optional[float] = None,
    ) -> Dict[str, Any]:
        """Runs cold-start, warmup, and timed iterations for a command and summarizes timings."""
        cold_code, _cold_out, cold_duration_ms, cold_rss_kb = self.execute_command(cmd)
        cold_start_ms = round(cold_duration_ms, 2) if cold_code == 0 else 0.0

        remaining_warmup = max(0, warmup_runs - 1)
        for _ in range(remaining_warmup):
            self.execute_command(cmd)

        durations: List[float] = []
        peak_rss_list: List[int] = []
        total_timed_elapsed_ms = 0.0
        required_samples = max(1, timed_runs)
        sample_counter = 0

        while sample_counter < required_samples or (
            benchmark_duration_ms is not None and total_timed_elapsed_ms < benchmark_duration_ms
        ):
            if scale_iterations > 1:
                batch_start = time.perf_counter()
                batch_peak_rss = 0
                batch_success = True
                for _ in range(scale_iterations):
                    iteration_code, _out, _dur, iteration_rss = self.execute_command(cmd)
                    if iteration_code != 0:
                        batch_success = False
                        break
                    batch_peak_rss = max(batch_peak_rss, iteration_rss)
                batch_elapsed_ms = (time.perf_counter() - batch_start) * 1000.0
                if batch_success:
                    effective_sample_ms = batch_elapsed_ms / scale_iterations
                    durations.append(effective_sample_ms)
                    total_timed_elapsed_ms += batch_elapsed_ms
                    if batch_peak_rss > 0:
                        peak_rss_list.append(batch_peak_rss)
            else:
                sample_code, _sample_out, sample_dur_ms, sample_rss = self.execute_command(cmd)
                if sample_code == 0:
                    durations.append(sample_dur_ms)
                    total_timed_elapsed_ms += sample_dur_ms
                    if sample_rss > 0:
                        peak_rss_list.append(sample_rss)

            sample_counter += 1
            if sample_counter >= 500:
                break

        if not durations:
            return {
                "cold_start_ms": cold_start_ms,
                "median_ms": 0.0,
                "min_ms": 0.0,
                "max_ms": 0.0,
                "stdev_ms": 0.0,
                "rss_kb": cold_rss_kb,
                "samples": 0,
            }

        median_duration = statistics.median(durations)
        minimum_duration = min(durations)
        maximum_duration = max(durations)
        standard_deviation = statistics.stdev(durations) if len(durations) > 1 else 0.0
        median_rss = int(statistics.median(peak_rss_list)) if peak_rss_list else cold_rss_kb

        return {
            "cold_start_ms": cold_start_ms,
            "median_ms": round(median_duration, 2),
            "min_ms": round(minimum_duration, 2),
            "max_ms": round(maximum_duration, 2),
            "stdev_ms": round(standard_deviation, 2),
            "rss_kb": median_rss,
            "samples": len(durations),
        }

    def run_all_benchmarks(
        self,
        mode: str = "fast",
        warmup: int = 3,
        iterations: int = 11,
        filter_pattern: Optional[str] = None,
        label: Optional[str] = None,
        solix_only: bool = False,
        scale_iterations: int = 1,
        benchmark_duration_ms: Optional[float] = None,
        no_precompile: bool = False,
    ) -> Dict[str, Any]:
        """Runs benchmarks across all workloads and active engines."""
        workload_dirs = self.get_workload_dirs(filter_pattern)
        solix_bin = self.resolve_engine("solix")
        py_bin = None if solix_only else self.resolve_engine("python3")
        lua_bin = None if solix_only else self.resolve_engine("lua")
        luajit_bin = None if solix_only else self.resolve_engine("luajit")
        java_bin = None if solix_only else self.resolve_engine("java")

        telemetry = get_hardware_telemetry()
        metadata: Dict[str, Any] = {
            "os": OS_NAME,
            "arch": platform.machine(),
            "mode": mode,
            "warmup": warmup,
            "iterations": iterations,
            "scale_iterations": scale_iterations,
            "precompiled": not no_precompile,
            "timestamp": time.time(),
            "telemetry": telemetry,
            "cpu_model": telemetry["cpu_model"],
            "cpu_cores": telemetry["cpu_cores"],
            "total_ram_mb": telemetry["total_ram_mb"],
            "os_kernel": telemetry["os_kernel"],
        }
        if benchmark_duration_ms is not None:
            metadata["benchmark_duration_ms"] = benchmark_duration_ms
        if label:
            metadata["label"] = label

        results: Dict[str, Any] = {
            "metadata": metadata,
            "workloads": {},
        }

        for wdir in workload_dirs:
            wname = wdir.name
            results["workloads"][wname] = {}
            slx_files = self._ordered_solix_files(wdir)

            compile_duration_ms = 0.0
            precompiled_path = None
            if solix_bin and slx_files and not no_precompile:
                precompiled_path, compile_duration_ms = self.compile_solix_workload(
                    slx_files, profile="release"
                )

            if solix_bin and slx_files:
                if precompiled_path:
                    fast_cmd = [solix_bin, "run", str(precompiled_path)]
                else:
                    fast_cmd = [solix_bin, "run"] + [str(path) for path in slx_files]
                engine_stats = self.benchmark_workload_engine(
                    fast_cmd,
                    warmup_runs=warmup,
                    timed_runs=iterations,
                    scale_iterations=scale_iterations,
                    benchmark_duration_ms=benchmark_duration_ms,
                )
                if compile_duration_ms > 0.0:
                    engine_stats["compile_ms"] = compile_duration_ms
                results["workloads"][wname]["solix_fast"] = engine_stats

            if solix_bin and slx_files and mode in ("safe", "parity"):
                if precompiled_path:
                    safe_cmd = [solix_bin, "run", str(precompiled_path), "--safe"]
                else:
                    safe_cmd = (
                        [solix_bin, "run"]
                        + [str(path) for path in slx_files]
                        + ["--safe"]
                    )
                safe_stats = self.benchmark_workload_engine(
                    safe_cmd,
                    warmup_runs=warmup,
                    timed_runs=iterations,
                    scale_iterations=scale_iterations,
                    benchmark_duration_ms=benchmark_duration_ms,
                )
                if compile_duration_ms > 0.0:
                    safe_stats["compile_ms"] = compile_duration_ms
                results["workloads"][wname]["solix_safe"] = safe_stats

            if solix_only or mode == "parity":
                continue

            py_files = sorted(wdir.glob("*.py"))
            if py_bin and py_files:
                results["workloads"][wname]["python3"] = (
                    self.benchmark_workload_engine(
                        [py_bin, str(py_files[0])],
                        warmup_runs=warmup,
                        timed_runs=iterations,
                        scale_iterations=scale_iterations,
                        benchmark_duration_ms=benchmark_duration_ms,
                    )
                )

            lua_files = sorted(wdir.glob("*.lua"))
            if lua_bin and lua_files:
                results["workloads"][wname]["lua"] = (
                    self.benchmark_workload_engine(
                        [lua_bin, str(lua_files[0])],
                        warmup_runs=warmup,
                        timed_runs=iterations,
                        scale_iterations=scale_iterations,
                        benchmark_duration_ms=benchmark_duration_ms,
                    )
                )

            if luajit_bin and lua_files:
                results["workloads"][wname]["luajit"] = (
                    self.benchmark_workload_engine(
                        [luajit_bin, str(lua_files[0])],
                        warmup_runs=warmup,
                        timed_runs=iterations,
                        scale_iterations=scale_iterations,
                        benchmark_duration_ms=benchmark_duration_ms,
                    )
                )

            c_files = sorted(wdir.glob("*.c"))
            if c_files:
                c_bin = self.compile_c_workload(c_files[0])
                if c_bin:
                    results["workloads"][wname]["c"] = (
                        self.benchmark_workload_engine(
                            [str(c_bin)],
                            warmup_runs=warmup,
                            timed_runs=iterations,
                            scale_iterations=scale_iterations,
                            benchmark_duration_ms=benchmark_duration_ms,
                        )
                    )

            java_files = sorted(wdir.glob("*.java"))
            if java_bin and java_files:
                java_target = self.compile_java_workload(java_files[0])
                if java_target:
                    class_dir, main_class = java_target
                    results["workloads"][wname]["java"] = (
                        self.benchmark_workload_engine(
                            [java_bin, "-cp", str(class_dir), main_class],
                            warmup_runs=warmup,
                            timed_runs=iterations,
                            scale_iterations=scale_iterations,
                            benchmark_duration_ms=benchmark_duration_ms,
                        )
                    )

        return results


def load_benchmark_data(source_target: str) -> Dict[str, Any]:
    """Loads benchmark JSON data from a filesystem path or raw JSON string."""
    candidate_path = Path(source_target)
    if candidate_path.exists() and candidate_path.is_file():
        with open(candidate_path, "r", encoding="utf-8") as file_handle:
            return json.load(file_handle)

    try:
        return json.loads(source_target)
    except json.JSONDecodeError as decode_error:
        raise ValueError(
            f"Failed to load benchmark data from '{source_target}': "
            f"not an existing file and invalid JSON ({decode_error})"
        ) from decode_error


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

    candidate_files.sort(
        key=lambda file_path: (file_path.stat().st_mtime, file_path.name),
        reverse=True,
    )
    return candidate_files[0]


def fetch_previous_release_baseline_from_github(
    target_tag: Optional[str] = None,
    exclude_tag: Optional[str] = None,
) -> Optional[Dict[str, Any]]:
    """Attempts to fetch previous release baseline JSON using GitHub CLI (`gh`)."""
    try:
        resolved_tag = target_tag
        if not resolved_tag:
            list_process = subprocess.run(
                [
                    "gh",
                    "release",
                    "list",
                    "--limit",
                    "10",
                    "--json",
                    "tagName,isPrerelease,isDraft",
                ],
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

        download_process = subprocess.run(
            [
                "gh",
                "release",
                "download",
                resolved_tag,
                "--pattern",
                "release_baseline.json",
                "--output",
                "-",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if download_process.returncode == 0 and download_process.stdout.strip():
            return json.loads(download_process.stdout)

    except Exception as network_error:
        sys.stderr.write(
            f"[WARNING] GitHub CLI baseline retrieval skipped: {network_error}\n"
        )

    return None


def calculate_geometric_mean(numbers_list: List[float]) -> float:
    """Calculates the geometric mean of positive float ratios."""
    positive_numbers = [number for number in numbers_list if number > 0.0]
    if not positive_numbers:
        return 1.0
    log_sum = sum(math.log(number) for number in positive_numbers)
    return math.exp(log_sum / len(positive_numbers))


def compare_benchmarks(
    current_data: Dict[str, Any],
    previous_data: Optional[Dict[str, Any]],
    regression_threshold_pct: float = 5.0,
    min_delta_ms: float = 0.5,
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
    total_memory_diff_kb = 0
    faster_workloads_count = 0
    stable_workloads_count = 0
    regressed_workloads_count = 0

    for workload_name in all_workload_keys:
        current_entry = current_workloads.get(workload_name, {}).get(
            preferred_engine, {}
        )
        previous_entry = previous_workloads.get(workload_name, {}).get(
            preferred_engine, {}
        )

        current_median_ms = float(current_entry.get("median_ms", 0.0))
        previous_median_ms = float(previous_entry.get("median_ms", 0.0))
        current_rss_kb = int(current_entry.get("rss_kb", 0))
        previous_rss_kb = int(previous_entry.get("rss_kb", 0))

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
            "badge": "UNKNOWN",
            "is_regression": False,
        }

        if previous_median_ms <= 0.0 and current_median_ms > 0.0:
            row_metric["status"] = "NEW"
            row_metric["badge"] = "NEW"
            row_metric["memory_diff_mb"] = round(current_rss_kb / 1024.0, 2)
        elif current_median_ms <= 0.0 and previous_median_ms > 0.0:
            row_metric["status"] = "REMOVED"
            row_metric["badge"] = "REMOVED"
            row_metric["memory_diff_mb"] = round(-previous_rss_kb / 1024.0, 2)
        elif current_median_ms > 0.0 and previous_median_ms > 0.0:
            speedup_ratio = previous_median_ms / current_median_ms
            speedup_ratios_list.append(speedup_ratio)
            speedup_percentage = (
                (previous_median_ms - current_median_ms) / previous_median_ms
            ) * 100.0
            abs_delta_ms = abs(current_median_ms - previous_median_ms)

            memory_diff_kb = current_rss_kb - previous_rss_kb
            total_memory_diff_kb += memory_diff_kb
            memory_diff_mb = round(memory_diff_kb / 1024.0, 2)

            row_metric["speedup_ratio"] = round(speedup_ratio, 2)
            row_metric["speedup_pct"] = round(speedup_percentage, 1)
            row_metric["memory_diff_mb"] = memory_diff_mb

            if (
                speedup_percentage > regression_threshold_pct
                and abs_delta_ms >= min_delta_ms
            ):
                row_metric["status"] = "FASTER"
                row_metric["badge"] = "FASTER"
                faster_workloads_count += 1
            elif (
                speedup_percentage < -regression_threshold_pct
                and abs_delta_ms >= min_delta_ms
            ):
                row_metric["status"] = "REGRESSION"
                row_metric["badge"] = "SLOWER"
                row_metric["is_regression"] = True
                regressed_workloads_count += 1
            else:
                row_metric["status"] = "STABLE"
                row_metric["badge"] = "STABLE"
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
            "min_delta_ms": min_delta_ms,
        },
    }


def compare_multiple_baselines(
    baseline_entries: List[Tuple[str, Dict[str, Any]]],
    preferred_engine: str = "solix_fast",
    regression_threshold_pct: float = 5.0,
    min_delta_ms: float = 0.5,
) -> Dict[str, Any]:
    """Compares two or more labeled benchmark baselines side-by-side."""
    if not baseline_entries:
        return {"labels": [], "workloads": [], "summary": {}}

    labels = [label for label, _data in baseline_entries]
    all_workloads = sorted(
        {
            wname
            for _label, dataset in baseline_entries
            for wname in dataset.get("workloads", {}).keys()
        }
    )

    rows: List[Dict[str, Any]] = []
    ratios_vs_first: List[List[float]] = [[] for _ in baseline_entries]
    regressed_latest = 0

    for wname in all_workloads:
        timings: List[float] = []
        rss_values: List[int] = []
        for idx, (_label, dataset) in enumerate(baseline_entries):
            engine_data = (
                dataset.get("workloads", {})
                .get(wname, {})
                .get(preferred_engine, {})
            )
            median_ms = float(engine_data.get("median_ms", 0.0))
            rss_kb = int(engine_data.get("rss_kb", 0))
            timings.append(median_ms)
            rss_values.append(rss_kb)
            if idx > 0 and timings[0] > 0.0 and median_ms > 0.0:
                ratios_vs_first[idx].append(timings[0] / median_ms)

        first_ms = timings[0]
        last_ms = timings[-1]
        net_speedup = (
            round(first_ms / last_ms, 2)
            if first_ms > 0.0 and last_ms > 0.0
            else 1.0
        )
        net_delta_pct = (
            round(((first_ms - last_ms) / first_ms) * 100.0, 1)
            if first_ms > 0.0 and last_ms > 0.0
            else 0.0
        )
        status = "BASELINE"
        if len(timings) >= 2:
            prev_ms = timings[-2]
            if prev_ms <= 0.0 and last_ms > 0.0:
                status = "NEW"
            elif last_ms <= 0.0 and prev_ms > 0.0:
                status = "REMOVED"
            elif prev_ms > 0.0 and last_ms > 0.0:
                step_pct = ((prev_ms - last_ms) / prev_ms) * 100.0
                abs_diff = abs(last_ms - prev_ms)
                if (
                    step_pct > regression_threshold_pct
                    and abs_diff >= min_delta_ms
                ):
                    status = "FASTER"
                elif (
                    step_pct < -regression_threshold_pct
                    and abs_diff >= min_delta_ms
                ):
                    status = "SLOWER"
                    regressed_latest += 1
                else:
                    status = "STABLE"

        rows.append(
            {
                "workload": wname,
                "timings_ms": timings,
                "rss_kb": rss_values,
                "net_speedup_vs_first": net_speedup,
                "net_delta_pct_vs_first": net_delta_pct,
                "latest_status": status,
            }
        )

    geomean_vs_first = [
        round(calculate_geometric_mean(col_ratios), 2) if idx > 0 else 1.0
        for idx, col_ratios in enumerate(ratios_vs_first)
    ]

    return {
        "labels": labels,
        "engine": preferred_engine,
        "workloads": rows,
        "summary": {
            "baseline_count": len(labels),
            "total_workloads": len(rows),
            "geomean_vs_first": geomean_vs_first,
            "latest_regressed_count": regressed_latest,
            "has_regression": regressed_latest > 0,
        },
    }


def extract_cross_language_rows(benchmark_data: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Extracts comparative language metrics (Solix, Python3, Lua, LuaJIT, C, Java) from a run."""
    workloads = benchmark_data.get("workloads", {})
    rows: List[Dict[str, Any]] = []
    for wname, engines in sorted(workloads.items()):
        solix_entry = engines.get("solix_fast", {})
        solix_ms = float(solix_entry.get("median_ms", 0.0))
        compile_ms = float(solix_entry.get("compile_ms", 0.0))
        cold_ms = float(solix_entry.get("cold_start_ms", 0.0))
        safe_ms = float(engines.get("solix_safe", {}).get("median_ms", 0.0))
        py_ms = float(engines.get("python3", {}).get("median_ms", 0.0))
        lua_ms = float(engines.get("lua", {}).get("median_ms", 0.0))
        luajit_ms = float(engines.get("luajit", {}).get("median_ms", 0.0))
        c_ms = float(engines.get("c", {}).get("median_ms", 0.0))
        java_ms = float(engines.get("java", {}).get("median_ms", 0.0))
        rss_kb = int(solix_entry.get("rss_kb", 0))
        has_any_other = any(
            value > 0.0 for value in (py_ms, lua_ms, luajit_ms, c_ms, java_ms, safe_ms)
        )
        if solix_ms <= 0.0 and not has_any_other:
            continue
        rows.append(
            {
                "workload": wname,
                "solix_compile_ms": compile_ms,
                "solix_cold_ms": cold_ms,
                "solix_ms": solix_ms,
                "solix_safe_ms": safe_ms,
                "python3_ms": py_ms,
                "lua_ms": lua_ms,
                "luajit_ms": luajit_ms,
                "c_ms": c_ms,
                "java_ms": java_ms,
                "solix_rss_kb": rss_kb,
                "vs_python": round(py_ms / solix_ms, 2)
                if solix_ms > 0.0 and py_ms > 0.0
                else None,
            }
        )
    return rows


def format_ascii_table(headers: List[str], rows: List[List[str]]) -> str:
    """Renders cleanly aligned monospace ASCII table for terminal inspection."""
    widths = [len(header) for header in headers]
    for row in rows:
        for idx, cell in enumerate(row):
            widths[idx] = max(widths[idx], len(cell))

    def fmt_row(values: List[str]) -> str:
        padded = [
            values[idx].ljust(widths[idx])
            if idx == 0 or idx == len(widths) - 1
            else values[idx].rjust(widths[idx])
            for idx in range(len(headers))
        ]
        return "| " + " | ".join(padded) + " |"

    separator = "+-" + "-+-".join("-" * width for width in widths) + "-+"
    lines = [separator, fmt_row(headers), separator]
    for row in rows:
        lines.append(fmt_row(row))
    lines.append(separator)
    return "\n".join(lines)


def render_cross_language_markdown(benchmark_data: Dict[str, Any]) -> str:
    """Renders cross-language comparison section in GitHub Flavored Markdown."""
    rows = extract_cross_language_rows(benchmark_data)
    if not rows:
        return ""

    has_c = any(row["c_ms"] > 0.0 for row in rows)
    has_lua = any(row["lua_ms"] > 0.0 for row in rows)
    has_luajit = any(row["luajit_ms"] > 0.0 for row in rows)
    has_py = any(row["python3_ms"] > 0.0 for row in rows)
    has_java = any(row.get("java_ms", 0.0) > 0.0 for row in rows)

    if not any((has_c, has_lua, has_luajit, has_py, has_java)):
        return ""

    lines = ["\n### Cross-Language Comparison\n"]
    lines.append(
        "| Workload | Precompile (ms) | Solix VM (ms) | Cold Start (ms) | Python 3 (ms) | Lua (ms) | LuaJIT (ms) | C -O3 (ms) | Java (ms) | vs Python | Solix RSS (KB) |"
    )
    lines.append("| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in rows:
        compile_str = (
            f"{row['solix_compile_ms']:.2f}"
            if row.get("solix_compile_ms", 0.0) > 0.0
            else "-"
        )
        cold_str = (
            f"{row['solix_cold_ms']:.2f}"
            if row.get("solix_cold_ms", 0.0) > 0.0
            else "-"
        )
        solix_str = f"**{row['solix_ms']:.2f}**" if row["solix_ms"] > 0.0 else "-"
        py_str = f"{row['python3_ms']:.2f}" if row["python3_ms"] > 0.0 else "-"
        lua_str = f"{row['lua_ms']:.2f}" if row["lua_ms"] > 0.0 else "-"
        luajit_str = f"{row['luajit_ms']:.2f}" if row["luajit_ms"] > 0.0 else "-"
        c_str = f"{row['c_ms']:.2f}" if row["c_ms"] > 0.0 else "-"
        java_str = f"{row['java_ms']:.2f}" if row.get("java_ms", 0.0) > 0.0 else "-"
        vs_py = f"{row['vs_python']:.2f}x" if row["vs_python"] is not None else "-"
        rss_str = f"{row['solix_rss_kb']}" if row["solix_rss_kb"] > 0 else "-"
        lines.append(
            f"| `{row['workload']}` | {compile_str} | {solix_str} | {cold_str} | {py_str} | {lua_str} | {luajit_str} | {c_str} | {java_str} | {vs_py} | {rss_str} |"
        )
    return "\n".join(lines) + "\n"


def render_run_console_table(
    benchmark_data: Dict[str, Any], include_cross_language: bool = True
) -> str:
    """Renders formatted console ASCII tables for a single benchmark run."""
    workloads = benchmark_data.get("workloads", {})
    metadata = benchmark_data.get("metadata", {})
    telemetry = metadata.get("telemetry", {})
    cpu_model = telemetry.get("cpu_model") or metadata.get("cpu_model")
    cores = telemetry.get("cpu_cores") or metadata.get("cpu_cores")
    ram_mb = telemetry.get("total_ram_mb") or metadata.get("total_ram_mb")
    kernel = telemetry.get("os_kernel") or metadata.get("os_kernel")

    telemetry_banner = ""
    if cpu_model and cores:
        telemetry_banner = (
            f"Hardware Telemetry: {cpu_model} ({cores} cores) | {ram_mb} MB RAM | {kernel}\n\n"
        )

    mode = metadata.get("mode", "fast")

    if mode == "parity":
        headers = [
            "Workload",
            "Precompile (ms)",
            "Solix Fast (ms)",
            "Solix Safe (ms)",
            "Overhead %",
            "Fast RSS (KB)",
            "Safe RSS (KB)",
        ]
        table_rows: List[List[str]] = []
        for wname, edata in sorted(workloads.items()):
            compile_ms = edata.get("solix_fast", {}).get("compile_ms", 0.0)
            compile_str = f"{compile_ms:.2f}" if compile_ms > 0 else "-"
            fast_ms = edata.get("solix_fast", {}).get("median_ms", 0.0)
            safe_ms = edata.get("solix_safe", {}).get("median_ms", 0.0)
            fast_rss = edata.get("solix_fast", {}).get("rss_kb", 0)
            safe_rss = edata.get("solix_safe", {}).get("rss_kb", 0)
            overhead = (
                f"{((safe_ms - fast_ms) / fast_ms * 100.0):+.1f}%"
                if fast_ms > 0
                else "N/A"
            )
            table_rows.append(
                [
                    wname,
                    compile_str,
                    f"{fast_ms:.2f}",
                    f"{safe_ms:.2f}",
                    overhead,
                    str(fast_rss),
                    str(safe_rss),
                ]
            )
        return telemetry_banner + format_ascii_table(headers, table_rows)

    cross_rows = extract_cross_language_rows(benchmark_data)
    if include_cross_language:
        headers = [
            "Workload",
            "Precompile (ms)",
            "Solix (ms)",
            "Cold (ms)",
            "Python 3 (ms)",
            "Lua (ms)",
            "LuaJIT (ms)",
            "C -O3 (ms)",
            "Java (ms)",
            "vs Python",
            "RSS (KB)",
        ]
        table_rows = []
        for row in cross_rows:
            compile_val = (
                f"{row['solix_compile_ms']:.2f}"
                if row.get("solix_compile_ms", 0.0) > 0.0
                else "-"
            )
            cold_val = (
                f"{row['solix_cold_ms']:.2f}"
                if row.get("solix_cold_ms", 0.0) > 0.0
                else "-"
            )
            table_rows.append(
                [
                    row["workload"],
                    compile_val,
                    f"{row['solix_ms']:.2f}" if row["solix_ms"] > 0 else "-",
                    cold_val,
                    f"{row['python3_ms']:.2f}" if row["python3_ms"] > 0 else "-",
                    f"{row['lua_ms']:.2f}" if row["lua_ms"] > 0 else "-",
                    f"{row['luajit_ms']:.2f}" if row["luajit_ms"] > 0 else "-",
                    f"{row['c_ms']:.2f}" if row["c_ms"] > 0 else "-",
                    f"{row['java_ms']:.2f}" if row.get("java_ms", 0.0) > 0 else "-",
                    f"{row['vs_python']:.2f}x"
                    if row["vs_python"] is not None
                    else "-",
                    str(row["solix_rss_kb"]),
                ]
            )
        return telemetry_banner + format_ascii_table(headers, table_rows)

    headers = [
        "Workload",
        "Precompile (ms)",
        "Solix Fast (ms)",
        "Cold (ms)",
        "Min (ms)",
        "Max (ms)",
        "RSS (KB)",
    ]
    table_rows = []
    for wname, edata in sorted(workloads.items()):
        stats = edata.get("solix_fast", {})
        compile_ms = stats.get("compile_ms", 0.0)
        cold_ms = stats.get("cold_start_ms", 0.0)
        table_rows.append(
            [
                wname,
                f"{compile_ms:.2f}" if compile_ms > 0 else "-",
                f"{stats.get('median_ms', 0.0):.2f}",
                f"{cold_ms:.2f}" if cold_ms > 0 else "-",
                f"{stats.get('min_ms', 0.0):.2f}",
                f"{stats.get('max_ms', 0.0):.2f}",
                str(stats.get("rss_kb", 0)),
            ]
        )
    return telemetry_banner + format_ascii_table(headers, table_rows)


def render_run_markdown_table(benchmark_data: Dict[str, Any]) -> str:
    """Renders GitHub Flavored Markdown table for `run --markdown`."""
    workloads = benchmark_data.get("workloads", {})
    metadata = benchmark_data.get("metadata", {})
    mode = metadata.get("mode", "fast")
    lines: List[str] = []

    telemetry = metadata.get("telemetry", {})
    cpu_model = telemetry.get("cpu_model") or metadata.get("cpu_model")
    cores = telemetry.get("cpu_cores") or metadata.get("cpu_cores")
    ram_mb = telemetry.get("total_ram_mb") or metadata.get("total_ram_mb")
    kernel = telemetry.get("os_kernel") or metadata.get("os_kernel")
    if cpu_model and cores:
        lines.append(
            f"> [!NOTE]\n> **Hardware Telemetry:** {cpu_model} ({cores} cores) | {ram_mb} MB RAM | {kernel}\n"
        )

    if mode == "parity":
        lines.append(
            "| Workload | Precompile (ms) | Solix Fast (ms) | Solix Safe (ms) | Overhead % | Fast RSS (KB) | Safe RSS (KB) |"
        )
        lines.append("| :--- | :---: | :---: | :---: | :---: | :---: | :---: |")
        for wname, edata in sorted(workloads.items()):
            compile_ms = edata.get("solix_fast", {}).get("compile_ms", 0.0)
            compile_str = f"{compile_ms:.2f}" if compile_ms > 0 else "-"
            fast_ms = edata.get("solix_fast", {}).get("median_ms", 0.0)
            safe_ms = edata.get("solix_safe", {}).get("median_ms", 0.0)
            fast_rss = edata.get("solix_fast", {}).get("rss_kb", 0)
            safe_rss = edata.get("solix_safe", {}).get("rss_kb", 0)
            overhead = (
                f"{((safe_ms - fast_ms) / fast_ms * 100.0):+.1f}%"
                if fast_ms > 0
                else "N/A"
            )
            lines.append(
                f"| `{wname}` | {compile_str} | {fast_ms:.2f} | {safe_ms:.2f} | {overhead} | {fast_rss} | {safe_rss} |"
            )
    else:
        lines.append(
            "| Workload | Precompile (ms) | Solix (ms) | Cold (ms) | Python (ms) | Lua (ms) | LuaJIT (ms) | C (ms) | Java (ms) | Speedup vs Python | RSS (KB) |"
        )
        lines.append("| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |")
        for wname, edata in sorted(workloads.items()):
            compile_ms = edata.get("solix_fast", {}).get("compile_ms", 0.0)
            cold_ms = edata.get("solix_fast", {}).get("cold_start_ms", 0.0)
            solix_ms = edata.get("solix_fast", {}).get("median_ms", 0.0)
            py_ms = edata.get("python3", {}).get("median_ms", 0.0)
            lua_ms = edata.get("lua", {}).get("median_ms", 0.0)
            luajit_ms = edata.get("luajit", {}).get("median_ms", 0.0)
            c_ms = edata.get("c", {}).get("median_ms", 0.0)
            java_ms = edata.get("java", {}).get("median_ms", 0.0)
            rss_kb = edata.get("solix_fast", {}).get("rss_kb", 0)

            compile_str = f"{compile_ms:.2f}" if compile_ms > 0 else "-"
            cold_str = f"{cold_ms:.2f}" if cold_ms > 0 else "-"
            speedup_str = (
                f"{(py_ms / solix_ms):.2f}x"
                if solix_ms > 0 and py_ms > 0
                else "N/A"
            )
            lua_str = f"{lua_ms:.2f}" if lua_ms > 0 else "-"
            luajit_str = f"{luajit_ms:.2f}" if luajit_ms > 0 else "-"
            c_str = f"{c_ms:.2f}" if c_ms > 0 else "-"
            java_str = f"{java_ms:.2f}" if java_ms > 0 else "-"
            py_str = f"{py_ms:.2f}" if py_ms > 0 else "-"

            lines.append(
                f"| `{wname}` | {compile_str} | **{solix_ms:.2f}** | {cold_str} | {py_str} | {lua_str} | {luajit_str} | {c_str} | {java_str} | {speedup_str} | {rss_kb} |"
            )

        lines.append("")
        lines.append(
            "> [!TIP]\n"
            "> **Cross-Language Column Glossary:**\n"
            "> - **Workload:** Standardized algorithm executed across all language runtimes.\n"
            "> - **Precompile (ms):** Solix compilation latency (`solix compile`) from source code to bytecode.\n"
            "> - **Solix (ms):** Median execution time of the compiled Solix binary on the fast register Virtual Machine.\n"
            "> - **Cold (ms):** Cold-start execution time (clean process invocation including compilation and execution).\n"
            "> - **Python (ms):** Median execution time of CPython 3.12.\n"
            "> - **Lua (ms):** Median execution time of the standard Lua 5.4 interpreter.\n"
            "> - **LuaJIT (ms):** Median execution time of the LuaJIT 2.1 JIT compiler.\n"
            "> - **C (ms):** Median execution time of native C compiled with GCC `-O3`.\n"
            "> - **Java (ms):** Median execution time of OpenJDK JVM HotSpot.\n"
            "> - **Speedup vs Python:** Speedup ratio of Solix relative to CPython (`Python / Solix`).\n"
            "> - **RSS (KB):** Peak Resident Set Size memory consumption in kilobytes."
        )

    return "\n".join(lines)


def render_markdown_comparison_table(
    comparison_results: Dict[str, Any],
    previous_tag_label: str = "Previous Release",
    current_tag_label: str = "Current Release",
    same_host: bool = False,
    cross_language_data: Optional[Dict[str, Any]] = None,
) -> str:
    """Renders formatted GitHub Flavored Markdown table for release notes or PR comments."""
    workload_rows: List[Dict[str, Any]] = comparison_results.get("workloads", [])
    summary_data: Dict[str, Any] = comparison_results.get("summary", {})
    has_previous_data = any(
        row.get("previous_ms", 0.0) > 0.0 for row in workload_rows
    )

    markdown_lines: List[str] = []

    if has_previous_data:
        markdown_lines.append(
            f"### Performance Benchmarks vs {previous_tag_label} (`{current_tag_label}`)\n"
        )
        markdown_lines.append(
            "| Workload | Previous (ms) | Current (ms) | Speedup | Memory Diff | Status |"
        )
        markdown_lines.append("| :--- | :--- | :--- | :--- | :--- | :--- |")

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

        geomean = summary_data.get("geometric_mean_speedup", 1.0)
        total_mem = summary_data.get("total_memory_diff_mb", 0.0)
        faster = summary_data.get("faster_count", 0)
        stable = summary_data.get("stable_count", 0)
        regressed = summary_data.get("regressed_count", 0)
        host_note = (
            " Measured back-to-back on the same CI runner." if same_host else ""
        )

        mem_sign = "+" if total_mem >= 0.0 else ""
        markdown_lines.append("")
        markdown_lines.append(
            f"> [!NOTE]\n"
            f"> **Benchmark Summary:** Geometric mean speedup: **{geomean:.2f}x** "
            f"across {len(workload_rows)} suites. "
            f"Total memory delta: **{mem_sign}{total_mem:.1f} MB**. "
            f"Distribution: {faster} faster, {stable} stable, {regressed} regressed.{host_note}"
        )
        markdown_lines.append("")
        markdown_lines.append(
            "> [!TIP]\n"
            "> **Column Glossary:**\n"
            "> - **Workload:** Standardized benchmark workload evaluating compiler and runtime performance.\n"
            "> - **Previous (ms):** Median wall-clock execution time (in milliseconds) of the baseline release.\n"
            "> - **Current (ms):** Median wall-clock execution time (in milliseconds) of the current pull request branch.\n"
            "> - **Speedup:** Execution ratio relative to previous baseline (`Previous / Current`); positive indicates faster execution.\n"
            "> - **Memory Diff:** Resident Set Size memory consumption delta compared to baseline (`Current - Previous`).\n"
            "> - **Status:** Performance classification (`FASTER` = speedup above threshold, `STABLE` = within ±5% margin, `SLOWER` = regressed)."
        )
    else:
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
                f"| `{workload_name}` | {current_ms:.2f} ms | {current_rss_kb} KB | BASELINE |"
            )

    if cross_language_data:
        cross_md = render_cross_language_markdown(cross_language_data)
        if cross_md:
            markdown_lines.append(cross_md.rstrip())

    return "\n".join(markdown_lines) + "\n"


def render_multi_baseline_console(multi_results: Dict[str, Any]) -> str:
    """Renders multi-baseline comparison as an aligned console ASCII table."""
    labels = multi_results.get("labels", [])
    rows = multi_results.get("workloads", [])
    summary = multi_results.get("summary", {})

    headers = (
        ["Workload"]
        + [f"{label} (ms)" for label in labels]
        + [f"vs {labels[0]}", "Latest Status"]
    )
    table_rows: List[List[str]] = []
    for row in rows:
        timing_cells = [
            f"{val:.2f}" if val > 0.0 else "-" for val in row["timings_ms"]
        ]
        sign = "+" if row["net_delta_pct_vs_first"] >= 0.0 else ""
        vs_first = f"{sign}{row['net_delta_pct_vs_first']:.1f}% ({row['net_speedup_vs_first']:.2f}x)"
        table_rows.append(
            [row["workload"]] + timing_cells + [vs_first, row["latest_status"]]
        )

    geomeans = summary.get("geomean_vs_first", [])
    geomean_line = ", ".join(
        f"{labels[idx]}={geomeans[idx]:.2f}x"
        for idx in range(1, len(labels))
        if idx < len(geomeans)
    )
    output = format_ascii_table(headers, table_rows)
    if geomean_line:
        output += f"\nGeometric mean speedup vs {labels[0]}: {geomean_line}"
    return output


def render_multi_baseline_markdown(
    multi_results: Dict[str, Any],
    cross_language_data: Optional[Dict[str, Any]] = None,
) -> str:
    """Renders multi-baseline comparison in GitHub Flavored Markdown."""
    labels = multi_results.get("labels", [])
    rows = multi_results.get("workloads", [])
    summary = multi_results.get("summary", {})

    lines = [f"### Multi-Baseline Benchmark Comparison ({' vs '.join(labels)})\n"]
    header_cols = (
        ["Workload"]
        + [f"{label} (ms)" for label in labels]
        + [f"Speedup vs {labels[0]}", "Latest Status"]
    )
    lines.append("| " + " | ".join(header_cols) + " |")
    lines.append("| :--- | " + " | ".join(["---:"] * (len(header_cols) - 2)) + " | :--- |")

    for row in rows:
        timing_cells = [
            f"{val:.2f}" if val > 0.0 else "-" for val in row["timings_ms"]
        ]
        sign = "+" if row["net_delta_pct_vs_first"] >= 0.0 else ""
        vs_first = f"{sign}{row['net_delta_pct_vs_first']:.1f}% ({row['net_speedup_vs_first']:.2f}x)"
        lines.append(
            "| "
            + " | ".join(
                [f"`{row['workload']}`"]
                + timing_cells
                + [vs_first, row["latest_status"]]
            )
            + " |"
        )

    geomeans = summary.get("geomean_vs_first", [])
    geomean_text = ", ".join(
        f"**{labels[idx]}**: {geomeans[idx]:.2f}x"
        for idx in range(1, len(labels))
        if idx < len(geomeans)
    )
    if geomean_text:
        lines.append("")
        lines.append(
            f"> [!NOTE]\n> **Geometric Mean Speedup vs `{labels[0]}`:** {geomean_text}."
        )

    lines.append("")
    lines.append(
        "> [!TIP]\n"
        "> **Multi-Baseline Column Glossary:**\n"
        "> - **Workload:** Standard benchmark suite evaluating compiler and runtime performance.\n"
        "> - **Labels (ms):** Median execution time in milliseconds across evaluated releases or branches.\n"
        "> - **Speedup vs First:** Relative execution speed ratio compared to the earliest baseline release.\n"
        "> - **Latest Status:** Current performance health categorization (`FASTER`, `STABLE`, or `SLOWER`)."
    )

    if cross_language_data:
        cross_md = render_cross_language_markdown(cross_language_data)
        if cross_md:
            lines.append(cross_md.rstrip())

    return "\n".join(lines) + "\n"


def inject_into_release_notes(
    notes_file_path: Path,
    markdown_content: str,
) -> None:
    """Appends markdown comparison content into an existing release notes file."""
    existing_content = ""
    if notes_file_path.exists():
        with open(notes_file_path, "r", encoding="utf-8") as notes_file:
            existing_content = notes_file.read()

    separator = (
        "\n\n"
        if existing_content and not existing_content.endswith("\n\n")
        else ""
    )
    updated_content = existing_content + separator + markdown_content

    with open(notes_file_path, "w", encoding="utf-8") as notes_file:
        notes_file.write(updated_content)


def build_parser() -> argparse.ArgumentParser:
    """Constructs the multi-command argument parser for `solix_bench`."""
    parser = argparse.ArgumentParser(
        prog="solix_bench",
        description="Solix Universal Cross-Platform Benchmark CLI Suite",
    )
    subparsers = parser.add_subparsers(dest="subcommand", required=True)

    # Subcommand: verify
    verify_parser = subparsers.add_parser(
        "verify",
        help="Run correctness checksum verification gate across all comparative engines",
    )
    verify_parser.add_argument(
        "--workload",
        type=str,
        default=None,
        help="Filter workloads by substring",
    )
    verify_parser.add_argument(
        "--solix-bin",
        type=str,
        default=None,
        help="Explicit path to solix binary",
    )
    verify_parser.add_argument(
        "--pin-cpu",
        action="store_true",
        help="Pin execution to CPU core 0 via taskset on Linux",
    )

    # Subcommand: run
    run_parser = subparsers.add_parser(
        "run",
        help="Execute benchmark suite and output console table, Markdown, and/or JSON",
    )
    run_parser.add_argument(
        "--mode",
        choices=["fast", "safe", "parity"],
        default="fast",
        help="Execution mode (default: fast)",
    )
    run_parser.add_argument(
        "--warmup",
        type=int,
        default=3,
        help="Number of warmup iterations (default: 3)",
    )
    run_parser.add_argument(
        "--iterations",
        type=int,
        default=11,
        help="Number of timed iterations (default: 11)",
    )
    run_parser.add_argument(
        "--workload",
        type=str,
        default=None,
        help="Filter to specific workload substring",
    )
    run_parser.add_argument(
        "--solix-bin",
        type=str,
        default=None,
        help="Explicit path to solix binary to benchmark",
    )
    run_parser.add_argument(
        "--label",
        type=str,
        default=None,
        help="Optional label stored in baseline metadata (e.g. v1.6.0 or pr-42)",
    )
    run_parser.add_argument(
        "--pin-cpu",
        action="store_true",
        help="Pin benchmark processes to CPU core 0 via taskset on Linux",
    )
    run_parser.add_argument(
        "--solix-only",
        action="store_true",
        help="Only run Solix benchmarks (skip comparative engines)",
    )
    run_parser.add_argument(
        "--scale-iterations",
        "--inner-loop",
        dest="scale_iterations",
        type=int,
        default=1,
        help="Scale benchmark execution passes per sample (default: 1)",
    )
    run_parser.add_argument(
        "--benchmark-duration-ms",
        type=float,
        default=None,
        help="Target minimum sample duration in milliseconds across iterations",
    )
    run_parser.add_argument(
        "--no-precompile",
        action="store_true",
        help="Execute Solix directly on sources without pre-compiling to .slxb",
    )
    run_parser.add_argument(
        "--cross-language",
        action="store_true",
        help="Include comparative language timings in table output",
    )
    run_parser.add_argument(
        "--format",
        choices=["table", "markdown", "json"],
        default="table",
        help="Console stdout output format (default: table)",
    )
    run_parser.add_argument(
        "--markdown",
        action="store_true",
        help="Alias for --format markdown",
    )
    run_parser.add_argument(
        "--json",
        action="store_true",
        help="Alias for --format json",
    )
    run_parser.add_argument(
        "--save-baseline",
        "--json-output",
        dest="save_baseline",
        type=str,
        default=None,
        help="Save full benchmark results to JSON file",
    )
    run_parser.add_argument(
        "--markdown-output",
        type=str,
        default=None,
        help="Save rendered Markdown table to file",
    )
    run_parser.add_argument(
        "--quiet",
        action="store_true",
        help="Suppress console table output",
    )

    # Subcommand: compare
    cmp_parser = subparsers.add_parser(
        "compare",
        help="Compare two or more JSON baselines (supports 2-way CI gate & multi-baseline tables)",
    )
    cmp_parser.add_argument(
        "baselines",
        nargs="*",
        help="Two or more baseline JSON paths for side-by-side comparison",
    )
    cmp_parser.add_argument(
        "--baselines",
        dest="baseline_flags",
        nargs="+",
        default=None,
        help="Explicit list of baseline JSON files to compare",
    )
    cmp_parser.add_argument(
        "--labels",
        nargs="+",
        default=None,
        help="Custom display labels for each baseline",
    )
    cmp_parser.add_argument(
        "--current",
        type=str,
        default=None,
        help="Path or raw JSON string for current benchmark baseline (2-way mode)",
    )
    cmp_parser.add_argument(
        "--previous",
        type=str,
        default=None,
        help="Path or raw JSON string for previous benchmark baseline (2-way mode)",
    )
    cmp_parser.add_argument(
        "--download-previous",
        action="store_true",
        help="Attempt downloading previous release baseline asset via GitHub CLI",
    )
    cmp_parser.add_argument(
        "--no-fallback",
        action="store_true",
        help="Do not fallback to local baselines directory if previous baseline is missing",
    )
    cmp_parser.add_argument(
        "--engine",
        type=str,
        default="solix_fast",
        help="Engine key to compare across baselines (default: solix_fast)",
    )
    cmp_parser.add_argument(
        "--threshold-pct",
        type=float,
        default=5.0,
        help="Performance regression percentage threshold (default: 5.0)",
    )
    cmp_parser.add_argument(
        "--min-delta-ms",
        type=float,
        default=0.5,
        help="Minimum absolute millisecond change required before flagging FASTER/SLOWER (default: 0.5)",
    )
    cmp_parser.add_argument(
        "--same-host",
        action="store_true",
        help="Annotate summary as back-to-back comparison on the same runner",
    )
    cmp_parser.add_argument(
        "--cross-language",
        action="store_true",
        help="Include comparative language table from the latest baseline in Markdown output",
    )
    cmp_parser.add_argument(
        "--current-tag",
        type=str,
        default="Current Build",
        help="Label or tag for current build version",
    )
    cmp_parser.add_argument(
        "--previous-tag",
        type=str,
        default="Previous Release",
        help="Label or tag for previous release version",
    )
    cmp_parser.add_argument(
        "--format",
        choices=["auto", "console", "markdown", "json"],
        default="auto",
        help="Output format for stdout (default: auto)",
    )
    cmp_parser.add_argument(
        "--markdown-output",
        type=str,
        default=None,
        help="Path to write generated GitHub Flavored Markdown comparison table",
    )
    cmp_parser.add_argument(
        "--json-output",
        type=str,
        default=None,
        help="Path to write structured JSON comparison results",
    )
    cmp_parser.add_argument(
        "--inject-notes",
        type=str,
        default=None,
        help="Path to release notes file where markdown table should be appended",
    )
    cmp_parser.add_argument(
        "--fail-on-regression",
        action="store_true",
        help="Exit non-zero if performance regresses beyond threshold",
    )
    cmp_parser.add_argument(
        "--quiet",
        action="store_true",
        help="Suppress stdout output",
    )

    return parser


def _handle_verify(args: argparse.Namespace, root_dir: Path) -> int:
    runner = BenchmarkRunner(
        root_dir, solix_bin_override=args.solix_bin, pin_cpu=args.pin_cpu
    )
    print(
        f"=== Running Solix Benchmark Correctness Verification Gate ({OS_NAME}) ==="
    )
    workload_dirs = runner.get_workload_dirs(args.workload)
    all_passed = True
    for wdir in workload_dirs:
        passed, checksum, results = runner.run_workload_verification(wdir)
        status_str = "PASS" if passed else "FAIL"
        print(
            f"  [{status_str}] {wdir.name:<25} Checksum: {checksum} (Engines: {list(results.keys())})"
        )
        if not passed:
            all_passed = False
            for eng, csum in results.items():
                print(f"       -> {eng}: {csum}")
    if not all_passed:
        print(
            "\n[FAIL] Verification gate failed: output checksum mismatch detected!"
        )
        return 1
    print(
        "\n[PASS] Verification gate passed: all workloads produced identical results across engines."
    )
    return 0


def _handle_run(args: argparse.Namespace, root_dir: Path) -> int:
    runner = BenchmarkRunner(
        root_dir, solix_bin_override=args.solix_bin, pin_cpu=args.pin_cpu
    )
    benchmark_data = runner.run_all_benchmarks(
        mode=args.mode,
        warmup=args.warmup,
        iterations=args.iterations,
        filter_pattern=args.workload,
        label=args.label,
        solix_only=args.solix_only,
        scale_iterations=args.scale_iterations,
        benchmark_duration_ms=args.benchmark_duration_ms,
        no_precompile=args.no_precompile,
    )

    output_format = args.format
    if args.markdown:
        output_format = "markdown"
    elif args.json:
        output_format = "json"

    markdown_text = render_run_markdown_table(benchmark_data)
    if args.cross_language:
        cross_md = render_cross_language_markdown(benchmark_data)
        if cross_md:
            markdown_text = f"{markdown_text}\n{cross_md.rstrip()}"

    if not args.quiet:
        if output_format == "json":
            print(json.dumps(benchmark_data, indent=2))
        elif output_format == "markdown":
            print(markdown_text)
        else:
            print(
                render_run_console_table(
                    benchmark_data,
                    include_cross_language=True,
                )
            )

    if args.save_baseline:
        out_path = Path(args.save_baseline)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(
            json.dumps(benchmark_data, indent=2) + "\n", encoding="utf-8"
        )
        if not args.quiet:
            print(f"\n[INFO] Saved benchmark baseline to '{out_path}'")

    if args.markdown_output:
        md_path = Path(args.markdown_output)
        md_path.parent.mkdir(parents=True, exist_ok=True)
        md_path.write_text(markdown_text + "\n", encoding="utf-8")

    return 0


def _resolve_baseline_label(
    source_path: str, dataset: Dict[str, Any], explicit_label: Optional[str]
) -> str:
    if explicit_label:
        return explicit_label
    meta_label = dataset.get("metadata", {}).get("label")
    if isinstance(meta_label, str) and meta_label.strip():
        return meta_label.strip()
    candidate = Path(source_path)
    if candidate.exists():
        return candidate.stem
    return "baseline"


def _handle_compare(args: argparse.Namespace, root_dir: Path) -> int:
    multi_sources = args.baseline_flags or args.baselines or []

    # Multi-baseline side-by-side comparison mode
    if multi_sources and not args.current:
        if len(multi_sources) < 2:
            raise ValueError(
                "Multi-baseline compare requires at least 2 baseline JSON files (or use --current/--previous)."
            )
        entries: List[Tuple[str, Dict[str, Any]]] = []
        custom_labels = args.labels or []
        for idx, src in enumerate(multi_sources):
            dataset = load_benchmark_data(src)
            label = (
                custom_labels[idx]
                if idx < len(custom_labels)
                else _resolve_baseline_label(src, dataset, None)
            )
            entries.append((label, dataset))

        multi_results = compare_multiple_baselines(
            baseline_entries=entries,
            preferred_engine=args.engine,
            regression_threshold_pct=args.threshold_pct,
            min_delta_ms=args.min_delta_ms,
        )
        cross_data = entries[-1][1] if args.cross_language else None
        markdown_table = render_multi_baseline_markdown(
            multi_results, cross_language_data=cross_data
        )
        console_table = render_multi_baseline_console(multi_results)

        fmt = "console" if args.format == "auto" else args.format
        if not args.quiet:
            if fmt == "json":
                print(json.dumps(multi_results, indent=2))
            elif fmt == "markdown":
                print(markdown_table)
            else:
                print(console_table)

        if args.markdown_output:
            target = Path(args.markdown_output)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(markdown_table, encoding="utf-8")

        if args.json_output:
            target = Path(args.json_output)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(
                json.dumps(multi_results, indent=2) + "\n", encoding="utf-8"
            )

        if args.inject_notes:
            inject_into_release_notes(Path(args.inject_notes), markdown_table)

        if args.fail_on_regression and multi_results.get("summary", {}).get(
            "has_regression", False
        ):
            return 1
        return 0

    # Standard 2-way comparison & CI release gate mode
    if not args.current:
        raise ValueError(
            "Provide either --current <baseline> or two or more baseline JSON files."
        )

    current_benchmark_data = load_benchmark_data(args.current)
    previous_benchmark_data: Optional[Dict[str, Any]] = None

    if args.previous:
        previous_benchmark_data = load_benchmark_data(args.previous)
    elif args.download_previous:
        previous_benchmark_data = fetch_previous_release_baseline_from_github(
            target_tag=args.previous_tag
            if args.previous_tag != "Previous Release"
            else None,
            exclude_tag=args.current_tag
            if args.current_tag != "Current Build"
            else None,
        )

    if previous_benchmark_data is None and not args.no_fallback:
        baselines_directory = root_dir / "benchmarks" / "baselines"
        local_fallback_file = find_local_baseline_fallback(baselines_directory)
        if local_fallback_file:
            try:
                previous_benchmark_data = load_benchmark_data(
                    str(local_fallback_file)
                )
                if not args.quiet:
                    print(
                        f"[INFO] Using local fallback baseline: {local_fallback_file.name}"
                    )
            except Exception:
                pass

    comparison_results = compare_benchmarks(
        current_data=current_benchmark_data,
        previous_data=previous_benchmark_data,
        regression_threshold_pct=args.threshold_pct,
        min_delta_ms=args.min_delta_ms,
        preferred_engine=args.engine,
    )

    markdown_table = render_markdown_comparison_table(
        comparison_results=comparison_results,
        previous_tag_label=args.previous_tag,
        current_tag_label=args.current_tag,
        same_host=args.same_host,
        cross_language_data=current_benchmark_data
        if args.cross_language
        else None,
    )

    fmt = "markdown" if args.format == "auto" else args.format
    if not args.quiet:
        if fmt == "json":
            print(json.dumps(comparison_results, indent=2))
        elif fmt == "console":
            # Build temporary 2-entry view for crisp ASCII rendering
            prev_label = args.previous_tag
            curr_label = args.current_tag
            if previous_benchmark_data:
                two_way = compare_multiple_baselines(
                    [
                        (prev_label, previous_benchmark_data),
                        (curr_label, current_benchmark_data),
                    ],
                    preferred_engine=args.engine,
                    regression_threshold_pct=args.threshold_pct,
                    min_delta_ms=args.min_delta_ms,
                )
                print(render_multi_baseline_console(two_way))
            else:
                print(render_run_console_table(current_benchmark_data))
        else:
            print(markdown_table)

    if args.markdown_output:
        markdown_target = Path(args.markdown_output)
        markdown_target.parent.mkdir(parents=True, exist_ok=True)
        markdown_target.write_text(markdown_table, encoding="utf-8")

    if args.json_output:
        json_target = Path(args.json_output)
        json_target.parent.mkdir(parents=True, exist_ok=True)
        json_target.write_text(
            json.dumps(comparison_results, indent=2) + "\n", encoding="utf-8"
        )

    if args.inject_notes:
        inject_into_release_notes(Path(args.inject_notes), markdown_table)

    if args.fail_on_regression and comparison_results.get("summary", {}).get(
        "has_regression", False
    ):
        regressed_count = comparison_results.get("summary", {}).get(
            "regressed_count", 0
        )
        sys.stderr.write(
            f"\n[FAIL] Quality Gate Failed: {regressed_count} workload(s) regressed beyond "
            f"{args.threshold_pct}% threshold!\n"
        )
        return 1

    return 0


def main() -> int:
    """CLI entrypoint for `solix_bench`."""
    parser = build_parser()
    args = parser.parse_args()
    root_dir = find_repo_root()

    try:
        if args.subcommand == "verify":
            return _handle_verify(args, root_dir)
        if args.subcommand == "run":
            return _handle_run(args, root_dir)
        if args.subcommand == "compare":
            return _handle_compare(args, root_dir)
    except ValueError as err:
        sys.stderr.write(f"[ERROR] {err}\n")
        return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
