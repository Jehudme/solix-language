# Milestone 004: Compiler Stabilization, Common Diagnostics & Incremental Architecture

## Document Metadata
* **Milestone ID:** MS-004
* **Target Version:** `v0.1.0-alpha.4`
* **Milestone Branch:** `milestone/004-stabilization-and-lsp`
* **Status:** COMPLETED
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-10-26
* **Completion Date:** 2026-10-25
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/specs/lexical.md` (SPEC-0001)
  * `docs/internals/milestones/002-lexical-analyzer.md` (MS-002)
  * `docs/internals/milestones/003-recursive-descent-parser.md` (MS-003)

---

## 1. Executive Summary & Purpose

Following the implementation of Milestone 002 (Lexer) and Milestone 003 (Recursive-Descent Parser), a stabilization and architectural alignment milestone is necessary before advancing to Semantic Analysis & Type Binding (MS-005).

In the previous design:
1. **Coupled Diagnostic & Span Infrastructure:** `Span`, `LexingError`, and `ParsingError` resided exclusively within `crates/solix-compilation`. However, `crates/solix-lsp` (and external tooling such as linters, debuggers, and formatters) requires direct access to `Span` and compiler diagnostics without pulling in the entire parser and code generator engine.
2. **Coarse-Grained Pipeline & Recompilation Bottleneck:** The compilation pipeline operated primarily on an all-or-nothing basis (`lex(&mut context)` and `parse(&mut context)`). In an interactive Language Server Protocol (LSP) environment, editing a single file forced whole-project re-lexing and re-parsing.
3. **Monolithic AST Storage in Context:** `context.ast` held a flat vector of programs without per-source isolation, preventing fine-grained error retention, partial AST updates, and dependency-based invalidation.

Milestone 004 achieves three primary architectural objectives:
* **Consolidate Common Diagnostics in `solix-common`:** Migrate `Span`, `LexingError`, and `ParsingError` into `crates/solix-common`. Provide standard diagnostic traits, coordinate conversions, and rich miette metadata usable across `solix-compilation`, `solix-lsp`, and `solix-client`.
* **Granular Single-Source Pipeline Entrypoints:** Establish symmetric single-source and context-level entrypoints:
  - `lex_source(source_id: u16, source_text: &str) -> Result<Vec<Token>, Vec<LexingError>>`
  - `lex_context(context: &mut CompilationContext) -> Result<MetaData, Vec<LexingError>>`
  - `parse_source(source_id: u16, tokens: &[Token], source_text: &str) -> Result<ProgramAst, Vec<ParsingError>>`
  - `parse_context(context: &mut CompilationContext) -> Result<AstMetaData, Vec<ParsingError>>`
* **Source-Partitioned AST Storage & Dependency Invalidation Tracking:** Partition AST storage in `CompilationContext` by `source_id` (`Vec<Option<ProgramAst>>`), alongside dependency tracking (`dependencies: Vec<Vec<u16>>`) and reverse dependency tracking (`dependents: Vec<Vec<u16>>`). When a file changes, the LSP can re-lex, re-parse, and selectively re-bind only the affected file and its downstream dependents.

---

## 2. Scope & Crates Affected

* **`crates/solix-common`**:
  * `src/span.rs`: Canonical `Span` struct, constructor, calculations (`length`, `is_empty`, `combine`), `Display` implementation, and `to_source_span()` conversion.
  * `src/diagnostics/mod.rs`: Diagnostic subsystem module re-exports.
  * `src/diagnostics/lexing.rs`: Canonical `LexingError` enum deriving `thiserror::Error` and `miette::Diagnostic` with standardized error codes.
  * `src/diagnostics/parsing.rs`: Canonical `ParsingError` enum deriving `thiserror::Error` and `miette::Diagnostic` with codes `E0101` through `E0153`.
  * `src/lib.rs`: Top-level re-exports (`Span`, `LexingError`, `ParsingError`).
  * `tests/diagnostic_tests.rs`: Comprehensive diagnostic unit and conversion tests.
* **`crates/solix-compilation`**:
  * `src/phases/lexing/tokens.rs`: Re-export `Span` from `solix_common::Span`.
  * `src/phases/lexing/errors.rs`: Re-export `LexingError` from `solix_common::LexingError`.
  * `src/phases/lexing/lexer.rs`: Expose both `lex_source` and `lex_context` (with `lex` alias).
  * `src/phases/parser/errors.rs`: Re-export `ParsingError` from `solix_common::ParsingError`.
  * `src/phases/parser/parser.rs`: Expose both `parse_source` and `parse_context` (with `parse` alias).
  * `src/CompilationContext.rs`:
    - Refactor `ast` to `Vec<Option<ProgramAst>>`.
    - Add `dependencies: Vec<Vec<u16>>` and `dependents: Vec<Vec<u16>>`.
    - Add methods for partial source compilation: `update_source()`, `lex_source()`, `parse_source()`, `invalidate_source()`, `get_dependents()`.
* **`crates/solix-lsp`**:
  * `src/main.rs`: Consume `solix_common::{Span, LexingError, ParsingError}` and `solix_compilation` partial compilation entrypoints.
  * `tests/lsp_tests.rs`: Verify diagnostic emission and incremental file recompilation harness.

---

## 3. Prioritized Deliverables & Task Matrix

### 3.1 P0: Critical Blockers (Release Gate)

- [x] `[P0]` **Dedicated Milestone Branch (`milestone/004-stabilization-and-lsp`):**
  Cut and isolate all stabilization work in `milestone/004-stabilization-and-lsp` conforming to GL-0002.

- [x] `[P0]` **Migrate `Span` to `solix-common` (`crates/solix-common/src/span.rs`):**
  Relocate the canonical `Span` definition to `solix-common`:
  * Fields: `id: u16`, `start: u32`, `end: u32`.
  * Methods: `new()`, `length()`, `is_empty()`, `to_source_span()`, `combine()`.
  * Derives: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`, `Hash`, `Serialize`, `Deserialize`.
  * Re-export in `solix_compilation::phases::lexing::tokens::Span` to maintain backwards compatibility.

- [x] `[P0]` **Migrate `LexingError` & `ParsingError` to `solix-common`:**
  * Move `LexingError` to `crates/solix-common/src/diagnostics/lexing.rs`.
  * Move `ParsingError` to `crates/solix-common/src/diagnostics/parsing.rs`.
  * Preserve all `miette` labels, format strings, and standardized error codes (`E0101`–`E0153`).
  * Re-export in `solix-compilation` without breaking any existing callers or tests.

### 3.2 P1: High Priority (Milestone Core)

- [x] `[P1]` **Single-Source & Context-Level Lexing Symmetrical APIs:**
  * Ensure `lex_source(source_id: u16, source_text: &str) -> Result<Vec<Token>, Vec<LexingError>>` is a first-class public primitive.
  * Implement `lex_context(context: &mut CompilationContext) -> Result<MetaData, Vec<LexingError>>` calling `lex_source` per source file.
  * Provide `lex` as a direct wrapper for `lex_context`.

- [x] `[P1]` **Single-Source & Context-Level Parsing Symmetrical APIs:**
  * Ensure `parse_source(source_id: u16, tokens: &[Token], source_text: &str) -> Result<ProgramAst, Vec<ParsingError>>` is a first-class public primitive.
  * Implement `parse_context(context: &mut CompilationContext) -> Result<AstMetaData, Vec<ParsingError>>` calling `parse_source` per source file.
  * Provide `parse` as a direct wrapper for `parse_context`.

- [x] `[P1]` **Source-Partitioned AST Storage (`CompilationContext.rs`):**
  * Update `context.ast` to `Vec<Option<ProgramAst>>`, sized dynamically to match `context.sources.len()`.
  * When parsing a source succeeds, record `context.ast[source_id as usize] = Some(ast)`.
  * When parsing a source fails, record `context.ast[source_id as usize] = None` while preserving other valid source ASTs.

- [x] `[P1]` **Incremental Recompilation Methods on `CompilationContext`:**
  * `update_source(&mut self, source_id: u16, new_code: impl Into<String>)`: Updates source text and invalidates cached tokens/AST.
  * `lex_source(&mut self, source_id: u16) -> Result<(), Vec<LexingError>>`: Tokenizes only the specified source into `self.tokens[source_id]`.
  * `parse_source(&mut self, source_id: u16) -> Result<(), Vec<ParsingError>>`: Parses only the specified source into `self.ast[source_id]`.

- [x] `[P1]` **Dependency & Invalidation Graph Scaffolding:**
  * Add `dependencies: Vec<Vec<u16>>` (files that file `i` imports).
  * Add `dependents: Vec<Vec<u16>>` (files that import file `i`).
  * Add `register_dependency(&mut self, from_source: u16, to_source: u16)`.
  * Add `get_dependents(&self, source_id: u16) -> &[u16]`.

### 3.3 P2: Medium Priority (LSP Verification & Integration)

- [x] `[P2]` **LSP Diagnostic Bridge in `solix-lsp`:**
  * Implement helper in `solix-lsp` converting `solix_common::Span` and `solix_common::{LexingError, ParsingError}` into `tower_lsp::lsp_types::Diagnostic`.
  * Verify in `tests/lsp_tests.rs` that syntax errors from a single modified file produce accurate LSP diagnostics.

- [x] `[P2]` **Incremental Compilation Verification Tests:**
  * Add tests in `crates/solix-compilation/tests/incremental_tests.rs`:
    - Register multiple files.
    - Mutate one file.
    - Re-lex and re-parse only that file.
    - Assert unaffected files retain their AST and tokens without re-execution.

---

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Compiles with 0 warnings across all targets | `cargo build --workspace --all-targets` | [x] Complete |
| **Automated Test Suite** | 100% pass across all unit and integration tests | `cargo test --workspace --all-targets` | [x] Complete |
| **Clippy Static Analysis** | 0 warnings, strict pass | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Complete |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [x] Complete |
| **Safety Invariants** | `#![forbid(unsafe_code)]` declared across all modified modules | Workspace source audit | [x] Complete |
| **LSP Integration** | `solix-lsp` compiles and consumes common diagnostics | `cargo test -p solix-lsp` | [x] Complete |
| **Incremental Verification** | Partial recompilation passes verify selective AST updates | `cargo test --test incremental_tests` | [x] Complete |

---

## 5. Architectural Blueprint

```
                      ┌───────────────────────────────────────┐
                      │             solix-common              │
                      │  • Span (id, start, end)              │
                      │  • LexingError                        │
                      │  • ParsingError (E0101..E0153)        │
                      └───────────────┬───────────────────────┘
                                      │
                 ┌────────────────────┴────────────────────┐
                 ▼                                         ▼
   ┌───────────────────────────┐             ┌───────────────────────────┐
   │     solix-compilation     │             │         solix-lsp         │
   │  • lex_source / lex_ctx   │             │  • Document synchronization│
   │  • parse_source / parse_ctx│            │  • Span -> LSP Position   │
   │  • CompilationContext     │             │  • Diagnostic translation │
   │    - tokens: Vec<Vec<T>>  │             │  • Incremental re-lex/    │
   │    - ast: Vec<Option<AST>>│             │    re-parse on did_change │
   │    - dependents: Vec<Vec> │             └───────────────────────────┘
   └───────────────────────────┘
```
