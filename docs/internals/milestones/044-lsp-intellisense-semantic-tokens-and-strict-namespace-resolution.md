# Milestone 044: LSP IntelliSense, Semantic Tokens Drift Resilience & Strict Namespace Resolution

## Document Metadata
* **Milestone ID:** MS-044
* **Target Version:** `v1.1.1`
* **Milestone Branch:** `milestone/044-lsp-intellisense-semantic-tokens-and-strict-namespace-resolution`
* **Status:** ACTIVE
* **Lead Architect:** Core Systems & Toolchain Team
* **Target Delivery Date:** 2026-11-15
* **Related Specifications:** [GL-0001](../../internals/guidelines/authoring-commits.md), [GL-0008](../../internals/guidelines/authoring-milestones.md), [GL-0011](../../internals/guidelines/language-conventions.md), [GL-0013](../../internals/guidelines/documentation-update-protocol.md), [SPEC-0001](../../spec/language-specification.md)

---

## 1. Executive Summary & Objectives

Developer experience in the official Visual Studio Code extension and the Language Server Protocol (`solix-lsp`) is critical to developer productivity. Recent dogfooding and game development sessions (such as the Raylib Breakout showcase) uncovered subtle edge cases where language server intelligence, syntax highlighting, and compiler semantic binding diverged:

1. **Semantic Token Character Offset Drift:** During partial typing or after syntax errors, semantic token delta encoding drifted out of alignment across lines and characters, causing VS Code to render split character colors inside single identifier tokens (e.g., `ClearBackground` and `bg_slate` visually fragmented into mismatched color fragments).
2. **Generic `arg0`, `arg1` Parameter Placeholders:** Inlay hints and signature help rendered generic parameter names (`arg0:`, `arg1:`, `arg2:`) instead of actual declared parameter names and types for both external library bindings and internal declarations.
3. **Foreign Package Symbol Leakage:** Classes and functions declared inside external dependency packages (such as `raylib::Raylib` and `raylib::Color`) could be accessed in user source files without an explicit `import` statement when the importing file resided in the default package.
4. **Diagnostic Parity on Imports (`import` vs `extern import`):** In projects without `solixlib` in `manifest.toml`, `import solix.collections.List;` was rejected by the CLI compiler (`cannot_resolve_import`), but silently accepted or unflagged by the LSP background analyzer due to standard library auto-injection.
5. **Permissive Type Path Syntax (`.` vs `::`):** Type annotations permitted dot notation (`solix.collections.List<int32>`), violating the Solix language invariant where dot (`.`) is strictly reserved for instance member access, and scope resolution (`::`) is strictly mandatory for namespace paths and types.
6. **Premature Completion on Single Colon (`:`):** Typing a single colon (`solix::collections:`) triggered premature scope member completion, presenting general keywords and invalid scope proposals.

Milestone 044 eliminates these defects across `solix-compilation`, `solix-lsp`, and the VS Code extension, restoring complete semantic parity between batch compilation and interactive language intelligence.

---

## 2. Scope & Crates Affected

* **`crates/solix-compilation`:**
  * Enforce strict `::` in `parse_type_annotation`; disallow `.` in type paths.
  * Allow both `::` and `.` in `import` and `package` statements.
  * Preserve formal parameter names in `SymbolEntry` for all methods, functions, constructors, and operators across Pass 1, Pass 2, and Pass 3.
  * Enforce strict foreign package symbol isolation in `SymbolTable::resolve_symbol`: require an explicit file import before resolving foreign symbols, regardless of whether the calling source declares a `package`.
* **`crates/solix-lsp`:**
  * Serve `semantic_tokens_full` and `semantic_tokens_range` from live document snapshots (`get_live_snapshot`) rather than stale debounced snapshots.
  * Protect delta encoding against negative deltas, token overlap, and multi-line token overflows; add support for primitive type keyword semantic tokens.
  * Extract real parameter names from `SymbolEntry::parameter_names` for inlay hints (`inlay_hints.rs`), signature help (`signature_help.rs`), and hover signatures (`hover.rs`).
  * In `completion.rs`, suppress general and member completions when `line_prefix` ends with a single colon (`:` without preceding `:`), reserving completion exclusively for double colon (`::`).
  * In `workspace.rs`, align dependency resolution with compiler requirements: do not auto-resolve unimported external packages unless explicitly declared in `manifest.toml` or imported via `extern import`.
* **`editors/vscode`:**
  * Package and verify the updated extension VSIX.
* **`docs/` & `scripts/`:**
  * Update living specifications (`docs/spec/`, `docs/guide/`, `docs/wiki/`) and verify doc link integrity via `python3 scripts/verify_docs.py`.

---

## 3. Priority Blocker Taxonomy & Deliverables

| Priority | Task ID | Subsystem | Description | Release Impact |
|:---|:---|:---|:---|:---|
| **P0** | `MS044-T01` | `solix-lsp` | Eliminate semantic token offset drift by querying live snapshots and delta sanitization. | Blocks interactive editor correctness |
| **P0** | `MS044-T02` | `solix-compilation` | Disallow `.` in type annotations; strictly require `::` in `parse_type_annotation`. | Blocks grammar soundness |
| **P0** | `MS044-T03` | `solix-compilation` | Enforce explicit import requirement for foreign package symbols in `SymbolTable::resolve_symbol`. | Blocks package encapsulation |
| **P1** | `MS044-T04` | `solix-compilation` | Record parameter names on `SymbolEntry` during symbol discovery and type hierarchy passes. | Blocks developer intelligence |
| **P1** | `MS044-T05` | `solix-lsp` | Populate real parameter names and types in Inlay Hints, Signature Help, and Hover cards. | Blocks LSP UX parity |
| **P1** | `MS044-T06` | `solix-lsp` | Suppress completion on single colon (`:`), requiring double colon (`::`) for scope resolution. | Blocks completion UX |
| **P1** | `MS044-T07` | `solix-lsp` | Synchronize LSP background diagnostics with compiler dependency resolution rules. | Blocks diagnostic parity |
| **P2** | `MS044-T08` | `editors/vscode` | Build, package, and install updated VS Code extension (`solix-1.1.1.vsix`). | Developer testing verification |
| **P2** | `MS044-T09` | `docs` | Update specification, guide, and wiki entries reflecting strict `::` type qualification. | GL-0013 compliance |

---

## 4. Phased Execution Roadmap

### Phase 1: Strict Scope Syntax (`::` vs `.`) & Single-Colon Trigger Fix
- Update `crates/solix-compilation/src/phases/parser/parser.rs` to forbid `TokenKind::PunctuationDot` in `parse_type_annotation`.
- Permit both `::` and `.` in `parse_import_statement` and `parse_package_statement`.
- In `crates/solix-lsp/src/completion.rs`, early-return empty completions if `line_prefix.ends_with(':') && !line_prefix.ends_with("::")`.

### Phase 2: Semantic Tokens Error Resilience, Live Snapshots & Delta Sanitization
- In `crates/solix-lsp/src/server.rs`, switch `semantic_tokens_full` and `semantic_tokens_range` to `self.workspace.get_live_snapshot(uri)`.
- In `crates/solix-lsp/src/semantic_tokens.rs`, classify primitive type tokens (`PrimitiveInt32`, etc.), clamp token lengths to line boundaries, and ensure tokens never overlap on the same line.
- In `crates/solix-lsp/src/workspace.rs`, ensure that when `last_good_semantic` is retained across syntax errors, token spans are sanitized or invalidated against mismatched source versions.

### Phase 3: Parameter Names and Types in Inlay Hints, Hover & Signature Help
- Extend `SymbolEntry` in `crates/solix-compilation/src/phases/binder/symbols.rs` with `pub parameter_names: Vec<String>`.
- In `pass1_symbol_discovery.rs` and `pass2_type_hierarchy.rs`, populate `parameter_names` for functions, class methods, and constructors from AST parameter nodes.
- Update `crates/solix-compilation/src/phases/binder/symbols.rs` `to_signature_string()` to format `Type Name` pairs.
- In `crates/solix-lsp/src/inlay_hints.rs`, replace `"arg{idx}: "` with `symbol.parameter_names.get(idx)`.
- In `crates/solix-lsp/src/signature_help.rs`, format parameter info with true parameter names.

### Phase 4: Strict Foreign Dependency Import Scoping & LSP Diagnostic Parity
- In `crates/solix-compilation/src/phases/binder/symbols.rs`: in `resolve_symbol`, ensure foreign symbols are never resolved unqualified without an explicit import, even if `caller_has_package` is false.
- Ensure qualified lookups like `raylib::Raylib` verify that `raylib` is in `file_imports` or the active package.
- In `crates/solix-lsp/src/workspace.rs`: remove special-case auto-resolution of `"solix"` when not declared in `dependencies_map` and not imported with `extern import`.

### Phase 5: Conformance Testing, VS Code Extension Packaging & Documentation Sync
- Add regression unit and integration tests covering:
  - Disallowing `.` in type paths (`solix.collections.List`).
  - Strict foreign symbol isolation (unimported `Raylib` fails to resolve).
  - Parameter names in inlay hints and signature help.
  - Single-colon completion suppression.
- Build and package `editors/vscode`.
- Run `python3 scripts/verify_docs.py` and synchronize documentation.

---

## 5. Non-Negotiable Definition of Done (DoD)

1. **Clean Compilation:** `cargo build --workspace --all-targets` succeeds with 0 warnings.
2. **100% Test Pass:** `cargo test --workspace --all-targets` passes all tests.
3. **Zero Clippy Defects:** `cargo clippy --workspace --all-targets -- -D warnings` passes cleanly.
4. **Zero Single-Letter Identifiers (GL-0011):** Code contains strictly 0 single-letter variables.
5. **Living Documentation (GL-0013):** Documentation passes `python3 scripts/verify_docs.py` with 0 defects.
