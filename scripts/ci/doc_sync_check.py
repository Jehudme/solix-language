#!/usr/bin/env python3
"""Automated Documentation Synchronization & Quality Gate Checker.

Enforces GL-0011 (no single-letter variables) and GL-0013 (Trigger Matrix sync)
by analyzing git diff against a base commit/branch.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
import subprocess
import sys

# Subsystem Trigger Matrix (Pattern -> Required Documentation Paths)
TRIGGER_MATRIX = [
    {
        "name": "CLI Subsystem",
        "pattern": r"^crates/solix-toolchain/src/(cli|commands)/",
        "required_docs": [
            "docs/spec/tools/cli/",
            "docs/wiki/commands/",
            "docs/guide/01_getting_started.md",
        ],
    },
    {
        "name": "Lexer & Tokens",
        "pattern": r"^crates/solix-compilation/src/phases/lexing/",
        "required_docs": [
            "docs/spec/lexical.md",
            "docs/wiki/keywords/",
            "docs/wiki/primitives/",
            "docs/wiki/errors/lexer-errors.md",
        ],
    },
    {
        "name": "Parser & AST",
        "pattern": r"^crates/solix-compilation/src/phases/parser/",
        "required_docs": [
            "docs/spec/statements/",
            "docs/spec/expressions/",
            "docs/spec/declarations/",
            "docs/wiki/errors/parser-errors.md",
        ],
    },
    {
        "name": "Type Binder",
        "pattern": r"^crates/solix-compilation/src/phases/binder/",
        "required_docs": [
            "docs/spec/types.md",
            "docs/spec/declarations/",
            "docs/architecture/binder_passes.md",
            "docs/wiki/errors/binder-errors.md",
        ],
    },
    {
        "name": "Bytecode Optimizer & Escape Analysis",
        "pattern": r"^crates/solix-compilation/src/phases/optimizer/",
        "required_docs": [
            "docs/architecture/pipeline.md",
            "docs/spec/vm_isa.md",
            "docs/architecture/arc_internals.md",
        ],
    },
    {
        "name": "Bytecode Emitter",
        "pattern": r"^crates/solix-compilation/src/phases/emitter/",
        "required_docs": [
            "docs/spec/vm_isa.md",
            "docs/architecture/pipeline.md",
        ],
    },
    {
        "name": "Register VM & Bytecode Opcodes",
        "pattern": r"^(crates/solix-runtime/src/vm/|crates/solix-common/src/bytecode/)",
        "required_docs": [
            "docs/spec/vm_isa.md",
            "docs/architecture/memory_model.md",
            "docs/architecture/arc_internals.md",
            "docs/wiki/errors/runtime-errors.md",
        ],
    },
    {
        "name": "Native Interop & FFI",
        "pattern": r"^(crates/solix-runtime/src/native/|ffi/)",
        "required_docs": [
            "docs/spec/native_interop.md",
            "docs/wiki/ffi/",
            "docs/guide/09_native_plugins.md",
        ],
    },
    {
        "name": "Standard Library",
        "pattern": r"^libraries/solixlib/src/",
        "required_docs": [
            "docs/spec/libraries/solixlib/",
        ],
    },
    {
        "name": "Project Manifest & Scaffolding",
        "pattern": r"^(crates/solix-toolchain/assets/scripts/(solix_scaffold|solix_build)\.py|crates/solix-toolchain/src/commands/new\.rs|crates/solix-common/src/project\.rs)$",
        "required_docs": [
            "docs/spec/tools/cli/new.md",
            "docs/spec/tools/cli/manifest.md",
            "docs/wiki/commands/package-commands.md",
            "docs/guide/01_getting_started.md",
        ],
    },
    {
        "name": "Diagnostics & Error Codes",
        "pattern": r"^crates/(solix-common|solix-compilation|solix-runtime)/src/(diagnostics/|exceptions/errors\.rs)",
        "required_docs": [
            "docs/wiki/errors/",
            "docs/spec/diagnostics.md",
        ],
    },
    {
        "name": "VS Code Extension & Grammars",
        "pattern": r"^editors/vscode/(src/|syntaxes/|package\.json|README\.md)",
        "required_docs": [
            "docs/guide/editors_vscode.md",
            "editors/vscode/README.md",
        ],
    },
    {
        "name": "LSP & Formatting Subsystem",
        "pattern": r"^(crates/solix-lsp/src/|crates/solix-toolchain/src/commands/(lsp|fmt)\.rs)",
        "required_docs": [
            "docs/wiki/commands/lsp.md",
            "docs/wiki/commands/fmt.md",
            "docs/spec/tools/lsp/protocol.md",
        ],
    },
    {
        "name": "CI Workflows & Release Governance",
        "pattern": r"^\.github/workflows/",
        "required_docs": [
            "docs/guidelines/pr-and-release-governance.md",
            "docs/guidelines/branching-conventions.md",
            "CONTRIBUTING.md",
            "docs/guide/release.md",
        ],
    },
    {
        "name": "Governance & Offline Docs Scripts",
        "pattern": r"^scripts/(ci|docs|release|bench)/.*\.py$",
        "required_docs": [
            "docs/guidelines/pr-and-release-governance.md",
            "docs/guidelines/documentation-update-protocol.md",
            "docs/guidelines/authoring-docs.md",
            "CONTRIBUTING.md",
            "docs/spec/tools/cli/wiki.md",
        ],
    },
]

# Regex for detecting single-letter variable declarations in Solix/Rust code blocks
SINGLE_LETTER_VAR_REGEX = re.compile(
    r"\b(int32|int64|float32|float64|bool|string|let|var|mut)\s+([a-zA-Z])\s*[:=;,)]"
)


def get_git_diff_files(base_ref: str | None) -> list[str]:
    """Retrieves the list of modified files relative to the base ref or working tree."""
    if base_ref and set(base_ref) == {"0"}:
        base_ref = "HEAD~1"
    cmd = ["git", "diff", "--name-only"]
    if base_ref:
        cmd.append(f"{base_ref}...HEAD" if "..." not in base_ref else base_ref)
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        files = [line.strip() for line in result.stdout.splitlines() if line.strip()]
        if not files and base_ref:
            # Fall back to direct diff against base_ref
            fallback_cmd = ["git", "diff", "--name-only", base_ref]
            fb_result = subprocess.run(fallback_cmd, capture_output=True, text=True, check=True)
            files = [line.strip() for line in fb_result.stdout.splitlines() if line.strip()]
        return files
    except subprocess.CalledProcessError:
        # Fall back to status if base_ref fails (e.g. shallow clone or initial repo)
        status_result = subprocess.run(
            ["git", "status", "--porcelain"], capture_output=True, text=True, check=True
        )
        files = []
        for line in status_result.stdout.splitlines():
            if len(line) > 3:
                files.append(line[3:].strip())
        return files


def check_trigger_matrix(modified_files: list[str]) -> list[str]:
    """Validates that modified code files have matching documentation changes."""
    violations = []
    
    for rule in TRIGGER_MATRIX:
        pattern = re.compile(rule["pattern"])
        matching_code_files = [f for f in modified_files if pattern.search(f)]
        
        if not matching_code_files:
            continue
            
        # Check if any required documentation path was also modified
        doc_updated = False
        for req in rule["required_docs"]:
            for modified in modified_files:
                if modified.startswith(req) or modified == req:
                    doc_updated = True
                    break
            if doc_updated:
                break
                
        if not doc_updated:
            violations.append(
                f"Subsystem '{rule['name']}' modified in: {', '.join(matching_code_files[:3])}\n"
                f"  Missing required documentation update in at least one of: {', '.join(rule['required_docs'])}"
            )
            
    return violations


def check_single_letter_variables(doc_files: list[Path]) -> list[str]:
    """Inspects markdown files for single-letter variable declarations in code blocks."""
    violations = []
    
    for doc_path in doc_files:
        if not doc_path.exists() or not doc_path.is_file():
            continue
            
        content = doc_path.read_text(encoding="utf-8")
        in_code_block = False
        code_lang = ""
        
        is_counter_example = False
        for line_number, line in enumerate(content.splitlines(), start=1):
            stripped = line.strip()
            if stripped.startswith("```"):
                if in_code_block:
                    in_code_block = False
                    code_lang = ""
                    is_counter_example = False
                else:
                    in_code_block = True
                    code_lang = stripped[3:].strip().lower()
                continue
                
            if in_code_block and code_lang in ("solix", "rust", ""):
                if "// FLAWED" in line or "// BAD" in line or "// Anti-Pattern" in line:
                    is_counter_example = True
                    continue
                if is_counter_example:
                    continue

                match = SINGLE_LETTER_VAR_REGEX.search(line)
                if match:
                    var_name = match.group(2)
                    violations.append(
                        f"{doc_path}:{line_number}: Single-letter variable '{var_name}' prohibited by GL-0011: '{line.strip()}'"
                    )
                    
    return violations


def main():
    parser = argparse.ArgumentParser(description="Solix Documentation Synchronization & Quality Gate")
    parser.add_argument("--base", default=None, help="Base git commit or branch to compare against (e.g. main)")
    parser.add_argument("--all-docs", action="store_true", help="Audit all documentation files for GL-0011")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent.parent
    os.chdir(repo_root)

    print("=== Solix Documentation Synchronization & Quality Gate ===")
    
    modified_files = get_git_diff_files(args.base)
    print(f"Discovered {len(modified_files)} modified file(s).")

    # 1. Check Trigger Matrix
    trigger_violations = check_trigger_matrix(modified_files)
    if trigger_violations:
        print("\n[FAIL] Trigger Matrix Violations (Documentation Drift Detected):")
        for violation in trigger_violations:
            print(f"  - {violation}")
    else:
        print("[PASS] Trigger Matrix Check: All modified code subsystems have synchronized documentation.")

    # 2. Check GL-0011 Single-Letter Variables
    if args.all_docs:
        docs_to_check = list((repo_root / "docs").rglob("*.md"))
    else:
        docs_to_check = [
            repo_root / f for f in modified_files if f.startswith("docs/") and f.endswith(".md")
        ]
        
    print(f"Auditing {len(docs_to_check)} markdown file(s) for GL-0011 compliance...")
    gl0011_violations = check_single_letter_variables(docs_to_check)
    
    if gl0011_violations:
        print("\n[FAIL] GL-0011 Single-Letter Variable Violations:")
        for violation in gl0011_violations:
            print(f"  - {violation}")
    else:
        print("[PASS] GL-0011 Check: Zero single-letter variables found.")

    if trigger_violations or gl0011_violations:
        print("\nAudit FAILED.")
        sys.exit(1)
    else:
        print("\nAudit PASSED successfully.")
        sys.exit(0)


if __name__ == "__main__":
    main()
