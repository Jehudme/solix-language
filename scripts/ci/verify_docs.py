#!/usr/bin/env python3
"""Automated Documentation Drift & Integrity Verifier for Solix.

Verifies:
1. All active CLI subcommands and long flags in crates/solix-toolchain/src/cli.rs are documented.
2. No deprecated terms or stale FFI/version patterns appear in active documentation.
3. All relative and local file Markdown links across docs/ and README.md resolve to existing files.
4. All top-level manifest.yaml (and legacy manifest.toml) sections are documented in manifest.md.
5. Complete error-code parity between crates/ / libraries/ diagnostics and docs/wiki/errors/.
6. Zero Rust Debug (`{:?}`) formatting leaks or stale `manifest.toml` wording in diagnostics.
7. FFI `solix_vm_*` runtime helper parity between ffi/ + solix-runtime and docs/wiki/ffi/.
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

DEPRECATED_PATTERNS = [
    (
        re.compile(r"\bSolix v2\.0(?:\.0)?\b"),
        "Stale 'Solix v2.0' version reference (use Solix v3 / v3.x)",
    ),
    (
        re.compile(r"\bsolix_vm_get_field\b"),
        "Removed C FFI helper name 'solix_vm_get_field' (use 'solix_vm_get_field_i32' or 'solix_vm_read_field_u64')",
    ),
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
        # Exclude historical milestone tracking files if present
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

            for pattern, reason in DEPRECATED_PATTERNS:
                if pattern.search(line):
                    errors.append(
                        f"{file_path}:{line_number}: {reason}"
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
    """Verifies that all required top-level manifest.yaml (or legacy manifest.toml) sections are documented."""
    errors: list[str] = []
    if not manifest_doc_path.exists():
        return [f"Manifest documentation file not found at '{manifest_doc_path}'"]

    manifest_doc_text = manifest_doc_path.read_text(encoding="utf-8")

    required_sections = [
        ("configuration:", "[configuration]"),
        ("profiles:", "[profiles]"),
        ("native_library_checksums:", "[native_library_checksums]"),
        ("pre_compile:", "`pre_compile`"),
        ("post_compile:", "`post_compile`"),
        ("natives:", "`natives`"),
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


def extract_cli_long_flags(cli_file_path: Path) -> set[str]:
    """Parses crates/solix-toolchain/src/cli.rs to discover visible CLI `--long` flags."""
    if not cli_file_path.exists():
        return set()

    flags: set[str] = set()
    attr_lines: list[str] = []
    in_attr = False
    paren_depth = 0

    for line in cli_file_path.read_text(encoding="utf-8").splitlines():
        if "#[arg(" in line:
            in_attr = True
            attr_lines = [line]
            paren_depth = line.count("(") - line.count(")")
            if paren_depth <= 0:
                in_attr = False
        elif in_attr:
            attr_lines.append(line)
            paren_depth += line.count("(") - line.count(")")
            if paren_depth <= 0:
                in_attr = False
        elif attr_lines:
            joined = " ".join(attr_lines)
            field_match = re.match(r"^\s*(?:pub\s+)?(\w+)\s*:", line)
            if (
                field_match
                and re.search(r"\blong\b", joined)
                and "hide = true" not in joined
            ):
                explicit_long = re.search(r'long\s*=\s*"([^"]+)"', joined)
                flag_name = (
                    explicit_long.group(1)
                    if explicit_long
                    else field_match.group(1).replace("_", "-")
                )
                flags.add(f"--{flag_name}")
                attr_lines = []
            elif line.strip() and not line.strip().startswith(("///", "#[")):
                attr_lines = []

    return flags


def check_cli_flag_parity(cli_file_path: Path, docs_directory: Path) -> list[str]:
    """Verifies every non-hidden CLI flag appears in both CLI specs and command wiki docs."""
    flags = extract_cli_long_flags(cli_file_path)
    if not flags:
        return []

    spec_dir = docs_directory / "spec" / "tools" / "cli"
    wiki_dir = docs_directory / "wiki" / "commands"
    spec_corpus = "\n".join(
        path.read_text(encoding="utf-8")
        for path in sorted(spec_dir.glob("*.md"))
    )
    wiki_corpus = "\n".join(
        path.read_text(encoding="utf-8")
        for path in sorted(wiki_dir.glob("*.md"))
    )

    errors: list[str] = []
    for flag in sorted(flags):
        if flag not in spec_corpus:
            errors.append(
                f"Active CLI flag '{flag}' missing from specification docs under '{spec_dir}'"
            )
        if flag not in wiki_corpus:
            errors.append(
                f"Active CLI flag '{flag}' missing from command wiki docs under '{wiki_dir}'"
            )
    return errors


def extract_codebase_error_codes(repo_root: Path) -> tuple[dict[str, list[str]], list[str]]:
    """Extracts active E/R diagnostic codes and reports duplicate formal enum allocations."""
    codes: dict[str, list[str]] = {}
    duplicate_errors: list[str] = []
    formal_code_owners: dict[str, str] = {}

    diagnostic_globs = [
        repo_root / "crates" / "solix-common" / "src" / "diagnostics",
        repo_root / "crates" / "solix-compilation" / "src" / "diagnostics",
    ]
    arm_re = re.compile(r'Self::(\w+)[^=]*=>\s*"([ER]\d{4})"')
    for diag_dir in diagnostic_globs:
        if not diag_dir.is_dir():
            continue
        for rs_file in sorted(diag_dir.glob("*.rs")):
            text = rs_file.read_text(encoding="utf-8")
            for match in arm_re.finditer(text):
                variant_name, code = match.group(1), match.group(2)
                owner = f"{rs_file.name}::{variant_name}"
                codes.setdefault(code, []).append(owner)
                if code in formal_code_owners and formal_code_owners[code] != owner:
                    duplicate_errors.append(
                        f"Duplicate formal diagnostic code '{code}': {formal_code_owners[code]} vs {owner}"
                    )
                else:
                    formal_code_owners[code] = owner

    runtime_err_file = (
        repo_root / "crates" / "solix-runtime" / "src" / "exceptions" / "errors.rs"
    )
    if runtime_err_file.is_file():
        rt_text = runtime_err_file.read_text(encoding="utf-8")
        for match in arm_re.finditer(rt_text):
            variant_name, code = match.group(1), match.group(2)
            codes.setdefault(code, []).append(f"RuntimeErrorCode::{variant_name}")

    bracket_re = re.compile(r"\[([E]\d{4})\]")
    for scan_dir in (
        repo_root / "crates" / "solix-toolchain" / "src",
        repo_root / "libraries" / "solixlib",
    ):
        if not scan_dir.is_dir():
            continue
        for file_path in sorted(scan_dir.rglob("*")):
            if file_path.suffix not in (".rs", ".slx"):
                continue
            for match in bracket_re.finditer(file_path.read_text(encoding="utf-8")):
                codes.setdefault(match.group(1), []).append(str(file_path))

    custom_test = (
        repo_root
        / "crates"
        / "solix-tests"
        / "tests"
        / "compilation"
        / "pipeline"
        / "test_pipeline_customization.rs"
    )
    if custom_test.is_file() and '"E9001"' in custom_test.read_text(encoding="utf-8"):
        codes.setdefault("E9001", []).append(str(custom_test))

    return codes, duplicate_errors


def extract_documented_error_codes(errors_dir: Path) -> set[str]:
    """Extracts documented E/R codes from heading/table definitions under docs/wiki/errors/."""
    documented: set[str] = set()
    if not errors_dir.is_dir():
        return documented

    entry_re = re.compile(r"(?:^###\s+`?|\|\s*\*\*`)([ER]\d{4})\b", re.M)
    for doc_file in sorted(errors_dir.glob("*.md")):
        if doc_file.name == "README.md":
            continue
        for match in entry_re.finditer(doc_file.read_text(encoding="utf-8")):
            documented.add(match.group(1))
    return documented


def check_error_code_parity(repo_root: Path, docs_directory: Path) -> list[str]:
    """Verifies bidirectional parity between codebase diagnostic codes and docs/wiki/errors/."""
    active_codes, errors = extract_codebase_error_codes(repo_root)
    errors_dir = docs_directory / "wiki" / "errors"
    documented_codes = extract_documented_error_codes(errors_dir)

    missing_docs = sorted(set(active_codes) - documented_codes)
    stale_docs = sorted(documented_codes - set(active_codes))
    for code in missing_docs:
        errors.append(
            f"Active diagnostic code '{code}' ({', '.join(active_codes[code][:2])}) is missing from '{errors_dir}'"
        )
    for code in stale_docs:
        errors.append(
            f"Stale diagnostic code '{code}' documented in '{errors_dir}' has no active definition in codebase"
        )
    return errors


def check_diagnostic_message_hygiene(repo_root: Path) -> list[str]:
    """Scans production diagnostic definitions and builders for Debug `{:?}` leaks or stale manifest.toml refs."""
    errors: list[str] = []
    targets: list[Path] = []
    for diag_dir in (
        repo_root / "crates" / "solix-common" / "src" / "diagnostics",
        repo_root / "crates" / "solix-compilation" / "src" / "diagnostics",
    ):
        if diag_dir.is_dir():
            targets.extend(sorted(diag_dir.glob("*.rs")))

    extra_builder_files = [
        repo_root / "crates" / "solix-common" / "src" / "project.rs",
        repo_root / "crates" / "solix-compilation" / "src" / "phases" / "binder" / "passes" / "pass3_type_checking.rs",
        repo_root / "crates" / "solix-runtime" / "src" / "vm" / "fast.rs",
        repo_root / "crates" / "solix-runtime" / "src" / "native" / "builtins.rs",
        repo_root / "crates" / "solix-runtime" / "src" / "context.rs",
        repo_root / "crates" / "solix-runtime" / "src" / "options.rs",
    ]

    debug_fmt_re = re.compile(r"\{[a-zA-Z0-9_]*:\?\}")
    for rs_file in targets:
        lines = rs_file.read_text(encoding="utf-8").splitlines()
        in_tests = False
        for line_num, line in enumerate(lines, start=1):
            if "#[cfg(test)]" in line:
                in_tests = True
            if in_tests:
                continue
            if ("#[error(" in line or "help(" in line) and debug_fmt_re.search(line):
                errors.append(
                    f"{rs_file}:{line_num}: Diagnostic template uses Rust Debug formatting '{{:?}}': {line.strip()}"
                )
            if "manifest.toml" in line and ("help(" in line or "#[error(" in line):
                errors.append(
                    f"{rs_file}:{line_num}: Stale 'manifest.toml' reference in diagnostic metadata: {line.strip()}"
                )

    leak_line_re = re.compile(
        r"format!\([^)]*(return_type|left_val|right_val|acc_val|found_val|pool_val|deser_err|deser_error|value|err|path):\?"
    )
    for builder_file in extra_builder_files:
        if not builder_file.is_file():
            continue
        lines = builder_file.read_text(encoding="utf-8").splitlines()
        in_tests = False
        for line_num, line in enumerate(lines, start=1):
            if "#[cfg(test)]" in line:
                in_tests = True
            if in_tests:
                continue
            if leak_line_re.search(line) or (
                "builtins.rs" in builder_file.name and "(tag={:?})" in line
            ):
                errors.append(
                    f"{builder_file}:{line_num}: User-facing error string leaks Rust Debug formatting '{{:?}}': {line.strip()}"
                )
    return errors


def extract_ffi_vm_symbols(repo_root: Path) -> set[str]:
    """Extracts exported `solix_vm_*` symbols from C header and Rust FFI runtime exports."""
    symbols: set[str] = set()
    runtime_ffi = repo_root / "crates" / "solix-runtime" / "src" / "native" / "ffi.rs"
    if runtime_ffi.is_file():
        symbols.update(
            re.findall(
                r'pub unsafe extern "C" fn (solix_vm_\w+)',
                runtime_ffi.read_text(encoding="utf-8"),
            )
        )
    header_file = repo_root / "ffi" / "c" / "include" / "solix_ffi.h"
    if header_file.is_file():
        symbols.update(
            re.findall(
                r"static inline\s+[\w\s\*]+\s+(solix_vm_\w+)\s*\(",
                header_file.read_text(encoding="utf-8"),
            )
        )
    return symbols


def check_ffi_symbol_parity(repo_root: Path, docs_directory: Path) -> list[str]:
    """Verifies exported `solix_vm_*` symbols match `docs/wiki/ffi/c-cpp-sdk.md`."""
    expected_symbols = extract_ffi_vm_symbols(repo_root)
    c_sdk_doc = docs_directory / "wiki" / "ffi" / "c-cpp-sdk.md"
    if not c_sdk_doc.is_file():
        return [f"FFI C SDK reference doc missing at '{c_sdk_doc}'"]

    doc_text = c_sdk_doc.read_text(encoding="utf-8")
    errors: list[str] = []
    for symbol in sorted(expected_symbols):
        if symbol not in doc_text:
            errors.append(
                f"Exported FFI helper '{symbol}' is missing from '{c_sdk_doc}'"
            )
    return errors


def main():
    repo_root = Path(__file__).resolve().parent.parent.parent
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
        print("   [PASS] All active CLI subcommands are documented.")

    # 2. Verify deprecated terms
    print("2. Checking for deprecated terminology...")
    deprecated_term_errors = check_deprecated_terms(all_markdown_files)
    if deprecated_term_errors:
        all_verification_failures.extend(deprecated_term_errors)
    else:
        print("   [PASS] Zero deprecated terms found in active documentation.")

    # 3. Verify markdown links
    print(f"3. Checking link integrity across {len(all_markdown_files)} file(s)...")
    broken_link_errors = verify_all_links(all_markdown_files, repo_root)
    if broken_link_errors:
        all_verification_failures.extend(broken_link_errors)
    else:
        print("   [PASS] Zero broken links found.")

    # 4. Verify manifest.yaml / legacy sections
    print("4. Checking top-level manifest section coverage...")
    manifest_section_errors = check_manifest_sections_documented(manifest_doc_file)
    if manifest_section_errors:
        all_verification_failures.extend(manifest_section_errors)
    else:
        print("   [PASS] All top-level manifest sections are documented.")

    # 5. Verify CLI --long flag parity across specs and command wiki
    print("5. Checking CLI --long flag parity across specs and command wiki...")
    cli_flag_errors = check_cli_flag_parity(cli_source_file, docs_directory)
    if cli_flag_errors:
        all_verification_failures.extend(cli_flag_errors)
    else:
        print("   [PASS] All visible CLI flags are documented in specs and wiki.")

    # 6. Verify error-code parity across crates/, libraries/, and docs/wiki/errors/
    print("6. Checking error-code coverage & uniqueness against docs/wiki/errors/...")
    error_code_parity_errors = check_error_code_parity(repo_root, docs_directory)
    if error_code_parity_errors:
        all_verification_failures.extend(error_code_parity_errors)
    else:
        print("   [PASS] Complete bidirectional error-code parity verified.")

    # 7. Verify diagnostic message hygiene (no Rust {:?} leaks or manifest.toml wording)
    print("7. Checking diagnostic templates & builders for Debug leaks...")
    diag_hygiene_errors = check_diagnostic_message_hygiene(repo_root)
    if diag_hygiene_errors:
        all_verification_failures.extend(diag_hygiene_errors)
    else:
        print("   [PASS] Zero diagnostic Debug leaks or stale manifest.toml references.")

    # 8. Verify C FFI solix_vm_* symbol parity
    print("8. Checking C FFI solix_vm_* helper parity...")
    ffi_symbol_errors = check_ffi_symbol_parity(repo_root, docs_directory)
    if ffi_symbol_errors:
        all_verification_failures.extend(ffi_symbol_errors)
    else:
        print("   [PASS] All exported solix_vm_* helpers match C/C++ SDK wiki.")

    # Summary
    if all_verification_failures:
        print(
            f"\n[FAIL] Documentation Verification FAILED with {len(all_verification_failures)} defect(s):"
        )
        for defect in all_verification_failures:
            print(f"  - {defect}")
        sys.exit(1)

    print("\n[PASS] Documentation Verification PASSED with 0 defects.")
    sys.exit(0)


if __name__ == "__main__":
    main()
