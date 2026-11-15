# Milestone 042: Complete Documentation Sync, Authoring Guidelines & Automated Drift Verification

## Document Metadata
* **Milestone ID:** MS-042
* **Target Version:** `v0.9.5`
* **Milestone Branch:** `milestone/042-documentation-sync-and-drift-verification`
* **Status:** DRAFT
* **Lead Architect:** @solix-lead
* **Target Delivery Date:** 2026-11-30
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md`
  * `docs/features/build-system.md`
  * `README.md`

---

## 1. Executive Summary & Objectives

Perform a comprehensive, file-by-file audit and synchronization of every documentation file in `docs/` and `README.md` against the actual codebase, update the documentation authoring guidelines, and implement an automated documentation drift verification gate (`scripts/verify_docs.py`) wired into GitHub Actions CI.

Following the major architectural upgrades in Milestones 036–041 (implicit numeric literal coercion, profile-aware package store `<name>-<profile>@<version>`, `[build]` manifest configuration, `solix_build.py` / `solix_scaffold.py`, `solix reinstall`, `solix clean`, `--metrics [<FORMAT>]`, `--no-verify`, `--deps-file`, and pre-bundled cross-platform installers), any stale CLI flag tables, deprecated script names (`solix_toolbox.py`, `init_project.py`), or outdated casts in documentation examples must be updated and permanently guarded against regression.

## 2. Scope & Crates Affected
* `crates/solix-common`: Inspected as source of truth for `ProjectManifest` (`manifest.toml`) schema and error codes.
* `crates/solix-compilation`: Inspected as source of truth for language grammar and type coercion rules.
* `crates/solix-runtime`: Inspected as source of truth for VM opcodes and FFI bridge contracts.
* `crates/solix-toolchain`: Inspected as source of truth (`src/cli.rs`) for all CLI subcommands and flags.
* `crates/solix-lsp`: Inspected as source of truth for LSP capabilities.
* `crates/solix-dap`: Inspected as source of truth for DAP debugger capabilities.
* `scripts/verify_docs.py`: Automated documentation drift and link verification script.
* `.github/workflows/ci.yml`: Wire `python3 scripts/verify_docs.py` into the CI lint gate.
* `docs/` & `README.md`: Full synchronization across all guides, references, and internal specs.

## 3. Prioritized Deliverables & Task Matrix

### 3.1 P0: Critical Blockers (Release Gate)
- [ ] `[P0]` **Synchronize All CLI Reference & Toolchain Docs (`README.md`, `docs/`):**
  Update every CLI reference table, tutorial, and command example across `README.md` and `docs/` to match `crates/solix-toolchain/src/cli.rs`:
  1. `solix compile`: `-d, --deps <SPEC>` (`<name>[-<profile>]@<version>`), `-l, --deps-file <PATH>`, `-f, --force-install`, `--metrics [<FORMAT>]` (`text` | `json`), `--no-verify` (replaces `--yolo`), `--inline-threshold <N>`.
  2. `solix run`: `--metrics [<FORMAT>]`, `--no-verify`, `--inline-threshold <N>`, `--gc-stats`, `--test`, `--filter`.
  3. `solix build [<PATH>] [--profile <PROFILE>]` and the `[build]` section in `manifest.toml`.
  4. `solix install [<PATH>] [--profile <PROFILE>] [-f, --force] [--bundled]` and the `$SOLIX_PKG_DIR/<name>-<profile>@<version>/` store layout.
  5. `solix reinstall [<SPEC>] [--all]` (replaces `solix sync`, using `source` in `registry.json`).
  6. `solix uninstall <SPEC>` (`<name>-<profile>@<version>`, `<name>@<version>`, `<name>-<profile>`, `<name>`).
  7. `solix list [--json] [-o, --output <FILE>]`.
  8. `solix info <TARGET> [--api] [--json]`.
  9. `solix clean [<PATH>] [--cache-only]`.
- [ ] `[P0]` **Synchronize Build System, Language & Library Docs (`docs/`):**
  1. Replace all non-historical references to `solix_toolbox.py` and `init_project.py` with `solix_build.py` and `solix_scaffold.py`.
  2. Document implicit numeric literal coercion and lossless widening (MS-037) in the language type-system documentation and remove obsolete `(uint8)` / `(float32)` literal casts from code examples.
  3. Document one-line cross-platform installation (`curl ... | sh` for Linux/macOS, `irm ... | iex` for Windows, Homebrew) and pre-bundled `solixlib` and `raylib` packages in `README.md` and getting-started guides.
- [ ] `[P0]` **Automated Documentation Drift Verifier (`scripts/verify_docs.py` & `.github/workflows/ci.yml`):**
  Create `scripts/verify_docs.py` and add it to `.github/workflows/ci.yml` so CI fails with exact `file:line` diagnostics if:
  1. Any active CLI subcommand in `crates/solix-toolchain/src/cli.rs` is missing from the documentation.
  2. Any non-historical doc (`README.md` or `docs/` excluding `docs/internals/milestones/`) contains removed/deprecated terms (`solix_toolbox.py`, `init_project.py`, `--metrics-json`, `--yolo`, `solix sync`).
  3. Any relative Markdown link (`[...](...)`) in `README.md` or `docs/` points to a non-existent file path.
  4. Any top-level `manifest.toml` section (`[package]`, `[build]`, `[dependencies]`, `[native_libraries]`) is undocumented.

### 3.2 P1: High Priority (Milestone Core)
- [ ] `[P1]` **Update Documentation Authoring Guidelines (`docs/internals/guidelines/`):** Update the documentation authoring specification to mandate running `python3 scripts/verify_docs.py` before closing any milestone or merging CLI/manifest/language changes.

### 3.3 P2: Medium Priority (Ergonomics & Polish)
- [ ] `[P2]` **Documentation Index & Navigation Table:** Ensure `docs/README.md` (or the root documentation index) links cleanly to `docs/features/build-system.md` and all current guides, specs, and milestones (`001` through `043`).

### 3.4 P3: Low Priority (Opportunistic Cleanups)
- [ ] `[P3]` **Code Block Syntax Highlighting Tags:** Ensure all Solix code blocks in Markdown files use consistent fencing and valid syntax.

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Zero warnings with `-D warnings` | `cargo build --workspace --all-targets` | [ ] Pending |
| **Automated Test Suite** | 100% pass across all unit, integration, and doc tests | `cargo test --workspace --all-targets` | [ ] Pending |
| **Clippy Static Analysis** | Zero warnings, zero `unwrap` / `expect` | `cargo clippy --workspace --all-targets -- -D warnings` | [ ] Pending |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [ ] Pending |
| **Documentation Sync** | `python3 scripts/verify_docs.py` exits `0` with 0 warnings | `python3 scripts/verify_docs.py` | [ ] Pending |

## 5. Risk Assessment & Mitigation Strategies
* **Risk 1:** Historical milestone files (`docs/internals/milestones/001-...` through `035-...`) legitimately mention historical names like `solix_toolbox.py` or `--yolo`.
  * *Mitigation:* Configure `scripts/verify_docs.py` to exclude historical milestone logs (`docs/internals/milestones/001-*` through `039-*`) from deprecated-term checks while still checking relative file links across the entire `docs/` tree.