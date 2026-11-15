# `solix info` — Binary & Archive Container Inspection

The `info` subcommand inspects compiled binary containers (`.slxb`), archive packages (`.slxarc`), or installed packages by name (`solix info <pkg_name>`), dumping container summaries, SHA-256 checksums, cache status, reconstructed Solix API interfaces, 64-bit Register VM disassemblies, or machine-readable JSON metadata.

---

## Synopsis

```bash
solix info <target> [options]
```

---

## Positional Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<target>` | Path or String | **Yes** | Path to target compiled binary (`*.slxb`), archive (`*.slxarc`), or registered package name (`<name>` or `<name>@<version>`). |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--output <file>` | `-o` | Path | `stdout` | Destination file path for inspection data. Parent directories are created automatically. |
| `--json` | | Flag | `false` | Output container metadata and export manifest formatted as structured JSON. |
| `--api` | | Flag | `false` | Print reconstructed Solix API interface source code (`.slx`) suitable for external consumption. |
| `--disasm` | | Flag | `false` | Disassemble all bytecode instructions into human-readable 64-bit Register VM opcodes. |
| `--pkg-dir <dir>` | | Path | `~/.solix/packages` | Root directory override for the local package store when resolving installed package names. |

---

## Target Semantics

### 1. Installed Package Resolution by Name

`<target>` can be an installed package identifier:
- `solix info solix`: Resolves the highest installed SemVer version of `solix` from `$SOLIX_PKG_DIR/registry.json`.
- `solix info solix@2.0.0`: Resolves the exact specified version.

```bash
solix info solix
```

Output:
```text
=== Solix Archive Container Inspection (.slxarc) ===
Archive:      /home/user/.solix/packages/solix/2.0.0/solix.slxarc
SHA-256:      46747ea15ac6f2f15d462438a63fb9ef71581361ab5facca844c97a488d24812
Cache Status: Cached (verified SHA-256 match in ~/.solix/cache/)
Package:      solix v2.0.0
Description:  Solix Standard Library
License:      Apache-2.0

--- Entry Points ---
  • main: Main::main
  • test: TestRunner::test

--- Native Shared Libraries (1) ---
  • lib/libsolixlib_native.so [SHA-256: af5f19f16fac4342322dc64262fc55e49b4119122e690957773c78aa25405a44]

--- Contained Files (3) ---
  • manifest.toml (size: 335B, compressed: 238B)
  • bin/solix.slxb (size: 1568252B, compressed: 346873B)
  • lib/libsolixlib_native.so (size: 936336B, compressed: 350746B)

--- Compiled Binaries (1) ---
  • bin/solix.slxb (profile: release, functions: 1872, classes: 134, static: 25B, SHA-256: 7251b9e23dfbb39087b5e6e8f4f37e9715b697e2f1d8cc395ac67cff6acd2b92)
```

### 2. Binary Container (`.slxb`) Inspection

When inspecting a standalone `.slxb` container:
- Displays package metadata, build profile, static storage size, total function count, class definitions, and SHA-256 digest.
- `--api`: Emits reconstructed public API interface source code.
- `--disasm`: Disassembles virtual machine instructions.
- `--json`: Emits raw JSON manifest.

```bash
solix info build/release/math_utils.slxb
```
