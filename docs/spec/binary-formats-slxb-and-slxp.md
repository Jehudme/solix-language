# Formal Specification: Binary Formats (`.slxb` & `.slxp`)

## Document Metadata

* **Specification ID:** SPEC-0009
* **Scope:** Bytecode Container & Package Container Layouts (`crates/solix-common/src/bytecode/`, `crates/solix-common/src/project.rs`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/guidelines/authoring-specs.md` (GL-0005)
* **Rust Implementation:** [`crates/solix-common/src/bytecode/binary.rs`](../../crates/solix-common/src/bytecode/binary.rs), [`crates/solix-common/src/project.rs`](../../crates/solix-common/src/project.rs)

---

## 1. `.slxb` Bytecode Container Layout

Every compiled `.slxb` file consists of a header (fixed 12-byte magic/version/flags prefix followed by variable-length entrypoint specifiers and static size) followed by sequentially serialized binary sections:

```text
┌────────────────────────────────────────────────────────┐
│ Header (Variable-Length)                               │
│   • Magic Bytes: b"SLXB" (0x53, 0x4C, 0x58, 0x42)      │
│   • Version Major (u16: 1)                             │
│   • Version Minor (u16: 0)                             │
│   • Target Flags (u32)                                 │
│   • Entrypoint Specifiers (entry_point_id / symbol) &  │
│     Static Size (u32)                                  │
├────────────────────────────────────────────────────────┤
│ Section 2: Static Symbols Map                          │
│   • List of (Symbol Name, Memory Slot Index) pairs     │
├────────────────────────────────────────────────────────┤
│ Section 3: Constant Pool                               │
│   • Typed Literals: Int64 (Tag 1), Float64 (Tag 2),   │
│     String (Tag 3)                                     │
├────────────────────────────────────────────────────────┤
│ Section 4: Class Descriptors                           │
│   • VTable IDs, Base Classes, Field Offsets & ARC maps │
├────────────────────────────────────────────────────────┤
│ Section 5: Compiled Functions                          │
│   • Parameters, Registers, 64-bit Instructions,        │
│     Exception Handlers & Source Maps                   │
├────────────────────────────────────────────────────────┤
│ Section 6: Metadata & ExportManifest                   │
│   • Package Name, Version, Description, Dependencies   │
├────────────────────────────────────────────────────────┤
│ Section 7: Exported AST (Pre-Parsed Blueprint)         │
│   • Length-prefixed binary ProgramAst                  │
└────────────────────────────────────────────────────────┘
```

### Strict Validation & Overflow Guarantees
* **`try_to_bytes()` Checked Encoding**: Validates `u32` section item counts and UTF-8 string byte lengths via `u32::try_from`, returning `EmissionError::CorruptBinaryContainer` cleanly if any container limit or JSON metadata serialization fails instead of truncating with `as u32` or masking errors.
* **`from_bytes()` Strict Decoding**: Rejects invalid boolean/option flag bytes (`> 1`, expecting strictly `0` or `1`), rejects malformed JSON `Type` descriptors rather than coercing to `Void`, enforces platform `usize` conversions and `MAX_INSTANCE_SIZE_BYTES` (`64 MiB`) class-size sanity limits, and pre-validates capacity via `ByteReader::checked_capacity`.

---

## 2. `.slxp` Package Container Layout

A `.slxp` file is a deterministic, cross-platform ZIP-compatible package encapsulating compiled bytecode, native shared libraries, and application resources:

```text
package.slxp (ZIP Container)
├── manifest.yaml              # Package metadata, entrypoints, and dependencies
├── binaries.slxb              # Primary compiled bytecode container at package root
├── bin/
│   ├── x86_64-unknown-linux-gnu/
│   │   ├── lib__native_core.so # Native library (prefixed with __native_ or lib__native_)
│   │   └── libhelper.so        # Transitive support library (unregistered in FFI table)
│   ├── x86_64-pc-windows-msvc/
│   │   ├── __native_core.dll
│   │   └── helper.dll
│   └── aarch64-apple-darwin/
│       ├── lib__native_core.dylib
│       └── libhelper.dylib
└── resources/                 # Application assets (fonts, images, data files)
```

### Root Bytecode Layout
* The primary compiled program is packaged directly at the root as `binaries.slxb`.
* Secondary bundled libraries reside under `lib/` as needed.
* The VM loads root `binaries.slxb` into runtime memory directly from the package without requiring disk extraction.

### Native Loader & Isolation Policy
* At runtime, native libraries matching the active host target triple (`host_triple`) are extracted to isolated cache directory `~/.solix/cache/packages/<name>-<version>/bin/<target>/`.
* **Strict Loading Rule**: The runtime ONLY loads and registers libraries whose filenames start with `__native_` (or platform equivalent `lib__native_`).
* **Transitive Dependency Isolation**: Auxiliary or transitive support libraries located in `bin/<target>/` that do not start with `__native_` remain un-registered in the Solix VM FFI symbol table, ensuring dependencies cannot collide with user FFI bindings.
* **Hermetic Search**: The runtime never searches host system `PATH` or arbitrary system directories; all native symbols are resolved strictly from the active package's `bin/<target>/` location.
* **Root Binary Deduplication & Interface Bounds**: When an archive bundles both root `binaries.slxb` and a byte-identical legacy `bin/<package>.slxb` alias, `SlxarcArchive::extract_and_load` deduplicates them by SHA-256 digest before runtime loading; across multi-module loads, `RuntimeContext` rejects relocated `CallInterface` interface IDs exceeding the 8-bit opcode ceiling (`255`) rather than wrapping.
