# Solix CLI & Toolchain Specification

The Solix toolchain provides a unified command-line executable (`solix`) designed for compiling, building, executing, scaffolding, inspecting, and managing Solix projects and packages across Linux, macOS, and Windows.

---

## Table of Contents

1. [Architecture & Philosophy](#architecture--philosophy)
2. [Global Invocation Syntax](#global-invocation-syntax)
3. [Subcommand Matrix](#subcommand-matrix)
4. [Environment Variables](#environment-variables)
5. [Standard Exit Codes](#standard-exit-codes)
6. [Detailed Subcommand Specifications](#detailed-subcommand-specifications)

---

## Architecture & Philosophy

The Solix CLI is designed around three core principles:

1. **Subcommand Modularity**: Every task is encapsulated in a dedicated, self-contained subcommand with clear option validation, deterministic error handling, and structured diagnostics.
2. **Deterministic Artifact Placement**: Source code compilation and project builds emit output binaries into explicit, predictable locations (`build/<profile>/<name>.slxarc` or `<name>.slxb`) without implicit system mutations.
3. **Isolated Package Management**: Package operations (`install`, `uninstall`, `list`) operate within an isolated, cross-platform user storage location (`~/.solix/packages`) governed by semantic versioning.

---

## Global Invocation Syntax

```bash
solix <subcommand> [options] [arguments...]
```

To display toolchain help or help for a specific subcommand:
```bash
solix --help
solix <subcommand> --help
```

---

## Subcommand Matrix

| Subcommand | Description | Specification Link |
|---|---|---|
| [`new`](new.md) | Scaffolds a new Solix project workspace with build automation, test harness, and native bridge | [new.md](new.md) |
| [`compile`](compile.md) | Compiles `.slx` source files into an executable binary container (`.slxb`) | [compile.md](compile.md) |
| [`build`](build.md) | Orchestrates building a project and packages it into an archive container (`.slxarc`) | [build.md](build.md) |
| [`run`](run.md) | Executes `.slxb` binaries, `.slxarc` archives, or `.slx` source files on the 64-bit Register VM | [run.md](run.md) |
| [`info`](info.md) | Inspects metadata, export manifests, disassembly, or synthesized APIs of `.slxb` or `.slxarc` | [info.md](info.md) |
| [`install`](install.md) | Installs a `.slxb` binary, `.slxarc` archive, or project directory into the local package store | [install.md](install.md) |
| [`sync`](sync.md) | Synchronizes and rebuilds installed packages from their recorded source directories | [sync.md](sync.md) |
| [`clean`](clean.md) | Cleans project build artifacts and intermediate caches | [clean.md](clean.md) |
| [`uninstall`](uninstall.md) | Removes an installed package or specific version from the local package store | [uninstall.md](uninstall.md) |
| [`list`](list.md) | Lists all installed packages and versions in the local package store | [list.md](list.md) |

---

## Environment Variables

| Variable | Description | Default Value |
|---|---|---|
| `SOLIX_PKG_DIR` | Root directory for local package installations | Linux/macOS: `~/.solix/packages`<br>Windows: `%USERPROFILE%\.solix\packages` |
| `SOLIX_RESOURCE_ROOT` | Root directory for bundled application resources | Auto-configured when executing `.slxarc` archives; falls back to `<cwd>/resources` |
| `SOLIX_BIN` | Explicit override path to the `solix` executable for build scripts | Current executable path or `PATH` discovery |

---

## Standard Exit Codes

| Exit Code | Classification | Meaning |
|---|---|---|
| `0` | Success | Command completed successfully with no errors |
| `1` | Error | Syntax error, semantic error, file not found, or unhandled runtime fault |
| `N` (`N > 1`) | Application Exit | Non-zero exit code explicitly returned by the Solix program's `main()` method |

---

## Detailed Subcommand Specifications

- [**`new` Subcommand**](new.md): Project scaffolding, applications, libraries (`--lib`), and pure Solix projects (`--no-native`).
- [**`compile` Subcommand**](compile.md): Multi-file compilation, static linking (`--static`), syntax checking (`--check`), and performance telemetry.
- [**`build` Subcommand**](build.md): Profile-driven (`--release`, `--debug`, `--test`) build orchestration and `.slxarc` archive generation.
- [**`run` Subcommand**](run.md): VM runtime execution, direct `.slx` execution, `.slxarc` bundle ingestion, and guest argument passing.
- [**`info` Subcommand**](info.md): Disassembly (`--disasm`), API reconstruction (`--api`), and JSON manifest extraction (`--json`).
- [**`install` Subcommand**](install.md): Installing `.slxb` and `.slxarc` packages into the local store.
- [**`clean` Subcommand**](clean.md): Cleaning project build artifacts (`build/`) and intermediate caches (`build/.cache/`).
- [**`uninstall` Subcommand**](uninstall.md): Removing packages and versions from the local store.
- [**`list` Subcommand**](list.md): Listing installed packages with tabular and `--json` formatting.
- [**Project Manifest (`manifest.toml`) Specification**](manifest.md): Complete schema documentation for project manifests.
