#!/usr/bin/env python3
"""Build and stage default first-party libraries from libraries/* into assets/packages/.

Scans the libraries directory, parses manifests, topologically sorts dependencies,
builds each library across requested profiles, and copies the resulting .slxarc archives
into crates/solix-toolchain/assets/packages/<name>-<profile>@<version>/<name>.slxarc.
Emits bundled_manifest.json indexing all staged bundled archives.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib  # type: ignore


def compute_sha256(file_path: Path) -> str:
    """Computes SHA-256 hexadecimal digest of a file in streaming chunks."""
    hasher = hashlib.sha256()
    with open(file_path, "rb") as stream:
        while chunk := stream.read(65536):
            hasher.update(chunk)
    return hasher.hexdigest()


def _strip_yaml_scalar(raw_val: str) -> str:
    """Strips surrounding quotes and inline comments from a simple YAML scalar value."""
    cleaned = raw_val.strip()
    if not cleaned:
        return ""
    if (cleaned.startswith('"') and cleaned.endswith('"')) or (
        cleaned.startswith("'") and cleaned.endswith("'")
    ):
        return cleaned[1:-1]
    if " #" in cleaned:
        cleaned = cleaned.split(" #", 1)[0].strip()
    return cleaned.strip("'\"")


def _parse_yaml_manifest_summary(manifest_path: Path) -> dict[str, Any]:
    """Parses configuration.{name,version} and profiles.release.dependencies from manifest.yaml using stdlib."""
    text = manifest_path.read_text(encoding="utf-8")
    section_stack: list[tuple[int, str]] = []
    config_table: dict[str, str] = {}
    release_dependencies: dict[str, Any] = {}

    for raw_line in text.splitlines():
        stripped = raw_line.strip()
        if not stripped or stripped.startswith("#") or stripped.startswith("-"):
            continue
        indent = len(raw_line) - len(raw_line.lstrip(" "))
        while section_stack and section_stack[-1][0] >= indent:
            section_stack.pop()

        if ":" not in stripped:
            continue
        key_part, val_part = stripped.split(":", 1)
        key = key_part.strip().strip("'\"")
        val = val_part.strip()

        parent_path = [item[1] for item in section_stack]
        if not val or val.startswith("#"):
            section_stack.append((indent, key))
            if parent_path == ["profiles", "release", "dependencies"]:
                release_dependencies.setdefault(key, {})
            continue

        if parent_path == ["configuration"] and key in ("name", "version"):
            config_table[key] = _strip_yaml_scalar(val)
        elif parent_path == ["profiles", "release", "dependencies"]:
            release_dependencies.setdefault(key, _strip_yaml_scalar(val))

    return {
        "configuration": config_table,
        "profiles": {"release": {"dependencies": release_dependencies}},
    }


def parse_manifest(manifest_path: Path) -> dict[str, Any]:
    """Parses manifest.yaml (or legacy manifest.toml) and returns its summary data dictionary."""
    if manifest_path.suffix.lower() in (".yaml", ".yml"):
        return _parse_yaml_manifest_summary(manifest_path)
    with open(manifest_path, "rb") as manifest_file:
        return tomllib.load(manifest_file)


def find_manifest_file(directory: Path) -> Path | None:
    """Returns manifest.yaml, manifest.yml, or manifest.toml if present in directory."""
    for candidate_name in ("manifest.yaml", "manifest.yml", "manifest.toml"):
        candidate_path = directory / candidate_name
        if candidate_path.is_file():
            return candidate_path
    return None


def discover_libraries(libraries_directory: Path) -> dict[str, dict[str, Any]]:
    """Scans libraries_directory for folders containing manifest.yaml or manifest.toml."""
    libraries_map: dict[str, dict[str, Any]] = {}
    if not libraries_directory.exists():
        return libraries_map

    for candidate_entry in sorted(libraries_directory.iterdir()):
        if not candidate_entry.is_dir():
            continue
        manifest_path = find_manifest_file(candidate_entry)
        if manifest_path is None:
            continue

        manifest_data = parse_manifest(manifest_path)
        config_table = manifest_data.get("configuration", manifest_data.get("info", {}))
        package_name = config_table.get("name")
        package_version = config_table.get("version", "0.1.0")
        dependencies_table = (
            manifest_data.get("profiles", {}).get("release", {}).get("dependencies", {})
            or manifest_data.get("dependencies", {})
        )

        if not package_name:
            continue

        dependency_names: list[str] = []
        for raw_dep_name in dependencies_table.keys():
            normalized_dep_name = raw_dep_name.strip()
            if normalized_dep_name:
                dependency_names.append(normalized_dep_name)

        libraries_map[package_name] = {
            "name": package_name,
            "version": package_version,
            "directory": candidate_entry,
            "dependencies": dependency_names,
            "manifest": manifest_data,
        }

    return libraries_map


def topological_sort(libraries_map: dict[str, dict[str, Any]]) -> list[str]:
    """Sorts libraries topologically based on dependency relationships."""
    sorted_order: list[str] = []
    visited_nodes: set[str] = set()
    visiting_nodes: set[str] = set()

    def depth_first_search(current_node: str) -> None:
        if current_node in visiting_nodes:
            raise RuntimeError(f"Cyclic dependency detected involving library: '{current_node}'")
        if current_node in visited_nodes:
            return

        visiting_nodes.add(current_node)
        library_info = libraries_map.get(current_node)
        if library_info:
            for dep_name in library_info["dependencies"]:
                if dep_name in libraries_map:
                    depth_first_search(dep_name)

        visiting_nodes.remove(current_node)
        visited_nodes.add(current_node)
        sorted_order.append(current_node)

    for library_name in sorted(libraries_map.keys()):
        if library_name not in visited_nodes:
            depth_first_search(library_name)

    return sorted_order


def build_library(
    solix_executable: list[str],
    library_directory: Path,
    target_profile: str,
    environment_variables: dict[str, str],
    target_override: str | None = None,
    platform_override: str | None = None,
) -> None:
    """Runs solix build on a library for a given profile."""
    build_command = [*solix_executable, "build", str(library_directory), "--profile", target_profile]
    if target_override:
        build_command.extend(["--target", target_override])
    if platform_override:
        build_command.extend(["--platform", platform_override])
    print(f"Building library at '{library_directory}' for profile '{target_profile}'...")
    subprocess_result = subprocess.run(
        build_command,
        env=environment_variables,
        check=False,
    )
    if subprocess_result.returncode != 0:
        raise RuntimeError(
            f"Failed to build library at '{library_directory}' with exit code {subprocess_result.returncode}"
        )


def is_cross_target_triple(target_triple: str | None) -> bool:
    """Returns True when target_triple explicitly targets a platform other than the current host."""
    if not target_triple:
        return False
    cleaned = target_triple.strip().lower()
    if not cleaned or cleaned == "host":
        return False
    if "," in cleaned or cleaned in ("all", "linux", "windows", "macos", "apple", "darwin"):
        return True

    sys_name = platform.system().lower()
    host_os = "macos" if sys_name == "darwin" else ("windows" if sys_name.startswith("win") else "linux")
    mach = platform.machine().lower()
    host_arch = "aarch64" if mach in ("arm64", "aarch64") else ("x86_64" if mach in ("x86_64", "amd64", "x64") else mach)
    host_id = f"{host_os}-{host_arch}"

    target_os = "macos" if ("apple-darwin" in cleaned or cleaned.startswith("macos")) else ("windows" if "windows" in cleaned else ("linux" if "linux" in cleaned else ""))
    target_arch = "aarch64" if ("aarch64" in cleaned or "arm64" in cleaned) else ("x86_64" if "x86_64" in cleaned else "")
    if not target_os or not target_arch:
        return False
    return f"{target_os}-{target_arch}" != host_id


def install_library_archive(
    solix_executable: list[str],
    archive_path: Path,
    target_profile: str,
    environment_variables: dict[str, str],
) -> None:
    """Installs an archive into the local package store so dependent libraries can resolve it."""
    install_command = [
        *solix_executable,
        "install",
        str(archive_path),
        "--profile",
        target_profile,
        "--force",
    ]
    print(f"Installing archive '{archive_path.name}' into local store ({target_profile})...")
    subprocess_result = subprocess.run(
        install_command,
        env=environment_variables,
        check=False,
    )
    if subprocess_result.returncode != 0:
        raise RuntimeError(
            f"Failed to install archive '{archive_path}' with exit code {subprocess_result.returncode}"
        )


def main() -> None:
    workspace_root = Path(__file__).resolve().parent.parent.parent
    default_libraries_dir = workspace_root / "libraries"
    default_output_dir = workspace_root / "crates" / "solix-toolchain" / "assets" / "packages"

    parser = argparse.ArgumentParser(description="Build and stage default Solix first-party libraries.")
    parser.add_argument(
        "--libraries-dir",
        type=Path,
        default=default_libraries_dir,
        help="Path to directory containing first-party libraries (default: libraries/)",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=default_output_dir,
        help="Destination directory for bundled packages (default: crates/solix-toolchain/assets/packages)",
    )
    parser.add_argument(
        "--profiles",
        type=str,
        default="release",
        help="Comma-separated target profiles to build (e.g. 'release' or 'release,debug')",
    )
    parser.add_argument(
        "--install",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Install host-compatible built packages into local package store (default: true)",
    )
    parser.add_argument(
        "--solix-binary",
        type=str,
        default=None,
        help="Path or command to invoke solix CLI",
    )
    parser.add_argument(
        "--target",
        type=str,
        default=None,
        help="Optional target triple (or comma-separated list) or 'host' forwarded to solix build",
    )
    parser.add_argument(
        "--platform",
        type=str,
        default=None,
        help="Optional platform filter (e.g. 'linux,windows' or 'macos') forwarded to solix build",
    )

    arguments = parser.parse_args()

    libraries_directory = arguments.libraries_dir.resolve()
    output_directory = arguments.output_dir.resolve()
    target_profiles = [profile_name.strip() for profile_name in arguments.profiles.split(",") if profile_name.strip()]

    if arguments.solix_binary:
        solix_executable = [arguments.solix_binary]
    elif os.environ.get("SOLIX_BIN"):
        solix_executable = [os.environ["SOLIX_BIN"]]
    else:
        # Check if cargo run is preferred or local target binary exists (including .exe on Windows)
        exe_suffix = ".exe" if os.name == "nt" else ""
        debug_binary = workspace_root / "target" / "debug" / f"solix{exe_suffix}"
        release_binary = workspace_root / "target" / "release" / f"solix{exe_suffix}"
        if debug_binary.exists():
            solix_executable = [str(debug_binary)]
        elif release_binary.exists():
            solix_executable = [str(release_binary)]
        else:
            solix_executable = ["cargo", "run", "--bin", "solix", "--"]

    environment_variables = os.environ.copy()

    print(f"Discovering libraries in '{libraries_directory}'...")
    libraries_map = discover_libraries(libraries_directory)
    if not libraries_map:
        print(f"No libraries with manifest.yaml or manifest.toml found in '{libraries_directory}'.")
        sys.exit(0)

    sorted_library_names = topological_sort(libraries_map)
    print(f"Topological build order: {' -> '.join(sorted_library_names)}")

    output_directory.mkdir(parents=True, exist_ok=True)
    expected_folders: set[str] = set()
    manifest_records: list[dict[str, Any]] = []

    for library_name in sorted_library_names:
        library_info = libraries_map[library_name]
        library_dir = library_info["directory"]
        package_version = library_info["version"]

        for target_profile in target_profiles:
            build_library(
                solix_executable,
                library_dir,
                target_profile,
                environment_variables,
                target_override=arguments.target,
                platform_override=arguments.platform,
            )

            candidate_archive = library_dir / "build" / target_profile / f"{library_name}.slxarc"
            if not candidate_archive.exists():
                candidate_archive = library_dir / "build" / f"{library_name}.slxarc"

            if not candidate_archive.exists():
                raise FileNotFoundError(
                    f"Expected build archive not found at '{candidate_archive}' for library '{library_name}'"
                )

            # Stage archive into output_dir/<name>-<profile>@<version>/<name>.slxarc
            staged_folder_name = f"{library_name}-{target_profile}@{package_version}"
            expected_folders.add(staged_folder_name)
            staged_directory = output_directory / staged_folder_name
            staged_directory.mkdir(parents=True, exist_ok=True)
            staged_archive_path = staged_directory / f"{library_name}.slxarc"

            shutil.copy2(candidate_archive, staged_archive_path)

            file_digest = compute_sha256(staged_archive_path)
            file_size_bytes = staged_archive_path.stat().st_size

            relative_archive_path = f"{staged_folder_name}/{library_name}.slxarc"
            manifest_records.append({
                "name": library_name,
                "version": package_version,
                "profile": target_profile,
                "archive_rel_path": relative_archive_path,
                "sha256": file_digest,
                "size_bytes": file_size_bytes,
            })

            print(f"Staged '{relative_archive_path}' ({file_size_bytes} bytes, SHA: {file_digest[:12]}...)")

            effective_target = arguments.target or environment_variables.get("SOLIX_TARGET_TRIPLE")
            should_skip_install = bool(arguments.platform) or is_cross_target_triple(effective_target)
            if arguments.install and not should_skip_install:
                install_library_archive(solix_executable, staged_archive_path, target_profile, environment_variables)
            elif arguments.install:
                print(f"Skipping host package store install for cross/multi-target '{arguments.platform or effective_target}'.")

    for existing_entry in output_directory.iterdir():
        if existing_entry.is_dir() and existing_entry.name not in expected_folders:
            shutil.rmtree(existing_entry)
            print(f"Removed unrequested stale bundled directory '{existing_entry.name}'")

    manifest_file_path = output_directory / "bundled_manifest.json"
    bundled_manifest_payload = {
        "schema_version": 1,
        "packages": manifest_records,
    }
    with open(manifest_file_path, "w", encoding="utf-8") as manifest_output:
        json.dump(bundled_manifest_payload, manifest_output, indent=2)

    print(f"\nWrote bundled manifest index to '{manifest_file_path}'")
    print("\nSummary of Bundled Default Libraries:")
    print("-" * 75)
    print(f"{'PACKAGE':<18} {'VERSION':<10} {'PROFILE':<10} {'SIZE':<12} SHA256")
    print("-" * 75)
    for record in manifest_records:
        size_kb = record["size_bytes"] / 1024.0
        print(f"{record['name']:<18} {record['version']:<10} {record['profile']:<10} {size_kb:.1f} KB    {record['sha256'][:16]}...")
    print("-" * 75)


if __name__ == "__main__":
    main()
