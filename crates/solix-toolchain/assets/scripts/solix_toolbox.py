#!/usr/bin/env python3
"""Two-Tier Solix Build Toolbox.

Tier 1: Composable low-level build primitives.
Tier 2: High-level turnkey auto_build orchestrator.
"""

from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import sys
import tomllib
import zipfile


# ============================================================================
# Tier 1: Composable Toolbox Primitives
# ============================================================================

def load_manifest(project_root: Path) -> dict:
    """Parses manifest.toml from the project root directory."""
    manifest_path = project_root / "manifest.toml"
    if not manifest_path.exists():
        raise FileNotFoundError(f"manifest.toml not found at '{manifest_path}'")

    with open(manifest_path, "rb") as manifest_file:
        data = tomllib.load(manifest_file)

    if "info" not in data or "name" not in data["info"]:
        raise ValueError("manifest.toml must contain a valid [info] section with 'name'")

    return data


def prepare_staging_dirs(project_root: Path, profile: str) -> dict[str, Path]:
    """Prepares and cleans the ephemeral staging and output directories."""
    cache_dir = project_root / "build" / ".cache"
    natives_cache_dir = cache_dir / "natives" / profile
    stage_dir = cache_dir / "stage" / profile
    build_dir = project_root / "build" / profile

    stage_bin_dir = stage_dir / "bin"
    stage_lib_dir = stage_dir / "lib"
    stage_res_dir = stage_dir / "resources"

    # Recreate stage directory cleanly
    if stage_dir.exists():
        shutil.rmtree(stage_dir)

    stage_bin_dir.mkdir(parents=True, exist_ok=True)
    stage_lib_dir.mkdir(parents=True, exist_ok=True)
    stage_res_dir.mkdir(parents=True, exist_ok=True)
    natives_cache_dir.mkdir(parents=True, exist_ok=True)
    build_dir.mkdir(parents=True, exist_ok=True)

    # Stage manifest.toml
    shutil.copy2(project_root / "manifest.toml", stage_dir / "manifest.toml")

    return {
        "cache": cache_dir,
        "natives_cache": natives_cache_dir,
        "stage": stage_dir,
        "stage_bin": stage_bin_dir,
        "stage_lib": stage_lib_dir,
        "stage_resources": stage_res_dir,
        "build": build_dir,
    }


def detect_native_projects(natives_dir: Path) -> list[Path]:
    """Scans natives/ for Rust cdylib projects containing Cargo.toml."""
    if not natives_dir.exists():
        return []

    discovered = []
    for entry in natives_dir.iterdir():
        if entry.is_dir() and (entry / "Cargo.toml").exists():
            discovered.append(entry)

    discovered.sort()
    return discovered


def compile_rust_project(
    project_dir: Path,
    cache_dir: Path,
    stage_lib_dir: Path,
    profile: str,
) -> list[Path]:
    """Compiles a Rust cdylib project via cargo build and copies output to stage/lib."""
    target_dir = cache_dir / "cargo"
    cmd = [
        "cargo",
        "build",
        "--manifest-path",
        str(project_dir / "Cargo.toml"),
        "--target-dir",
        str(target_dir),
    ]

    is_release = (profile == "release")
    if is_release:
        cmd.append("--release")

    print(f"  [cargo] Building native Rust project '{project_dir.name}' ({profile})...")
    result = subprocess.run(cmd, check=False)
    if result.returncode != 0:
        raise RuntimeError(f"Cargo build failed for '{project_dir.name}' with code {result.returncode}")

    artifact_dir = target_dir / ("release" if is_release else "debug")

    # Match platform shared library extension
    if sys.platform == "win32":
        extensions = [".dll"]
    elif sys.platform == "darwin":
        extensions = [".dylib"]
    else:
        extensions = [".so"]

    emitted_libraries = []
    for file_path in artifact_dir.iterdir():
        if file_path.is_file() and any(file_path.name.endswith(ext) for ext in extensions):
            # Exclude Cargo build script artifacts and dylib deps if any
            dest_path = stage_lib_dir / file_path.name
            shutil.copy2(file_path, dest_path)
            emitted_libraries.append(dest_path)

    return emitted_libraries


def copy_project_libs(project_lib_dir: Path, stage_lib_dir: Path) -> None:
    """Copies pre-existing external library files into stage/lib."""
    if not project_lib_dir.exists():
        return

    for item in project_lib_dir.iterdir():
        if item.is_file() and not item.name.startswith("."):
            shutil.copy2(item, stage_lib_dir / item.name)


def copy_project_resources(resources_dir: Path, stage_res_dir: Path) -> None:
    """Copies static runtime assets into stage/resources."""
    if not resources_dir.exists():
        return

    for item in resources_dir.iterdir():
        if item.name.startswith("."):
            continue
        dest = stage_res_dir / item.name
        if item.is_dir():
            shutil.copytree(item, dest, dirs_exist_ok=True)
        else:
            shutil.copy2(item, dest)


def resolve_solix_binary() -> str:
    """Resolves the solix CLI executable path."""
    if "SOLIX_BIN" in os.environ and os.path.exists(os.environ["SOLIX_BIN"]):
        candidate_bin = Path(os.environ["SOLIX_BIN"])
        if candidate_bin.is_file() and candidate_bin.name in ("solix", "solix.exe"):
            return str(candidate_bin)

    # Search workspace targets starting from toolbox location up through parent directories
    current = Path(__file__).resolve().parent
    while current != current.parent:
        for candidate in [
            current / "target" / "release" / "solix",
            current / "target" / "debug" / "solix",
        ]:
            if candidate.exists() and os.access(candidate, os.X_OK):
                return str(candidate)
        current = current.parent

    which_result = shutil.which("solix")
    if which_result:
        return which_result

    return "solix"


def compile_solix_sources(
    project_root: Path,
    manifest: dict,
    stage_bin_dir: Path,
    stage_lib_dir: Path,
    profile: str,
) -> Path:
    """Invokes solix compile to emit the main application bytecode binary."""
    solix_bin = resolve_solix_binary()
    package_name = manifest["info"]["name"]
    package_version = manifest["info"].get("version", "0.1.0")
    package_desc = manifest["info"].get("description", "")

    output_slxb = stage_bin_dir / f"{package_name}.slxb"

    src_dir = project_root / "src"
    tests_dir = project_root / "tests"

    sources_to_compile = [str(src_dir)]
    if profile == "test" and tests_dir.exists():
        sources_to_compile.append(str(tests_dir))

    cmd = [
        solix_bin,
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

    if package_desc:
        cmd.extend(["--description", package_desc])

    # Pass any dependency .slxb libraries in stage/lib
    for lib_file in stage_lib_dir.iterdir():
        if lib_file.name.endswith(".slxb"):
            cmd.extend(["-l", str(lib_file)])

    # Pass explicit dependencies declared in manifest.toml [dependencies]
    dependencies = manifest.get("dependencies", {})
    if isinstance(dependencies, dict):
        for dep_name, dep_version in sorted(dependencies.items()):
            cmd.extend(["--deps", f"{dep_name}@{dep_version}"])

    print(f"  [solix] Compiling Solix sources for '{package_name}' ({profile})...")
    result = subprocess.run(cmd, check=False)
    if result.returncode != 0:
        raise RuntimeError(f"solix compile failed with exit code {result.returncode}")

    if not output_slxb.exists():
        raise FileNotFoundError(f"Expected compiled binary not found at '{output_slxb}'")

    return output_slxb


def package_slxarc(stage_dir: Path, output_slxarc: Path) -> Path:
    """Creates a deterministic ZIP archive of the staging directory as a .slxarc."""
    if output_slxarc.exists():
        output_slxarc.unlink()

    output_slxarc.parent.mkdir(parents=True, exist_ok=True)

    with zipfile.ZipFile(output_slxarc, "w", compression=zipfile.ZIP_DEFLATED) as zip_file:
        for root, dirs, files in os.walk(stage_dir):
            dirs.sort()
            files.sort()
            for file_name in files:
                file_path = Path(root) / file_name
                archive_rel_path = file_path.relative_to(stage_dir)
                zip_file.write(file_path, arcname=str(archive_rel_path).replace("\\", "/"))

    print(f"  [package] Emitted archive: {output_slxarc}")
    return output_slxarc


# ============================================================================
# Tier 2: Turnkey Orchestrator
# ============================================================================

def auto_build(profile: str = "debug", project_root: Path | None = None) -> Path:
    """One-call automatic project build orchestrator.

    1. Loads manifest.toml
    2. Prepares cache/stage/<profile>/
    3. Auto-detects & compiles all Rust cdylib projects in natives/
    4. Copies lib/ and resources/
    5. Compiles Solix sources into cache/stage/<profile>/bin/<name>.slxb
    6. Packages build/<profile>/<name>.slxarc
    """
    if project_root is None:
        project_root = Path.cwd()

    profile = profile.lower()
    if profile not in ("debug", "release", "test"):
        raise ValueError(f"Unknown build profile '{profile}'; expected debug, release, or test")

    print(f"=== Building Solix Project: {project_root.name} [{profile}] ===")

    manifest = load_manifest(project_root)
    package_name = manifest["info"]["name"]

    staging = prepare_staging_dirs(project_root, profile)

    # Compile native Rust subprojects
    native_projects = detect_native_projects(project_root / "natives")
    for native_proj in native_projects:
        compile_rust_project(
            native_proj,
            staging["natives_cache"],
            staging["stage_lib"],
            profile,
        )

    # Copy external libraries and static resources
    copy_project_libs(project_root / "lib", staging["stage_lib"])
    copy_project_resources(project_root / "resources", staging["stage_resources"])

    # Compile Solix sources
    compile_solix_sources(
        project_root,
        manifest,
        staging["stage_bin"],
        staging["stage_lib"],
        profile,
    )

    # Package distributable .slxarc
    final_archive_path = staging["build"] / f"{package_name}.slxarc"
    package_slxarc(staging["stage"], final_archive_path)

    print(f"=== Build Completed Successfully: {final_archive_path} ===")
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
    """Convenience wrapper delegating to init_project.generate_project."""
    import init_project as project_generator

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
    auto_build()
