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

The release workflow triggers on semantic tag pushes (e.g. `v2.0.0`):

1. **Build Matrix**: Concurrently compiles release binaries across `ubuntu-latest`, `macos-latest`, and `windows-latest` using `cross` / `cargo`.
2. **Packaging**: Generates native `.deb`, `.rpm`, `.tar.gz`, and `.zip` archives.
3. **Asset Publishing**: Publishes all artifacts, checksums (SHA-256), and the VS Code `.vsix` extension to GitHub Releases automatically.
