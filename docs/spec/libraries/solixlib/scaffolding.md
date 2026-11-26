# Solix Standard Library (`solix`) Architecture & Scaffolding

This document specifies the layout, build pipeline, and package lifecycle of the bundled Solix standard library (`libraries/solixlib/`).

---

## 1. Architectural Philosophy

The Solix standard library (`solix`) is structured as a first-party Solix package rather than hardcoded runtime sources:

1. **Clean Separation:** Pure Solix modules (`src/solix/...`) and the companion Rust FFI plugin (`natives/solixlib_native/`) live side-by-side inside `libraries/solixlib/`.
2. **First-Class Manifest (`manifest.yaml`):** Governed directly by [`manifest.yaml`](../../tools/cli/manifest.md) with self-contained `release`, `debug`, and `test` profiles.
3. **Deterministic SemVer & Bootstrapping:** Consumer projects pin compatible versions via `dependencies: { solix: { version: "..." } }`, while `solix` automatically installs bundled releases into `~/.solix/packages` on first run unless `--no-std` is used.
4. **Opt-In (`--no-std`):** Bare-metal or self-contained projects omit the standard library cleanly via `solix new --no-std`.
5. **Dogfooding:** Exercises the same dependency solver (`ClientModuleSolver`), FFI macro SDK (`solix-ffi`), and `.slxp` container staging used by third-party libraries.

---

## 2. Directory Layout

```text
libraries/solixlib/
├── manifest.yaml                    # Package manifest (name: "solix", profiles: release/debug/test)
├── src/
│   └── solix/                       # Pure Solix stdlib modules (core, collections, io, json, net, text, sync, ...)
├── tests/                           # Standard library integration & unit test suite
├── scripts/                         # Optional cross-target helper scripts
└── natives/
    └── solixlib_native/             # Companion Rust FFI bridge crate (`solix-ffi`)
        ├── Cargo.toml               # `crate-type = ["cdylib", "rlib"]`, optional `solix-native` feature
        ├── Cross.toml
        └── src/
            └── lib.rs               # `solix_register_natives` registration entrypoint
```

---

## 3. Package Manifest (`libraries/solixlib/manifest.yaml`)

```yaml
configuration:
  name: solix
  version: "2.2.0"
  description: Solix Standard Library
  license: Apache-2.0
  authors: []

profiles:
  release:
    optimization_level: 3
    inline_threshold: 16
    debug_symbols: false
    static: true
    sources:
      - src
    platforms:
      - all
    natives:
      solixlib_native:
        profile: release
        docker: true

  debug:
    optimization_level: 0
    inline_threshold: 8
    debug_symbols: true
    static: false
    sources:
      - src
    platforms:
      - host
    natives:
      solixlib_native:
        profile: debug
```

---

## 4. Native Companion Plugin (`natives/solixlib_native`)

The native companion library (`solixlib_native`) builds through `solix build` using the Rust FFI SDK (`solix-ffi`) and outputs platform-indexed shared libraries (`libsolixlib_native.so`, `solixlib_native.dll`, `libsolixlib_native.dylib`) under `build/<profile>/stage/bin/<target_triple>/` before packaging into the `.slxp` archive.

---

## 5. Build, Verification & Installation Lifecycle

### Step 1: Build and Stage Locally
```bash
# Build standard library package and host native plugin
solix build libraries/solixlib --profile release --install --force

# Execute standard library test suite
solix build libraries/solixlib --profile test
solix run libraries/solixlib/build/test/solix.slxp
```

### Step 2: Consumer Dependency Declaration
In consumer projects (`manifest.yaml`), reference the `solix` standard library under active profiles:
```yaml
profiles:
  release:
    entrypoint: Main::main
    optimization_level: 3
    sources:
      - src
    dependencies:
      solix:
        version: "2.2.0"
        profile: release
```
When running `solix build` or `solix run`, `ClientModuleSolver` resolves the matching package profile from `~/.solix/packages` and loads the host native shared library from `bin/<target_triple>/` inside the `.slxp` archive.
