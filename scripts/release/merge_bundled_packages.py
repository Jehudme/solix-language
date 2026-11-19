#!/usr/bin/env python3
"""Merge per-platform bundled `.slxarc` archives into universal cross-platform packages.

Given two or more input directories formatted as `<name>-<profile>@<version>/<name>.slxarc`,
this utility combines all target-qualified native libraries (`lib/<target_triple>/*`) and
recomputes a unified `[native_library_checksums]` table in each archive's `manifest.toml`,
emitting cross-platform `.slxarc` containers and an updated `bundled_manifest.json`.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import sys
import tempfile
import tomllib
import zipfile
from typing import Any


NATIVE_EXTENSIONS = (".so", ".dll", ".dylib")


def compute_sha256_bytes(payload: bytes) -> str:
    """Computes lowercase hexadecimal SHA-256 digest of `payload`."""
    return hashlib.sha256(payload).hexdigest()


TRIPLE_TO_PLATFORM = {
    "x86_64-unknown-linux-gnu": "linux-x86_64",
    "aarch64-unknown-linux-gnu": "linux-aarch64",
    "x86_64-pc-windows-msvc": "windows-x86_64",
    "x86_64-pc-windows-gnu": "windows-x86_64",
    "x86_64-apple-darwin": "macos-x86_64",
    "aarch64-apple-darwin": "macos-aarch64",
}


def strip_native_checksum_section(manifest_text: str) -> str:
    """Removes any existing `[native_library_checksums]` table and `supported_platforms` line."""
    output_lines: list[str] = []
    skipping_checksum_table = False

    for raw_line in manifest_text.splitlines():
        stripped = raw_line.strip()
        if stripped.startswith("supported_platforms") and not skipping_checksum_table:
            continue
        if stripped.startswith("[") and not stripped.startswith("[["):
            if stripped == "[native_library_checksums]":
                skipping_checksum_table = True
                continue
            skipping_checksum_table = False
        elif stripped.startswith("[["):
            skipping_checksum_table = False

        if not skipping_checksum_table:
            output_lines.append(raw_line)

    cleaned = "\n".join(output_lines).rstrip()
    return f"{cleaned}\n" if cleaned else ""


def strip_yaml_native_checksum_section(manifest_text: str) -> str:
    """Removes any existing top-level `supported_platforms:` or `native_library_checksums:` YAML blocks."""
    output_lines: list[str] = []
    skipping_block = False

    for raw_line in manifest_text.splitlines():
        is_top_level = bool(raw_line) and not raw_line[0].isspace() and not raw_line.startswith("#")
        if is_top_level:
            stripped = raw_line.strip()
            if stripped.startswith("supported_platforms:") or stripped.startswith("native_library_checksums:"):
                skipping_block = True
                continue
            skipping_block = False

        if not skipping_block:
            output_lines.append(raw_line)

    cleaned = "\n".join(output_lines).rstrip()
    return f"{cleaned}\n" if cleaned else ""


def parse_folder_identity(
    folder_name: str,
    manifest_text: str,
    manifest_filename: str = "manifest.toml",
) -> tuple[str, str, str]:
    """Extracts `(package_name, build_profile, version)` from folder naming + manifest."""
    package_name = ""
    build_profile = "release"
    package_version = ""

    if "@" in folder_name:
        left_part, package_version = folder_name.split("@", 1)
        for candidate_profile in ("release", "debug", "test"):
            suffix = f"-{candidate_profile}"
            if left_part.endswith(suffix):
                package_name = left_part[: -len(suffix)]
                build_profile = candidate_profile
                break
        if not package_name:
            package_name = left_part

    if manifest_text:
        if manifest_filename.endswith((".yaml", ".yml")):
            in_configuration = False
            for raw_line in manifest_text.splitlines():
                stripped = raw_line.strip()
                if not stripped or stripped.startswith("#"):
                    continue
                if not raw_line[0].isspace():
                    in_configuration = stripped.startswith("configuration:")
                    continue
                if in_configuration and ":" in stripped:
                    key, raw_val = stripped.split(":", 1)
                    cleaned_val = raw_val.strip().strip("'\"")
                    if key.strip() == "name" and cleaned_val:
                        package_name = cleaned_val
                    elif key.strip() == "version" and cleaned_val:
                        package_version = cleaned_val
        else:
            try:
                manifest_data = tomllib.loads(manifest_text)
                config = manifest_data.get("configuration", {})
                package_name = str(config.get("name", package_name) or package_name)
                package_version = str(config.get("version", package_version) or package_version)
            except tomllib.TOMLDecodeError:
                pass

    return package_name, build_profile, package_version


def infer_triple_from_archive_path(source_archive: Path) -> str | None:
    """Infers target triple from a parent directory named `bundled-packages-<triple>`."""
    for parent in source_archive.parents:
        if parent.name.startswith("bundled-packages-"):
            candidate = parent.name[len("bundled-packages-"):].strip()
            if candidate and candidate != "universal":
                return candidate
    return None


def merge_package_group(
    archive_paths: list[Path],
    destination_archive: Path,
    required_targets: list[str],
) -> dict[str, Any]:
    """Merges matching `.slxarc` archives into `destination_archive` and returns manifest metadata."""
    base_manifest_text = ""
    manifest_filename = "manifest.yaml"
    static_entries: dict[str, bytes] = {}
    qualified_natives: dict[str, bytes] = {}
    unqualified_natives: dict[str, bytes] = {}

    for source_archive in archive_paths:
        archive_qualified: dict[str, bytes] = {}
        archive_unqualified: dict[str, bytes] = {}

        with zipfile.ZipFile(source_archive, "r") as zip_reader:
            for entry_info in zip_reader.infolist():
                if entry_info.is_dir():
                    continue
                entry_name = entry_info.filename.replace("\\", "/")
                payload = zip_reader.read(entry_info.filename)

                if entry_name in ("manifest.yaml", "manifest.yml", "manifest.toml"):
                    if not base_manifest_text:
                        base_manifest_text = payload.decode("utf-8")
                        manifest_filename = entry_name
                    continue

                if entry_name.startswith("lib/") and entry_name.endswith(NATIVE_EXTENSIONS):
                    relative_lib_path = entry_name[len("lib/"):]
                    if "/" in relative_lib_path:
                        archive_qualified[entry_name] = payload
                    else:
                        archive_unqualified[entry_name] = payload
                    continue

                static_entries.setdefault(entry_name, payload)

        for entry_name, payload in archive_qualified.items():
            qualified_natives.setdefault(entry_name, payload)

        if archive_unqualified:
            inferred_triple = infer_triple_from_archive_path(source_archive)
            if not archive_qualified and inferred_triple:
                for entry_name, payload in archive_unqualified.items():
                    file_name = Path(entry_name).name
                    qualified_natives.setdefault(f"lib/{inferred_triple}/{file_name}", payload)
            else:
                for entry_name, payload in archive_unqualified.items():
                    unqualified_natives.setdefault(entry_name, payload)

    if not base_manifest_text:
        raise ValueError(f"Cannot merge archives without manifest.yaml or manifest.toml: {archive_paths}")

    qualified_basenames = {Path(entry_name).name for entry_name in qualified_natives}
    retained_natives: dict[str, bytes] = dict(qualified_natives)
    for unqualified_name, unqualified_payload in unqualified_natives.items():
        if Path(unqualified_name).name not in qualified_basenames:
            retained_natives[unqualified_name] = unqualified_payload

    present_targets = {
        entry_name.split("/")[1]
        for entry_name in retained_natives
        if entry_name.startswith("lib/") and entry_name.count("/") >= 2
    }
    missing_required = [target for target in required_targets if target not in present_targets]
    if missing_required:
        raise RuntimeError(
            f"Merged archive '{destination_archive.name}' is missing required targets: "
            f"{', '.join(missing_required)} (present: {', '.join(sorted(present_targets))})"
        )

    native_checksums = {
        entry_name: compute_sha256_bytes(payload)
        for entry_name, payload in sorted(retained_natives.items())
    }

    merged_platforms = sorted({
        TRIPLE_TO_PLATFORM[target_name]
        for target_name in present_targets
        if target_name in TRIPLE_TO_PLATFORM
    }) or ["universal"]

    if manifest_filename.endswith((".yaml", ".yml")):
        cleaned_manifest = strip_yaml_native_checksum_section(base_manifest_text)
        platform_lines = "\n".join(f"  - {plat}" for plat in merged_platforms)
        sections = [cleaned_manifest.rstrip(), f"supported_platforms:\n{platform_lines}"]
        if native_checksums:
            checksum_lines = "\n".join(
                f'  "{key}": "{digest}"'
                for key, digest in sorted(native_checksums.items())
            )
            sections.append(f"native_library_checksums:\n{checksum_lines}")
        unified_manifest = "\n".join(part for part in sections if part) + "\n"
    else:
        cleaned_manifest = strip_native_checksum_section(base_manifest_text)
        formatted_platforms = ", ".join(f'"{plat}"' for plat in merged_platforms)
        header_line = f"supported_platforms = [{formatted_platforms}]\n\n"
        if native_checksums:
            checksum_lines = [
                f'"{key}" = "{digest}"'
                for key, digest in sorted(native_checksums.items())
            ]
            unified_manifest = (
                header_line
                + f"{cleaned_manifest}\n[native_library_checksums]\n"
                + "\n".join(checksum_lines)
                + "\n"
            )
        else:
            unified_manifest = header_line + cleaned_manifest

    all_entries: dict[str, bytes] = {
        manifest_filename: unified_manifest.encode("utf-8"),
        **static_entries,
        **retained_natives,
    }

    destination_archive.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(
        dir=destination_archive.parent,
        prefix=".merge_",
        suffix=".slxarc",
        delete=False,
    ) as temp_file:
        temp_archive_path = Path(temp_file.name)

    try:
        with zipfile.ZipFile(temp_archive_path, "w", compression=zipfile.ZIP_DEFLATED) as zip_writer:
            for entry_name in sorted(all_entries.keys()):
                zip_info = zipfile.ZipInfo(entry_name, date_time=(2026, 1, 1, 0, 0, 0))
                zip_info.compress_type = zipfile.ZIP_DEFLATED
                if entry_name.endswith(NATIVE_EXTENSIONS):
                    zip_info.external_attr = (0o755 & 0xFFFF) << 16
                else:
                    zip_info.external_attr = (0o644 & 0xFFFF) << 16
                zip_writer.writestr(zip_info, all_entries[entry_name])
        shutil.move(str(temp_archive_path), str(destination_archive))
    finally:
        if temp_archive_path.exists():
            temp_archive_path.unlink(missing_ok=True)

    folder_name = destination_archive.parent.name
    package_name, build_profile, package_version = parse_folder_identity(
        folder_name, unified_manifest, manifest_filename
    )
    archive_bytes = destination_archive.read_bytes()

    return {
        "name": package_name,
        "version": package_version,
        "profile": build_profile,
        "archive_rel_path": f"{folder_name}/{destination_archive.name}",
        "sha256": compute_sha256_bytes(archive_bytes),
        "size_bytes": len(archive_bytes),
        "targets": sorted(present_targets),
    }


def merge_bundled_directories(
    input_directories: list[Path],
    output_directory: Path,
    required_targets: list[str] | None = None,
    allowed_profiles: set[str] | None = None,
) -> dict[str, Any]:
    """Discovers and merges all per-platform `.slxarc` archives across `input_directories`."""
    effective_profiles = allowed_profiles if allowed_profiles is not None else {"all"}
    grouped_archives: dict[tuple[str, str], list[Path]] = {}
    for input_dir in input_directories:
        resolved_input = input_dir.resolve()
        if not resolved_input.is_dir():
            continue
        for folder_entry in sorted(resolved_input.iterdir()):
            if not folder_entry.is_dir():
                continue
            for file_entry in sorted(folder_entry.iterdir()):
                if file_entry.is_file() and file_entry.suffix.lower() == ".slxarc":
                    _pkg_name, build_profile, _pkg_ver = parse_folder_identity(folder_entry.name, "")
                    if "all" not in effective_profiles and build_profile not in effective_profiles:
                        continue
                    key = (folder_entry.name, file_entry.name)
                    grouped_archives.setdefault(key, []).append(file_entry)

    if not grouped_archives:
        raise RuntimeError(
            f"No '.slxarc' packages found in input directories: {[str(item) for item in input_directories]}"
        )

    required_target_list = required_targets or []
    output_directory.mkdir(parents=True, exist_ok=True)

    manifest_records: list[dict[str, Any]] = []
    for (folder_name, archive_filename), source_list in sorted(grouped_archives.items()):
        destination_archive = output_directory / folder_name / archive_filename
        record = merge_package_group(
            source_list,
            destination_archive,
            required_target_list,
        )
        targets_display = ", ".join(record.pop("targets")) or "(pure bytecode)"
        print(
            f"Merged '{record['archive_rel_path']}' "
            f"({record['size_bytes']} bytes, targets: {targets_display})"
        )
        manifest_records.append(record)

    bundled_manifest = {
        "schema_version": 1,
        "packages": manifest_records,
    }
    manifest_output_path = output_directory / "bundled_manifest.json"
    manifest_output_path.write_text(json.dumps(bundled_manifest, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote universal bundled manifest to '{manifest_output_path}'")
    return bundled_manifest


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Merge per-platform bundled .slxarc packages into universal cross-platform packages."
    )
    parser.add_argument(
        "--input-dirs",
        nargs="+",
        required=True,
        type=Path,
        help="Input bundled packages directories (one or more platform package roots).",
    )
    parser.add_argument(
        "--output-dir",
        required=True,
        type=Path,
        help="Destination directory for merged universal .slxarc packages and bundled_manifest.json.",
    )
    parser.add_argument(
        "--require-targets",
        type=str,
        default="",
        help="Comma-separated list of target triples that must be present in each merged archive.",
    )
    parser.add_argument(
        "--profiles",
        type=str,
        default="all",
        help="Comma-separated profiles to merge (default: all, or e.g. 'release').",
    )
    return parser.parse_args()


def main() -> None:
    arguments = parse_args()
    required_targets = [
        item.strip()
        for item in arguments.require_targets.split(",")
        if item.strip()
    ]
    allowed_profiles = {
        item.strip()
        for item in arguments.profiles.split(",")
        if item.strip()
    } or {"all"}
    try:
        merge_bundled_directories(
            input_directories=arguments.input_dirs,
            output_directory=arguments.output_dir.resolve(),
            required_targets=required_targets,
            allowed_profiles=allowed_profiles,
        )
    except Exception as error_details:
        print(f"Error: {error_details}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
