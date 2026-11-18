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

## 4. Automated Two-Wave CI/CD Release Pipeline (`dev -> master`)

The CI/CD pipeline separates continuous integration (`dev`) from release cutting and multi-architecture binary distribution (`master`):

1. **Wave 1 — Continuous Integration, Security & Sensitive-Path Audits ([`.github/workflows/ci.yml`](../../.github/workflows/ci.yml))**:
   - Triggered on pull requests and pushes targeting `dev` (and validates every PR safely before landing).
   - Enforces `rustfmt`, strict production Clippy lints, documentation synchronization (`doc_sync_check.py`, `verify_docs.py`, `verify_doc_links.py`), offline `solix wiki` portal compilation (`scripts/build_docs_site.py`), security checks (`cargo-deny`, `cargo-audit`, `actionlint`, `zizmor`), non-blocking sensitive-path risk advisories (`scripts/pr_risk_audit.py`), and 3-OS test matrices (`ubuntu-latest`, `macos-latest`, `windows-latest`).
2. **Wave 2 — Release Train & Publication (`dev -> master`)**:
   - Direct contributor PRs into `master` are rejected by **`Validate Release PR Source`** (`scripts/check_release_pr_source.py`). Only the curated `dev -> master` release train merges into `master`.
   - When the maintainer merges the release PR to `master`, `release-plz` tags `vX.Y.Z`, updates changelogs, publishes workspace crates, and syncs the version bump back into `dev`.
3. **Multi-Platform Distribution ([`.github/workflows/release.yml`](../../.github/workflows/release.yml))**:
   - Triggered on `v*.*.*` tag pushes and manual `workflow_dispatch`.
   - Builds the static offline `solix wiki` portal via `scripts/build_docs_site.py`, compiles universal cross-platform `solix` and `raylib` `.slxarc` archives via `scripts/merge_bundled_packages.py`, embeds both together with `solix-language.vsix` into every `cargo-dist` installer, publishes GitHub Releases, and mirrors updated user docs to GitHub Wiki with version links.
