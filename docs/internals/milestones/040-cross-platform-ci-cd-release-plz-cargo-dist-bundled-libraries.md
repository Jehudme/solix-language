# Milestone 040: Cross-Platform CI/CD, `release-plz`, `cargo-dist`, Auto-Bundled Libraries & `v0.1.0` Test Release

## Document Metadata
* **Milestone ID:** MS-040
* **Target Version:** `v0.1.0`
* **Milestone Branch:** `milestone/040-cicd-release-plz-cargo-dist-and-bundled-libs`
* **Status:** COMPLETED
* **Lead Architect:** @solix-lead
* **Target Delivery Date:** 2026-11-25
* **Related Specifications:**
  * `docs/internals/specs/toolchain/ci-cd-and-distribution.md`
  * `docs/features/build-system.md`

---

## 1. Executive Summary & Objectives

Establish a production-grade, automated CI/CD and multi-platform release pipeline on GitHub Actions using `release-plz` (automated semantic version bumps, `CHANGELOG.md` generation via `git-cliff`, and Release PR management) and `cargo-dist` (cross-platform binary archives and one-line installers across Linux distros, macOS, and Windows), complete with automatic discovery, compilation, and bundling of all first-party libraries in `libraries/*` (`solixlib`, `raylib`, and any future library added to `libraries/`).

When a user installs Solix on any platform (via the `curl | sh` installer on Ubuntu, Debian, Fedora, Arch, openSUSE, or macOS; the `irm | iex` PowerShell installer on Windows; Homebrew; or release archives), the bundled default libraries (`solix-release@2.0.0`, `raylib-release@5.5.0`) are packaged inside `<exe_dir>/assets/packages/` and automatically bootstrapped into `$SOLIX_PKG_DIR` on first run. This milestone validates the entire release and installer pipeline using version **`v0.1.0`**.

## 2. Scope & Crates Affected
* `crates/solix-common`: Not affected.
* `crates/solix-compilation`: Not affected.
* `crates/solix-runtime`: Not affected.
* `crates/solix-toolchain`:
  * `build.rs` and `src/assets.rs`: Stage `assets/packages/` alongside the compiled `solix` binary and implement `ensure_bundled_packages_installed(pkg_dir: &Path)` for automatic first-run package store bootstrap (plus `solix install --bundled`).
  * `src/commands/install.rs`: Add `--bundled` flag to explicitly install or repair bundled default libraries.
* `scripts/build_default_libraries.py`: Topological discovery and build script for all `libraries/*` packages.
* `.github/workflows/`: `ci.yml`, `release-plz.yml`, and `release.yml`.
* Root configuration files: `Cargo.toml` (`v0.1.0` baseline + `[workspace.metadata.dist]`), `release-plz.toml`, `cliff.toml`, and `dist-workspace.toml`.
* `docs/`: Document CI/CD architecture and one-line cross-platform installation commands.

## 3. Prioritized Deliverables & Task Matrix

### 3.1 P0: Critical Blockers (Release Gate)
- [x] `[P0]` **Dynamic Default Library Builder (`scripts/build_default_libraries.py`):** Create a Python script that scans `libraries/*` for directories containing `manifest.toml`, topologically sorts them by their `[dependencies]` table (so `solixlib` is always built and installed before dependent libraries like `raylib`), builds each library in `release` (and optionally `debug`) profile, and stages the resulting `.slxarc` archives into `crates/solix-toolchain/assets/packages/<name>-<profile>@<version>/<name>.slxarc` along with a `bundled_manifest.json` index. Adding a new library under `libraries/` in the future requires zero workflow changes.
- [x] `[P0]` **First-Run Auto-Bootstrap of Bundled Libraries (`crates/solix-toolchain/build.rs`, `src/assets.rs`, `src/commands/install.rs`):**
  1. Update `crates/solix-toolchain/build.rs` to copy `crates/solix-toolchain/assets/packages/` (if present) into `<target_dir>/assets/packages/`.
  2. Implement `ensure_bundled_packages_installed(pkg_dir: &Path)` in `crates/solix-toolchain/src/assets.rs` and invoke it automatically when initializing/accessing the default package store (unless `SOLIX_NO_AUTO_BOOTSTRAP=1` is set): for each `<name>-<profile>@<version>/<pkg>.slxarc` in `<exe_dir>/assets/packages/`, if that `(name, profile, version)` is not yet registered in `registry.json`, install it silently into `pkg_dir`.
  3. Add `solix install --bundled` (with `--force` support) so users can manually re-extract bundled standard libraries at any time.
- [x] `[P0]` **Cross-Platform Continuous Integration Workflow (`.github/workflows/ci.yml`):**
  Configure GitHub Actions triggered on `push` to `main`/`master` and `pull_request`:
  1. **Lint & Format Gate:** `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
  2. **Multi-OS Test & Library Build Matrix (`ubuntu-latest`, `macos-latest`, `windows-latest`):**
     * Install Python 3, CMake, Zig (`goto-bus-stop/setup-zig@v2`), and Linux X11/OpenGL development headers on Ubuntu.
     * Run `cargo test --workspace --all-targets`.
     * Run `python3 scripts/build_default_libraries.py` and run `solix run --test` on `solixlib` and `raylib`.

### 3.2 P1: High Priority (Milestone Core)
- [x] `[P1]` **Automated Versioning & Changelog with `release-plz` (`release-plz.toml`, `cliff.toml`, `.github/workflows/release-plz.yml`):**
  1. Configure `release-plz.toml` with `publish = false` (since `solix` is distributed as prebuilt binaries rather than crates.io source crates), `git_tag_enable = true`, and `git_release_enable = false` (allowing `cargo-dist` / `release.yml` to publish the GitHub Release with binary installers when the `v*` tag is pushed).
  2. Configure `cliff.toml` for clean Conventional Commits changelog generation in `CHANGELOG.md`.
  3. Create `.github/workflows/release-plz.yml` triggered on pushes to the default branch.
- [x] `[P1]` **Cross-Platform Installers & Release Workflow (`dist-workspace.toml`, `.github/workflows/release.yml`):**
  1. Set workspace version in `Cargo.toml` to `0.1.0` for the test release baseline.
  2. Configure `dist-workspace.toml` for targets `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, and `x86_64-pc-windows-msvc` with installers `shell` (universal Linux distros & macOS `curl | sh`), `powershell` (Windows `irm | iex`), and `homebrew`, including `crates/solix-toolchain/assets` in every archive.
  3. Create `.github/workflows/release.yml` triggered on `v*.*.*` tags (and `workflow_dispatch`) that builds all `libraries/*` via `python3 scripts/build_default_libraries.py`, packages the `solix` toolchain binaries with bundled `assets/packages/`, generates the platform installer scripts and SHA-256 checksums, and publishes the GitHub Release.
- [x] `[P1]` **Integration Test for Bundled Package Bootstrap (`crates/solix-toolchain/tests/`):** Add an automated integration test verifying that running `solix list` or `solix install --bundled` against a clean temporary `SOLIX_PKG_DIR` automatically installs all bundled `.slxarc` archives from `assets/packages/`.

### 3.3 P2: Medium Priority (Ergonomics & Polish)
- [x] `[P2]` **VS Code Extension Packaging Check in CI (`.github/workflows/ci.yml`):** Validate `editors/vscode/package.json` syntax in CI so editor tooling stays synchronized with toolchain releases.

### 3.4 P3: Low Priority (Opportunistic Cleanups)
- [x] `[P3]` **Cargo Cache Optimization in GitHub Actions:** Configure `Swatinem/rust-cache@v2` across CI and release workflows to minimize build times.

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Zero warnings with `-D warnings` | `cargo build --workspace --all-targets` | [x] Completed |
| **Automated Test Suite** | 100% pass across all unit, integration, and doc tests | `cargo test --workspace --all-targets` | [x] Completed |
| **Clippy Static Analysis** | Zero warnings, zero `unwrap` / `expect` | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Completed |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [x] Completed |
| **Documentation Sync** | Specs and guides updated to GL-0010 | GL-0010 audit checklist | [x] Completed |

## 5. Risk Assessment & Mitigation Strategies
* **Risk 1:** `release-plz` by default attempts to run `cargo publish` to crates.io, which fails for private/binary-distributed workspaces without a `CARGO_REGISTRY_TOKEN`.
  * *Mitigation:* Explicitly set `publish = false` under `[workspace]` in `release-plz.toml` so `release-plz` operates purely on Git version bumps, `CHANGELOG.md`, Release PRs, and Git tags.
* **Risk 2:** First-run auto-bootstrap could slow down CLI startup if it parses archives on every invocation.
  * *Mitigation:* Check if `$SOLIX_PKG_DIR/.bundled_bootstrapped_<version>` marker file exists (or do a fast filename check against `registry.json`); if already bootstrapped for the current toolchain version, skip directory scanning in $< 0.1\text{ ms}$.