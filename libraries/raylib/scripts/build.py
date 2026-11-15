#!/usr/bin/env python3
"""Build script for Solix Raylib package.

Automated 3-Step Execution Sequence:
Step 1: Host rlparser compilation and API JSON extraction (build/.cache/api/raylib_api.json).
Step 2: Unified Two-Pass IR Codegen for Solix API (src/raylib.slx) and Rust bridge (natives/raylib/src/lib.rs).
Step 3: Platform compilation (debug/test for host, release multi-target cross-compilation) and packaging.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

scripts_directory = Path(__file__).resolve().parent
project_root = scripts_directory.parent

sys.path.insert(0, str(scripts_directory))
import solix_toolbox


def step_1_extract_api(root_path: Path) -> Path:
    """Configures CMake in build/.cache/cmake/host, builds rlparser and extracts API JSON."""
    cache_directory = root_path / "build" / ".cache"
    cmake_host_directory = cache_directory / "cmake" / "host"
    api_schema_path = cache_directory / "api" / "raylib_api.json"
    natives_raylib_directory = root_path / "natives" / "raylib"

    print("=== [Step 1] CMake Configuration & rlparser Extraction ===")
    cmake_configure_cmd = [
        "cmake",
        "-B", str(cmake_host_directory),
        "-S", str(natives_raylib_directory),
        "-DBUILD_HOST_PARSER_ONLY=ON",
        f"-DAPI_OUTPUT_FILE={api_schema_path}",
    ]
    configure_result = subprocess.run(cmake_configure_cmd, check=False)
    if configure_result.returncode != 0:
        raise RuntimeError(f"CMake configure failed with code {configure_result.returncode}")

    print("  [cmake] Extracting API JSON via rlparser...")
    extract_result = subprocess.run(
        ["cmake", "--build", str(cmake_host_directory), "--target", "extract_api_json"],
        check=False,
    )
    if extract_result.returncode != 0:
        raise RuntimeError(f"CMake extract_api_json failed with code {extract_result.returncode}")

    if not api_schema_path.exists():
        raise FileNotFoundError(f"Expected generated API JSON schema not found at '{api_schema_path}'")

    print(f"  [step 1] Schema extracted successfully: {api_schema_path}")
    return api_schema_path


def step_2_codegen(root_path: Path, api_schema_path: Path) -> None:
    """Executes schema-driven Two-Pass IR codegen for Solix declarations and Rust FFI bridge."""
    scripts_dir = root_path / "scripts"
    solix_output_file = root_path / "src" / "raylib.slx"
    rust_output_file = root_path / "natives" / "raylib" / "src" / "lib.rs"

    print("=== [Step 2] Unified Two-Pass IR Code Generation ===")
    codegen_cmd = [
        sys.executable,
        str(scripts_dir / "generate_bindings.py"),
        "--api", str(api_schema_path),
        "--out-slx", str(solix_output_file),
        "--out-rs", str(rust_output_file),
        "--package", "raylib",
        "--class-name", "Raylib",
    ]
    print(f"  [codegen] Generating Solix API and Rust companion bridge...")
    codegen_result = subprocess.run(codegen_cmd, check=False)
    if codegen_result.returncode != 0:
        raise RuntimeError(f"Binding codegen failed with code {codegen_result.returncode}")

    print("  [step 2] Codegen completed successfully.")


def run_pipeline(build_profile: str = "debug", root_path: Path | None = None) -> Path:
    """Executes the complete 3-step pipeline."""
    target_root = root_path if root_path is not None else project_root

    # Step 1: Host rlparser & API extraction
    api_schema = step_1_extract_api(target_root)

    # Step 2: Schema-driven Codegen
    step_2_codegen(target_root, api_schema)

    # Step 3: Platform compilation and packaging
    print("=== [Step 3] Platform Compilation & Distribution Packaging ===")
    return solix_toolbox.auto_build(profile=build_profile, project_root=target_root)


def main() -> None:
    build_profile = "debug"
    for argument_item in sys.argv[1:]:
        if argument_item in ("debug", "release", "test"):
            build_profile = argument_item
        elif argument_item.startswith("--profile="):
            build_profile = argument_item.split("=", 1)[1]
        elif argument_item == "--profile":
            continue
        elif argument_item == "--release":
            build_profile = "release"
        elif argument_item == "--test":
            build_profile = "test"
        elif argument_item == "--debug":
            build_profile = "debug"

    run_pipeline(build_profile=build_profile, root_path=project_root)


if __name__ == "__main__":
    main()
