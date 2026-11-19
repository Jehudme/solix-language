#!/usr/bin/env python3
"""Pre-compile binding generator for the Solix Raylib library.

Extracts the Raylib API schema using the host `rlparser` CMake target and regenerates
`src/raylib.slx` and `natives/raylib/src/lib.rs` via `scripts/generate_bindings.py`.
"""

from __future__ import annotations

from pathlib import Path
import subprocess
import sys


def run_step(command: list[str], cwd: Path, label: str) -> None:
    """Runs a build step and raises an informative error on non-zero exit."""
    result = subprocess.run(command, cwd=cwd, check=False)
    if result.returncode != 0:
        raise SystemExit(
            f"Raylib pre_compile step '{label}' failed with exit code {result.returncode}: "
            f"{' '.join(command)}"
        )


def main() -> None:
    """Runs host rlparser extraction followed by Two-Pass IR binding generation."""
    project_root = Path(__file__).resolve().parent.parent
    cache_directory = project_root / "build" / ".cache"
    cmake_host_directory = cache_directory / "cmake" / "host"
    api_schema_path = cache_directory / "api" / "raylib_api.json"
    natives_raylib_directory = project_root / "natives" / "raylib"

    print("=== [pre_compile] CMake Configuration & rlparser Extraction ===")
    run_step(
        [
            "cmake",
            "-B",
            str(cmake_host_directory),
            "-S",
            str(natives_raylib_directory),
            "-DBUILD_HOST_PARSER_ONLY=ON",
            f"-DAPI_OUTPUT_FILE={api_schema_path}",
        ],
        cwd=project_root,
        label="cmake-configure-host-parser",
    )

    run_step(
        [
            "cmake",
            "--build",
            str(cmake_host_directory),
            "--target",
            "extract_api_json",
        ],
        cwd=project_root,
        label="cmake-extract-api-json",
    )

    if not api_schema_path.is_file():
        raise SystemExit(
            f"Expected generated API JSON schema not found at '{api_schema_path}'"
        )

    print("=== [pre_compile] Unified Two-Pass IR Code Generation ===")
    run_step(
        [
            sys.executable,
            str(project_root / "scripts" / "generate_bindings.py"),
            "--api",
            str(api_schema_path),
            "--out-slx",
            str(project_root / "src" / "raylib.slx"),
            "--out-rs",
            str(natives_raylib_directory / "src" / "lib.rs"),
            "--package",
            "raylib",
            "--class-name",
            "Raylib",
        ],
        cwd=project_root,
        label="generate-bindings",
    )


if __name__ == "__main__":
    main()
