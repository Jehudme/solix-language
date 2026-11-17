# Milestone 062: Cross-Platform CI/CD Overhaul, Hermetic Bundled Libraries & Release Automation

## Document Metadata
* **Milestone ID:** MS-062
* **Target Version:** `v1.3.1`
* **Milestone Branch:** `milestone/062-cicd-overhaul-cross-platform-bundling-and-release-automation`
* **Status:** COMPLETED
* **Lead Architect:** Release Engineering, Build Tooling & Platform Maintainers
* **Target Delivery Date:** 2026-12-01
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * [GL-0001: Conventional Commits & Release Specification](../guidelines/authoring-commits.md)
  * [GL-0002: Trunk-Based & Milestone Branching Topology](../guidelines/branching-conventions.md)
  * [GL-0008: Milestone Planning & Delivery Specification](../guidelines/authoring-milestones.md)
  * [GL-0009: Test Engineering & Verification Matrices](../guidelines/authoring-test.md)
  * [GL-0010: Living Documentation & Drift Prevention](../guidelines/authoring-docs.md)
  * [GL-0013: Pre-Merge Git-Diff Documentation Audit Protocol](../guidelines/documentation-update-protocol.md)
  * [MS-040: Cross-Platform CI/CD, `release-plz`, `cargo-dist` & Bundled Libraries](040-cross-platform-ci-cd-release-plz-cargo-dist-bundled-libraries.md)
  * [MS-058: Dynamic Release Benchmarking & CI/CD Gate](058-dynamic-release-benchmarking-and-ci-cd.md)

---

## 1. Executive Summary & Forensic CI/CD Audit

Live inspection of GitHub Actions runs [`44004369200`](https://github.com/Jehudme/solix-language/actions/runs/44004369200), [`44004369291`](https://github.com/Jehudme/solix-language/actions/runs/44004369291), and [`43823786105`](https://github.com/Jehudme/solix-language/actions/runs/43823786105), repository workflow permissions, release artifacts, and local Cargo/Python tooling exposed **14 distinct CI/CD defects and bad practices** across `.github/workflows/`, `release-plz`, `cargo-dist`, `Cargo.toml`, and workspace integration tests.

Because of these defects:
- `release-plz` fails every run with HTTP `403` (`GitHub Actions is not permitted to create or approve pull requests`), yet reports **green** due to `continue-on-error: true`.
- `ci.yml` passes on `ubuntu-latest` while **failing 100% of runs on `macos-latest` and `windows-latest`** due to un-bootstrapped `raylib@5.5.0` packages in `test_breakout_real_inlay_hints`.
- `doc_sync_check.py` silently audits **`0` files** in CI and holds stale regex patterns that miss half the compiler subsystems.
- `release.yml` compiles Linux x86_64 `.so` native shared libraries once on Ubuntu and bakes them into Windows, macOS Intel, Apple Silicon, and ARM64 release binaries, while running heavy build jobs on every pull request.

This milestone establishes an authoritative architectural cleanup that makes `release-plz`, cross-platform CI, documentation drift gating, and multi-target binary/archive publishing reliable, fast, and hermetic.

---

### 1.1 Forensic Audit Inventory (`CICD-01` .. `CICD-14`)

| ID | Priority | Exact Location | Observed Defect / Root Cause | Architectural Decision |
| :--- | :---: | :--- | :--- | :--- |
| **CICD-01**<br>*(Release-plz `403`, Unused `RELEASE_TOKEN` & Un-Gated Parallel Run)* | **`P0`** | `.github/workflows/release-plz.yml` (lines 27-34, removed) | Repo Actions permissions (`can_approve_pull_request_reviews: false`) block `secrets.GITHUB_TOKEN` from creating PRs (`HTTP 403`), `GITHUB_TOKEN`-created PRs/tags do not trigger downstream CI or `release.yml`, `RELEASE_TOKEN` goes unused, and `release-plz.yml` runs un-gated in parallel with failing `ci.yml` runs. | Consolidate `release-plz` into [`ci.yml`](../../../.github/workflows/ci.yml) gated behind `needs: [lint-and-docs, test-matrix]` (delete standalone `release-plz.yml`), keep `master` push tests + branch `concurrency:` rules, and authenticate with `secrets.RELEASE_TOKEN`. |
| **CICD-02**<br>*(`continue-on-error: true` Masks Broken Releases)* | **`P0`** | `.github/workflows/release-plz.yml` (line 29, removed) | `continue-on-error: true` causes failed `release-plz` runs (`44004369291`, `43823786410`, etc.) to show ✓ `success` in GitHub Actions despite failing to open a release PR. | Remove `continue-on-error: true` so release automation failures fail loudly. |
| **CICD-03**<br>*(Split Publishing + Unchecked `2.0.0` Bump + `\|\| true` Publish)* | **`P0`** | `.github/workflows/release-plz.yml` (lines 30-31, removed)<br>[release-plz.toml](../../../release-plz.toml#L1-L7)<br>[release.yml](../../../.github/workflows/release.yml#L379-L398)<br>[solix_scaffold.py](../../../crates/solix-toolchain/assets/scripts/solix_scaffold.py#L239) | `release-plz.yml` only calls `release-pr` (never `release`), leaving `git_tag_enable = true` unused; `release.yml` manually runs `cargo publish ... \|\| true` swallowing real crates.io failures; internal crate changes trigger unexpected `2.0.0` `cargo-semver-checks` bumps that conflict with `solix_scaffold.py`'s `<=1.3.0` pin. | Unify versioning, tagging, and ordered crates.io publishing in `release-plz` (`command: release` + `release-pr`, `publish = true`); remove duplicate `publish-crates` (`\|\| true`) from `release.yml`; configure `semver_check = false` for internal crates (`semver_check = true` on `solix-ffi` only) or decouple `solix-ffi` until deliberate `2.0.0`, and relax scaffold constraint cleanly. |
| **CICD-04**<br>*(macOS & Windows CI Fail `test_breakout_real_inlay_hints`)* | **`P0`** | [ci.yml](../../../.github/workflows/ci.yml#L91-L93)<br>[lsp_tests.rs](../../../crates/solix-lsp/tests/lsp_tests.rs#L1072-L1117)<br>[workspace.rs](../../../crates/solix-lsp/src/workspace.rs#L280-L289) | `ci.yml` only runs `build_default_libraries.py` on Linux (`--install` writes `raylib@5.5.0` into `~/.solix/packages`), leaving macOS and Windows package stores empty. `test_breakout_real_inlay_hints` depends on ambient `~/.solix/packages` instead of a hermetic store and panics with `Got 0 hints`. | Make `test_breakout_real_inlay_hints` self-contained (resolve/install from repo bundled assets into an isolated `TempDir` store via `ClientModuleSolver::with_store_dir`) **and** build/verify default libraries across Linux, macOS, and Windows in `ci.yml`. |
| **CICD-05**<br>*(Linux x86_64 `.so` Baked Into Windows, macOS & ARM64 Releases)* | **`P0`** | [release.yml](../../../.github/workflows/release.yml#L21-L65)<br>[release.yml](../../../.github/workflows/release.yml#L209-L213)<br>[project.rs](../../../crates/solix-common/src/project.rs#L800-L819) | 1. `build-bundled-libraries` builds **only on `ubuntu-latest`** without `cargo-zigbuild`/cross-targets, packing Linux x86_64 `.so` libraries into `raylib.slxarc` and `solix.slxarc` and downloading them into Windows MSVC, macOS x86_64/arm64, Linux ARM64, and Linux MUSL `dist build` runners.<br>2. `project.rs:808-810` checks `lib/<OS>-<ARCH>` (`"macos-aarch64"`), never matching standard Rust triples (`aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, etc.). | Build host-native bundled `.slxarc` packages **inside each `build-local-artifacts` target runner** (with `cargo-zigbuild` on Linux ARM64/MUSL) before `dist build`, and fix `project.rs` to match canonical target triples (`OS`, `ARCH`, vendor/ABI aliases). |
| **CICD-06**<br>*(`build_default_libraries.py` Misses `.exe` on Windows & Overwrites Profiles in Release Staging)* | **`P1`** | [build_default_libraries.py](../../../scripts/build_default_libraries.py#L209-L214)<br>[release.yml](../../../.github/workflows/release.yml#L47-L58) | 1. `build_default_libraries.py` checks `target/debug/solix` without `.exe` on Windows.<br>2. `release.yml:47` omits `debug` profile (`--profiles release,debug`), and `cp .../*/*.slxarc staging-packages/` (`line 58`) collides on `raylib.slxarc`/`solix.slxarc` if multiple profiles exist. | Add `solix.exe` detection on `nt` in `build_default_libraries.py`, build `--profiles release,debug` consistently, and stage release archive attachments with profile+version+target disambiguation. |
| **CICD-07**<br>*(`doc_sync_check.py` Silent No-Op in CI & Stale `TRIGGER_MATRIX` Regexes)* | **`P1`** | [ci.yml](../../../.github/workflows/ci.yml#L48-L49)<br>[doc_sync_check.py](../../../scripts/doc_sync_check.py#L30-L131) | 1. `ci.yml` runs `doc_sync_check.py` with no `--base`, diffing clean runner working tree -> always `0 modified file(s)`.<br>2. `TRIGGER_MATRIX` still matches deleted pre-refactor paths (`src/binder/`, `src/(parser\|ast)/`, `src/phases/emitter\.rs`, `bytecode\.rs`) and omits `src/phases/optimizer/`. | Pass explicit `--base` in `ci.yml` (`origin/${{ github.base_ref }}` on PRs, `${{ github.event.before }}` on pushes with `fetch-depth: 0`), update `TRIGGER_MATRIX` to real module paths, and add `verify_doc_links.py` + `doc_sync_check.py --all-docs` to CI. |
| **CICD-08**<br>*(macOS & Windows Skip `--verify` & `examples/breakout` End-to-End Smoke Gate)* | **`P1`** | [ci.yml](../../../.github/workflows/ci.yml#L91-L115) | `--verify`, `solix list`, and `examples/breakout` debug/release compilation only run on `ubuntu-latest`; Windows/macOS CLI linking and execution are unverified. | Enable `--verify` (supporting available host compilers/engines or explicit `--engines solix,c`), `solix list`, and `examples/breakout` build verification on Windows and macOS runners too, plus `git config --global core.longpaths true` on Windows. |
| **CICD-09**<br>*(Release Workflow Runs Heavy Jobs on Every PR & Downloads Artifacts Twice)* | **`P1`** | [release.yml](../../../.github/workflows/release.yml#L9)<br>[release.yml](../../../.github/workflows/release.yml#L217-L222) | `on: pull_request:` triggers `build-bundled-libraries`, `build-vscode-extension`, and `plan` on every PR; `build-local-artifacts` downloads `artifacts-*` unnecessarily before building. | Remove `on: pull_request:` from `release.yml` (keep tag pushes + `workflow_dispatch`), and remove pre-build `download-artifact` from `build-local-artifacts`. |
| **CICD-10**<br>*(9 Cargo Manifest Warnings Across Workspace on Rust `1.100.0`)* | **`P2`** | [Cargo.toml](../../../Cargo.toml#L24-L35)<br>[crates/solix-common/Cargo.toml](../../../crates/solix-common/Cargo.toml)<br>[crates/solix-runtime/Cargo.toml](../../../crates/solix-runtime/Cargo.toml) | Cargo emits 9 manifest warnings: unused `workspace.package.readme`, unused `solix-ffi` workspace dep, unused `compact_str`/`unicode-ident` in `solix-common`, unused `tracing` in `solix-runtime`, and redundant `homepage` alongside `repository` across 7 crates. | Remove unused dependencies and redundant manifest fields so `cargo` produces zero manifest warnings. |
| **CICD-11**<br>*(Broken `Swatinem/rust-cache@v2` & Deprecated Node 20 Actions)* | **`P2`** | [ci.yml](../../../.github/workflows/ci.yml#L21-L90)<br>[release.yml](../../../.github/workflows/release.yml) | Every CI job fails cache restore (`400`) & save (`5xx`), and `@v4`/`@v5` actions log Node 20 deprecation warnings across all runners. | Upgrade to current Node-24-ready actions (`actions/checkout@v7`, `actions/setup-python@v7`, `actions/setup-node@v7`, `actions/upload-artifact@v7`, `actions/download-artifact@v8`, `Swatinem/rust-cache@v2.9.3` or clean conditional caching) and `cargo-dist` `0.34.0`. |
| **CICD-12**<br>*(Unsafe Quoting, Duplicate Release Call & Empty `announce` Job)* | **`P2`** | [release.yml](../../../.github/workflows/release.yml#L359-L411) | `host` leaves `$RELEASE_FILES`, `$PRERELEASE_FLAG`, and `$ANNOUNCEMENT_TITLE` unquoted and invokes `gh release create` redundantly with `dist host`; `announce` job has only `checkout` with zero steps. | Quote all shell expansions safely, rely on `gh release edit/upload` cleanly after `dist host`, and remove the dead `announce` job. |
| **CICD-13**<br>*(`benchmark-release` Self-Comparison on Tag Re-Run & No Release Build Check in `ci.yml`)* | **`P2`** | [release.yml](../../../.github/workflows/release.yml#L128-L141)<br>[compare_releases.py](../../../benchmarks/compare_releases.py#L64-L96) | `fetch_previous_release_baseline_from_github` downloads top `gh release list` tag without excluding `current_tag`, risking self-comparison on `workflow_dispatch`; `ci.yml` never builds `--release` or verifies `compare_releases.py`. | Exclude `current_tag` in `compare_releases.py` when finding previous release baseline, add `--release` build smoke test to `ci.yml`, and preserve pinned benchmark flags where applicable. |
| **CICD-14**<br>*(Strict Clippy Gaps & 30+ Stale Remote `release-plz-*` Branches)* | **`P3`** | [ci.yml](../../../.github/workflows/ci.yml#L45-L46)<br>GitHub remote `origin` | `ci.yml` does not run strict `--lib --bins` checks (`clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `clippy::min_ident_chars`), and dozens of abandoned `release-plz-*` branches remain on `origin`. | Enforce strict production Clippy lints on `--lib --bins` in `ci.yml` and prune orphaned `release-plz-*` branches. |

---

## 2. Architectural Design Decisions

```mermaid
flowchart LR
    PR["Pull Request or Push to master"] --> CI["ci.yml (Workflow 1: CI + Gated Release-plz)"]
    CI -->|"1. fmt, strict clippy, manifest clean, doc_sync --base"| DocsGate["Lint & Docs Gate (Pass/Fail)"]
    DocsGate -->|"2. build_default_libraries (All 3 OS) + tests + breakout + release smoke"| MatrixGate["3-OS Test Matrix"]
    MatrixGate -->|"3. push to master/main only (needs: lint-and-docs, test-matrix)"| RPlz["release-plz job (RELEASE_TOKEN, fail-loud)"]
    RPlz -->|"release-pr"| ReleasePR["Version Bump + CHANGELOG PR (Triggers CI cleanly)"]
    ReleasePR -->|"Merged to master"| RPlzRelease["release-plz release: ordered crates.io publish + tag vX.Y.Z"]
    RPlzRelease -->|"Push tag vX.Y.Z"| Release["release.yml (Workflow 2: Tag / Dispatch only)"]
    Release --> NativeTargets["Each target runner compiles native bundled .slxarc + dist build"]
    NativeTargets --> HostRelease["host: GitHub Release + Benchmark Table + VS Code VSIX"]
```

### 2.1 Two-Workflow Topology, `master` Test Gate & Authority Split (`RELEASE_TOKEN` + `release-plz`)
- **Consolidate From 3 Workflows to 2 (`ci.yml` + `release.yml`, Delete `release-plz.yml`):**
  - Merge `release-plz` directly into [`ci.yml`](../../../.github/workflows/ci.yml) as the final job (`needs: [lint-and-docs, test-matrix]`, `if: github.event_name == 'push' && (github.ref == 'refs/heads/master' || github.ref == 'refs/heads/main')`). Removing standalone `release-plz.yml` prevents un-gated `release-plz` runs when `master` CI fails on Windows or macOS.
  - Keep `release.yml` separate and triggered **only on version tags (`v*.*.*`) and `workflow_dispatch`** (never on pull requests), preserving read-only least privilege on PRs while letting `cargo-dist` manage its release matrix independently.
- **Keep Tests Running on Pushes to `master` + Add Concurrency Controls:**
  - `ci.yml` continues running on both `pull_request` (targeting `master`/`main`) and `push` to `master`/`main`, protecting direct merges and gating `release-plz` so broken commits never get tagged or published.
  - Configure workflow `concurrency:` (`cancel-in-progress: true` for PR branches, `cancel-in-progress: false` for `master`/`main`) to eliminate redundant queued runs without skipping `master` verification.
- **Single Authority for Versions, Tags, and Crates.io:** `release-plz` handles semantic bumps, `CHANGELOG.md` updates (`cliff.toml`), ordered crates.io publishing (`publish = true`, both `release` and `release-pr`), and GitHub tag creation (`git_tag_enable = true`) using `secrets.RELEASE_TOKEN`.
- **Single Authority for Binaries & GitHub Releases:** Pushing the `vX.Y.Z` tag (via `RELEASE_TOKEN`) triggers `release.yml`, which builds OS-native binaries, VS Code VSIX, benchmark release comparisons, and the finalized GitHub Release (`git_release_enable = false` stays in `release-plz.toml` so `cargo-dist` owns release creation cleanly).
- **Prevent Accidental `2.0.0` Cascade on `solix-ffi`:** Set `semver_check = false` at `[workspace]` level in `release-plz.toml` with `[[package]] name = "solix-ffi" semver_check = true` (or explicit `version_group` policy) so internal compiler/VM struct refactors do not force an unintentional `2.0.0` bump while keeping `solix-ffi` strictly SemVer-checked for native FFI users, and relax `solix_scaffold.py` to `>=1.1.3, <2.0.0`.

### 2.2 Per-Target Native Bundling + Canonical Triple Matching
- Fix `ArchivePackage` extraction in [`crates/solix-common/src/project.rs`](../../../crates/solix-common/src/project.rs#L800-L819) so `lib/<subdir>/` matches canonical Rust target triples (`x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-gnu`, `x86_64-pc-windows-msvc`) accurately rather than `std::env::consts::OS-ARCH`.
- Move `build_default_libraries.py --profiles release,debug` inside each `build-local-artifacts` matrix runner in `release.yml` (cross-compiling Linux `aarch64`/`musl` via `cargo-zigbuild` when needed, and compiling natively on Windows MSVC and macOS x86_64/arm64) so every distributed `solix` binary embeds the exact native `.so`/`.dll`/`.dylib` matching its platform.

---

## 3. Scope & Files Affected
* **GitHub Workflows:** [`.github/workflows/ci.yml`](../../../.github/workflows/ci.yml), [`.github/workflows/release-plz.yml`](../../../.github/workflows/release-plz.yml), [`.github/workflows/release.yml`](../../../.github/workflows/release.yml)
* **Release & Manifest Configs:** [`release-plz.toml`](../../../release-plz.toml), [`Cargo.toml`](../../../Cargo.toml), and workspace crate `Cargo.toml` manifests
* **Automation Scripts:** [`scripts/doc_sync_check.py`](../../../scripts/doc_sync_check.py), [`scripts/build_default_libraries.py`](../../../scripts/build_default_libraries.py), [`benchmarks/compare_releases.py`](../../../benchmarks/compare_releases.py), [`crates/solix-toolchain/assets/scripts/solix_scaffold.py`](../../../crates/solix-toolchain/assets/scripts/solix_scaffold.py)
* **Archive Loader & Hermetic Tests:** [`crates/solix-common/src/project.rs`](../../../crates/solix-common/src/project.rs), [`crates/solix-lsp/tests/lsp_tests.rs`](../../../crates/solix-lsp/tests/lsp_tests.rs), [`crates/solix-toolchain/tests/new_command_tests.rs`](../../../crates/solix-toolchain/tests/new_command_tests.rs)

---

## 4. Phased Implementation Plan & Priority Task Matrix

### 4.1 P0: Critical Blockers (Must Land First)
- [x] `[P0]` **`MS062-T01` (Consolidate to 2 Workflows, Fix `release-plz` Auth, Fail-Loud Behavior & Ordered Publishing — `CICD-01`, `CICD-02`, `CICD-03`):** Move `release-plz` into [`ci.yml`](../../../.github/workflows/ci.yml) gated on `needs: [lint-and-docs, test-matrix]` (push to `master`/`main` only, keeping `push` tests on `master` + branch `concurrency:` rules), delete standalone `release-plz.yml`, authenticate with `secrets.RELEASE_TOKEN`, remove `continue-on-error: true`, run `release` + `release-pr`, configure `publish = true` and `semver_check` scoping in `release-plz.toml`, update `solix_scaffold.py` FFI version range cleanly, and delete `publish-crates` (`|| true`) from `release.yml`.
- [x] `[P0]` **`MS062-T02` (Make `test_breakout_real_inlay_hints` Hermetic & Unblock macOS/Windows CI — `CICD-04`):** Update [`crates/solix-lsp/tests/lsp_tests.rs`](../../../crates/solix-lsp/tests/lsp_tests.rs) so Breakout LSP tests stage `raylib` into a `tempfile::TempDir` store via `ClientModuleSolver::with_store_dir` instead of relying on `~/.solix/packages`, and build/install default libraries across `ubuntu-latest`, `macos-latest`, and `windows-latest` in `ci.yml`.
- [x] `[P0]` **`MS062-T03` (Per-Target Native Bundling & Canonical Triple Extraction — `CICD-05`):** Update [`crates/solix-common/src/project.rs`](../../../crates/solix-common/src/project.rs#L800-L819) to match canonical target triples cleanly, and update `release.yml` so every `build-local-artifacts` target compiles/bundles its matching OS/arch native `.slxarc` libraries instead of copying Ubuntu x86_64 `.so` assets across Windows, macOS, and ARM64. *(Verified live in `v1.3.1` release workflow run `44019840669` across all 6 target triples and uploaded release assets.)*

### 4.2 P1: High Priority (Core Pipeline Correctness)
- [x] `[P1]` **`MS062-T04` (Windows `.exe` Resolution & Multi-Profile Release Staging — `CICD-06`):** Update [`scripts/build_default_libraries.py`](../../../scripts/build_default_libraries.py#L209-L214) to locate `solix.exe` on Windows, build both `release` and `debug` profiles cleanly, and name uploaded standalone `.slxarc` assets unambiguously (`<name>-<profile>-<target>@<version>.slxarc` or archive zip) without overwriting `debug` with `release`.
- [x] `[P1]` **`MS062-T05` (Enforce Real `doc_sync_check.py` Gating & Sync `TRIGGER_MATRIX` — `CICD-07`):** Update `ci.yml` (`fetch-depth: 0` + `--base`) so `doc_sync_check.py` diffs real PR/push commits instead of `0` files, update `TRIGGER_MATRIX` in [`scripts/doc_sync_check.py`](../../../scripts/doc_sync_check.py) to match current `src/phases/*` and `solix-common/src/bytecode/` layout, and add `verify_doc_links.py` + `doc_sync_check.py --all-docs` to CI.
- [x] `[P1]` **`MS062-T06` (Cross-Platform Verification & Release Build Gate — `CICD-08`, `CICD-09`, `CICD-13`):** Remove `on: pull_request:` from `release.yml`, remove redundant pre-build `download-artifact` from `build-local-artifacts`, run `examples/breakout` debug/release build + `--verify` checks across Linux/macOS/Windows + `core.longpaths true` on Windows in `ci.yml`, and add a `cargo build --release --bin solix` smoke step.

### 4.3 P2: Medium Priority (Hygiene, Action Upgrades & Benchmarks)
- [x] `[P2]` **`MS062-T07` (Eliminate All 9 Cargo Manifest Warnings — `CICD-10`):** Remove unused `compact_str`, `unicode-ident`, `tracing`, `solix-ffi` workspace dep, unused `readme`, and redundant `homepage` fields from [`Cargo.toml`](../../../Cargo.toml) and member crate manifests.
- [x] `[P2]` **`MS062-T08` (Upgrade GitHub Actions, Fix Cache & Harden `release.yml` Shell Steps — `CICD-11`, `CICD-12`, `CICD-13`):** Upgrade workflow actions to Node-24 native versions and `cargo-dist` `0.34.0`, fix/stabilize Rust caching, quote shell expansions and remove the empty `announce` job in `release.yml`, and update [`benchmarks/compare_releases.py`](../../../benchmarks/compare_releases.py) to exclude `current_tag` when resolving previous releases.

### 4.4 P3: Low Priority (Strict Lint Parity & Branch Cleanup)
- [x] `[P3]` **`MS062-T09` (Enforce Strict Production Clippy Lints & Clean Stale Branches — `CICD-14`):** Add `cargo clippy --workspace --exclude solix-tests --lib --bins --no-deps -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::min_ident_chars` to `ci.yml` and delete orphaned `origin/release-plz-*` remote branches.

---

## 5. Verification Matrix (`GL-0009`)
1. **Local & CI Workspace Parity:** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` (zero Cargo manifest warnings), strict `--lib --bins` Clippy, and `cargo test --workspace --all-targets` pass on `ubuntu-latest`, `macos-latest`, and `windows-latest` with zero pre-installed `~/.solix/packages` dependency.
2. **Documentation & Trigger Verification:** Deliberately testing `doc_sync_check.py --base master` detects compiler/VM edits without docs and passes once synchronized across all 265+ docs.
3. **Cross-Platform Triple Archive Unit Tests:** Unit test in `solix-common` verifying `ArchivePackage` selects native `.dll` / `.dylib` / `.so` from `lib/<rust-target-triple>/` and rejects wrong-architecture subdirectories cleanly.
4. **`release-plz` Dry-Run & Tag Chain Verification:** Confirm `release-plz` creates/updates the release PR using `RELEASE_TOKEN` with `workflow_run`/`pull_request` CI triggering cleanly on the generated PR, and confirm `release.yml` validates workflow YAML syntax cleanly.

---

## 6. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Verification | Tool / Workflow | Sign-off Status |
| :--- | :--- | :--- | :---: |
| **Zero Manifest & Clippy Warnings** | Strict production + workspace Clippy clean | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Verified |
| **3-OS CI Green** | `ubuntu-latest`, `macos-latest`, `windows-latest` all pass | `.github/workflows/ci.yml` | [x] Verified |
| **Real Doc Sync Gate** | Non-zero diff check & updated `TRIGGER_MATRIX` | `scripts/doc_sync_check.py`, `verify_docs.py`, `verify_doc_links.py` | [x] Verified |
| **Release-plz Green Without Masking** | Opens Release PR cleanly inside `ci.yml` only after 3-OS CI passes | `.github/workflows/ci.yml`, `release-plz.toml` | [x] Verified |
| **True Multi-Platform Bundles** | Native `.dll`/`.dylib`/`.so` per target triple | `.github/workflows/release.yml`, `crates/solix-common/src/project.rs` | [x] Verified |
