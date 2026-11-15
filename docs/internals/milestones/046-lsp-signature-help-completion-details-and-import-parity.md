# Milestone 046: LSP Signature Help, Completion Details, Import Diagnostic Parity & Semantic Resilience

## Document Metadata
* **Milestone ID:** MS-046
* **Target Version:** `v1.1.3`
* **Milestone Branch:** `milestone/046-lsp-signature-help-completion-details-and-import-parity`
* **Status:** COMPLETED
* **Lead Architect:** Core Systems & Language Tooling Team
* **Target Delivery Date:** 2026-11-15
* **Related Specifications:** [GL-0001](../../internals/guidelines/authoring-commits.md), [GL-0008](../../internals/guidelines/authoring-milestones.md), [GL-0011](../../internals/guidelines/language-conventions.md), [GL-0013](../../internals/guidelines/documentation-update-protocol.md), [SPEC-0001](../../spec/README.md)

---

## 1. Executive Summary & Objectives

Real-world usage of the Solix Visual Studio Code extension on multi-file projects (such as `examples/breakout`) identified three tooling defects in Language Server Protocol (LSP) diagnostics, code completion, and syntax highlighting:

1. **IntelliSense Missing Parameter Names & Types:**
   - In autocompletion lists, methods and functions only presented bare symbol names (`DrawRectangleRounded`) without displaying parameter signatures (`(Rectangle rec, float32 roundness, int32 segments, Color color)`) or return types in the primary suggestion label.
   - When developers typed open parentheses or commas (e.g. `Raylib::DrawRectangleRounded(` or `DrawRectangleRounded(arg, `), signature help failed to display because the resilient parser does not produce an AST `MethodCall` node before the closing parenthesis is typed, leaving `find_enclosing_call_in_ast` with zero matches.
2. **Semantic Highlighting Character-Level Drift on Syntax Errors:**
   - When syntax errors occur or during in-editor debounce editing, character offsets can drift or lexical fallback misclassifies method calls starting with uppercase letters (`ClearBackground`) as classes rather than methods. Furthermore, `\r` carriage-return characters in CRLF files could skew character column calculations.
3. **Import Diagnostic Parity (`import` vs `extern import`):**
   - The CLI compiler strictly requires external packages not declared in `manifest.toml` to be imported via `extern import` (e.g. `extern import solix.collections.List;` or `extern import raylib.*;`).
   - However, wildcard imports starting with `std` or `solix` were exempted from package validation in `pass1_symbol_discovery.rs` (`let is_std = path.starts_with("std") || path.starts_with("solix");`), causing LSP background analysis to silently accept unimported wildcard packages while CLI compilation failed with `solix::binder::cannot_resolve_import` (`E0232`).

Milestone 046 resolves all three defects across `crates/solix-compilation`, `crates/solix-lsp`, and `editors/vscode`.

---

## 2. Technical Architecture & Implementation Plan

### 2.1 Subsystem 1: Live Signature Help & Completion Details (`crates/solix-lsp`)

* **LSP 3.17 `label_details` in Code Completion:**
  - Enhance `make_symbol_completion_item` in `crates/solix-lsp/src/completion.rs` to populate `CompletionItemLabelDetails`:
    - `detail`: Formatted parameter list (e.g. `(Rectangle rec, float32 roundness, int32 segments, Color color)`).
    - `description`: Return type (e.g. `void`, `int32`).
* **Lexical Fallback for Active Signature Help:**
  - In `crates/solix-lsp/src/signature_help.rs`, when AST traversal returns no enclosing call (due to unclosed parentheses during active typing):
    - Implement `find_enclosing_call_lexical` to scan backwards from `target_offset` across the token stream / source text to locate the active unclosed `(`.
    - Track parenthesis depth and count commas at depth 0 to determine `active_parameter_index`.
    - Extract the callee symbol (and optional receiver class qualifier).
    - Query `symbol_table` and construct active `SignatureInformation` with parameter details.

### 2.2 Subsystem 2: Semantic Highlighting Robustness (`crates/solix-lsp`)

* **Call-Site Lexical Fallback Classification:**
  - In `crates/solix-lsp/src/semantic_tokens.rs`, update identifier classification during syntax errors: if an identifier is immediately followed by `(` (or preceded by `::` in call context), classify it as `SemanticTokenType::METHOD` / `FUNCTION` regardless of whether its first character is uppercase PascalCase (`ClearBackground`).
* **CRLF Sanitization in Coordinate Calculations:**
  - In `crates/solix-lsp/src/diagnostics.rs`, update `byte_offset_to_position` to ignore `\r` carriage-return characters so character columns precisely match VS Code UTF-16 visual column indices.

### 2.3 Subsystem 3: Strict Import Diagnostic Parity (`crates/solix-compilation`)

* **Eliminate Unchecked `solix` / `std` Wildcard Pass-Through:**
  - In `crates/solix-compilation/src/phases/binder/passes/pass1_symbol_discovery.rs`, restrict built-in package exemption strictly to `blt` and `blt::*`.
  - External packages (`solix.*`, `raylib.*`) must exist in `symbol_table.known_packages` (populated via `manifest.toml` dependencies, CLI `--deps`, or `extern import`). Otherwise, `BindingError::CannotResolveImport` (`E0232`) is emitted consistently in both compiler and LSP.

---

## 3. Verification Plan

1. **Unit & Integration Tests:**
   - Add tests verifying signature help popup on unclosed calls: `Raylib::DrawRectangleRounded(brick_rect, `.
   - Add tests verifying `label_details` parameter signatures on method completion items.
   - Add tests verifying `E0232` diagnostic emission on wildcard imports `import solix.collections.*;` without `extern` or manifest dependency.
   - Add tests verifying semantic token classification for PascalCase method calls under syntax error states.
2. **Quality Gate:**
   - Full workspace tests: `cargo test --workspace --all-targets`.
   - Linter checks: `cargo clippy --workspace --all-targets -- -D warnings`.
   - Format checks: `cargo fmt --check`.
   - GL-0011 compliance: strictly zero single-letter variables.
   - Living documentation drift audit: `python3 scripts/verify_docs.py`.
3. **VS Code Extension Release:**
   - Bump extension version to `1.1.3` in `editors/vscode/package.json`.
   - Package `editors/vscode/solix-1.1.3.vsix` and reinstall into local VS Code.
