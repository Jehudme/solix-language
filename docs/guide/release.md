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
| **Windows (Generic)**| `.zip` | `zip` | Portable archive with `solix.exe`, `solix-lsp.exe`, `solix-dbg.exe` |
| **VS Code Extension**| `.vsix` | `vsce` | Language Server client, syntax highlighting, and snippets |

---

## 2. Generating Release Packages Locally

To build and package Solix locally using Cargo:

### Step 1: Build Workspace in Release Mode
```bash
cargo build --release --workspace
```

The compiled binaries are generated in `target/release/`:
- `solix` — Unified CLI compiler, runner, and package manager
- `solix-lsp` — Language Server Protocol daemon
- `solix-dbg` — Interactive command-line debugger

### Step 2: Assemble Portable Distribution Archive
```bash
# Package portable distribution bundle
tar -czvf solix-x86_64-unknown-linux-gnu.tar.gz \
    -C target/release solix solix-lsp solix-dbg \
    -C ../libraries solixlib
```

---

## 3. Package Structure

Generated release packages distribute the following directory hierarchy:

```text
<install_prefix>/
├── bin/
│   ├── solix            # CLI compiler, runner, and package tool
│   ├── solix-lsp        # Language server daemon
│   └── solix-dbg        # Interactive bytecode debugger
└── share/
    ├── doc/solix/
    │   ├── LICENSE
    │   └── README.md
    └── solix/
        ├── solixlib/    # Bundled official standard library
        │   ├── manifest.toml
        │   ├── lib/libsolixlib_native.so
        │   └── src/solix/...
        └── vscode/
            └── solix-vscode.vsix # Bundled editor extension
```

---

## 4. Automated Two-Workflow CI/CD Pipeline (`ci.yml` + `release.yml`)

Solix separates continuous integration (`dev`) from on-demand release promotion and multi-architecture packaging (`master`):

1. **Continuous Integration ([`.github/workflows/ci.yml`](../../.github/workflows/ci.yml))**:
   - Triggered on pull requests and pushes targeting **`dev`**.
   - Enforces `rustfmt`, strict production Clippy lints, documentation synchronization (`doc_sync_check.py`, `verify_docs.py`, `verify_doc_links.py`), offline `solix wiki` portal compilation (`scripts/build_docs_site.py`), cached VS Code extension compilation, security checks (`cargo-deny`, `cargo-audit`, `actionlint`, `zizmor`), concise sensitive-path notices (`scripts/pr_risk_audit.py`), and 3-OS test matrices (`ubuntu-latest`, `macos-latest`, `windows-latest`).
2. **Unified On-Demand Release Pipeline ([`.github/workflows/release.yml`](../../.github/workflows/release.yml))**:
   - Triggered explicitly via **`workflow_dispatch`** with selectable version bump (`auto`, `patch`, `minor`, `major`), optional exact `X.Y.Z` override, and optional `dry_run` verification mode.
   - Verifies that `dev` HEAD passed CI, computes and applies version/changelog updates via `scripts/prepare_release_train.py`, runs post-bump workspace tests and `cargo package --no-verify` publishability dry-runs, fast-forwards `dev -> master` via `RELEASE_TOKEN`, tags `vX.Y.Z`, publishes crates to `crates.io`, compiles cached cross-platform `cargo-dist` installers + universal bundled `solixlib` / `raylib` packages + `solix-language.vsix`, runs same-host release benchmarks, publishes clean emoji-free release notes, mirrors docs to GitHub Wiki, and syncs `master -> dev`.
