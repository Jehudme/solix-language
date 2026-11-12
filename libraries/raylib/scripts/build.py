#!/usr/bin/env python3
"""Build script for Solix Raylib package.

Automated 3-Step Execution Sequence:
Step 1: Host rlparser compilation and API JSON extraction (build/.cache/api/raylib_api.json).
Step 2: Schema-driven 1-to-1 codegen for Solix API (src/raylib.slx) and Rust bridge (natives/raylib/src/lib.rs).
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
        "-DCMAKE_POSITION_INDEPENDENT_CODE=ON",
        "-DBUILD_EXAMPLES=OFF",
        "-DBUILD_GAMES=OFF",
        "-DBUILD_SHARED_LIBS=OFF",
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

    print("  [cmake] Compiling static raylib library...")
    raylib_lib_result = subprocess.run(
        ["cmake", "--build", str(cmake_host_directory), "--target", "raylib"],
        check=False,
    )
    if raylib_lib_result.returncode != 0:
        raise RuntimeError(f"CMake raylib build failed with code {raylib_lib_result.returncode}")

    if not api_schema_path.exists():
        raise FileNotFoundError(f"Expected generated API JSON schema not found at '{api_schema_path}'")

    print(f"  [step 1] Schema extracted successfully: {api_schema_path}")
    return api_schema_path


def step_2_codegen(root_path: Path, api_schema_path: Path) -> None:
    """Executes schema-driven codegen for Solix declarations and Rust FFI companion bridge."""
    scripts_dir = root_path / "scripts"
    solix_output_file = root_path / "src" / "raylib.slx"
    rust_output_file = root_path / "natives" / "raylib" / "src" / "lib.rs"

    print("=== [Step 2] Schema-Driven Code Generation ===")
    solix_codegen_cmd = [
        sys.executable,
        str(scripts_dir / "generate_solix_api.py"),
        "--api", str(api_schema_path),
        "--output", str(solix_output_file),
    ]
    print(f"  [codegen] Generating Solix API: {solix_output_file}...")
    solix_result = subprocess.run(solix_codegen_cmd, check=False)
    if solix_result.returncode != 0:
        raise RuntimeError(f"Solix API codegen failed with code {solix_result.returncode}")

    rust_codegen_cmd = [
        sys.executable,
        str(scripts_dir / "generate_rust_bridge.py"),
        "--api", str(api_schema_path),
        "--output", str(rust_output_file),
    ]
    print(f"  [codegen] Generating Rust FFI bridge: {rust_output_file}...")
    rust_result = subprocess.run(rust_codegen_cmd, check=False)
    if rust_result.returncode != 0:
        raise RuntimeError(f"Rust bridge codegen failed with code {rust_result.returncode}")

    print("  [step 2] Codegen completed successfully.")


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

    # Step 1: Host rlparser & API extraction
    api_schema = step_1_extract_api(project_root)

    # Step 2: Schema-driven Codegen
    step_2_codegen(project_root, api_schema)

    # Step 3: Platform compilation and packaging
    print("=== [Step 3] Platform Compilation & Distribution Packaging ===")
    solix_toolbox.auto_build(profile=build_profile, project_root=project_root)


if __name__ == "__main__":
    main()
