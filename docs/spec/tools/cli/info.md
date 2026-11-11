# `solix info` — Binary & Archive Container Inspection

The `info` subcommand inspects compiled binary containers (`.slxb`) and archive packages (`.slxarc`), dumping container summaries, reconstructed Solix API interfaces, 64-bit Register VM disassemblies, or raw machine-readable JSON metadata.

---

## Synopsis

```bash
solix info <binary> [options]
```

---

## Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<binary>` | Path | **Yes** | Path to target compiled binary container (`*.slxb`) or archive container (`*.slxarc`). |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--output <file>` | `-o` | Path | `stdout` | Destination file path for inspection data. Parent directories are created automatically. |
| `--json` | | Flag | `false` | Output container metadata and export manifest formatted as structured JSON. |
| `--api` | | Flag | `false` | Print reconstructed Solix API interface source code (`.slx`) suitable for external consumption. |
| `--disasm` | | Flag | `false` | Disassemble all bytecode instructions into human-readable 64-bit Register VM opcodes. |

---

## Container Type Semantics

### 1. Binary Container (`.slxb`) Inspection

When inspecting a standalone `.slxb` container:

- **Default (Summary)**: Displays package metadata (name, version, description), static storage size, total function count, class definitions, and method signatures.
- **`--api`**: Extracts the embedded `ExportManifest`. If an `api_header` is present in the binary, it is emitted directly; otherwise, the interface is dynamically reconstructed via `manifest_to_source`.
- **`--disasm`**: Disassembles all functions in the binary into 64-bit Register VM instructions, detailing register indices, constant pool references, branch offsets, and exception handler ranges.
- **`--json`**: Emits the raw `ExportManifest` as formatted JSON containing classes, fields, vtable layouts, and function signatures.

```bash
# View summary of a compiled binary
solix info build/release/math_utils.slxb

# Emit public API interface source
solix info build/release/math_utils.slxb --api

# Disassemble bytecode to a log file
solix info build/release/math_utils.slxb --disasm -o target/disasm.txt

# Extract export manifest as JSON
solix info build/release/math_utils.slxb --json
```

### 2. Archive Container (`.slxarc`) Inspection

When inspecting a packaged `.slxarc` archive container:

- **Default (Summary)**: Inspects the ZIP container, displaying:
  - Package metadata from embedded `manifest.toml` (`[info]`, `[entrypoints]`).
  - Inventory of all contained files with uncompressed and compressed byte counts.
  - Native shared libraries (`natives/*.so`, `*.dll`, `*.dylib`).
  - Compiled binaries (`bin/*.slxb`).
- **`--api`**: Iterates across all contained `.slxb` binary files in the archive and emits reconstructed Solix API interface source blocks for each binary.
- **`--disasm`**: Iterates across all contained `.slxb` binary files and prints full Register VM disassemblies for each unit.
- **`--json`**: Emits an archive inspection object formatted as JSON containing archive path, manifest data, contained file entries, native library paths, and binary program descriptors.

```bash
# Inspect archive package structure
solix info build/release/game_engine.slxarc

# Output full archive structure and manifests as JSON
solix info build/release/game_engine.slxarc --json
```

---

## Exit Codes

| Exit Code | Classification | Meaning |
|---|---|---|
| `0` | Success | Inspection succeeded and output was printed or written. |
| `1` | Error | Binary file missing, invalid magic header, container decode failure, or I/O error. |
