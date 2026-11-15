# Formal Specification: Binary Formats (`.slxb` & `.slxarc`)

## Document Metadata

* **Specification ID:** SPEC-0009
* **Scope:** Bytecode Container & Project Archive Container Layouts (`crates/solix-common/src/bytecode/`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)
* **Rust Implementation:** [`crates/solix-common/src/bytecode/binary.rs`](../../crates/solix-common/src/bytecode/binary.rs)

---

## 1. `.slxb` Bytecode Container Layout

Every compiled `.slxb` file consists of a 16-byte fixed container header followed by sequentially serialized binary sections:

```text
┌────────────────────────────────────────────────────────┐
│ Header (16 Bytes)                                      │
│   • Magic Bytes: b"SLXB" (0x53, 0x4C, 0x58, 0x42)      │
│   • Version Major (u16: 1)                             │
│   • Version Minor (u16: 0)                             │
│   • Target Flags (u32)                                 │
│   • Entrypoint Specifiers & Static Size (u32)          │
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

---

## 2. `.slxarc` Archive Package Container

A `.slxarc` file is a deterministic, cross-platform archive encapsulating compiled bytecode, native dynamic libraries, and application resources:

```text
package.slxarc (ZIP Container)
├── manifest.toml              # Project metadata, entrypoints, and dependencies
├── bin/
│   └── package.slxb           # Primary compiled bytecode container
├── lib/
│   ├── secondary.slxb         # Bundled dependency bytecode
│   ├── x86_64-linux/
│   │   └── libnative.so       # Host platform native shared library
│   ├── x86_64-windows/
│   │   └── native.dll
│   └── aarch64-macos/
│       └── libnative.dylib
└── resources/                 # Application assets (fonts, images, data files)
```

### Extraction & Caching Behavior
* At runtime, the package manager extracts native libraries matching the host platform (`host_triple`) into `~/.solix/cache/natives/<name>-<version>/`.
* Primary and secondary `.slxb` files are loaded directly into the VM runtime without requiring filesystem extraction.
