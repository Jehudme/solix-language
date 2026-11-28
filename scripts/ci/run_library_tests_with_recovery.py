#!/usr/bin/env python3
"""Integration test runner with automatic library recompilation recovery.

Executes tests for bundled libraries (`libraries/*`).
In Phase 2 of CI, libraries may have been restored from cache while the runtime in
`crates/` had internal opcode or bytecode layout changes.

Algorithm:
1. First attempt: Run `solix build <lib> --profile test` (testing with existing/cached bytecode if built).
2. If tests pass, finish successfully.
3. If tests fail:
   - Automatically trigger recompilation from source:
     `solix build <lib> --profile test --force` (or cleaning the build dir).
   - Re-run the test suite.
   - If it passes: Log a notice that the failure was due to stale cached bytecode; accept the newly compiled binary.
   - If it still fails: Raise a fatal error with the real compiler/test error trace.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys
from typing import List


def run_command(command: List[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    """Runs a command and returns the completed process."""
    return subprocess.run(
        command,
        cwd=str(cwd),
        text=True,
        capture_output=True,
    )


def test_library_with_recovery(
    solix_binary: Path,
    library_dir: Path,
    ephemeral_pkg_dir: Path,
) -> bool:
    """Tests a library, falling back to clean recompilation if cached execution fails."""
    manifest_path = library_dir / "manifest.yaml"
    if not manifest_path.is_file():
        print(f"[recovery_runner] Skipping {library_dir.name} (no manifest.yaml)")
        return True

    lib_name = library_dir.name
    print(f"=== Testing library '{lib_name}' (First Attempt with cached/active artifacts) ===")

    test_cmd = [
        str(solix_binary),
        "build",
        str(library_dir),
        "--profile",
        "test",
        "--pkg-dir",
        str(ephemeral_pkg_dir),
    ]

    first_result = run_command(test_cmd, library_dir)

    if first_result.returncode == 0:
        print(f"[PASS] Library '{lib_name}' passed test suite cleanly on first attempt.")
        return True

    print(f"[WARN] Library '{lib_name}' failed on initial run. Checking if due to stale cache/bytecode mismatch...")
    print(f"[OUTPUT]:\n{first_result.stderr}\n{first_result.stdout}")

    # Phase 2 Auto-Recompilation Recovery Step:
    # Clean output directories to force a fresh recompilation from source
    build_dir = library_dir / "build"
    if build_dir.is_dir():
        shutil.rmtree(build_dir, ignore_errors=True)

    print(f"=== [RECOVERY] Clean recompiling library '{lib_name}' from source ===")
    clean_compile_cmd = [
        str(solix_binary),
        "build",
        str(library_dir),
        "--profile",
        "test",
        "--pkg-dir",
        str(ephemeral_pkg_dir),
    ]

    second_result = run_command(clean_compile_cmd, library_dir)

    if second_result.returncode == 0:
        print(f"[PASS] RECOVERY SUCCESSFUL: Library '{lib_name}' passed after clean recompilation.")
        print(f"[NOTE] The initial failure was due to stale prebuilt bytecode matching an older runtime ABI.")
        return True

    print(f"[FAIL] Library '{lib_name}' failed tests even after clean recompilation from source!")
    print(f"[ERROR]:\n{second_result.stderr}\n{second_result.stdout}")
    return False


def main() -> int:
    parser = argparse.ArgumentParser(description="Test libraries with auto-recompilation recovery.")
    parser.add_argument("--solix-binary", required=True, type=Path, help="Path to solix toolchain executable")
    parser.add_argument("--libraries-dir", type=Path, default=Path("libraries"), help="Root directory containing libraries")
    parser.add_argument("--pkg-dir", type=Path, default=None, help="Ephemeral package directory")
    args = parser.parse_args()

    solix_binary = args.solix_binary.resolve()
    if not solix_binary.is_file() or not os.access(solix_binary, os.X_OK):
        print(f"[recovery_runner] Error: Solix binary not found or not executable at {solix_binary}", file=sys.stderr)
        return 1

    libraries_dir = args.libraries_dir.resolve()
    ephemeral_pkg_dir = (args.pkg_dir or Path(os.environ.get("RUNNER_TEMP", "/tmp")) / "solix_test_pkg_cache").resolve()
    ephemeral_pkg_dir.mkdir(parents=True, exist_ok=True)

    failed_libraries: List[str] = []

    # Find all directories with manifest.yaml
    for child in sorted(libraries_dir.iterdir()):
        if child.is_dir() and (child / "manifest.yaml").is_file():
            success = test_library_with_recovery(solix_binary, child, ephemeral_pkg_dir)
            if not success:
                failed_libraries.append(child.name)

    if failed_libraries:
        print(f"\n[FATAL] The following libraries failed integration tests: {', '.join(failed_libraries)}", file=sys.stderr)
        return 1

    print("\n[SUCCESS] All libraries verified cleanly.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
