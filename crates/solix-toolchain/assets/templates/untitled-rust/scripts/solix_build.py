#!/usr/bin/env python3
"""Two-Tier Solix Build Engine (`solix_build.py`).

Tier 1: Composable low-level build primitives with strict cache isolation and structured error reporting.
Tier 2: High-level turnkey auto_build orchestrator with timing metrics and declarative [build] integration.
"""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import tomllib
from typing import Any, Callable
import zipfile


SUPPORTED_RELEASE_TARGETS: list[tuple[str, str]] = [
    ("x86_64-unknown-linux-gnu", ".so"),
    ("aarch64-unknown-linux-gnu", ".so"),
    ("x86_64-pc-windows-gnu", ".dll"),
    ("x86_64-apple-darwin", ".dylib"),
    ("aarch64-apple-darwin", ".dylib"),
]


# ============================================================================
# Structured Build Error & Diagnostic UI
# ============================================================================

class BuildError(Exception):
    """Encapsulates a structured build failure with contextual diagnostics."""

    def __init__(
        self,
        phase: str,
        message: str,
        command: list[str] | None = None,
        cwd: Path | None = None,
        exit_code: int | None = None,
        stderr: str | None = None,
        hint: str | None = None,
    ) -> None:
        super().__init__(message)
        self.phase = phase
        self.message = message
        self.command = command
        self.cwd = cwd
        self.exit_code = exit_code
        self.stderr = stderr
        self.hint = hint


def format_build_error(error: BuildError) -> str:
    """Renders a structured, human-readable terminal diagnostic panel."""
    separator_line = "=" * 80
    sub_separator = "-" * 80
    command_display = " ".join(error.command) if error.command else "N/A"
    cwd_display = str(error.cwd) if error.cwd else str(Path.cwd())
    exit_code_display = str(error.exit_code) if error.exit_code is not None else "1"
    stderr_text = error.stderr.strip() if error.stderr and error.stderr.strip() else "(no output recorded)"
    hint_text = error.hint or "Check compiler and process error output above for details."

    lines = [
        separator_line,
        f"[SOLIX BUILD ERROR] Phase: {error.phase}",
        separator_line,
        f"Message:           {error.message}",
        f"Command:           {command_display}",
        f"Working Directory: {cwd_display}",
        f"Exit Code:         {exit_code_display}",
        sub_separator,
        "Compiler Output:",
        stderr_text,
        sub_separator,
        f"Remediation Hint:  {hint_text}",
        separator_line,
    ]
    return "\n".join(lines)


def run_command(
    command_arguments: list[str],
    *,
    cwd: Path | None = None,
    env: dict[str, str] | None = None,
    phase: str = "Subprocess Execution",
    hint: str | None = None,
    capture_output: bool = True,
) -> subprocess.CompletedProcess[str]:
    """Executes a subprocess command, capturing output and raising structured BuildError on failure."""
    try:
        process_result = subprocess.run(
            command_arguments,
            cwd=cwd,
            env=env,
            capture_output=capture_output,
            text=True,
            check=False,
        )
    except FileNotFoundError as file_not_found_error:
        executable_name = command_arguments[0] if command_arguments else "command"
        raise BuildError(
            phase=phase,
            message=f"Executable '{executable_name}' was not found in PATH",
            command=command_arguments,
            cwd=cwd,
            exit_code=127,
            stderr=str(file_not_found_error),
            hint=hint or f"Ensure that '{executable_name}' is installed and present in your system PATH.",
        ) from file_not_found_error

    if process_result.returncode != 0:
        error_output = (process_result.stderr or "") + "\n" + (process_result.stdout or "")
        raise BuildError(
            phase=phase,
            message=f"Command execution failed with exit code {process_result.returncode}",
            command=command_arguments,
            cwd=cwd,
            exit_code=process_result.returncode,
            stderr=error_output,
            hint=hint,
        )

    return process_result


def run_cli(main_entrypoint: Callable[[], Any]) -> None:
    """Wraps CLI execution to display structured diagnostics and suppress raw tracebacks."""
    try:
        main_entrypoint()
    except BuildError as build_error:
        if os.environ.get("SOLIX_BUILD_TRACEBACK") == "1":
            raise
        sys.stderr.write(format_build_error(build_error) + "\n")
        sys.exit(build_error.exit_code if build_error.exit_code is not None else 1)
    except Exception as generic_error:
        if os.environ.get("SOLIX_BUILD_TRACEBACK") == "1":
            raise
        wrapped_error = BuildError(
            phase="Build Initialization",
            message=str(generic_error),
            exit_code=1,
            hint="Set SOLIX_BUILD_TRACEBACK=1 to inspect the full Python exception traceback.",
        )
        sys.stderr.write(format_build_error(wrapped_error) + "\n")
        sys.exit(1)


# ============================================================================
# Tier 1: Composable Toolbox Primitives
# ============================================================================

def get_host_target_triple() -> str:
    """Resolves the current machine's Rust target triple via rustc."""
    try:
        process_result = subprocess.run(
            ["rustc", "-vV"], capture_output=True, text=True, check=True
        )
        for line_text in process_result.stdout.splitlines():
            if line_text.startswith("host:"):
                return line_text.split(":", 1)[1].strip()
    except Exception:
        pass
    if sys.platform == "win32":
        return "x86_64-pc-windows-gnu"
    if sys.platform == "darwin":
        return "x86_64-apple-darwin"
    return "x86_64-unknown-linux-gnu"


def load_manifest(project_root: Path) -> dict[str, Any]:
    """Parses manifest.toml from the project root directory."""
    manifest_path = project_root / "manifest.toml"
    if not manifest_path.exists():
        raise BuildError(
            phase="Manifest Loading",
            message=f"manifest.toml not found at '{manifest_path}'",
            cwd=project_root,
            hint="Ensure that you are running the build from a valid Solix project directory.",
        )

    try:
        with open(manifest_path, "rb") as manifest_file:
            manifest_data = tomllib.load(manifest_file)
    except Exception as parse_error:
        raise BuildError(
            phase="Manifest Parsing",
            message=f"Failed to parse manifest.toml: {parse_error}",
            cwd=project_root,
            hint="Ensure manifest.toml is valid TOML syntax.",
        ) from parse_error

    if "info" not in manifest_data or "name" not in manifest_data["info"]:
        raise BuildError(
            phase="Manifest Validation",
            message="manifest.toml must contain a valid [info] section with 'name'",
            cwd=project_root,
            hint="Add [info] section with name = \"<package_name>\" to manifest.toml.",
        )

    return manifest_data


def prepare_staging_dirs(project_root: Path, profile: str) -> dict[str, Path]:
    """Prepares and cleans the ephemeral staging and output directories strictly within build/.cache/."""
    cache_directory = project_root / "build" / ".cache"
    natives_cache_directory = cache_directory / "natives" / profile
    cargo_target_directory = cache_directory / "cargo"
    stage_directory = cache_directory / "stage" / profile
    build_output_directory = project_root / "build" / profile

    stage_bin_directory = stage_directory / "bin"
    stage_lib_directory = stage_directory / "lib"
    stage_resources_directory = stage_directory / "resources"

    # Recreate stage directory cleanly
    if stage_directory.exists():
        shutil.rmtree(stage_directory)

    stage_bin_directory.mkdir(parents=True, exist_ok=True)
    stage_lib_directory.mkdir(parents=True, exist_ok=True)
    stage_resources_directory.mkdir(parents=True, exist_ok=True)
    natives_cache_directory.mkdir(parents=True, exist_ok=True)
    cargo_target_directory.mkdir(parents=True, exist_ok=True)
    build_output_directory.mkdir(parents=True, exist_ok=True)

    # Stage manifest.toml
    shutil.copy2(project_root / "manifest.toml", stage_directory / "manifest.toml")

    return {
        "cache": cache_directory,
        "natives_cache": natives_cache_directory,
        "cargo_target": cargo_target_directory,
        "stage": stage_directory,
        "stage_bin": stage_bin_directory,
        "stage_lib": stage_lib_directory,
        "stage_resources": stage_resources_directory,
        "build": build_output_directory,
    }


def detect_native_projects(natives_directory: Path) -> list[Path]:
    """Scans natives/ for Rust cdylib projects containing Cargo.toml."""
    if not natives_directory.exists():
        return []

    discovered_projects: list[Path] = []
    for entry_path in natives_directory.iterdir():
        if entry_path.is_dir() and (entry_path / "Cargo.toml").exists():
            discovered_projects.append(entry_path)

    discovered_projects.sort()
    return discovered_projects


def compile_rust_project(
    project_directory: Path,
    cache_directory: Path,
    stage_lib_directory: Path,
    profile: str,
) -> list[Path]:
    """Compiles a Rust cdylib project with strict CARGO_TARGET_DIR isolation."""
    cargo_target_directory = cache_directory / "cargo"
    host_target_triple = get_host_target_triple()

    cargo_environment = os.environ.copy()
    cargo_environment["CARGO_TARGET_DIR"] = str(cargo_target_directory)
    home_cargo_bin = str(Path.home() / ".cargo" / "bin")
    current_system_path = cargo_environment.get("PATH", "")
    if home_cargo_bin not in current_system_path:
        cargo_environment["PATH"] = f"{home_cargo_bin}:{current_system_path}"

    if sys.platform.startswith("linux"):
        cargo_environment["RUSTFLAGS"] = "-C link-arg=-L/usr/lib " + cargo_environment.get("RUSTFLAGS", "")

    emitted_libraries: list[Path] = []

    if profile != "release":
        # Debug/Test: Compile only for the host machine
        cargo_command = [
            "cargo",
            "build",
            "--manifest-path",
            str(project_directory / "Cargo.toml"),
            "--target-dir",
            str(cargo_target_directory),
        ]
        print(f"  [cargo] Building native Rust project '{project_directory.name}' ({profile})...")
        run_command(
            cargo_command,
            cwd=project_directory,
            env=cargo_environment,
            phase="Native Rust Host Build",
            hint="Check that Rust toolchain is installed and Cargo.toml has no errors.",
            capture_output=False,
        )

        artifact_directory = cargo_target_directory / "debug"
        if sys.platform == "win32":
            allowed_extensions = [".dll"]
        elif sys.platform == "darwin":
            allowed_extensions = [".dylib"]
        else:
            allowed_extensions = [".so"]

        if artifact_directory.exists():
            for file_path in artifact_directory.iterdir():
                if file_path.is_file() and any(file_path.name.endswith(extension) for extension in allowed_extensions):
                    destination_path = stage_lib_directory / file_path.name
                    shutil.copy2(file_path, destination_path)
                    emitted_libraries.append(destination_path)
        return emitted_libraries

    # Release mode: Multi-platform cross-compilation via cargo zigbuild
    has_cargo_zigbuild = bool(shutil.which("cargo-zigbuild", path=cargo_environment["PATH"]))
    successful_targets = 0

    for target_triple, target_extension in SUPPORTED_RELEASE_TARGETS:
        is_host_platform = (target_triple == host_target_triple)
        target_stage_directory = stage_lib_directory / target_triple
        target_stage_directory.mkdir(parents=True, exist_ok=True)

        if has_cargo_zigbuild:
            cargo_command = [
                "cargo",
                "zigbuild",
                "--manifest-path",
                str(project_directory / "Cargo.toml"),
                "--target-dir",
                str(cargo_target_directory),
                "--release",
                "--target",
                target_triple,
            ]
        else:
            if not is_host_platform:
                print(f"  [warning] Skipping cross-target '{target_triple}': cargo-zigbuild not installed")
                continue
            cargo_command = [
                "cargo",
                "build",
                "--manifest-path",
                str(project_directory / "Cargo.toml"),
                "--target-dir",
                str(cargo_target_directory),
                "--release",
            ]

        print(f"  [cargo] Building target '{target_triple}' ({profile})...")
        try:
            run_command(
                cargo_command,
                cwd=project_directory,
                env=cargo_environment,
                phase=f"Native Rust Target Build ({target_triple})",
                hint="Install target standard library via: rustup target add " + target_triple,
            )
            successful_targets += 1
        except BuildError as target_error:
            error_output = target_error.stderr or ""
            if "can't find crate for `std`" in error_output or "can't find crate for std" in error_output:
                print(f"  [warning] Cross-compilation skipped for '{target_triple}': rust standard library not installed")
            else:
                lines = error_output.strip().splitlines()
                last_line = lines[-1] if lines else "build failed"
                print(f"  [warning] Target '{target_triple}' compilation failed: {last_line}")
            continue

        artifact_directory = (
            cargo_target_directory / target_triple / "release"
            if has_cargo_zigbuild
            else cargo_target_directory / "release"
        )

        if artifact_directory.exists():
            for file_path in artifact_directory.iterdir():
                if file_path.is_file() and file_path.name.endswith(target_extension):
                    destination_target_path = target_stage_directory / file_path.name
                    shutil.copy2(file_path, destination_target_path)
                    emitted_libraries.append(destination_target_path)

                    if is_host_platform:
                        destination_host_path = stage_lib_directory / file_path.name
                        shutil.copy2(file_path, destination_host_path)
                        emitted_libraries.append(destination_host_path)

    if successful_targets == 0:
        raise BuildError(
            phase="Native Rust Build",
            message=f"All target builds failed for native project '{project_directory.name}'",
            cwd=project_directory,
            hint="Ensure host build dependencies and headers are available.",
        )

    return emitted_libraries


def copy_project_libs(project_lib_directory: Path, stage_lib_directory: Path) -> None:
    """Copies pre-existing external library files into stage/lib."""
    if not project_lib_directory.exists():
        return

    for item_path in project_lib_directory.iterdir():
        if item_path.is_file() and not item_path.name.startswith("."):
            shutil.copy2(item_path, stage_lib_directory / item_path.name)


def copy_project_resources(resources_directory: Path, stage_resources_directory: Path) -> None:
    """Copies static runtime assets into stage/resources."""
    if not resources_directory.exists():
        return

    for item_path in resources_directory.iterdir():
        if item_path.name.startswith("."):
            continue
        destination_path = stage_resources_directory / item_path.name
        if item_path.is_dir():
            shutil.copytree(item_path, destination_path, dirs_exist_ok=True)
        else:
            shutil.copy2(item_path, destination_path)


def resolve_solix_binary() -> str:
    """Resolves the solix CLI executable path."""
    if "SOLIX_BIN" in os.environ and os.path.exists(os.environ["SOLIX_BIN"]):
        candidate_binary = Path(os.environ["SOLIX_BIN"])
        if candidate_binary.is_file() and candidate_binary.name in ("solix", "solix.exe"):
            return str(candidate_binary)

    current_path = Path(__file__).resolve().parent
    while current_path != current_path.parent:
        for candidate in [
            current_path / "target" / "release" / "solix",
            current_path / "target" / "debug" / "solix",
        ]:
            if candidate.exists() and os.access(candidate, os.X_OK):
                return str(candidate)
        current_path = current_path.parent

    which_result = shutil.which("solix")
    if which_result:
        return which_result

    return "solix"


def compile_solix_sources(
    project_root: Path,
    manifest: dict[str, Any],
    stage_bin_directory: Path,
    stage_lib_directory: Path,
    profile: str,
) -> Path:
    """Invokes solix compile to emit the main application bytecode binary."""
    solix_executable = resolve_solix_binary()
    package_name = manifest["info"]["name"]
    package_version = manifest["info"].get("version", "0.1.0")
    package_description = manifest["info"].get("description", "")

    output_slxb = stage_bin_directory / f"{package_name}.slxb"

    source_directory = project_root / "src"
    tests_directory = project_root / "tests"

    sources_to_compile: list[str] = [str(source_directory)]
    if profile == "test" and tests_directory.exists():
        sources_to_compile.append(str(tests_directory))

    compile_command: list[str] = [
        solix_executable,
        "compile",
        *sources_to_compile,
        "--profile",
        profile,
        "--name",
        package_name,
        "--version",
        package_version,
        "-o",
        str(output_slxb),
    ]

    if package_description:
        compile_command.extend(["--description", package_description])

    entrypoints = manifest.get("entrypoints", {})
    if isinstance(entrypoints, dict):
        selected_entrypoint = (
            entrypoints.get("test") or entrypoints.get("main")
            if profile == "test"
            else entrypoints.get("main")
        )
        if selected_entrypoint:
            compile_command.extend(["--entry", str(selected_entrypoint)])

    build_config = manifest.get("build", {})
    link_mode = os.environ.get("SOLIX_LINK_MODE") or build_config.get("link")
    if link_mode:
        compile_command.extend(["--link", str(link_mode)])

    pkg_dir_override = os.environ.get("SOLIX_PKG_DIR") or build_config.get("pkg_dir")
    if pkg_dir_override:
        compile_command.extend(["--pkg-dir", str(pkg_dir_override)])

    verify_checksum = build_config.get("verify_checksum", True)
    if not verify_checksum or os.environ.get("SOLIX_VERIFY_CHECKSUM") == "0":
        compile_command.append("--no-verify")

    inline_threshold = os.environ.get("SOLIX_INLINE_THRESHOLD") or build_config.get("inline_threshold")
    if inline_threshold is not None:
        compile_command.extend(["--inline-threshold", str(inline_threshold)])

    for lib_file in stage_lib_directory.iterdir():
        if lib_file.name.endswith(".slxb"):
            compile_command.extend(["-l", str(lib_file)])

    dependencies = manifest.get("dependencies", {})
    if isinstance(dependencies, dict):
        for dependency_name, dependency_version in sorted(dependencies.items()):
            compile_command.extend(["--deps", f"{dependency_name}@{dependency_version}"])

    print(f"  [solix] Compiling Solix sources for '{package_name}' ({profile})...")
    run_command(
        compile_command,
        cwd=project_root,
        phase="Solix Bytecode Compilation",
        hint="Check compiler diagnostics above for syntax, type, or binding errors.",
        capture_output=False,
    )

    if not output_slxb.exists():
        raise BuildError(
            phase="Solix Bytecode Compilation",
            message=f"Expected compiled binary not found at '{output_slxb}'",
            command=compile_command,
            cwd=project_root,
            hint="Ensure solix compile produced the expected binary artifact.",
        )

    return output_slxb


def package_slxarc(stage_directory: Path, output_slxarc: Path) -> Path:
    """Creates a deterministic ZIP archive of the staging directory as a .slxarc."""
    if output_slxarc.exists():
        output_slxarc.unlink()

    output_slxarc.parent.mkdir(parents=True, exist_ok=True)

    with zipfile.ZipFile(output_slxarc, "w", compression=zipfile.ZIP_DEFLATED) as zip_file:
        for root_path, directory_names, file_names in os.walk(stage_directory):
            directory_names.sort()
            file_names.sort()
            for file_name in file_names:
                file_path = Path(root_path) / file_name
                archive_relative_path = file_path.relative_to(stage_directory)
                zip_file.write(file_path, arcname=str(archive_relative_path).replace("\\", "/"))

    print(f"  [package] Emitted archive: {output_slxarc}")
    return output_slxarc


# ============================================================================
# Tier 2: Turnkey Orchestrator
# ============================================================================

def auto_build(profile: str | None = None, project_root: Path | None = None) -> Path:
    """One-call automatic project build orchestrator.

    1. Loads manifest.toml and resolves effective build configuration
    2. Prepares build/.cache/stage/<profile>/
    3. Auto-detects & compiles all Rust cdylib projects in natives/
    4. Copies lib/ and resources/
    5. Compiles Solix sources into build/.cache/stage/<profile>/bin/<name>.slxb
    6. Records cryptographic checksums in staged manifest.toml
    7. Packages build/<profile>/<name>.slxarc
    8. Optionally auto-installs into local package store
    """
    total_start_time = time.perf_counter()

    if project_root is None:
        project_root = Path.cwd()

    manifest = load_manifest(project_root)
    build_config = manifest.get("build", {})

    # Resolve effective profile
    effective_profile = profile or os.environ.get("SOLIX_PROFILE") or build_config.get("profile", "debug")
    effective_profile = effective_profile.lower()
    if effective_profile not in ("debug", "release", "test"):
        raise BuildError(
            phase="Profile Resolution",
            message=f"Unknown build profile '{effective_profile}'; expected 'debug', 'release', or 'test'",
            cwd=project_root,
            hint="Set profile to 'debug', 'release', or 'test' via CLI or [build].profile in manifest.toml.",
        )

    print(f"=== Building Solix Project: {project_root.name} [{effective_profile}] ===")

    package_name = manifest["info"]["name"]
    staging = prepare_staging_dirs(project_root, effective_profile)

    # Compile native Rust subprojects
    native_start_time = time.perf_counter()
    native_projects = detect_native_projects(project_root / "natives")
    for native_project in native_projects:
        compile_rust_project(
            native_project,
            staging["natives_cache"],
            staging["stage_lib"],
            effective_profile,
        )
    native_duration_ms = (time.perf_counter() - native_start_time) * 1000.0

    # Copy external libraries and static resources
    copy_project_libs(project_root / "lib", staging["stage_lib"])
    copy_project_resources(project_root / "resources", staging["stage_resources"])

    # Compile Solix sources
    solix_start_time = time.perf_counter()
    compile_solix_sources(
        project_root,
        manifest,
        staging["stage_bin"],
        staging["stage_lib"],
        effective_profile,
    )
    solix_duration_ms = (time.perf_counter() - solix_start_time) * 1000.0

    # Record cryptographic checksums for native shared libraries
    staged_manifest_file = staging["stage"] / "manifest.toml"
    native_checksums: dict[str, str] = {}
    if staging["stage_lib"].exists():
        for root_path, _, file_names in os.walk(staging["stage_lib"]):
            for file_name in file_names:
                if file_name.endswith((".so", ".dll", ".dylib")):
                    lib_file = Path(root_path) / file_name
                    relative_path = str(lib_file.relative_to(staging["stage"])).replace("\\", "/")
                    hasher = hashlib.sha256()
                    hasher.update(lib_file.read_bytes())
                    native_checksums[relative_path] = hasher.hexdigest()

    if native_checksums and staged_manifest_file.exists():
        manifest_text = staged_manifest_file.read_text(encoding="utf-8")
        checksum_section = "\n[native_library_checksums]\n" + "\n".join(
            f'"{key}" = "{digest}"' for key, digest in sorted(native_checksums.items())
        ) + "\n"
        staged_manifest_file.write_text(manifest_text + checksum_section, encoding="utf-8")

    # Package distributable .slxarc
    packaging_start_time = time.perf_counter()
    final_archive_path = staging["build"] / f"{package_name}.slxarc"
    package_slxarc(staging["stage"], final_archive_path)
    packaging_duration_ms = (time.perf_counter() - packaging_start_time) * 1000.0

    # Phase timing summary
    total_duration_ms = (time.perf_counter() - total_start_time) * 1000.0
    print(f"=== Build Completed Successfully: {final_archive_path} ===")
    print("Phase Timings:")
    print(f"  - Native C/Rust Compilation:  {native_duration_ms:.1f} ms")
    print(f"  - Solix Bytecode Compilation: {solix_duration_ms:.1f} ms")
    print(f"  - Package Packaging:          {packaging_duration_ms:.1f} ms")
    print(f"  Total Elapsed Time:           {total_duration_ms:.1f} ms")

    # Optional automatic installation into package store
    should_install = (
        os.environ.get("SOLIX_AUTO_INSTALL") == "1"
        or build_config.get("install", False)
    )
    if should_install:
        force_install = (
            os.environ.get("SOLIX_FORCE_INSTALL") == "1"
            or build_config.get("force_install", False)
        )
        custom_pkg_dir = os.environ.get("SOLIX_PKG_DIR") or build_config.get("pkg_dir")

        solix_executable = resolve_solix_binary()
        install_command: list[str] = [
            solix_executable,
            "install",
            str(final_archive_path),
            "--profile",
            effective_profile,
        ]
        if force_install:
            install_command.append("--force")
        if custom_pkg_dir:
            install_command.extend(["--pkg-dir", str(custom_pkg_dir)])

        print(f"  [install] Auto-installing '{package_name}' ({effective_profile}) into package store...")
        run_command(
            install_command,
            cwd=project_root,
            phase="Store Installation",
            hint="Use 'solix install --force' or check package store permissions.",
            capture_output=False,
        )

    return final_archive_path


def init_project(
    target_path: Path | str,
    name: str | None = None,
    version: str = "0.1.0",
    description: str = "A Solix project",
    license_id: str = "Apache-2.0",
    authors: list[str] | None = None,
    main_entry: str = "Main::main",
    test_entry: str = "TestRunner::test",
    is_library: bool = False,
    no_native: bool = False,
    dependencies: list[str] | None = None,
    force: bool = False,
) -> Path:
    """Convenience wrapper delegating to solix_scaffold.generate_project."""
    import solix_scaffold as project_generator

    return project_generator.generate_project(
        target_path=Path(target_path),
        name=name,
        version=version,
        description=description,
        license_id=license_id,
        authors=authors,
        main_entry=main_entry,
        test_entry=test_entry,
        is_library=is_library,
        no_native=no_native,
        dependencies=dependencies,
        force=force,
    )


if __name__ == "__main__":
    run_cli(auto_build)
