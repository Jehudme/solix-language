# CLI Reference: `run` & `info`

---

## 1. `solix run`

Executes compiled `.slxb` binaries, packaged `.slxp` containers, or executes `.slx` source files on-the-fly:

```bash
solix run <binaries...> [options] [-- [guest_args...]]
solix run package [path] [options] [-- [guest_args...]]
solix run source <file.slx> [options] [-- [guest_args...]]
```

### Key Options & Flags
* `<binaries...>`: One or more target binary files (`.slxb`), package containers (`.slxp`), or raw source files (`.slx`).
* `-l, --lib <path>`: External library binary or native shared object (`.so`, `.dll`, `.dylib`) to link (can be repeated).
* `-e, --entry <symbol>`: Overrides application entry point method [default: `Main::main`].
* `--test`: Executes test entry point defined in package `manifest.yaml` (`profiles.test.entrypoint`, or legacy `[entrypoints].test`).
* `-p, --profile <profile>`: Target build profile to execute (`release`, `debug`, `test`).
* `--inline-threshold <N>`: Inlining statement threshold limit for in-memory source compilation [default: `8`].
* `--heap-size <size>`: Configures VirtualHeap capacity limit (e.g. `16MB`, `64MB`, `1GB`) [default: `32MB`]. Returns `RuntimeError::OutOfMemory` immediately if initial constant pool string interning exceeds this limit.
* `--max-stack <depth>`: Maximum call stack frame depth [default: `1024`].
* `-v, --verbose`: Prints VM execution summary and heap statistics upon completion.
* `--safe`: Executes in defensive safe mode with bounds checking and invariant assertions.
* `--metrics [<format>]`: Prints runtime execution telemetry and memory profiling (`text` or `json`).
* `--no-verify`: Bypasses dependency and native library cryptographic SHA-256 checksum verification (visible alias `--ignore-checksum`).
* `--pkg-dir <dir>`: Package store root directory override [default: `~/.solix/packages`].
* `-- [guest_args...]`: Trailing arguments passed directly to the guest Solix entry point.

### Practical Examples
```bash
# Run a source file directly on-the-fly (no intermediate .slxb file needed)
solix run src/main.slx
solix run source src/main.slx

# Execute a compiled binary container
solix run bin/app.slxb

# Execute a packaged project container
solix run build/release/my_game.slxp
solix run package ./my_game

# Execute project unit tests packaged within the container
solix run --test build/debug/my_game.slxp

# Run an installed package directly by name, version, or profile
solix run my-app
solix run my-app@1.0.0
solix run my-app:debug@1.0.0
solix run my-app --profile debug

# Pass command-line arguments to the guest Solix program
solix run bin/app.slxb -- --port 8080 --host 0.0.0.0

# Execute with expanded virtual heap capacity and verbose diagnostics
solix run bin/app.slxb --heap-size 128MB -v
```

---

## 2. `solix info`

Inspects binary container (`.slxb`) or package container (`.slxp`) metadata, export manifests, disassemblies, and reconstructed APIs:

```bash
solix info <binary> [options]
```

### Key Options & Flags
* `<binary>`: Path to compiled binary container (`*.slxb`) or package container (`*.slxp`).
* `-o, --output <file>`: Destination file path for inspection data [default: `stdout`].
* `--disasm`: Disassembles all functions into human-readable 64-bit Register VM opcodes.
* `--api`: Emits reconstructed Solix public API interface declarations (`.slx`).
* `--json`: Emits container metadata, sections, classes, and manifests as structured JSON.

### Practical Examples
```bash
# View summary of a project directory (profiles, platforms, dependencies, built artifacts)
solix info ./libraries/raylib/

# View summary of a binary container (classes, methods, static size)
solix info bin/app.slxb

# Inspect a package container (manifest, files, native libraries, binaries)
solix info build/release/my_game.slxp

# Print reconstructed Solix API interface for a library
solix info ~/.solix/packages/math_utils/1.0.0/math_utils.slxb --api

# Disassemble bytecode to inspect low-level 64-bit Register VM instructions
solix info bin/app.slxb --disasm

# Export full binary container manifest as machine-readable JSON
solix info bin/app.slxb --json -o target/manifest.json
```

---

## Related Specifications
* Formal Spec: [`docs/spec/tools/cli/run.md`](../../spec/tools/cli/run.md)
* Formal Spec: [`docs/spec/tools/cli/info.md`](../../spec/tools/cli/info.md)
