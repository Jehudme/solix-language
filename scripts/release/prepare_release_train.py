#!/usr/bin/env python3
"""Two-Wave Release Train Version Preview & Bump Preparer (`dev -> master`).

Analyzes Conventional Commits between `master` and `dev`, honors maintainer
SemVer overrides (`release:major`, `release:minor`, `release:patch`, or explicit
`X.Y.Z` overrides), updates `Cargo.toml` / `Cargo.lock` / crate changelogs, and
renders the Release Train PR description for the single `master` release gate.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import datetime as dt
import json
from pathlib import Path
import re
import subprocess
import sys
import tomllib


SEMVER_RE = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")
HEADER_RE = re.compile(r"^([a-zA-Z]+)(?:\(([^)]+)\))?(!)?:\s*(.+)$")
WORKSPACE_PKG_VERSION_RE = re.compile(
    r'(\[workspace\.package\]\s*(?:[^\[]*?\n)?version\s*=\s*")([^"]+)(")',
    re.MULTILINE,
)
INTERNAL_DEP_VERSION_RE = re.compile(
    r'(solix-(?:common|compilation|runtime|lsp|dap)\s*=\s*\{[^}]*version\s*=\s*")([^"]+)(")'
)

CHANGELOG_FILES = (
    "CHANGELOG.md",
    "crates/solix-common/CHANGELOG.md",
    "crates/solix-compilation/CHANGELOG.md",
    "crates/solix-runtime/CHANGELOG.md",
    "ffi/rust/CHANGELOG.md",
    "ffi/c/CHANGELOG.md",
)

CMAKE_FFI_VERSION_RE = re.compile(
    r"(project\(\s*solix_ffi\s+VERSION\s+)(\d+\.\d+\.\d+)(\s+LANGUAGES\b)"
)
HEADER_FFI_VERSION_RE = re.compile(
    r'(#define\s+SOLIX_FFI_VERSION\s+")(\d+\.\d+\.\d+)(")'
)
HEADER_FFI_PART_RE = re.compile(
    r"(#define\s+SOLIX_FFI_VERSION_(MAJOR|MINOR|PATCH)\s+)(\d+)"
)


@dataclass(frozen=True)
class CommitSummary:
    """Structured parsed representation of a Git commit for changelog/bump computation."""

    sha: str
    subject: str
    commit_type: str
    scope: str
    breaking: bool
    description: str


def parse_semver(version_text: str) -> tuple[int, int, int]:
    """Parses `X.Y.Z` into integer tuple."""
    match = SEMVER_RE.match(version_text.strip().lstrip("v"))
    if not match:
        raise ValueError(f"Unsupported workspace version format: '{version_text}'")
    return int(match.group(1)), int(match.group(2)), int(match.group(3))


def bump_semver(current_version: str, bump_kind: str) -> str:
    """Applies `major`, `minor`, or `patch` SemVer bump."""
    major, minor, patch = parse_semver(current_version)
    if bump_kind == "major":
        return f"{major + 1}.0.0"
    if bump_kind == "minor":
        return f"{major}.{minor + 1}.0"
    if bump_kind == "patch":
        return f"{major}.{minor}.{patch + 1}"
    raise ValueError(f"Unsupported bump kind: '{bump_kind}'")


def parse_commits(raw_log: str) -> list[CommitSummary]:
    """Parses `git log --format=%H%x1f%s%x1f%b%x1e` output into CommitSummary records."""
    commits: list[CommitSummary] = []
    for record in raw_log.split("\x1e"):
        cleaned = record.strip("\r\n ")
        if not cleaned:
            continue
        fields = cleaned.split("\x1f")
        sha = fields[0].strip() if len(fields) > 0 else ""
        subject = fields[1].strip() if len(fields) > 1 else ""
        body = fields[2].strip() if len(fields) > 2 else ""

        if subject.startswith("chore(release):") or subject.startswith("Merge "):
            continue

        match = HEADER_RE.match(subject)
        if match:
            commit_type = match.group(1).lower()
            scope = match.group(2) or ""
            is_breaking = bool(match.group(3)) or (
                "BREAKING CHANGE:" in body or "BREAKING-CHANGE:" in body
            )
            description = match.group(4).strip()
        else:
            commit_type = "chore"
            scope = ""
            is_breaking = "BREAKING CHANGE:" in body or "BREAKING-CHANGE:" in body
            description = subject

        commits.append(
            CommitSummary(
                sha=sha,
                subject=subject,
                commit_type=commit_type,
                scope=scope,
                breaking=is_breaking,
                description=description,
            )
        )
    return commits


def determine_bump_kind(
    commits: list[CommitSummary],
    *,
    pr_labels: list[str] | None = None,
    override: str = "auto",
) -> tuple[str, str]:
    """Determines `(bump_or_exact_version, reason)` from override, PR labels, and commits."""
    cleaned_override = override.strip().lower()
    if SEMVER_RE.match(cleaned_override.lstrip("v")):
        return cleaned_override.lstrip("v"), f"explicit version override ({cleaned_override})"
    if cleaned_override in ("major", "minor", "patch"):
        return cleaned_override, f"manual workflow bump override ({cleaned_override})"

    normalized_labels = {label.strip().lower() for label in (pr_labels or []) if label.strip()}
    if "release:major" in normalized_labels:
        return "major", "PR label override `release:major`"
    if "release:minor" in normalized_labels:
        return "minor", "PR label override `release:minor`"
    if "release:patch" in normalized_labels:
        return "patch", "PR label override `release:patch`"

    if any(commit.breaking for commit in commits):
        return "major", "detected breaking change (`!` or `BREAKING CHANGE:`)"
    if any(commit.commit_type == "feat" for commit in commits):
        return "minor", "detected Conventional Commit `feat`"
    return "patch", "detected non-breaking fixes/tooling/docs changes"


def collect_commits(repo_root: Path, base_ref: str, head_ref: str) -> list[CommitSummary]:
    """Collects commits present in `head_ref` that are not yet in `base_ref`."""
    proc = subprocess.run(
        [
            "git",
            "-C",
            str(repo_root),
            "log",
            "--no-merges",
            "--format=%H%x1f%s%x1f%b%x1e",
            f"{base_ref}..{head_ref}",
        ],
        capture_output=True,
        text=True,
        check=True,
    )
    return parse_commits(proc.stdout)


def read_base_version(repo_root: Path, base_ref: str) -> str:
    """Reads workspace version from `Cargo.toml` on `base_ref` (or working tree fallback)."""
    try:
        raw_toml = subprocess.check_output(
            ["git", "-C", str(repo_root), "show", f"{base_ref}:Cargo.toml"],
            text=True,
        )
    except subprocess.CalledProcessError:
        raw_toml = (repo_root / "Cargo.toml").read_text(encoding="utf-8")

    parsed = tomllib.loads(raw_toml)
    version = parsed.get("workspace", {}).get("package", {}).get("version", "")
    if not isinstance(version, str) or not version.strip():
        raise RuntimeError("Could not determine [workspace.package].version from Cargo.toml")
    return version.strip()


def update_cargo_toml_version(cargo_toml_path: Path, new_version: str) -> None:
    """Updates workspace package version and internal workspace dependency versions."""
    original = cargo_toml_path.read_text(encoding="utf-8")
    updated, pkg_subs = WORKSPACE_PKG_VERSION_RE.subn(rf"\g<1>{new_version}\g<3>", original, count=1)
    if pkg_subs != 1:
        raise RuntimeError(f"Failed to update [workspace.package].version in '{cargo_toml_path}'")
    updated = INTERNAL_DEP_VERSION_RE.sub(rf"\g<1>{new_version}\g<3>", updated)
    cargo_toml_path.write_text(updated, encoding="utf-8")


def render_changelog_section(
    new_version: str,
    commits: list[CommitSummary],
    date_str: str | None = None,
) -> str:
    """Renders a Keep-a-Changelog Markdown section matching `cliff.toml` grouping."""
    release_date = date_str or dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%d")
    group_map = {
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
    grouped: dict[str, list[CommitSummary]] = {}
    for commit in commits:
        group_name = group_map.get(commit.commit_type, "Miscellaneous Tasks")
        grouped.setdefault(group_name, []).append(commit)

    lines: list[str] = [f"## [{new_version}] - {release_date}", ""]
    for group_name in sorted(grouped.keys()):
        lines.append(f"### {group_name}")
        lines.append("")
        for commit in grouped[group_name]:
            scope_prefix = f"*({commit.scope})* " if commit.scope else ""
            desc = commit.description[:1].upper() + commit.description[1:] if commit.description else commit.subject
            lines.append(f"- {scope_prefix}{desc}")
        lines.append("")

    if not grouped:
        lines.extend(["### Miscellaneous Tasks", "", f"- Prepare release {new_version}", ""])

    return "\n".join(lines).rstrip() + "\n"


def prepend_changelog_section(changelog_path: Path, new_version: str, section_md: str) -> None:
    """Inserts `section_md` directly below the `# Changelog` header unless version already exists."""
    header_prefix = (
        "# Changelog\n\n"
        "All notable changes to the Solix Programming Language will be documented in this file.\n\n"
    )
    if changelog_path.is_file():
        existing = changelog_path.read_text(encoding="utf-8")
    else:
        existing = header_prefix

    if f"## [{new_version}]" in existing:
        return

    divider = "\n---\n"
    marker = "documented in this file."
    if divider in existing:
        head_part, _, tail_part = existing.partition(divider)
        cleaned_tail = tail_part.lstrip("\r\n")
        updated = f"{head_part}{divider}\n{section_md}\n{cleaned_tail}"
    elif marker in existing:
        head_part, _, tail_part = existing.partition(marker)
        cleaned_tail = tail_part.lstrip("\r\n")
        updated = f"{head_part}{marker}\n\n{section_md}\n{cleaned_tail}"
    else:
        updated = f"{header_prefix}{section_md}\n{existing.lstrip()}"
    changelog_path.write_text(updated, encoding="utf-8")


def update_c_ffi_version(repo_root: Path, new_version: str) -> None:
    """Updates `ffi/c/CMakeLists.txt` and `ffi/c/include/solix_ffi.h` SDK versions."""
    major_val, minor_val, patch_val = parse_semver(new_version)
    cmake_path = repo_root / "ffi" / "c" / "CMakeLists.txt"
    if cmake_path.is_file():
        cmake_text = cmake_path.read_text(encoding="utf-8")
        updated_cmake, subs = CMAKE_FFI_VERSION_RE.subn(
            rf"\g<1>{new_version}\g<3>", cmake_text, count=1
        )
        if subs != 1:
            raise RuntimeError(f"Failed to update solix_ffi VERSION in '{cmake_path}'")
        cmake_path.write_text(updated_cmake, encoding="utf-8")

    header_path = repo_root / "ffi" / "c" / "include" / "solix_ffi.h"
    if header_path.is_file():
        header_text = header_path.read_text(encoding="utf-8")
        parts_map = {
            "MAJOR": str(major_val),
            "MINOR": str(minor_val),
            "PATCH": str(patch_val),
        }

        def replace_part(match_obj: re.Match[str]) -> str:
            return f"{match_obj.group(1)}{parts_map[match_obj.group(2)]}"

        updated_header = HEADER_FFI_PART_RE.sub(replace_part, header_text)
        updated_header, str_subs = HEADER_FFI_VERSION_RE.subn(
            rf"\g<1>{new_version}\g<3>", updated_header, count=1
        )
        if str_subs != 1:
            raise RuntimeError(f"Failed to update SOLIX_FFI_VERSION in '{header_path}'")
        header_path.write_text(updated_header, encoding="utf-8")


def sync_cargo_lock(repo_root: Path) -> None:
    """Refreshes Cargo.lock workspace crate entries cleanly without touching upstream deps."""
    proc = subprocess.run(
        ["cargo", "update", "--workspace", "--offline"],
        cwd=str(repo_root),
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        subprocess.run(
            ["cargo", "metadata", "--format-version", "1"],
            cwd=str(repo_root),
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            text=True,
            check=True,
        )


def format_pr_body(
    *,
    current_version: str,
    next_version: str,
    reason: str,
    commits: list[CommitSummary],
    section_md: str,
) -> str:
    """Formats the Release Train PR description for `@Jehudme` review."""
    return f"""## Solix Release Train: `v{current_version}` -> `v{next_version}`

* **Proposed Version:** `{next_version}` *(reason: {reason})*
* **Queued Commits Ahead of `master`:** `{len(commits)}`
* **Source Gate:** `release-plz/dev-to-master` (tracks verified `dev` head)

### How to Adjust Version Before Merging
Add one of the following labels on this PR (and wait a few seconds for auto-sync), then re-review before merging into `master`:
- `release:patch` — force Patch bump (`x.y.Z+1`)
- `release:minor` — force Minor bump (`x.Y+1.0`)
- `release:major` — force Major bump (`X+1.0.0`)

---

### Changelog Preview

{section_md}
---
*Merging this PR into `master` triggers crate publishing (`crates.io`), Git tag `v{next_version}`, universal multi-arch installers + bundled `solix wiki` docs (`release.yml`), GitHub Wiki mirroring, and automatic `master -> dev` back-sync.*
"""


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    """Parses CLI flags for release train preparation."""
    parser = argparse.ArgumentParser(description="Prepare or preview Two-Wave Release Train PR.")
    parser.add_argument("--base-ref", type=str, default="origin/master", help="Production branch ref")
    parser.add_argument("--head-ref", type=str, default="HEAD", help="Integration branch ref")
    parser.add_argument(
        "--override",
        type=str,
        default="auto",
        help="Bump override: auto, patch, minor, major, or exact X.Y.Z",
    )
    parser.add_argument(
        "--pr-labels",
        type=str,
        default="",
        help="Comma-separated PR labels checked for release:patch/minor/major",
    )
    parser.add_argument("--write", action="store_true", help="Apply version bump and changelog edits")
    parser.add_argument("--no-lock-sync", action="store_true", help="Skip running cargo lockfile sync")
    parser.add_argument("--json-out", type=Path, default=None, help="Optional JSON metadata output")
    parser.add_argument("--pr-body-out", type=Path, default=None, help="Optional Markdown PR body file")
    parser.add_argument(
        "--changelog-out",
        type=Path,
        default=None,
        help="Optional Markdown file containing only the new version's changelog section",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    """CLI entrypoint."""
    args = parse_args(argv)
    repo_root = Path(__file__).resolve().parent.parent.parent

    current_version = read_base_version(repo_root, args.base_ref)
    commits = collect_commits(repo_root, args.base_ref, args.head_ref)
    labels = [item.strip() for item in args.pr_labels.split(",") if item.strip()]
    bump_or_exact, reason = determine_bump_kind(
        commits,
        pr_labels=labels,
        override=args.override,
    )
    next_version = (
        bump_or_exact
        if SEMVER_RE.match(bump_or_exact)
        else bump_semver(current_version, bump_or_exact)
    )

    section_md = render_changelog_section(next_version, commits)
    pr_body = format_pr_body(
        current_version=current_version,
        next_version=next_version,
        reason=reason,
        commits=commits,
        section_md=section_md,
    )

    if args.write and commits:
        update_cargo_toml_version(repo_root / "Cargo.toml", next_version)
        update_c_ffi_version(repo_root, next_version)
        for rel_changelog in CHANGELOG_FILES:
            prepend_changelog_section(repo_root / rel_changelog, next_version, section_md)
        if not args.no_lock_sync:
            sync_cargo_lock(repo_root)

    payload = {
        "has_changes": bool(commits),
        "current_version": current_version,
        "next_version": next_version,
        "reason": reason,
        "commit_count": len(commits),
    }

    if args.json_out is not None:
        args.json_out.parent.mkdir(parents=True, exist_ok=True)
        args.json_out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    if args.pr_body_out is not None:
        args.pr_body_out.parent.mkdir(parents=True, exist_ok=True)
        args.pr_body_out.write_text(pr_body, encoding="utf-8")

    if args.changelog_out is not None:
        args.changelog_out.parent.mkdir(parents=True, exist_ok=True)
        args.changelog_out.write_text(section_md, encoding="utf-8")

    print(
        f"[prepare_release_train] {current_version} -> {next_version} "
        f"({len(commits)} commit(s); {reason})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
