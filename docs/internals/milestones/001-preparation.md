# Milestone 001: Preparation & Workspace Foundation

## Document Metadata
* **Milestone ID:** MS-001
* **Target Version:** `v0.1.0-alpha.1`
* **Milestone Branch:** `milestone/001-preparation`
* **Status:** ACTIVE
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-11-05
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)

---

## 1. Executive Summary & Objectives

The Solix language compiler and virtual machine runtime are being completely re-architected in modern Rust (Edition 2024). Before implementing complex lexing, recursive-descent parsing, semantic analysis, or opcode dispatch, the project requires an uncompromising engineering foundation.

Milestone 001 establishes the **complete architectural foundation, linting and security gates, diagnostic primitives, fixture test harness, and specification directory tree** for the entire workspace. This guarantees that all subsequent compiler passes, virtual machine instructions, and developer tooling are built upon a deterministic, military-grade foundation with zero compilation warnings, zero unhandled panics, and strict automated testing from day one.

---

## 2. Scope & Crates Affected

* **`crates/solix-common`**: Source span primitives (`SourceSpan`, `SourceLocation`, `SourceId`), structured diagnostic foundations with `miette`, binary magic header definitions, and common error traits.
* **`crates/solix-compilation`**: Crate structure, module partitioning (`lexer/`, `parser/`, `binder/`, `codegen/`), Tier 1 unit test harnesses, and Tier 2 automated data-driven test runner (`tests/test_fixtures.rs`).
* **`crates/solix-runtime`**: Virtual machine module partitioning (`vm.rs`, `stack.rs`, `frame.rs`, `memory.rs`), runtime error types, and fixture execution directories.
* **`crates/solix-client`**: Unified CLI entrypoint using `clap` (derive), subcommand hierarchy (`compile`, `run`, `check`, `build`, `pack`, `package`, `lsp`, `dbg`), and `SOLIX_HOME` environment handling.
* **`crates/solix-lsp`**: Language Server Protocol crate skeleton, `tower-lsp` service scaffolding, and asynchronous Tokio runtime integration.
* **`crates/solix-dbg`**: Debugger CLI skeleton and terminal interactive command dispatch.
* **`docs/`**: Specification directory taxonomy (`docs/internals/specs/`), guidelines suite verification, and milestone contract tracking.

---

## 3. Prioritized Deliverables & Task Matrix

### 3.1 P0: Critical Blockers (Release Gate)

- [ ] `[P0]` **Workspace Security & Lint Invariants:**
  Enforce `#![forbid(unsafe_code)]` and `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` across all workspace crate root files (`src/lib.rs` and `src/main.rs`).
- [ ] `[P0]` **Hierarchical Diagnostic & Error Architecture:**
  Enforce the hierarchical error tree architecture (enums of sub-enums: `SolixError` -> `CompilerError` -> `LexerError`) with structured error code metadata (`E0XXX`), precise `SourceSpan` tracking, actionable help recommendations, and unbroken causal `#[source]` chaining across all crates.
- [ ] `[P0]` **Source Span Primitives:**
  Implement `SourceSpan`, `SourceLocation`, and `SourceFile` in `solix-common` with exact byte offset, line, and column tracking, implementing `miette::Diagnostic` with source code rendering.
- [ ] `[P0]` **Dedicated Milestone Branch Initialization:**
  Initialize and protect the dedicated `milestone/001-preparation` Git branch as mandated by GL-0002 and GL-0008. All PRs for this milestone must target this branch.
- [ ] `[P0]` **Cargo Workspace Dependency Hardening:**
  Update root `Cargo.toml` and crate dependencies with `compact_str = "0.8"`, `unicode-ident = "1.0"`, `tempfile = "3.13"`, and configure release profiles with `panic = "abort"`, `lto = "thin"`, and `codegen-units = 1`.

### 3.2 P1: High Priority (Milestone Core)

- [ ] `[P1]` **Conventional Integration Test Structure:**
  Establish standard Cargo integration test targets in `crates/*/tests/*.rs`, shared test utilities in `crates/*/tests/common/mod.rs`, and multi-domain test fixture directory trees in `crates/*/tests/fixtures/` (`syntax/`, `types/`, `codegen/`, `execution/`).
- [ ] `[P1]` **Data-Driven Language Fixture Harness:**
  Implement the automated fixture runner in `crates/solix-compilation/tests/syntax_fixtures_tests.rs` capable of executing `.slx` files and parsing `// EXPECT_ERROR: E0XXX` header directives.
- [ ] `[P1]` **Unified CLI Command Scaffolding:**
  Implement the complete `clap` derive command hierarchy in `crates/solix-client/src/main.rs` with typed arguments for `compile`, `run`, `check`, `build`, `pack`, `package`, `lsp`, and `dbg`.
- [ ] `[P1]` **Specification Directory Taxonomy Scaffolding:**
  Scaffold the complete specification tree in `docs/internals/specs/`:
  * `docs/internals/specs/statements/` (control flow, declarations, expressions, modules)
  * `docs/internals/specs/vm_isa.md` (bytecode layout and opcode specifications)
  * `docs/internals/specs/runtime/` (execution loop, ARC, exception unwinding)
  * `docs/internals/specs/cli/` (command specifications)
- [ ] `[P1]` **Bytecode Container Binary Format (`.slxbin`):**
  Specify and implement the 16-byte magic header (`SOLX\x02\x00...`), section table offsets, and target VM version constants in `solix-common`.

### 3.3 P2: Medium Priority (Ergonomics & Tooling)

- [ ] `[P2]` **Multi-Platform CI Pipeline:**
  Establish GitHub Actions workflow (`.github/workflows/ci.yml`) validating `cargo check`, `cargo test --workspace --all-targets`, `cargo clippy -- -D warnings`, and `cargo fmt --check` across Ubuntu, macOS, and Windows runners.
- [ ] `[P2]` **Local Developer Automation (`Justfile` or scripts):**
  Provide simple, standardized local commands for running format, clippy, test suites, and Miri checks.

### 3.4 P3: Low Priority (Opportunistic Cleanups)

- [ ] `[P3]` **Benchmark Scaffolding (`criterion`):**
  Add `criterion` to workspace dev-dependencies and create a baseline benchmark harness in `crates/solix-compilation/benches/` for tracking compiler throughput.

---

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Compiles with 0 warnings across all targets | `cargo build --workspace --all-targets` | [ ] Pending |
| **Automated Test Suite** | 100% pass across all unit and integration tests | `cargo test --workspace --all-targets` | [ ] Pending |
| **Clippy Static Analysis** | 0 warnings, 0 unwraps, strict pedantic pass | `cargo clippy --workspace --all-targets -- -D warnings -D clippy::unwrap_used -D clippy::expect_used` | [ ] Pending |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [ ] Pending |
| **Safety Invariants** | `#![forbid(unsafe_code)]` declared in all crate roots | Workspace source audit | [ ] Pending |
| **Living Documentation** | Guidelines (GL-0001 - GL-0011) and Spec directory scaffolded | Audit against GL-0010 | [x] Verified |

---

## 5. Risk Assessment & Mitigation Strategies

* **Risk 1: Diagnostic and Span Representation Inefficiencies**
  * *Impact:* Storing redundant strings or complex file paths in every AST token increases memory consumption during compilation.
  * *Mitigation:* Implement `SourceId` (a lightweight 32-bit integer indexing a global `SourceMap`) and `SourceSpan` (a compact pair of 32-bit start/end byte offsets), ensuring zero-copy token spans.
* **Risk 2: Circular Dependency Across Workspace Crates**
  * *Impact:* Coupling compiler, runtime, and CLI crates directly creates compilation cycles and slow build times.
  * *Mitigation:* Strictly enforce a Directed Acyclic Graph (DAG):
    * `solix-common` is the zero-dependency shared root.
    * `solix-compilation` and `solix-runtime` depend **only** on `solix-common`.
    * `solix-client`, `solix-lsp`, and `solix-dbg` sit at the top, depending on the compiler and runtime.
