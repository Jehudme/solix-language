# Milestone 041: Full-Codebase Deep Audit, Logic Verification, Performance & Ergonomic Enhancements

## Document Metadata
* **Milestone ID:** MS-041
* **Target Version:** `v0.9.0`
* **Milestone Branch:** `milestone/041-full-codebase-audit-and-performance`
* **Status:** DRAFT
* **Lead Architect:** @solix-lead
* **Target Delivery Date:** 2026-11-28
* **Related Specifications:**
  * `docs/internals/specs/compilation/bytecode-emitter.md`
  * `docs/internals/specs/runtime/virtual-machine.md`
  * `docs/internals/specs/runtime/memory-management.md`

---

## 1. Executive Summary & Objectives

Execute a comprehensive, file-by-file engineering audit across every crate in the Solix workspace (`solix-common`, `solix-compilation`, `solix-runtime`, `solix-lsp`, `solix-dap`, `solix-toolchain`) prior to the `v1.0.0` freeze.

This milestone identifies and resolves edge-case soundness defects, eliminates any remaining panic hazards (`unwrap()` / `expect()` or unchecked UTF-8 string slicing on user input), verifies ARC reference-counting invariants across non-local control flow (`break`, `continue`, `return`, `throw`), implements high-impact compiler and VM performance optimizations, and adds targeted developer-experience commands (`solix clean` and enhanced `solix info` diagnostics).

## 2. Scope & Crates Affected
* `crates/solix-common`: Audit archive (`archive.rs`), bytecode (`bytecode.rs`), solver (`solver.rs`), and registry (`registry.rs`) parsers for malformed input resilience.
* `crates/solix-compilation`: Audit lexer, parser, 4-pass semantic binder, AST inliner, bytecode emitter (specifically scoped `DecRef` emission on `break`/`continue`/`return`), and peephole optimizer.
* `crates/solix-runtime`: Audit VM fast execution loop (`vm/fast.rs`), ARC virtual heap (`memory/heap.rs`), NaN-boxed value model (`value/model.rs`), FFI C-ABI bridge (`native/ffi.rs`), and AOT bytecode verifier (`verifier.rs`).
* `crates/solix-toolchain`: Add `solix clean` subcommand, enrich `solix info` output, and audit all CLI command error paths.
* `crates/solix-lsp`: Audit UTF-8 / UTF-16 position conversion safety across all handlers and add `PackageExportIndex` caching in `WorkspaceManager`.
* `crates/solix-dap`: Audit expression evaluator and stack/variable inspection for bounds safety.
* `docs/`: Record all audit findings, bug fixes, and performance benchmarks in the milestone report.

## 3. Prioritized Deliverables & Task Matrix

### 3.1 P0: Critical Blockers (Release Gate)
- [ ] `[P0]` **Compiler & Emitter Soundness Audit (`crates/solix-compilation`):**
  1. Audit `emitter/` for register lifetime and ARC `DecRef` emission on early loop exits (`break`, `continue`) and early `return` statements inside nested blocks containing reference-typed local variables, ensuring zero memory leaks (`verify_clean_memory()` passes 100%).
  2. Audit short-circuit logical operators (`&&`, `||`), ternary expressions (`? :`), and `try`/`catch` blocks for register clobbering or unbalanced refcounts.
  3. Audit parser and binder passes to ensure malformed syntax or unresolved types never trigger an unhandled panic (`unwrap()` / index out-of-bounds).
- [ ] `[P0]` **Runtime VM, Heap & FFI Safety Audit (`crates/solix-runtime`):**
  1. Audit `vm/fast.rs` and `verifier.rs` for division/modulo by zero (`i32::MIN / -1` overflow), out-of-bounds array/string indexing, and stack overflow protection.
  2. Audit `native/ffi.rs` (`solix_vm_read_field_u64`, `solix_vm_write_field_u64`, `solix_vm_alloc_object`, `solix_vm_array_get_u64`, `solix_vm_array_set_u64`) to guarantee strict alignment, type-tag fidelity, and memory safety.
- [ ] `[P0]` **UTF-8 Multibyte Slicing Safety in LSP & DAP (`crates/solix-lsp`, `crates/solix-dap`):** Audit every string slice in `solix-lsp` and `solix-dap` where LSP column offsets are converted to byte indices, ensuring multibyte UTF-8 characters in comments or string literals never cause a byte-boundary slice panic.

### 3.2 P1: High Priority (Milestone Core)
- [ ] `[P1]` **High-Impact Compiler & Runtime Performance Optimizations:**
  1. Cache resolved external `PackageExportIndex` entries in `WorkspaceManager` (`crates/solix-lsp/src/workspace.rs`) keyed by `(package_name, profile, version, archive_mtime)` so `solix-lsp` does not re-read and deserialize `.slxarc` archives from disk on every keystroke!
  2. Add `#[inline(always)]` to hot register/value decoding helpers in `crates/solix-runtime/src/vm/fast.rs` and pre-allocate `Vec` / `HashMap` capacities in hot compiler passes (`SymbolTable`, `BytecodeEmitter`).
- [ ] `[P1]` **Add `solix clean` Subcommand (`crates/solix-toolchain/src/cli.rs`, `src/commands/clean.rs`):** Implement `solix clean [<PATH>] [--cache-only]` to safely remove a project's `build/` directory (or only `build/.cache/` when `--cache-only` is passed) and report the number of removed files and reclaimed disk space.
- [ ] `[P1]` **Enrich `solix info` Diagnostics (`crates/solix-toolchain/src/commands/info.rs`):** Display the package's `build_profile` (`debug`, `release`, `test`), `source` directory (if installed from local source), archive file size, bundled native target platforms inside the `.slxarc` (`lib/<os>-<arch>/`), and exported symbol counts.

### 3.3 P2: Medium Priority (Ergonomics & Polish)
- [ ] `[P2]` **Archive & Registry Corruption Resilience (`crates/solix-common`):** Ensure truncated `.slxarc` / `.slxb` files or corrupted `registry.json` files return descriptive `SolixError` diagnostics indicating the file path and failure reason.

### 3.4 P3: Low Priority (Opportunistic Cleanups)
- [ ] `[P3]` **Dead Code & Unused Import Sweep:** Eliminate any dead internal helper functions or redundant clones across all workspace crates.

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Zero warnings with `-D warnings` | `cargo build --workspace --all-targets` | [ ] Pending |
| **Automated Test Suite** | 100% pass across all unit, integration, and doc tests | `cargo test --workspace --all-targets` | [ ] Pending |
| **Clippy Static Analysis** | Zero warnings, zero `unwrap` / `expect` | `cargo clippy --workspace --all-targets -- -D warnings` | [ ] Pending |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [ ] Pending |
| **Documentation Sync** | Specs and guides updated to GL-0010 | GL-0010 audit checklist | [ ] Pending |

## 5. Risk Assessment & Mitigation Strategies
* **Risk 1:** Caching `PackageExportIndex` in `WorkspaceManager` could serve stale symbols if a user rebuilds and reinstalls a dependency while the IDE is open.
  * *Mitigation:* Include the `.slxarc` file's last-modified timestamp (`mtime`) and file size in the cache key so reinstalling a package automatically invalidates its cache entry.
* **Risk 2:** Emitting extra `DecRef` instructions on `break` / `continue` could double-free a variable declared outside the loop.
  * *Mitigation:* Only emit `DecRef` for reference variables whose lexical scope depth is strictly greater than the target loop's outer scope depth (i.e., variables declared inside the loop body before the `break` / `continue`).