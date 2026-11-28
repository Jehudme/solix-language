# Solix Packaging & Automated Release Guide

This guide covers building, packaging, and releasing the Solix programming language across Linux, macOS, and Windows.

---

## 1. Overview of Release Artifacts

Every Solix release includes binary installers and standalone portable archives:

| Platform | Format | Packaging Tool | Description |
|---|---|---|---|
| **Ubuntu / Debian** | `.deb` | `cargo-deb` | System package with `/usr/bin/solix` and companion tooling |
| **Fedora / RHEL** | `.rpm` | `cargo-generate-rpm` | Red Hat package installer |
| **Linux (Generic)** | `.tar.gz` | `tar` | Portable archive with precompiled standard library |
| **macOS (Generic)** | `.tar.gz` | `tar` | Portable archive for Apple Silicon (`aarch64`) & Intel (`x86_64`) |
| **Windows (Generic)**| `.zip` | `zip` | Portable archive with `solix.exe` unified CLI |
| **VS Code Extension**| `.vsix` | `vsce` | Language Server client, syntax highlighting, and snippets |

---

## 2. Generating Release Packages Locally

To build and package Solix locally using Cargo:

### Step 1: Build Workspace in Release Mode
```bash
cargo build --release --workspace
```

The compiled unified CLI binary is generated in `target/release/`:
- `solix` — Unified CLI compiler, runner, package manager, LSP (`solix lsp`), and DAP (`solix dap`)

### Step 2: Assemble Portable Distribution Archive
```bash
# Package portable distribution bundle
tar -czvf solix-x86_64-unknown-linux-gnu.tar.gz \
    -C target/release solix \
    -C ../libraries solixlib
```

---

## 3. Package Structure

Generated release packages distribute the following directory hierarchy:

```text
<install_prefix>/
├── bin/
│   └── solix            # Unified CLI compiler, runner, package tool, LSP, and DAP
└── share/
    ├── doc/solix/
    │   ├── LICENSE
    │   └── README.md
    └── solix/
        ├── solixlib/    # Bundled official standard library
        │   ├── manifest.yaml
        │   ├── bin/x86_64-unknown-linux-gnu/libsolixlib_native.so
        │   └── src/solix/...
        └── vscode/
            └── solix-vscode.vsix # Bundled editor extension
```

---

## 4. Automated Two-Workflow CI/CD Pipeline (`ci.yml` + `release.yml`)

Solix separates continuous integration (`dev`) from on-demand release promotion and multi-architecture packaging (`master`):

1. **Continuous Integration ([`.github/workflows/ci.yml`](../../.github/workflows/ci.yml))**:
   - Triggered on pull requests targeting **`dev`** (plus weekly security `schedule` and manual `workflow_dispatch`).
   - Uses [`.github/actions/solix-setup`](../../.github/actions/solix-setup/action.yml) for checkout-adjacent Rust toolchain setup, Cargo caching, Python/Node setup, and bundled package cache restoration, and reuses `staged-offline-docs` (`crates/solix-toolchain/assets/docs/`) built in `lint-and-docs` across downstream test, coverage, and benchmark jobs.
   - Enforces `rustfmt`, strict production Clippy lints, documentation synchronization (`scripts/ci/doc_sync_check.py`, `scripts/ci/verify_docs.py`, `scripts/ci/verify_doc_links.py`), offline `solix wiki` portal compilation (`scripts/docs/build_docs_site.py`), headless 3-layer VS Code/LSP/DAP extension verification and `.vsix` packaging (`editors/vscode/build/solix-language.vsix`), security checks (cached prebuilt `cargo-deny`, `cargo-audit`, `actionlint`, `zizmor`, and consolidated conditional `codeql`), `proptest` differential `-O0` vs `-O3` equivalence fuzzing, `.slxb`/`.slxp` corruption fuzzing, ARC/exception torture suites, `cargo-llvm-cov` workspace coverage gate (`>= 55%` lines), concise sensitive-path notices (`scripts/ci/pr_risk_audit.py` with automatic `sensitive` label removal once reverted), PR same-host and cross-language sticky benchmarks (`scripts/bench/solix_bench.py` after staging containerized `cross` native library builds), and 3-OS test matrices (`ubuntu-latest`, `macos-latest`, `windows-latest`) including cross-platform `run_ffi_suite.py`, `test_cli_lifecycle_matrix.py`, and headless VS Code LSP/DAP RPC tests.
2. **Unified On-Demand Release Pipeline ([`.github/workflows/release.yml`](../../.github/workflows/release.yml))**:
   - Triggered explicitly via **`workflow_dispatch`** with selectable version bump (`auto`, `patch`, `minor`, `major`), optional exact `X.Y.Z` override, and optional `dry_run` verification mode.
   - Verifies that `dev` HEAD passed CI, computes and applies version/changelog updates via `scripts/release/prepare_release_train.py`, runs cached post-bump workspace tests (setting `SOLIX_FFI_GIT_TAG=dev` for pre-tag scaffold verification alongside host-staged bundled packages built with `--target host`) and `cargo package` packaging checks, shares `staged-offline-docs` with downstream release build and benchmark jobs, fast-forwards `dev -> master` via `RELEASE_TOKEN`, tags `vX.Y.Z`, publishes crates to `crates.io` (including `solix-ffi`), compiles cached cross-platform `cargo-dist` installers + universal release-only bundled `solix-release` / `raylib-release` packages with merged `supported_platforms` metadata + `solix-language.vsix`, restores cached bundled packages or compiles host libraries prior to running same-host release and cross-language benchmarks via `scripts/bench/solix_bench.py`, publishes clean emoji-free release notes, mirrors docs to GitHub Wiki (`scripts/docs/sync_github_wiki.py`), and syncs `master -> dev`.
