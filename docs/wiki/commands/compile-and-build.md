# CLI Reference: `compile` & `build`

---

## 1. `solix compile`

Compiles one or more standalone Solix source files into an executable `.slxb` container:

```bash
solix compile <source.slx> [flags]
```

### Key Flags
* `-o, --output <path>`: Specifies destination `.slxb` output file path.
* `--profile <debug|release|test>`: Compilation profile (default: `release`).
* `--deps <pkg@version>`: Explicit external dependency requirements.
* `--static-link`: Statically merges dependencies and executes Dead-Code Elimination (DCE).
* `--metrics-json`: Emits compiler phase latency telemetry as machine-readable JSON.
* `--yolo, --ignore-checksum`: Bypasses dependency cryptographic SHA-256 validation.

---

## 2. `solix build`

Orchestrates multi-file, multi-language projects containing a `manifest.toml`:

```bash
solix build [project_path] [flags]
```

### Key Flags
* `--release`: Builds optimized release archive (`build/release/<name>.slxarc`).
* `--debug`: Builds unoptimized debug archive with source maps.
* `--test`: Builds and packages project tests (`build/test/<name>.slxarc`).
* `--install, --sync`: Installs the built package into the local store immediately upon build success.
* `--clean`: Purges existing build artifacts in `build/` before building.
