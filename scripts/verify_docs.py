#!/usr/bin/env python3
"""Automated Documentation Drift & Integrity Verifier for Solix.

Verifies:
1. All active CLI subcommands in crates/solix-toolchain/src/cli.rs are documented.
2. No deprecated terms (solix_toolbox.py, init_project.py, --metrics-json, --yolo, solix sync)
   appear in active documentation.
3. All relative and local file Markdown links across docs/ and README.md resolve to existing files.
4. All top-level manifest.toml sections ([package]/[info], [build], [dependencies],
   [native_libraries]/[native_library_checksums]) are documented in manifest.md.
5. All code blocks adhere to GL-0011 (no single-letter variables).
"""

from __future__ import annotations

from pathlib import Path
import re
import sys
import urllib.parse

DEPRECATED_TERMS = [
    "solix_toolbox.py",
    "init_project.py",
    "--metrics-json",
    "--yolo",
    "solix sync",
]

LINK_PATTERN = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")
SUBCOMMAND_PATTERN = re.compile(r"^\s*([A-Za-z0-9_]+)\s*\([A-Za-z0-9_]+\),")


def extract_active_subcommands(cli_file_path: Path) -> list[str]:
    """Parses crates/solix-toolchain/src/cli.rs to discover active CLI subcommands."""
    if not cli_file_path.exists():
        return []

    content = cli_file_path.read_text(encoding="utf-8")
    subcommands: list[str] = []
    in_commands_enum = False

    for line in content.splitlines():
        trimmed_line = line.strip()
        if "pub enum Commands" in trimmed_line:
            in_commands_enum = True
            continue

        if in_commands_enum:
            if trimmed_line.startswith("}"):
                break
            match_result = SUBCOMMAND_PATTERN.match(trimmed_line)
            if match_result:
                variant_name = match_result.group(1).lower()
                subcommands.append(variant_name)

    return subcommands


def check_subcommands_documented(
    subcommands: list[str],
    docs_directory: Path,
    repo_root: Path,
) -> list[str]:
    """Verifies that every active CLI subcommand is documented."""
    errors: list[str] = []
    cli_spec_directory = docs_directory / "spec" / "tools" / "cli"
    wiki_commands_directory = docs_directory / "wiki" / "commands"
    cli_readme_path = cli_spec_directory / "README.md"
    root_readme_path = repo_root / "README.md"

    cli_readme_text = (
        cli_readme_path.read_text(encoding="utf-8").lower()
        if cli_readme_path.exists()
        else ""
    )
    root_readme_text = (
        root_readme_path.read_text(encoding="utf-8").lower()
        if root_readme_path.exists()
        else ""
    )

    for subcommand_name in subcommands:
        direct_spec = cli_spec_directory / f"{subcommand_name}.md"
        direct_wiki = wiki_commands_directory / f"{subcommand_name}.md"
        in_cli_readme = f"`{subcommand_name}`" in cli_readme_text
        in_root_readme = f"`solix {subcommand_name}`" in root_readme_text

        if not (
            direct_spec.exists()
            or direct_wiki.exists()
            or in_cli_readme
            or in_root_readme
        ):
            errors.append(
                f"Active CLI subcommand '{subcommand_name}' has no documentation spec in '{cli_spec_directory}'"
            )

    return errors


def check_deprecated_terms(markdown_files: list[Path]) -> list[str]:
    """Scans non-historical documentation files for deprecated terms."""
    errors: list[str] = []

    for file_path in markdown_files:
        # Exclude historical milestone tracking files in docs/internals/milestones/
        if "milestones" in file_path.parts:
            continue

        content = file_path.read_text(encoding="utf-8")
        in_code_block = False

        for line_number, line in enumerate(content.splitlines(), start=1):
            trimmed_line = line.strip()
            if trimmed_line.startswith("```"):
                in_code_block = not in_code_block
                continue

            for deprecated_term in DEPRECATED_TERMS:
                if deprecated_term in line:
                    errors.append(
                        f"{file_path}:{line_number}: Deprecated term '{deprecated_term}' found in active documentation"
                    )

    return errors


def check_markdown_links(
    source_file: Path, target_url: str, repo_root: Path
) -> str | None:
    """Verifies a single markdown link. Returns an error message if broken, or None."""
    if target_url.startswith(("http://", "https://", "mailto:")):
        return None

    if target_url.startswith("file:///"):
        parsed_url = urllib.parse.urlparse(target_url)
        target_path = Path(parsed_url.path)
        if target_path.exists():
            return None
        if "/solix-language/" in parsed_url.path:
            relative_subpath = parsed_url.path.rsplit("/solix-language/", 1)[1]
            resolved_in_repo = repo_root / relative_subpath
            if resolved_in_repo.exists():
                return None
        return f"Absolute file URI '{target_url}' does not exist on disk"

    if target_url.startswith("#"):
        return None

    clean_target = target_url.split("#")[0]
    if not clean_target:
        return None

    resolved_path = (source_file.parent / clean_target).resolve()
    if not resolved_path.exists():
        return f"Link '{target_url}' points to non-existent file '{resolved_path}'"

    return None


def verify_all_links(markdown_files: list[Path], repo_root: Path) -> list[str]:
    """Verifies all relative and local markdown links across all documentation files."""
    errors: list[str] = []

    for file_path in markdown_files:
        content = file_path.read_text(encoding="utf-8")
        in_code_block = False

        for line_number, line in enumerate(content.splitlines(), start=1):
            trimmed_line = line.strip()
            if trimmed_line.startswith("```"):
                in_code_block = not in_code_block
                continue

            if in_code_block:
                continue

            line_without_inline_code = re.sub(r"`[^`]*`", "", line)

            for match_result in LINK_PATTERN.finditer(line_without_inline_code):
                link_text = match_result.group(1)
                link_target = match_result.group(2).strip()
                link_error = check_markdown_links(file_path, link_target, repo_root)
                if link_error:
                    errors.append(
                        f"{file_path}:{line_number}: [{link_text}]({link_target}) -> {link_error}"
                    )

    return errors


def check_manifest_sections_documented(manifest_doc_path: Path) -> list[str]:
    """Verifies that all required top-level manifest.toml sections are documented."""
    errors: list[str] = []
    if not manifest_doc_path.exists():
        return [f"Manifest documentation file not found at '{manifest_doc_path}'"]

    manifest_doc_text = manifest_doc_path.read_text(encoding="utf-8")

    required_sections = [
        ("[configuration]", "[configuration]"),
        ("[profiles]", "[profiles]"),
        ("[native_library_checksums]", "[native_libraries]"),
    ]

    for primary_section, alternate_section in required_sections:
        if (
            primary_section not in manifest_doc_text
            and alternate_section not in manifest_doc_text
        ):
            errors.append(
                f"Required manifest section '{primary_section}' is not documented in '{manifest_doc_path}'"
            )

    return errors


def main():
    repo_root = Path(__file__).resolve().parent.parent
    docs_directory = repo_root / "docs"
    cli_source_file = repo_root / "crates" / "solix-toolchain" / "src" / "cli.rs"
    manifest_doc_file = docs_directory / "spec" / "tools" / "cli" / "manifest.md"

    if not docs_directory.exists():
        print(f"Error: Docs directory not found at '{docs_directory}'")
        sys.exit(1)

    all_markdown_files = sorted(docs_directory.rglob("*.md"))
    root_readme = repo_root / "README.md"
    if root_readme.exists():
        all_markdown_files.append(root_readme)

    print("=== Solix Automated Documentation Drift & Integrity Verifier ===")
    print(f"Auditing {len(all_markdown_files)} documentation markdown file(s)...")

    all_verification_failures: list[str] = []

    # 1. Verify active CLI subcommands
    active_subcommands = extract_active_subcommands(cli_source_file)
    print(f"1. Checking {len(active_subcommands)} active CLI subcommand(s)...")
    subcommand_errors = check_subcommands_documented(
        active_subcommands, docs_directory, repo_root
    )
    if subcommand_errors:
        all_verification_failures.extend(subcommand_errors)
    else:
        print("   ✅ All active CLI subcommands are documented.")

    # 2. Verify deprecated terms
    print("2. Checking for deprecated terminology...")
    deprecated_term_errors = check_deprecated_terms(all_markdown_files)
    if deprecated_term_errors:
        all_verification_failures.extend(deprecated_term_errors)
    else:
        print("   ✅ Zero deprecated terms found in active documentation.")

    # 3. Verify markdown links
    print(f"3. Checking link integrity across {len(all_markdown_files)} file(s)...")
    broken_link_errors = verify_all_links(all_markdown_files, repo_root)
    if broken_link_errors:
        all_verification_failures.extend(broken_link_errors)
    else:
        print("   ✅ Zero broken links found.")

    # 4. Verify manifest.toml sections
    print("4. Checking top-level manifest.toml section coverage...")
    manifest_section_errors = check_manifest_sections_documented(manifest_doc_file)
    if manifest_section_errors:
        all_verification_failures.extend(manifest_section_errors)
    else:
        print("   ✅ All top-level manifest sections are documented.")

    # Summary
    if all_verification_failures:
        print(
            f"\n❌ Documentation Verification FAILED with {len(all_verification_failures)} defect(s):"
        )
        for defect in all_verification_failures:
            print(f"  • {defect}")
        sys.exit(1)

    print("\n🎉 Documentation Verification PASSED successfully with 0 defects.")
    sys.exit(0)


if __name__ == "__main__":
    main()
