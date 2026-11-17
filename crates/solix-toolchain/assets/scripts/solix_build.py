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
    ("x86_64-pc-windows-msvc", ".dll"),
    ("x86_64-apple-darwin", ".dylib"),
    ("aarch64-apple-darwin", ".dylib"),
]


def normalize_host_path(path: Path) -> Path:
    """Strips Windows verbatim path prefixes (`\\\\?\\`) so external tools like CMake work cleanly."""
    if os.name == "nt":
        raw_path = str(path)
        if raw_path.startswith("\\\\?\\UNC\\"):
            return Path("\\\\" + raw_path[8:])
        if raw_path.startswith("\\\\?\\"):
            return Path(raw_path[4:])
    return path


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

    if "configuration" not in manifest_data or "name" not in manifest_data["configuration"]:
        raise BuildError(
            phase="Manifest Validation",
            message="manifest.toml must contain a valid [configuration] section with 'name'",
            cwd=project_root,
            hint="Add [configuration] section with name = \"<package_name>\" to manifest.toml.",
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


_SYSTEM_CROSS_LINKERS: dict[str, tuple[str, str]] = {
    "aarch64-unknown-linux-gnu": ("aarch64-linux-gnu-gcc", "aarch64-linux-gnu-ar"),
    "x86_64-pc-windows-gnu": ("x86_64-w64-mingw32-gcc", "x86_64-w64-mingw32-ar"),
}


def _resolve_rustup_installed_targets(cargo_environment: dict[str, str]) -> set[str] | None:
    """Returns installed rustup target triples, or None when rustup is unavailable."""
    rustup_executable = shutil.which("rustup", path=cargo_environment.get("PATH", ""))
    if not rustup_executable:
        return None
    try:
        result = subprocess.run(
            [rustup_executable, "target", "list", "--installed"],
            capture_output=True,
            text=True,
            env=cargo_environment,
            check=False,
        )
        if result.returncode == 0:
            return {line.strip() for line in result.stdout.splitlines() if line.strip()}
    except OSError:
        pass
    return None


def _ensure_rustup_target(
    target_triple: str,
    installed_targets: set[str] | None,
    cargo_environment: dict[str, str],
) -> tuple[bool, str | None]:
    """Ensures `target_triple` stdlib is installed via rustup when available."""
    if installed_targets is None:
        return True, None
    if target_triple in installed_targets:
        return True, None
    if os.environ.get("SOLIX_NO_AUTO_RUSTUP", "").strip().lower() in ("1", "true", "yes"):
        return False, f"rustup target '{target_triple}' not installed (SOLIX_NO_AUTO_RUSTUP=1)"

    rustup_executable = shutil.which("rustup", path=cargo_environment.get("PATH", ""))
    if not rustup_executable:
        return False, f"rustup target '{target_triple}' not installed"

    print(f"  [rustup] Installing target standard library '{target_triple}'...")
    add_result = subprocess.run(
        [rustup_executable, "target", "add", target_triple],
        capture_output=True,
        text=True,
        env=cargo_environment,
        check=False,
    )
    if add_result.returncode == 0:
        installed_targets.add(target_triple)
        return True, None

    error_detail = (add_result.stderr or add_result.stdout or "").strip()
    return False, f"rustup target add failed ({error_detail})" if error_detail else "rustup target add failed"


def _select_release_target_plan(
    target_triple: str,
    host_target_triple: str,
    project_directory: Path,
    cargo_target_directory: Path,
    cargo_environment: dict[str, str],
    *,
    has_cargo_zigbuild: bool,
    has_zig: bool,
    has_cargo_xwin: bool,
    forced_target: bool,
) -> tuple[list[str] | None, dict[str, str], bool, str]:
    """Resolves Cargo invocation strategy for a single release target triple.

    Returns `(command_or_none, env_copy, uses_explicit_target_dir, label_or_skip_reason)`.
    """
    target_env = cargo_environment.copy()
    is_exact_host = target_triple == host_target_triple
    is_darwin_host = sys.platform == "darwin" and "apple-darwin" in target_triple
    is_msvc_target = "-pc-windows-msvc" in target_triple

    if sys.platform.startswith("linux") and is_exact_host:
        target_env["RUSTFLAGS"] = "-C link-arg=-L/usr/lib " + target_env.get("RUSTFLAGS", "")

    base_manifest_args = [
        "--manifest-path",
        str(project_directory / "Cargo.toml"),
        "--target-dir",
        str(cargo_target_directory),
        "--release",
    ]

    # 1. Native macOS Clang compiles both Intel & Apple Silicon Darwin binaries cleanly.
    if is_darwin_host:
        return ["cargo", "build", *base_manifest_args, "--target", target_triple], target_env, True, "native-clang"

    # 2. Zig + cargo-zigbuild cross-compiles all non-MSVC targets across Linux/macOS/Windows hosts.
    if has_cargo_zigbuild and has_zig and not is_msvc_target:
        return ["cargo", "zigbuild", *base_manifest_args, "--target", target_triple], target_env, True, "zigbuild"

    # 3. cargo-xwin cross-compiles Windows MSVC targets on non-Windows hosts when installed.
    if has_cargo_xwin and is_msvc_target and sys.platform != "win32":
        return ["cargo", "xwin", "build", *base_manifest_args, "--target", target_triple], target_env, True, "cargo-xwin"

    # 4. System cross-GCC (e.g. aarch64-linux-gnu-gcc, x86_64-w64-mingw32-gcc) when present.
    cross_tools = _SYSTEM_CROSS_LINKERS.get(target_triple)
    if cross_tools and not is_exact_host:
        cross_cc, cross_ar = cross_tools
        resolved_cc = shutil.which(cross_cc, path=cargo_environment.get("PATH", ""))
        if resolved_cc:
            underscored = target_triple.replace("-", "_")
            upper_triple = underscored.upper()
            target_env[f"CARGO_TARGET_{upper_triple}_LINKER"] = resolved_cc
            target_env[f"CC_{underscored}"] = resolved_cc
            resolved_ar = shutil.which(cross_ar, path=cargo_environment.get("PATH", ""))
            if resolved_ar:
                target_env[f"AR_{underscored}"] = resolved_ar
            return ["cargo", "build", *base_manifest_args, "--target", target_triple], target_env, True, cross_cc

    # 5. Exact host toolchain fallback (or explicit forced target).
    if is_exact_host:
        if sys.platform.startswith("linux"):
            return ["cargo", "build", *base_manifest_args], target_env, False, "host-cargo"
        return ["cargo", "build", *base_manifest_args, "--target", target_triple], target_env, True, "host-cargo"

    if forced_target:
        return ["cargo", "build", *base_manifest_args, "--target", target_triple], target_env, True, "explicit-cargo"

    if is_msvc_target:
        return None, target_env, False, "requires Windows MSVC host or cargo-xwin"
    if has_cargo_zigbuild and not has_zig:
        return None, target_env, False, "cargo-zigbuild found but 'zig' executable is missing in PATH"
    return None, target_env, False, "requires cargo-zigbuild + zig (or system cross-compiler)"


def compile_rust_project(
    project_directory: Path,
    cache_directory: Path,
    stage_lib_directory: Path,
    profile: str,
) -> list[Path]:
    """Compiles a Rust cdylib project with strict CARGO_TARGET_DIR isolation."""
    project_directory = normalize_host_path(project_directory)
    cache_directory = normalize_host_path(cache_directory)
    cargo_target_directory = cache_directory / "cargo"
    host_target_triple = get_host_target_triple()

    cargo_environment = os.environ.copy()
    cargo_environment["CARGO_TARGET_DIR"] = str(cargo_target_directory)
    home_cargo_bin = str(Path.home() / ".cargo" / "bin")
    current_system_path = cargo_environment.get("PATH", "")
    if home_cargo_bin not in current_system_path.split(os.pathsep):
        cargo_environment["PATH"] = f"{home_cargo_bin}{os.pathsep}{current_system_path}"

    emitted_libraries: list[Path] = []
    forced_target_triple = os.environ.get("SOLIX_TARGET_TRIPLE", "").strip()
    installed_targets = _resolve_rustup_installed_targets(cargo_environment)

    if profile != "release":
        # Debug/Test: Compile only for the active host (or explicitly requested cross target)
        active_target_triple = forced_target_triple or host_target_triple
        is_cross_target = active_target_triple != host_target_triple
        target_cargo_env = cargo_environment.copy()
        if sys.platform.startswith("linux") and not is_cross_target:
            target_cargo_env["RUSTFLAGS"] = "-C link-arg=-L/usr/lib " + target_cargo_env.get("RUSTFLAGS", "")

        use_zigbuild_for_debug = (
            is_cross_target
            and bool(shutil.which("cargo-zigbuild", path=cargo_environment["PATH"]))
            and bool(shutil.which("zig", path=cargo_environment["PATH"]))
            and "-pc-windows-msvc" not in active_target_triple
        )
        if is_cross_target:
            _ensure_rustup_target(active_target_triple, installed_targets, cargo_environment)

        cargo_command = [
            "cargo",
            "zigbuild" if use_zigbuild_for_debug else "build",
            "--manifest-path",
            str(project_directory / "Cargo.toml"),
            "--target-dir",
            str(cargo_target_directory),
        ]
        if is_cross_target:
            cargo_command.extend(["--target", active_target_triple])

        print(f"  [cargo] Building native Rust project '{project_directory.name}' ({profile}, {active_target_triple})...")
        run_command(
            cargo_command,
            cwd=project_directory,
            env=target_cargo_env,
            phase="Native Rust Host Build",
            hint="Check that Rust toolchain is installed and Cargo.toml has no errors.",
            capture_output=False,
        )

        artifact_directory = (
            cargo_target_directory / active_target_triple / "debug"
            if is_cross_target
            else cargo_target_directory / "debug"
        )
        if active_target_triple.endswith("-windows-msvc") or active_target_triple.endswith("-windows-gnu") or (not is_cross_target and sys.platform == "win32"):
            allowed_extensions = [".dll"]
        elif active_target_triple.endswith("-apple-darwin") or (not is_cross_target and sys.platform == "darwin"):
            allowed_extensions = [".dylib"]
        else:
            allowed_extensions = [".so"]

        if artifact_directory.exists():
            for file_path in artifact_directory.iterdir():
                if file_path.is_file() and any(file_path.name.endswith(extension) for extension in allowed_extensions):
                    if is_cross_target or forced_target_triple:
                        target_stage_dir = stage_lib_directory / active_target_triple
                        target_stage_dir.mkdir(parents=True, exist_ok=True)
                        destination_path = target_stage_dir / file_path.name
                    else:
                        destination_path = stage_lib_directory / file_path.name
                    shutil.copy2(file_path, destination_path)
                    emitted_libraries.append(destination_path)
        return emitted_libraries

    # Release mode: Build forced target (when SOLIX_TARGET_TRIPLE is set) or all supported platforms
    has_cargo_zigbuild = bool(shutil.which("cargo-zigbuild", path=cargo_environment["PATH"]))
    has_zig = bool(shutil.which("zig", path=cargo_environment["PATH"]))
    has_cargo_xwin = bool(shutil.which("cargo-xwin", path=cargo_environment["PATH"]))
    strict_cross = os.environ.get("SOLIX_STRICT_CROSS", "").strip().lower() in ("1", "true", "yes")

    built_targets: list[str] = []
    skipped_targets: list[tuple[str, str]] = []
    last_target_error_output: str | None = None
    candidate_targets = (
        [
            (
                forced_target_triple,
                ".dll"
                if "windows" in forced_target_triple
                else (".dylib" if "apple-darwin" in forced_target_triple else ".so"),
            )
        ]
        if forced_target_triple
        else SUPPORTED_RELEASE_TARGETS
    )

    for target_triple, target_extension in candidate_targets:
        cargo_command, target_cargo_env, uses_explicit_target_dir, strategy_label = (
            _select_release_target_plan(
                target_triple,
                host_target_triple,
                project_directory,
                cargo_target_directory,
                cargo_environment,
                has_cargo_zigbuild=has_cargo_zigbuild,
                has_zig=has_zig,
                has_cargo_xwin=has_cargo_xwin,
                forced_target=bool(forced_target_triple),
            )
        )
        if cargo_command is None:
            skipped_targets.append((target_triple, strategy_label))
            print(f"  [warning] Skipping cross-target '{target_triple}': {strategy_label}")
            continue

        if installed_targets is not None:
            target_ready, rustup_reason = _ensure_rustup_target(
                target_triple,
                installed_targets,
                cargo_environment,
            )
            if not target_ready:
                skipped_targets.append((target_triple, rustup_reason or "target not installed"))
                print(f"  [warning] Skipping cross-target '{target_triple}': {rustup_reason}")
                continue

        target_stage_directory = stage_lib_directory / target_triple
        target_stage_directory.mkdir(parents=True, exist_ok=True)

        print(f"  [cargo] Building target '{target_triple}' ({profile}, {strategy_label})...")
        try:
            run_command(
                cargo_command,
                cwd=project_directory,
                env=target_cargo_env,
                phase=f"Native Rust Target Build ({target_triple})",
                hint="Install target standard library via: rustup target add " + target_triple,
            )
        except BuildError as target_error:
            error_output = target_error.stderr or ""
            if "can't find crate for `std`" in error_output or "can't find crate for std" in error_output:
                reason = (
                    "rust stdlib target not installed (install rustup or add target sysroot)"
                    if installed_targets is None
                    else "rustup stdlib target not installed"
                )
                skipped_targets.append((target_triple, reason))
                print(f"  [warning] Cross-compilation skipped for '{target_triple}': {reason}")
            else:
                trimmed_output = error_output.strip()
                last_target_error_output = trimmed_output or None
                first_line = trimmed_output.splitlines()[-1] if trimmed_output else "compilation error"
                skipped_targets.append((target_triple, first_line))
                print(f"  [warning] Target '{target_triple}' compilation failed:")
                if trimmed_output:
                    print(trimmed_output)
            try:
                if target_stage_directory.exists() and not any(target_stage_directory.iterdir()):
                    target_stage_directory.rmdir()
            except OSError:
                pass
            continue

        artifact_directory = (
            cargo_target_directory / target_triple / "release"
            if uses_explicit_target_dir
            else cargo_target_directory / "release"
        )

        copied_for_target = False
        if artifact_directory.exists():
            for file_path in sorted(artifact_directory.iterdir()):
                if file_path.is_file() and file_path.name.endswith(target_extension):
                    destination_target_path = target_stage_directory / file_path.name
                    shutil.copy2(file_path, destination_target_path)
                    emitted_libraries.append(destination_target_path)
                    copied_for_target = True

        if copied_for_target:
            built_targets.append(target_triple)
        else:
            skipped_targets.append((target_triple, f"no *{target_extension} produced"))

    if not built_targets:
        raise BuildError(
            phase="Native Rust Build",
            message=f"All target builds failed for native project '{project_directory.name}'",
            cwd=project_directory,
            stderr=last_target_error_output,
            hint="Ensure host build dependencies and headers are available.",
        )

    if not forced_target_triple:
        print(
            f"  [cross] Built native targets ({len(built_targets)}/{len(candidate_targets)}) "
            f"for '{project_directory.name}': {', '.join(built_targets)}"
        )
        if skipped_targets:
            if installed_targets is None:
                print(
                    "  [cross] Hint: Active rustc is not managed by rustup; install rustup + "
                    "cargo-zigbuild + zig for multi-architecture release cross-compilation."
                )
            elif not (has_cargo_zigbuild and has_zig):
                print(
                    "  [cross] Hint: Install zig and cargo-zigbuild to cross-compile all desktop "
                    "architectures from this host."
                )

    if strict_cross:
        required_platforms = {"linux-x86_64", "linux-aarch64", "windows-x86_64", "macos-x86_64", "macos-aarch64"}
        covered_platforms: set[str] = set()
        for built in built_targets:
            if built == "x86_64-unknown-linux-gnu":
                covered_platforms.add("linux-x86_64")
            elif built == "aarch64-unknown-linux-gnu":
                covered_platforms.add("linux-aarch64")
            elif built in ("x86_64-pc-windows-gnu", "x86_64-pc-windows-msvc"):
                covered_platforms.add("windows-x86_64")
            elif built == "x86_64-apple-darwin":
                covered_platforms.add("macos-x86_64")
            elif built == "aarch64-apple-darwin":
                covered_platforms.add("macos-aarch64")
        missing_platforms = sorted(required_platforms - covered_platforms)
        if missing_platforms and not forced_target_triple:
            raise BuildError(
                phase="Cross-Platform Release Gate",
                message=(
                    f"SOLIX_STRICT_CROSS=1 enabled, missing platforms for '{project_directory.name}': "
                    f"{', '.join(missing_platforms)}"
                ),
                cwd=project_directory,
                stderr=last_target_error_output,
                hint="Install rustup cross targets, zig, cargo-zigbuild, and any required platform SDKs.",
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


def resolve_profile_options(manifest: dict[str, Any], target_profile: str) -> dict[str, Any]:
    """Resolves hierarchical profile inheritance (e.g. test -> debug -> release)."""
    profiles_table = manifest.get("profiles", {})
    if "release" not in profiles_table:
        raise BuildError(
            phase="Profile Resolution",
            message="manifest.toml must define mandatory [profiles.release] profile",
            hint="Add [profiles.release] section to manifest.toml.",
        )

    visited_profiles: list[str] = []
    resolution_chain: list[str] = []
    current_profile_name = target_profile

    while current_profile_name:
        if current_profile_name in visited_profiles:
            raise BuildError(
                phase="Profile Resolution",
                message=f"Cyclic profile inheritance detected: {' -> '.join(visited_profiles)} -> {current_profile_name}",
                hint="Ensure profile inheritance hierarchy does not contain cycles.",
            )
        visited_profiles.append(current_profile_name)
        resolution_chain.append(current_profile_name)
        if current_profile_name == "release":
            break

        current_config = profiles_table.get(current_profile_name, {})
        parent_name = current_config.get("inherits")
        if not parent_name:
            parent_name = "release"
        current_profile_name = parent_name

    resolved_options: dict[str, Any] = {
        "name": target_profile,
        "entrypoint": "Main::main",
        "opt_level": 3,
        "inline_threshold": 16,
        "debug_symbols": False,
        "static": True,
        "emit_disassembly": False,
        "warnings_as_errors": False,
        "verify_checksum": True,
        "link": "static",
        "sources": ["src"],
        "dependencies": {},
    }

    while resolution_chain:
        step_profile = resolution_chain.pop()
        step_config = profiles_table.get(step_profile)
        if step_config is None:
            if step_profile == "debug":
                resolved_options["opt_level"] = 0
                resolved_options["inline_threshold"] = 8
                resolved_options["debug_symbols"] = True
                resolved_options["static"] = False
                resolved_options["link"] = "dynamic"
            elif step_profile == "test":
                resolved_options["opt_level"] = 0
                resolved_options["inline_threshold"] = 8
                resolved_options["debug_symbols"] = True
                resolved_options["entrypoint"] = "TestRunner::test"
                if "tests" not in resolved_options["sources"] and "test" not in resolved_options["sources"]:
                    resolved_options["sources"].append("tests")
            continue

        if "entrypoint" in step_config:
            resolved_options["entrypoint"] = step_config["entrypoint"]
        if "optimization_level" in step_config:
            resolved_options["opt_level"] = step_config["optimization_level"]
        if "opt_level" in step_config:
            resolved_options["opt_level"] = step_config["opt_level"]
        if "inline_threshold" in step_config:
            resolved_options["inline_threshold"] = step_config["inline_threshold"]
        if "debug_symbols" in step_config:
            resolved_options["debug_symbols"] = step_config["debug_symbols"]
        if "static" in step_config:
            resolved_options["static"] = step_config["static"]
            resolved_options["link"] = "static" if step_config["static"] else "dynamic"
        if "link" in step_config:
            resolved_options["link"] = step_config["link"]
            resolved_options["static"] = (step_config["link"] == "static")
        if "emit_disassembly" in step_config:
            resolved_options["emit_disassembly"] = step_config["emit_disassembly"]
        if "warnings_as_errors" in step_config:
            resolved_options["warnings_as_errors"] = step_config["warnings_as_errors"]
        if "verify_checksum" in step_config:
            resolved_options["verify_checksum"] = step_config["verify_checksum"]

        if "sources" in step_config:
            if step_profile == "release":
                resolved_options["sources"] = list(step_config["sources"])
            else:
                for source_item in step_config["sources"]:
                    if source_item not in resolved_options["sources"]:
                        resolved_options["sources"].append(source_item)

        if "dependencies" in step_config and isinstance(step_config["dependencies"], dict):
            for raw_dep_key, raw_dep_spec in step_config["dependencies"].items():
                resolved_options["dependencies"][raw_dep_key] = raw_dep_spec

    return resolved_options


def compile_solix_sources(
    project_root: Path,
    manifest: dict[str, Any],
    stage_bin_directory: Path,
    stage_lib_directory: Path,
    profile: str,
) -> Path:
    """Invokes solix compile to emit the main application bytecode binary."""
    solix_executable = resolve_solix_binary()
    package_name = manifest["configuration"]["name"]
    package_version = manifest["configuration"].get("version", "0.1.0")
    package_description = manifest["configuration"].get("description", "")

    output_slxb = stage_bin_directory / f"{package_name}.slxb"

    resolved_profile_options = resolve_profile_options(manifest, profile)

    sources_to_compile: list[str] = []
    for source_subpath in resolved_profile_options["sources"]:
        full_source_path = project_root / source_subpath
        if full_source_path.exists():
            sources_to_compile.append(str(full_source_path))

    if not sources_to_compile:
        fallback_src = project_root / "src"
        if fallback_src.exists():
            sources_to_compile.append(str(fallback_src))

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

    selected_entrypoint = resolved_profile_options.get("entrypoint")
    if selected_entrypoint:
        compile_command.extend(["--entry", str(selected_entrypoint)])

    link_mode = os.environ.get("SOLIX_LINK_MODE") or resolved_profile_options.get("link")
    if link_mode:
        compile_command.extend(["--link", str(link_mode)])

    pkg_dir_override = os.environ.get("SOLIX_PKG_DIR")
    if pkg_dir_override:
        compile_command.extend(["--pkg-dir", str(pkg_dir_override)])

    verify_checksum = resolved_profile_options.get("verify_checksum", True)
    if not verify_checksum or os.environ.get("SOLIX_VERIFY_CHECKSUM") == "0":
        compile_command.append("--no-verify")

    inline_threshold = os.environ.get("SOLIX_INLINE_THRESHOLD") or resolved_profile_options.get("inline_threshold")
    if inline_threshold is not None:
        compile_command.extend(["--inline-threshold", str(inline_threshold)])

    if resolved_profile_options.get("emit_disassembly"):
        compile_command.append("--disasm")

    for lib_file in stage_lib_directory.iterdir():
        if lib_file.name.endswith(".slxb"):
            compile_command.extend(["-l", str(lib_file)])

    dependencies = resolved_profile_options.get("dependencies", {})
    if isinstance(dependencies, dict):
        for raw_dependency_key, dependency_val in sorted(dependencies.items()):
            if not isinstance(dependency_val, dict):
                raise BuildError(
                    phase="Manifest Dependency Validation",
                    message=(
                        f"Invalid dependency format for '{raw_dependency_key}' in manifest.toml. "
                        f"Only table notation is supported: {raw_dependency_key} = {{ version = \"...\", profile = \"...\" }}"
                    ),
                    command=[],
                    cwd=project_root,
                    hint=f"Update manifest.toml dependencies to use table notation, e.g. {raw_dependency_key} = {{ version = \"5.5.0\", profile = \"release\" }}",
                )
            dep_version = dependency_val.get("version", "*")
            dep_profile = dependency_val.get("profile")
            if dep_profile:
                compile_command.extend(["--deps", f"{raw_dependency_key}@{dep_version}#{dep_profile}"])
            else:
                compile_command.extend(["--deps", f"{raw_dependency_key}@{dep_version}"])

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


def resolve_package_store_directory(resolved_options: dict[str, Any]) -> Path:
    """Resolves the local package store directory using the same precedence as the Solix toolchain."""
    pkg_dir_override = os.environ.get("SOLIX_PKG_DIR", "").strip()
    if pkg_dir_override:
        return Path(pkg_dir_override)

    profile_pkg_dir = resolved_options.get("pkg_dir")
    if profile_pkg_dir:
        return Path(str(profile_pkg_dir))

    solix_home = os.environ.get("SOLIX_HOME", "").strip()
    if solix_home:
        return Path(solix_home) / "packages"

    if sys.platform == "win32":
        user_profile = os.environ.get("USERPROFILE", "").strip()
        if user_profile:
            return Path(user_profile) / ".solix" / "packages"
        local_app_data = os.environ.get("LOCALAPPDATA", "").strip()
        if local_app_data:
            return Path(local_app_data) / "solix" / "packages"

    home_env = os.environ.get("HOME", "").strip()
    if home_env:
        return Path(home_env) / ".solix" / "packages"

    return Path.home() / ".solix" / "packages"


def _find_dependency_archive(
    package_store: Path,
    dependency_name: str,
    dependency_version: str,
    dependency_profile: str | None,
) -> Path | None:
    """Finds the best matching `.slxarc` archive for a dependency requirement in `package_store`."""
    candidate_archives: list[Path] = []
    if dependency_profile:
        candidate_archives.append(
            package_store
            / f"{dependency_name}-{dependency_profile}@{dependency_version}"
            / f"{dependency_name}.slxarc"
        )
        candidate_archives.extend(
            reversed(sorted(package_store.glob(f"{dependency_name}-{dependency_profile}@*/{dependency_name}.slxarc")))
        )
    else:
        for fallback_profile in ("release", "debug", "test"):
            candidate_archives.append(
                package_store
                / f"{dependency_name}-{fallback_profile}@{dependency_version}"
                / f"{dependency_name}.slxarc"
            )
            candidate_archives.extend(
                reversed(sorted(package_store.glob(f"{dependency_name}-{fallback_profile}@*/{dependency_name}.slxarc")))
            )

    candidate_archives.append(
        package_store / f"{dependency_name}@{dependency_version}" / f"{dependency_name}.slxarc"
    )
    candidate_archives.append(
        package_store / dependency_name / dependency_version / f"{dependency_name}.slxarc"
    )
    candidate_archives.extend(
        reversed(sorted(package_store.glob(f"{dependency_name}/*/{dependency_name}.slxarc")))
    )
    return next((path for path in candidate_archives if path.is_file()), None)


def stage_dependency_native_libraries(
    manifest: dict[str, Any],
    profile: str,
    stage_lib_directory: Path,
) -> list[Path]:
    """Bundles native shared libraries from statically linked dependencies into `stage/lib/`.

    Static linking (`--link static`) merges reachable Solix bytecode and clears external bytecode
    dependencies from the compiled binary's runtime requirements. Any native `.so`/`.dll`/`.dylib`
    shared libraries owned by those dependencies must therefore be packaged inside the static
    `.slxarc` archive so `solix run` can load them cleanly without requiring the dependency package
    at runtime.
    """
    resolved_options = resolve_profile_options(manifest, profile)
    link_mode = os.environ.get("SOLIX_LINK_MODE") or resolved_options.get("link")
    if link_mode != "static":
        return []

    dependencies = resolved_options.get("dependencies", {})
    if not isinstance(dependencies, dict) or not dependencies:
        return []

    package_store = resolve_package_store_directory(resolved_options)
    if not package_store.exists():
        return []

    pending_dependencies: list[tuple[str, str, str | None]] = []
    for dependency_name, dependency_spec in sorted(dependencies.items()):
        if isinstance(dependency_spec, dict):
            dep_version = str(dependency_spec.get("version", "*"))
            dep_profile = dependency_spec.get("profile")
            pending_dependencies.append((dependency_name, dep_version, dep_profile))

    visited_archives: set[Path] = set()
    staged_libraries: list[Path] = []

    while pending_dependencies:
        dep_name, dep_version, dep_profile = pending_dependencies.pop(0)
        dependency_archive = _find_dependency_archive(
            package_store,
            dep_name,
            dep_version,
            dep_profile,
        )
        if dependency_archive is None or dependency_archive in visited_archives:
            continue
        visited_archives.add(dependency_archive)

        with zipfile.ZipFile(dependency_archive) as archive_file:
            native_entries = [
                archive_entry
                for archive_entry in archive_file.infolist()
                if archive_entry.filename.startswith("lib/")
                and archive_entry.filename.endswith((".so", ".dll", ".dylib"))
            ]
            qualified_basenames = {
                Path(archive_entry.filename).name
                for archive_entry in native_entries
                if "/" in archive_entry.filename[len("lib/"):]
            }
            for archive_entry in native_entries:
                entry_name = archive_entry.filename
                relative_subpath = entry_name[len("lib/"):]
                is_qualified = "/" in relative_subpath
                if not is_qualified and Path(entry_name).name in qualified_basenames:
                    continue
                destination_path = stage_lib_directory / relative_subpath
                if destination_path.exists():
                    continue
                destination_path.parent.mkdir(parents=True, exist_ok=True)
                destination_path.write_bytes(archive_file.read(archive_entry))
                staged_libraries.append(destination_path)

            if "manifest.toml" in archive_file.namelist():
                try:
                    dep_manifest = tomllib.loads(archive_file.read("manifest.toml").decode("utf-8"))
                    dep_resolved = resolve_profile_options(dep_manifest, dep_profile or "release")
                    transitive_deps = dep_resolved.get("dependencies", {})
                    if isinstance(transitive_deps, dict):
                        for trans_name, trans_spec in sorted(transitive_deps.items()):
                            if isinstance(trans_spec, dict):
                                pending_dependencies.append((
                                    trans_name,
                                    str(trans_spec.get("version", "*")),
                                    trans_spec.get("profile"),
                                ))
                except (tomllib.TOMLDecodeError, BuildError, UnicodeDecodeError):
                    pass

    if staged_libraries:
        print(
            f"  [static] Bundled {len(staged_libraries)} dependency native "
            f"librar{'y' if len(staged_libraries) == 1 else 'ies'} into static archive"
        )
    return staged_libraries


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

    # Resolve effective profile
    effective_profile = profile or os.environ.get("SOLIX_PROFILE") or "release"
    effective_profile = effective_profile.lower()
    if effective_profile not in ("debug", "release", "test") and effective_profile not in manifest.get("profiles", {}):
        raise BuildError(
            phase="Profile Resolution",
            message=f"Unknown build profile '{effective_profile}'; expected 'debug', 'release', 'test', or a profile defined in manifest.toml",
            cwd=project_root,
            hint="Set profile to 'debug', 'release', 'test', or a custom profile defined in manifest.toml via CLI or build command.",
        )

    print(f"=== Building Solix Project: {project_root.name} [{effective_profile}] ===")

    package_name = manifest["configuration"]["name"]
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

    # Bundle native shared libraries from dependencies for statically linked archives
    stage_dependency_native_libraries(manifest, effective_profile, staging["stage_lib"])

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

    packaged_targets = sorted({
        key.split("/")[1]
        for key in native_checksums
        if key.startswith("lib/") and key.count("/") >= 2
    })
    if packaged_targets:
        print(
            f"  [platforms] Packaged architectures ({len(packaged_targets)}): "
            f"{', '.join(packaged_targets)}"
        )

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
    profile_options = resolve_profile_options(manifest, effective_profile)
    should_install = (
        os.environ.get("SOLIX_AUTO_INSTALL") == "1"
        or profile_options.get("install", False)
    )
    if should_install:
        force_install = (
            os.environ.get("SOLIX_FORCE_INSTALL") == "1"
            or profile_options.get("force_install", False)
        )
        custom_pkg_dir = os.environ.get("SOLIX_PKG_DIR") or profile_options.get("pkg_dir")

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
