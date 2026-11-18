#!/usr/bin/env python3
"""Build script for Solix Raylib package.

Automated 3-Step Execution Sequence:
Step 1: Host rlparser compilation and API JSON extraction (build/.cache/api/raylib_api.json).
Step 2: Unified Two-Pass IR Codegen for Solix API (src/raylib.slx) and Rust bridge (natives/raylib/src/lib.rs).
Step 3: Platform compilation (debug/test for host, release multi-target cross-compilation) and packaging.
"""

from __future__ import annotations

import os
from pathlib import Path
import sys

scripts_directory = Path(__file__).resolve().parent
project_root = scripts_directory.parent

sys.path.insert(0, str(scripts_directory))
import solix_build

project_root = solix_build.normalize_host_path(project_root)


def step_1_extract_api(root_path: Path) -> Path:
    """Configures CMake in build/.cache/cmake/host, builds rlparser and extracts API JSON."""
    root_path = solix_build.normalize_host_path(root_path)
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
    solix_build.run_command(
        cmake_configure_cmd,
        cwd=root_path,
        phase="CMake Host Parser Configuration",
        hint="Ensure cmake and a C host compiler (gcc/clang) are installed.",
    )

    print("  [cmake] Extracting API JSON via rlparser...")
    solix_build.run_command(
        ["cmake", "--build", str(cmake_host_directory), "--target", "extract_api_json"],
        cwd=root_path,
        phase="CMake rlparser API JSON Extraction",
        hint="Check build/.cache/cmake/host/CMakeFiles/CMakeError.log for compiler error details.",
    )

    if not api_schema_path.exists():
        raise solix_build.BuildError(
            phase="API Schema Validation",
            message=f"Expected generated API JSON schema not found at '{api_schema_path}'",
            cwd=root_path,
            hint="Ensure extract_api_json target successfully wrote the output file.",
        )

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
    print("  [codegen] Generating Solix API and Rust companion bridge...")
    solix_build.run_command(
        codegen_cmd,
        cwd=root_path,
        phase="Unified Two-Pass IR Codegen",
        hint="Inspect generate_bindings.py error output for schema parsing or formatting issues.",
    )

    print("  [step 2] Codegen completed successfully.")


def verify(ctx: solix_build.BuildContext) -> None:
    """Pre-flight verification hook checking CMake 3.20+ and host C compiler for rlparser."""
    _ = ctx
    cmake_info = solix_build.require_tool(
        "cmake",
        min_version="3.20.0",
        hint="Install CMake >= 3.20.0 to extract Raylib API headers.",
    )
    print(f"  [check] cmake {cmake_info['version']} ({cmake_info['path']})")


def run_pipeline(
    build_profile: str = "debug",
    root_path: Path | None = None,
    verify_only: bool = False,
) -> Path:
    """Executes the complete 3-step pipeline."""
    target_root = solix_build.normalize_host_path(root_path if root_path is not None else project_root)
    manifest = solix_build.load_manifest(target_root)

    if verify_only:
        solix_build.verify_environment(target_root, manifest, build_profile, verify_fn=verify)
        return target_root

    solix_build.verify_environment(target_root, manifest, build_profile, verify_fn=verify)

    # Step 1: Host rlparser & API extraction
    api_schema = step_1_extract_api(target_root)

    # Step 2: Schema-driven Codegen
    step_2_codegen(target_root, api_schema)

    # Step 3: Platform compilation and packaging
    print("=== [Step 3] Platform Compilation & Distribution Packaging ===")
    return solix_build.auto_build(profile=build_profile, project_root=target_root)


def main() -> None:
    build_profile = os.environ.get("SOLIX_PROFILE", "").strip() or "release"
    verify_only = os.environ.get("SOLIX_VERIFY_ONLY", "").strip().lower() in ("1", "true", "yes")
    arguments = sys.argv[1:]
    index = 0
    while index < len(arguments):
        argument_item = arguments[index]
        if argument_item in ("debug", "release", "test"):
            build_profile = argument_item
        elif argument_item.startswith("--profile="):
            build_profile = argument_item.split("=", 1)[1]
        elif argument_item == "--profile" and index + 1 < len(arguments):
            build_profile = arguments[index + 1]
            index += 1
        elif argument_item.startswith("--target="):
            raw_target = argument_item.split("=", 1)[1]
            os.environ["SOLIX_TARGET_TRIPLE"] = (
                solix_build.get_host_target_triple() if raw_target.lower() == "host" else raw_target
            )
        elif argument_item == "--target" and index + 1 < len(arguments):
            raw_target = arguments[index + 1]
            os.environ["SOLIX_TARGET_TRIPLE"] = (
                solix_build.get_host_target_triple() if raw_target.lower() == "host" else raw_target
            )
            index += 1
        elif argument_item in ("--check-env", "--verify-only"):
            verify_only = True
        elif argument_item == "--release":
            build_profile = "release"
        elif argument_item == "--test":
            build_profile = "test"
        elif argument_item == "--debug":
            build_profile = "debug"
        index += 1

    run_pipeline(build_profile=build_profile, root_path=project_root, verify_only=verify_only)


if __name__ == "__main__":
    solix_build.run_cli(main)
