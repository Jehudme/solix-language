# Milestone 053: Toolchain CLI Refactoring & Documentation Intelligence

## Document Metadata
* **Milestone ID:** MS-053
* **Target Version:** `v2.0.0`
* **Milestone Branch:** `milestone/053-toolchain-cli-refactoring-and-doc-intelligence`
* **Status:** COMPLETED
* **Lead Architect:** Core Systems & Toolchain Infrastructure Team
* **Target Delivery Date:** 2026-11-16
* **Related Specifications:** [GL-0001](../../internals/guidelines/authoring-commits.md), [GL-0008](../../internals/guidelines/authoring-milestones.md), [GL-0011](../../internals/guidelines/language-conventions.md), [GL-0013](../../internals/guidelines/documentation-update-protocol.md), [SPEC-0001](../../spec/README.md)

---

## 1. Executive Summary & Objectives

Milestone 053 focuses on decoupling build profile configuration from runtime execution in the CLI, establishing strict entrypoint validation, and delivering a comprehensive offline documentation generator (`solix doc`) coupled with rich doc comment extraction across the Language Server Protocol (`solix-lsp`) and VS Code extension.

Key objectives:
1. **Toolchain CLI Refactoring**:
   - Remove `--test` from `solix run`. Tests are compiled at build time via profile inheritance (`solix build --profile test`).
   - Preserve `--entry <symbol>` override on `solix run`, enforcing strict validation that aborts execution with a runtime error (`E0301`) if the requested symbol does not exist in the loaded program or archive manifest.
2. **Static Documentation Generator (`solix doc`)**:
   - Build a high-performance offline documentation tool in `solix-toolchain`.
   - Parse triple-slash (`///`) and block (`/** ... */`) doc comments across classes, interfaces, enums, functions, methods, constructors, and fields.
   - Support standard documentation tags: `@param`, `@return`, `@throws`, `@deprecated`.
   - Export both responsive, searchable static HTML and GitHub-flavored Markdown.
3. **LSP & VS Code IntelliSense Integration**:
   - Surface doc comments directly in completion items (`textDocument/completion`), completion resolve (`completionItem/resolve`), signature help (`textDocument/signatureHelp`), and hover (`textDocument/hover`).
   - Ensure the VS Code extension declares full markdown documentation capabilities.

---

## 2. Priority Blocker Taxonomy & Deliverables

| Priority | Task ID | Subsystem | Description | Release Impact |
|:---|:---|:---|:---|:---|
| **P0** | `MS053-T01` | `toolchain` | Remove `--test` flag from `RunArgs` and `execute_run`. | Architectural consistency |
| **P0** | `MS053-T02` | `toolchain` | Enforce entrypoint symbol existence validation in `solix run --entry <symbol>`; emit `E0301` if symbol is absent. | Runtime safety |
| **P1** | `MS053-T03` | `compiler/toolchain` | Implement `solix doc` static generator with HTML and Markdown export. | Developer tooling |
| **P1** | `MS053-T04` | `lsp` | Surface doc comments in completions, signature help, and hovers. | IDE intelligence |
| **P2** | `MS053-T05` | `vscode` | Verify client capability configurations for markdown doc rendering in VS Code extension. | User experience |
| **P2** | `MS053-T06` | `docs` | Update CLI guides, LSP specs, and user manual reflecting new commands and options. | Documentation sync |

---

## 3. New Diagnostics & Error Codes

| Error Code | Subsystem | Name | Description & Template |
|:---|:---|:---|:---|
| **`E0301`** | Runtime / Toolchain | `EntrypointSymbolNotFound` | `Specified entrypoint symbol '{symbol}' was not found in the loaded program or container.` |
| **`E0302`** | Toolchain (`solix doc`) | `DocOutputDirectoryError` | `Failed to create documentation output directory '{path}': {reason}` |
| **`E0303`** | Toolchain (`solix doc`) | `DocGenerationFailed` | `Failed to generate documentation for workspace: {reason}` |

---

## 4. Comprehensive Test Matrix

### A. Positive Conformance Tests
1. `test_run_container_default_entrypoint`: Running a `.slxb` or `.slxarc` artifact executes the profile's baked entrypoint without extra runtime flags.
2. `test_run_container_valid_entrypoint_override`: Supplying `--entry Alternative::run` successfully executes the designated entrypoint.
3. `test_doc_generator_html_output`: `solix doc --output docs/html` extracts classes, methods, fields, and doc tags into valid HTML files with CSS assets.
4. `test_doc_generator_markdown_output`: `solix doc --format markdown` emits formatted Markdown documentation.
5. `test_lsp_completion_docs_rendered`: LSP completion list and completion resolve contain formatted doc comments for discovered symbols.
6. `test_lsp_signature_help_param_docs`: Typing parameter arguments triggers signature help with active parameter doc annotations.

### B. Negative Rejection Tests
1. `test_run_missing_entrypoint_override_fails`: Supplying `--entry NonExistent::symbol` fails immediately with code `E0301`.
2. `test_run_legacy_test_flag_rejected`: Invoking `solix run --test` produces unknown CLI flag error.
3. `test_doc_invalid_output_path_fails`: Providing an unwritable path emits diagnostic `E0302`.

---

## 5. Definition of Done (DoD)
1. Zero compiler errors and zero clippy warnings (`cargo clippy --workspace --all-targets -- -D warnings`).
2. 100% test pass rate across unit and integration tests (`cargo test --workspace`).
3. CLI documentation updated with zero drift (`scripts/verify_docs.py`).

---

## 6. Completed Deliverables & Verification Record

* **CLI `--test` Flag Removal:** Cleanly removed redundant `--test` flag from `RunArgs` and `execute_run`. Profile-scoped test entrypoint compilation now drives test execution directly.
* **Strict Entrypoint Validation (`E0301`):** Verified `--entry <symbol>` checks that requested symbols exist in loaded bytecode containers, throwing `[E0301]` when absent.
* **Static Documentation Generator (`solix doc`):** Implemented high-performance offline documentation tool supporting both modern static HTML with responsive CSS and Markdown exports, parsing classes, interfaces, enums, functions, methods, constructors, fields, and doc tags (`@param`, `@return`, `@throws`, `@deprecated`).
* **LSP & IntelliSense Intelligence:** Updated `solix-lsp` to attach doc comments upfront in completion lists and populated active `@param` documentation in signature help tooltips.
* **Hermetic Integration Tests:** Added 5 new integration tests in `milestone053_tests.rs` and 2 new LSP tests in `lsp_tests.rs`, passing with 100% success rate.
