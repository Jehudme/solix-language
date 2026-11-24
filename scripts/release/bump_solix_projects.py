#!/usr/bin/env python3
"""Automated Semantic Version Manager for Solix Native Packages and Example Projects.

Analyzes Conventional Commits affecting `libraries/` and `examples/`, computes independent
SemVer bumps (`major`, `minor`, `patch`), updates `manifest.yaml` version strings, cascades
dependency version updates to consumer projects, and maintains per-project CHANGELOG.md files.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import datetime as datetime_module
import json
import os
from pathlib import Path
import re
import subprocess
import sys
from typing import Any, Dict, List, Optional, Tuple


SEMVER_PATTERN = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")
CONVENTIONAL_HEADER_PATTERN = re.compile(r"^([a-zA-Z]+)(?:\(([^)]+)\))?(!)?:\s*(.+)$")
MANIFEST_VERSION_PATTERN = re.compile(
    r'(version:\s*["\'])([^"\']+)(["\'])',
    re.MULTILINE,
)


@dataclass(frozen=True)
class SolixProjectMetadata:
    """Represents a discovered Solix library or example project package."""

    project_name: str
    project_directory: Path
    manifest_path: Path
    current_version: str
    is_library: bool


@dataclass(frozen=True)
class ParsedCommitRecord:
    """Structured representation of a Git commit record."""

    commit_hash: str
    commit_subject: str
    commit_type: str
    commit_scope: str
    is_breaking_change: bool
    commit_description: str


def parse_semver_components(version_string: str) -> Tuple[int, int, int]:
    """Parses an `X.Y.Z` SemVer string into integer tuple components."""
    matched_pattern = SEMVER_PATTERN.match(version_string.strip().lstrip("v"))
    if not matched_pattern:
        raise ValueError(f"Unsupported package version format: '{version_string}'")
    return (
        int(matched_pattern.group(1)),
        int(matched_pattern.group(2)),
        int(matched_pattern.group(3)),
    )


def increment_semver_version(current_version_string: str, bump_kind: str) -> str:
    """Applies `major`, `minor`, or `patch` increment to an existing version string."""
    major_number, minor_number, patch_number = parse_semver_components(current_version_string)
    if bump_kind == "major":
        return f"{major_number + 1}.0.0"
    if bump_kind == "minor":
        return f"{major_number}.{minor_number + 1}.0"
    if bump_kind == "patch":
        return f"{major_number}.{minor_number}.{patch_number + 1}"
    raise ValueError(f"Unsupported bump kind requested: '{bump_kind}'")


def discover_solix_projects(repository_root_directory: Path) -> List[SolixProjectMetadata]:
    """Discovers all Solix library and example projects with manifest.yaml files."""
    discovered_projects: List[SolixProjectMetadata] = []

    search_roots = [
        (repository_root_directory / "libraries", True),
        (repository_root_directory / "examples", False),
    ]

    for search_root_path, is_library_flag in search_roots:
        if not search_root_path.is_dir():
            continue
        for candidate_directory in sorted(search_root_path.iterdir()):
            manifest_candidate_path = candidate_directory / "manifest.yaml"
            if not manifest_candidate_path.is_file():
                continue

            manifest_content = manifest_candidate_path.read_text(encoding="utf-8")
            version_match = MANIFEST_VERSION_PATTERN.search(manifest_content)
            if not version_match:
                continue

            discovered_version = version_match.group(2)
            project_name = candidate_directory.name

            name_match = re.search(r'name:\s*["\']?([^"\s\n]+)["\']?', manifest_content)
            if name_match:
                project_name = name_match.group(1)

            discovered_projects.append(
                SolixProjectMetadata(
                    project_name=project_name,
                    project_directory=candidate_directory,
                    manifest_path=manifest_candidate_path,
                    current_version=discovered_version,
                    is_library=is_library_flag,
                )
            )

    return discovered_projects


def collect_project_commits(
    repository_root_directory: Path,
    base_reference: str,
    head_reference: str,
    project_relative_path: str,
) -> List[ParsedCommitRecord]:
    """Collects git commits touching a specific project relative path."""
    git_command = [
        "git",
        "-C",
        str(repository_root_directory),
        "log",
        "--no-merges",
        "--format=%H%x1f%s%x1f%b%x1e",
        f"{base_reference}..{head_reference}",
        "--",
        project_relative_path,
    ]

    execution_result = subprocess.run(
        git_command,
        capture_output=True,
        text=True,
        check=False,
    )

    if execution_result.returncode != 0:
        return []

    collected_records: List[ParsedCommitRecord] = []
    raw_output = execution_result.stdout

    for raw_record in raw_output.split("\x1e"):
        cleaned_record = raw_record.strip("\r\n ")
        if not cleaned_record:
            continue
        record_fields = cleaned_record.split("\x1f")
        commit_hash = record_fields[0].strip() if len(record_fields) > 0 else ""
        commit_subject = record_fields[1].strip() if len(record_fields) > 1 else ""
        commit_body = record_fields[2].strip() if len(record_fields) > 2 else ""

        if commit_subject.startswith("chore(release):") or commit_subject.startswith("Merge "):
            continue

        matched_header = CONVENTIONAL_HEADER_PATTERN.match(commit_subject)
        if matched_header:
            commit_type = matched_header.group(1).lower()
            commit_scope = matched_header.group(2) or ""
            is_breaking_change = bool(matched_header.group(3)) or (
                "BREAKING CHANGE:" in commit_body or "BREAKING-CHANGE:" in commit_body
            )
            commit_description = matched_header.group(4).strip()
        else:
            commit_type = "chore"
            commit_scope = ""
            is_breaking_change = "BREAKING CHANGE:" in commit_body or "BREAKING-CHANGE:" in commit_body
            commit_description = commit_subject

        collected_records.append(
            ParsedCommitRecord(
                commit_hash=commit_hash,
                commit_subject=commit_subject,
                commit_type=commit_type,
                commit_scope=commit_scope,
                is_breaking_change=is_breaking_change,
                commit_description=commit_description,
            )
        )

    return collected_records


def determine_project_bump_strategy(commits: List[ParsedCommitRecord]) -> Optional[Tuple[str, str]]:
    """Determines bump kind ('major', 'minor', 'patch') and justification reason."""
    if not commits:
        return None

    if any(commit.is_breaking_change for commit in commits):
        return ("major", "breaking changes detected")

    if any(commit.commit_type == "feat" for commit in commits):
        return ("minor", "new feature additions detected")

    return ("patch", "bug fixes or minor improvements detected")


def render_project_changelog_entry(
    new_version_string: str,
    commits: List[ParsedCommitRecord],
    release_date_string: Optional[str] = None,
) -> str:
    """Formats a Keep-a-Changelog section for a specific Solix project."""
    effective_date = release_date_string or datetime_module.datetime.now(
        datetime_module.timezone.utc
    ).strftime("%Y-%m-%d")

    group_mapping = {
        "feat": "Features",
        "fix": "Bug Fixes",
        "perf": "Performance",
        "docs": "Documentation",
        "refactor": "Refactoring",
        "test": "Testing",
        "build": "Build System",
        "ci": "Continuous Integration",
        "style": "Style",
        "chore": "Miscellaneous Tasks",
    }

    grouped_commits: Dict[str, List[ParsedCommitRecord]] = {}
    for commit_record in commits:
        group_name = group_mapping.get(commit_record.commit_type, "Miscellaneous Tasks")
        grouped_commits.setdefault(group_name, []).append(commit_record)

    output_lines: List[str] = [f"## [{new_version_string}] - {effective_date}", ""]
    for group_name in sorted(grouped_commits.keys()):
        output_lines.append(f"### {group_name}")
        output_lines.append("")
        for commit_item in grouped_commits[group_name]:
            scope_prefix = f"*({commit_item.commit_scope})* " if commit_item.commit_scope else ""
            formatted_description = (
                commit_item.commit_description[:1].upper() + commit_item.commit_description[1:]
                if commit_item.commit_description
                else commit_item.commit_subject
            )
            output_lines.append(f"- {scope_prefix}{formatted_description}")
        output_lines.append("")

    return "\n".join(output_lines).rstrip() + "\n"


def prepend_project_changelog(
    changelog_file_path: Path,
    project_display_name: str,
    new_version_string: str,
    rendered_section_markdown: str,
) -> None:
    """Inserts a new changelog release section at the top of the project's CHANGELOG.md."""
    initial_header = (
        f"# Changelog - {project_display_name}\n\n"
        "All notable changes to this Solix package are documented in this file.\n\n"
    )

    if changelog_file_path.is_file():
        existing_content = changelog_file_path.read_text(encoding="utf-8")
    else:
        existing_content = initial_header

    if f"## [{new_version_string}]" in existing_content:
        return

    marker_phrase = "documented in this file."
    if marker_phrase in existing_content:
        prefix_part, separator, suffix_part = existing_content.partition(marker_phrase)
        cleaned_suffix = suffix_part.lstrip("\r\n")
        updated_content = f"{prefix_part}{marker_phrase}\n\n{rendered_section_markdown}\n{cleaned_suffix}"
    else:
        updated_content = f"{initial_header}{rendered_section_markdown}\n{existing_content.lstrip()}"

    changelog_file_path.write_text(updated_content, encoding="utf-8")


def update_manifest_package_version(manifest_path: Path, new_version_string: str) -> None:
    """Updates the package version declared in manifest.yaml."""
    original_text = manifest_path.read_text(encoding="utf-8")
    updated_text, substitution_count = MANIFEST_VERSION_PATTERN.subn(
        rf"\g<1>{new_version_string}\g<3>",
        original_text,
        count=1,
    )
    if substitution_count != 1:
        raise RuntimeError(f"Failed to update version field in manifest '{manifest_path}'")
    manifest_path.write_text(updated_text, encoding="utf-8")


def cascade_dependency_versions(
    all_projects: List[SolixProjectMetadata],
    updated_library_versions: Dict[str, str],
) -> List[Tuple[str, str, str]]:
    """Cascades newly bumped library versions to dependent project manifests."""
    cascaded_updates: List[Tuple[str, str, str]] = []

    for project in all_projects:
        manifest_text = project.manifest_path.read_text(encoding="utf-8")
        has_modifications = False

        for library_name, next_library_version in updated_library_versions.items():
            dependency_pattern = re.compile(
                rf'({library_name}:\s*(?:[^\{{\n]*?\n)?\s*version:\s*["\'])([^"\']+)(["\'])',
                re.MULTILINE,
            )

            def dependency_replacement(match_object: re.Match[str]) -> str:
                nonlocal has_modifications
                previous_version = match_object.group(2)
                if previous_version != next_library_version:
                    has_modifications = True
                    cascaded_updates.append(
                        (project.project_name, library_name, next_library_version)
                    )
                return f"{match_object.group(1)}{next_library_version}{match_object.group(3)}"

            manifest_text = dependency_pattern.sub(dependency_replacement, manifest_text)

        if has_modifications:
            project.manifest_path.write_text(manifest_text, encoding="utf-8")

    return cascaded_updates


def parse_command_line_arguments(argument_list: Optional[List[str]] = None) -> argparse.Namespace:
    """Parses command-line arguments for the Solix project bump manager."""
    argument_parser = argparse.ArgumentParser(
        description="Inspect and bump Solix library and example package versions automatically."
    )
    argument_parser.add_argument(
        "--base-ref",
        type=str,
        default="origin/dev",
        help="Base git reference to compare commits against",
    )
    argument_parser.add_argument(
        "--head-ref",
        type=str,
        default="HEAD",
        help="Head git reference to compare commits from",
    )
    argument_parser.add_argument(
        "--write",
        action="store_true",
        help="Persist version updates to manifest.yaml and CHANGELOG.md files",
    )
    argument_parser.add_argument(
        "--json-out",
        type=Path,
        default=None,
        help="Optional path to output structured JSON bump telemetry",
    )
    return argument_parser.parse_args(argument_list)


def main(argument_list: Optional[List[str]] = None) -> int:
    """Main CLI execution routine."""
    parsed_arguments = parse_command_line_arguments(argument_list)
    repository_root_directory = Path(__file__).resolve().parent.parent.parent

    discovered_projects = discover_solix_projects(repository_root_directory)
    project_bump_results: List[Dict[str, Any]] = []
    updated_library_versions: Dict[str, str] = {}

    for project in discovered_projects:
        relative_project_path = str(
            project.project_directory.relative_to(repository_root_directory)
        )
        commits_for_project = collect_project_commits(
            repository_root_directory,
            parsed_arguments.base_ref,
            parsed_arguments.head_ref,
            relative_project_path,
        )

        bump_strategy = determine_project_bump_strategy(commits_for_project)
        if bump_strategy is None:
            continue

        bump_kind, bump_reason = bump_strategy
        new_version_string = increment_semver_version(project.current_version, bump_kind)
        changelog_section = render_project_changelog_entry(
            new_version_string, commits_for_project
        )

        project_bump_results.append(
            {
                "project_name": project.project_name,
                "project_path": relative_project_path,
                "current_version": project.current_version,
                "next_version": new_version_string,
                "bump_kind": bump_kind,
                "bump_reason": bump_reason,
                "commit_count": len(commits_for_project),
                "is_library": project.is_library,
            }
        )

        if project.is_library:
            updated_library_versions[project.project_name] = new_version_string

        if parsed_arguments.write:
            update_manifest_package_version(project.manifest_path, new_version_string)
            prepend_project_changelog(
                project.project_directory / "CHANGELOG.md",
                project.project_name,
                new_version_string,
                changelog_section,
            )

    cascaded_dependency_updates: List[Tuple[str, str, str]] = []
    if parsed_arguments.write and updated_library_versions:
        cascaded_dependency_updates = cascade_dependency_versions(
            discovered_projects,
            updated_library_versions,
        )

    execution_payload = {
        "bumped_projects": project_bump_results,
        "cascaded_dependency_updates": [
            {
                "consumer_project": consumer_name,
                "dependency_name": dependency_name,
                "new_version": target_version,
            }
            for consumer_name, dependency_name, target_version in cascaded_dependency_updates
        ],
    }

    if parsed_arguments.json_out is not None:
        parsed_arguments.json_out.parent.mkdir(parents=True, exist_ok=True)
        parsed_arguments.json_out.write_text(
            json.dumps(execution_payload, indent=2) + "\n",
            encoding="utf-8",
        )

    print(
        f"[bump_solix_projects] Evaluated {len(discovered_projects)} project(s); "
        f"bumped {len(project_bump_results)} package(s)."
    )
    for bumped_item in project_bump_results:
        print(
            f"  - {bumped_item['project_name']} ({bumped_item['project_path']}): "
            f"{bumped_item['current_version']} -> {bumped_item['next_version']} "
            f"[{bumped_item['bump_kind']}] ({bumped_item['bump_reason']})"
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
