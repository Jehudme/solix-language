# Milestone 043: Production Release `v1.0.0` Readiness & Publication

## Document Metadata
* **Milestone ID:** MS-043
* **Target Version:** `v1.0.0`
* **Milestone Branch:** `milestone/043-release-v1.0.0`
* **Status:** COMPLETED
* **Lead Architect:** @solix-lead
* **Target Delivery Date:** 2026-12-01
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md`
  * `CHANGELOG.md`
  * `Cargo.toml`

---

## 1. Executive Summary & Objectives

Complete the final production release verification, bump the Solix workspace, crates, and VS Code extension to **`v1.0.0`**, rebuild and bundle the production `debug` and `release` archives for `libraries/solixlib` and `libraries/raylib`, verify `examples/breakout`, execute the full quality and documentation verification gates, and finalize `CHANGELOG.md` for the official **`v1.0.0`** release.

## 2. Scope & Crates Affected
* `Cargo.toml` & `crates/*/Cargo.toml`: Bump workspace and internal crate versions to `1.0.0`.
* `editors/vscode/package.json`: Bump VS Code extension version to `1.0.0`.
* `libraries/solixlib`: Rebuild and test `debug` (`solix-debug@2.0.0`) and `release` (`solix-release@2.0.0`) archives.
* `libraries/raylib`: Rebuild and test `debug` (`raylib-debug@5.5.0`) and `release` (`raylib-release@5.5.0`) archives.
* `examples/breakout`: Rebuild and verify `debug` and `release` archives against the `v1.0.0` toolchain.
* `CHANGELOG.md` & `docs/`: Finalize `v1.0.0` release notes and mark Milestones `036`–`043` as `VALIDATED` / `CLOSED`.

## 3. Prioritized Deliverables & Task Matrix

### 3.1 P0: Critical Blockers (Release Gate)
- [x] `[P0]` **Workspace & Extension Version Bump to `1.0.0` (`Cargo.toml`, `editors/vscode/package.json`):**
  1. Set `[workspace.package] version = "1.0.0"` in root `Cargo.toml` and update any internal crate dependency version pins across `crates/*/Cargo.toml` to `"1.0.0"`.
  2. Set `"version": "1.0.0"` in `editors/vscode/package.json`.
  3. Verify `solix --version` outputs `solix 1.0.0`.
- [x] `[P0]` **Full Rebuild & Verification of Bundled Default Libraries (`solixlib` & `raylib`):**
  1. Run `python3 scripts/build_default_libraries.py` to build and stage fresh `release` and `debug` `.slxarc` archives into `crates/solix-toolchain/assets/packages/`.
  2. Run `solix build libraries/solixlib --profile test` and `solix run --test --gc-stats libraries/solixlib/build/test/solix.slxarc`, verifying 100% test pass and 0 memory leaks.
  3. Run `solix build libraries/raylib --profile test` and `solix run --test --gc-stats libraries/raylib/build/test/raylib.slxarc`, verifying 100% test pass and 0 memory leaks.
- [x] `[P0]` **End-to-End Verification of `examples/breakout`:**
  Build `examples/breakout` in both `debug` (`solix build examples/breakout --profile debug`) and `release` (`solix build examples/breakout --profile release`) profiles and verify clean compilation and archive packaging.

### 3.2 P1: High Priority (Milestone Core)
- [x] `[P1]` **Execute Complete 5-Pillar DoD & Documentation Gate:**
  1. `cargo build --workspace --all-targets`
  2. `cargo test --workspace --all-targets`
  3. `cargo clippy --workspace --all-targets -- -D warnings`
  4. `cargo fmt --all -- --check`
  5. `python3 scripts/verify_docs.py`
- [x] `[P1]` **Finalize `CHANGELOG.md` for `v1.0.0`:** Author the comprehensive `[1.0.0]` release entry in `CHANGELOG.md` covering:
  * **Compiler & Language:** 4-pass semantic binder, context-typed numeric literal coercion & safe widening, AST inliner, peephole optimizer, AOT bytecode verifier.
  * **Runtime & VM:** 64-bit NaN-boxed register VM, deterministic ARC with POD fast-path, generational handles, zero-marshal FFI bridge.
  * **Package Manager & Build System:** Profile-aware `<name>-<profile>@<version>` store, SemVer + SHA-256 solver with automatic `release` $\to$ `debug` fallback, declarative `[build]` manifest section, `solix_build.py` engine with hermetic `build/.cache/` isolation.
  * **First-Party Ecosystem:** `solixlib` (`2.0.0`), `raylib` (`5.5.0` / `6.2` generic Two-Pass IR bindings), and `examples/breakout`.
  * **IDE & Tooling:** `solix-lsp` (zero-lag `.` / `::` and hierarchical `import` completion), `solix-dap` debugger, VS Code extension, and cross-platform installers with pre-bundled default libraries.

### 3.3 P2: Medium Priority (Ergonomics & Polish)
- [x] `[P2]` **Milestone Closure Audit (`docs/internals/milestones/`):** Ensure all milestone documents from `001` through `043` have complete DoD sign-off matrices and accurate status fields.

### 3.4 P3: Low Priority (Opportunistic Cleanups)
- [x] `[P3]` **Final Git Status Cleanliness Check:** Verify `git status` has zero untracked temporary artifacts or stray build caches outside `.gitignore`.

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Zero warnings with `-D warnings` | `cargo build --workspace --all-targets` | [x] VALIDATED |
| **Automated Test Suite** | 100% pass across all unit, integration, and doc tests | `cargo test --workspace --all-targets` | [x] VALIDATED |
| **Clippy Static Analysis** | Zero warnings, zero `unwrap` / `expect` | `cargo clippy --workspace --all-targets -- -D warnings` | [x] VALIDATED |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [x] VALIDATED |
| **Documentation Sync** | `python3 scripts/verify_docs.py` exits `0` | `python3 scripts/verify_docs.py` | [x] VALIDATED |

## 5. Risk Assessment & Mitigation Strategies
* **Risk 1:** Bumping the workspace version to `1.0.0` could cause CLI version assertion tests expecting `0.1.0` to fail.
  * *Mitigation:* Use `env!("CARGO_PKG_VERSION")` in version tests or update any hardcoded version strings in test assertions during task 3.1.
* **Risk 2:** Stale `.slxarc` archives in `crates/solix-toolchain/assets/packages/` could be packaged into the `1.0.0` binary if `build_default_libraries.py` is not run after the final compiler changes.
  * *Mitigation:* Always run `python3 scripts/build_default_libraries.py` after compiling the `1.0.0` toolchain and before running the final `cargo test --workspace` suite.