# Mandatory Pre-Merge Git-Diff Documentation Audit Protocol

## Document Metadata

* **Guideline ID:** GL-0013
* **Scope:** Repository-Wide Pre-Merge Quality Gate (`crates/`, `libraries/`, `docs/`)
* **Status:** Final Implementation Standard
* **Audience:** Core Systems Engineers, Compiler Architects, PR Reviewers, Autonomous Agents
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)

---

## 1. Executive Summary & Purpose

A fundamental vulnerability of software documentation is asynchronous drift: code is updated to fix a bug or add syntax, but the documentation is forgotten, deferred, or updated superficially. Within several release cycles, the documentation becomes an untrustworthy artifact of historical compromises.

This protocol enforces a **Hard Pre-Merge Gate**: before any milestone or feature branch can be approved or merged into `main`, the author **MUST** run a Git-diff inventory against the trunk branch, evaluate the **Strict Cross-Update Trigger Matrix**, and verify that all affected specifications, guides, and wiki entries are fully synchronized within the exact same pull request.

---

## 2. Normative Invariants & Core Principles

All contributors and CI verification pipelines **MUST** enforce the following rules:

1. **The Pre-Merge Audit Command:**
   Before marking any milestone or task complete, the engineer/agent **MUST** execute:
   ```bash
   git diff main...HEAD --stat
   git diff main...HEAD -- crates/ libraries/
   ```
   and inventory all touched Rust files, diagnostic enums, AST nodes, opcodes, and CLI flags.
2. **Zero-Drift Enforcement Gate:**
   If a Rust source file in `crates/` or standard library Solix file in `libraries/solixlib/` is modified, the corresponding documentation tiers defined in the Trigger Matrix **MUST** be updated in the same branch. Commits modifying compiler functionality without matching documentation changes will be rejected by review.
3. **Link Integrity Mandate:**
   All markdown links across `docs/` **MUST** resolve to valid files. Broken links, dangling relative anchors, and references to deleted files are blocking defects.
4. **No Legacy Syntax in Documentation:**
   All new and updated documentation **MUST** use modern Solix v2.0 semantics:
   * Scope resolution `::` for static methods, static fields, and enums (`Math::sin`, `Color::Red`).
   * `.slxb` and `.slxarc` container extensions.
   * `manifest.toml` package metadata.
   * Canonical diagnostic error codes (`E0000`–`E0290`, `R0001`–`R0023`).

---

## 3. Strict Cross-Update Trigger Matrix

When `git diff main...HEAD` reveals modifications in any of the left-hand files or subsystems, the author **MUST** inspect and synchronize the corresponding documentation tiers:

| If `git diff main...HEAD` Modifies... | You MUST Inspect & Update in `docs/wiki/` | You MUST Inspect & Update in `docs/spec/` | You MUST Inspect & Update in `docs/guide/` |
| :--- | :--- | :--- | :--- |
| **`TokenKind` / Keywords (`tokens.rs`, `lexer.rs`)** | `wiki/keywords/` + `wiki/primitives/` + `wiki/errors/lexer-errors.md` | `spec/lexical-grammar.md` | Relevant guide in `guide/02..07` |
| **`ParsingError` / Grammar (`parsing.rs`, `parser.rs`)** | `wiki/errors/parser-errors.md` (`E0101`–`E0200`) | `spec/statements/*.md` | Relevant guide if syntax changed |
| **`BindingError` / Type Rules (`binding.rs`, `binder/`)** | `wiki/errors/binder-errors.md` (`E0201`–`E0290`) | `spec/type-system-and-semantics.md` & `spec/binder-and-monomorphization.md` | Relevant guide in `guide/02..08` |
| **`OpCode` / VM Execution (`opcodes.rs`, `vm/fast.rs`)** | `wiki/errors/runtime-errors.md` (`R0001`–`R0023`) | `spec/vm-isa-and-execution.md` & `spec/statements/*.md` bytecode listings | `guide/05-memory-management-arc-and-raii.md` (if ARC/memory changed) |
| **Compiler Phases / `.slxb` / `.slxarc` (`pipeline.rs`, `binary.rs`)** | `wiki/errors/emission-errors.md` | `spec/compilation-pipeline.md` & `spec/binary-formats-slxb-and-slxarc.md` | `guide/08-packages-dependencies-and-linking.md` |
| **CLI Commands / Flags (`cli.rs`, `commands/*.rs`)** | `wiki/commands/*.md` | `spec/package-solver-and-linking.md` | `guide/01-getting-started-and-toolchain.md` |
| **Standard Library (`libraries/solixlib/src/`)** | N/A | `spec/solixlib/*.md` | `guide/10-standard-library-tour.md` |
| **Workflow / Repo Conventions (`crates/` layout)** | N/A | `docs/internals/guidelines/*.md` | `guide/01-getting-started-and-toolchain.md` |

---

## 4. Verification & Audit Script

Before submitting a milestone for merge:
1. Run automated documentation link verification:
   ```bash
   python3 scripts/verify_doc_links.py
   ```
2. Verify that all documented Solix code snippets compile cleanly:
   ```bash
   cargo test --workspace --test documentation_conformance
   ```
3. Run `git status` to ensure all generated and updated documentation files are committed.
