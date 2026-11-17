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

## 4. Automated CI/CD Release Pipeline

The two-workflow CI/CD pipeline separates continuous verification from binary distribution:

1. **Continuous Verification & `release-plz` ([`.github/workflows/ci.yml`](../../.github/workflows/ci.yml))**:
   - Enforces formatting, strict production Clippy lints, documentation synchronization (`doc_sync_check.py`, `verify_docs.py`, `verify_doc_links.py`), hermetic workspace tests, `--verify` correctness checks, and `examples/breakout` debug/release compilation across `ubuntu-latest`, `macos-latest`, and `windows-latest`.
   - On green `master` pushes, runs `release-plz` (`release` + `release-pr`) via `RELEASE_TOKEN` to publish crates in topological order, tag releases, and open/update changelog PRs.
2. **Multi-Platform Distribution ([`.github/workflows/release.yml`](../../.github/workflows/release.yml))**:
   - Triggered on `v*.*.*` tag pushes and manual `workflow_dispatch`.
   - Compiles target-native `solix` and `raylib` archives across `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, `x86_64-apple-darwin`, and `aarch64-apple-darwin`, merges all architectures via `scripts/merge_bundled_packages.py` into universal cross-platform `solix` and `raylib` `.slxarc` containers (`release` and `debug` profiles), embeds those universal packages plus `solix-language.vsix` into every `cargo-dist` installer, generates release-to-release benchmark comparisons, and finalizes the GitHub Release.
