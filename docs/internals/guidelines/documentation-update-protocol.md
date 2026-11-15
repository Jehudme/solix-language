# Mandatory Pre-Merge Git-Diff Documentation Audit Protocol

## Document Metadata

* **Guideline ID:** GL-0013
* **Scope:** Repository-Wide Pre-Merge Quality Gate (`crates/`, `libraries/`, `docs/`)
* **Status:** Final Implementation Standard
* **Audience:** Core Systems Engineers, Compiler Architects, PR Reviewers, Autonomous Agents
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)

---

## 1. Executive Summary & Purpose

A fundamental vulnerability of software engineering is asynchronous documentation drift: code is modified to add flags, change bytecode lowering, or refine type rules, but documentation is deferred, forgotten, or updated superficially. Within a few release cycles, documentation drifts from truth to historical fiction.

This protocol enforces a **Hard Pre-Merge Gate**: before any milestone branch or pull request can be merged into `main`, the author **MUST** run a Git-diff analysis against the base trunk branch, evaluate the **Strict Cross-Update Trigger Matrix**, and verify that all affected specifications, guides, and wiki entries are fully synchronized within the exact same pull request.

---

## 2. Normative Invariants & Core Principles

All contributors and CI verification pipelines **MUST** enforce the following rules:

1. **The Git-Diff Analysis Command:**
   Before marking any milestone complete or submitting a pull request, the author **MUST** execute:
   ```bash
   git diff main...HEAD --stat
   git diff main...HEAD --name-only -- crates/ libraries/
   ```
   to inventory all touched Rust crates, diagnostics, AST nodes, opcodes, and CLI flags.
2. **Zero-Drift Enforcement Gate:**
   If a Rust source file in `crates/` or standard library Solix file in `libraries/solixlib/` is modified, the corresponding documentation tiers defined in the Trigger Matrix **MUST** be updated in the same branch. Commits modifying functionality without matching documentation updates will be rejected at code review.
3. **Automated Documentation Sync Check:**
   Every pull request **MUST** pass the automated documentation synchronization verification script:
   ```bash
   python3 scripts/doc_sync_check.py --base main
   ```
4. **Link Integrity Mandate:**
   All markdown links across `docs/` **MUST** resolve to valid files. Broken links, dangling relative anchors, and references to deleted files are blocking defects verified via:
   ```bash
   python3 scripts/verify_doc_links.py
   ```
5. **No Single-Letter Variables Mandate (GL-0011):**
   All code snippets in updated documentation **MUST NOT** use single-letter variable names (`x`, `y`, `i`, `n`, `s`, `v`, `e`). Every identifier must be descriptive (`coord_x`, `coord_y`, `index`, `item_count`, `error_val`).
6. **No Legacy Syntax in Documentation:**
   All new and updated documentation **MUST** use modern Solix v2.0 semantics:
   * Scope resolution `::` for static methods, static fields, and enums (`Math::sin`, `Color::Red`).
   * `.slxb` and `.slxarc` container extensions.
   * `manifest.toml` package metadata.
   * Canonical diagnostic error codes (`E0000`–`E0290`, `R0001`–`R0023`).

---

## 3. Strict Cross-Update Trigger Matrix

When `git diff main...HEAD` reveals modifications in any left-hand subsystem, the author **MUST** inspect and synchronize the corresponding documentation tiers:

| Modified Subsystem / Path | Inspect & Update in `docs/wiki/` | Inspect & Update in `docs/spec/` | Inspect & Update in `docs/guide/` & `docs/architecture/` |
| :--- | :--- | :--- | :--- |
| **CLI Arguments & Flags**<br>`crates/solix-toolchain/src/cli.rs`<br>`crates/solix-toolchain/src/commands/` | `docs/wiki/commands/*.md` | `docs/spec/tools/cli/*.md` (`new.md`, `compile.md`, `build.md`, `run.md`, `info.md`, `install.md`, `uninstall.md`, `list.md`, `manifest.md`) | `docs/guide/01_getting_started.md` |
| **Lexer & Tokens**<br>`crates/solix-compilation/src/lexer/`<br>`crates/solix-compilation/src/tokens/` | `docs/wiki/keywords/`<br>`docs/wiki/primitives/`<br>`docs/wiki/errors/lexer-errors.md` | `docs/spec/lexical.md` | `docs/guide/02_basic_syntax.md` |
| **Parser & AST Nodes**<br>`crates/solix-compilation/src/parser/`<br>`crates/solix-compilation/src/ast/` | `docs/wiki/errors/parser-errors.md` (`E0101`–`E0200`) | `docs/spec/statements/*.md`<br>`docs/spec/expressions/*.md`<br>`docs/spec/declarations/*.md` | Relevant guides in `docs/guide/02..07` |
| **Type Binder & Semantics**<br>`crates/solix-compilation/src/binder/` | `docs/wiki/errors/binder-errors.md` (`E0201`–`E0290`) | `docs/spec/types.md`<br>`docs/spec/declarations/*.md` | `docs/architecture/binder_passes.md`<br>`docs/guide/04_oop_in_solix.md` |
| **Emitter & Bytecode Codegen**<br>`crates/solix-compilation/src/phases/emitter.rs` | N/A | `docs/spec/vm_isa.md`<br>`docs/spec/statements/*.md` (Register VM lowering listings) | `docs/architecture/pipeline.md` |
| **Register VM & Opcodes**<br>`crates/solix-runtime/src/vm/`<br>`crates/solix-common/src/bytecode.rs` | `docs/wiki/errors/runtime-errors.md` (`R0001`–`R0023`) | `docs/spec/vm_isa.md` | `docs/architecture/memory_model.md`<br>`docs/architecture/arc_internals.md`<br>`docs/architecture/exception_unwinding.md`<br>`docs/guide/05_memory_management_arc.md` |
| **Native Interop & FFI**<br>`crates/solix-runtime/src/native/` | N/A | `docs/spec/native_interop.md` | `docs/guide/09_native_plugins.md` |
| **Standard Library**<br>`libraries/solixlib/src/` | N/A | `docs/spec/libraries/solixlib/*.md` | `docs/guide/` (relevant guides) |
| **Project Manifest & Scaffolding**<br>`crates/solix-toolchain/assets/scripts/init_project.py`<br>`crates/solix-toolchain/assets/scripts/solix_toolbox.py` | `docs/wiki/commands/package-commands.md` | `docs/spec/tools/cli/new.md`<br>`docs/spec/tools/cli/manifest.md` | `docs/guide/01_getting_started.md`<br>`docs/guide/packaging_and_resources.md` |

---

## 4. Post-Milestone Synchronization Workflow

At the conclusion of each milestone:

```mermaid
flowchart TD
    A["git diff main...HEAD --name-only"] --> B["Identify Modified Subsystems"]
    B --> C["Lookup in Trigger Matrix"]
    C --> D["Update Affected Docs (Guide, Spec, Wiki, Architecture)"]
    D --> E["Run python3 scripts/doc_sync_check.py --base main"]
    E --> F["Run python3 scripts/verify_doc_links.py"]
    F --> G["Run cargo test & cargo clippy (Double Clean Pass)"]
    G --> H["Commit & Submit Milestone PR"]
```

### Step 1: Git-Diff Inventory
```bash
git diff main...HEAD --name-only
```
Review the list of changed files in `crates/` and `libraries/`.

### Step 2: Trigger Evaluation
Match each modified file against Section 3's Trigger Matrix. For example, if `crates/solix-toolchain/src/cli.rs` was modified:
- Ensure `docs/spec/tools/cli/` has up-to-date specs for all added/modified flags.
- Ensure `docs/wiki/commands/` reflects all added/modified flags.
- Ensure `docs/guide/01_getting_started.md` documents the practical developer usage.

### Step 3: Run Automated Sync Checker
```bash
python3 scripts/doc_sync_check.py --base main
```

### Step 4: Verify Markdown Links
```bash
python3 scripts/verify_doc_links.py
```

### Step 5: Double Verification Pass
Execute two consecutive clean test passes:
```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
