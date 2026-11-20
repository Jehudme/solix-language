# Solix Language Documentation Hub

Welcome to the documentation for **Solix**, an ahead-of-time compiled, memory-safe, statically typed programming language powered by a deterministic Automatic Reference Counting (ARC) memory model and a 64-bit register-based virtual machine.

---

## Documentation Pillars

Solix documentation is organized into four user-facing pillars plus contributor engineering governance:

| Pillar | Purpose | Target Audience | Entry Point |
| :--- | :--- | :--- | :--- |
| **User Guides** | Tutorials covering syntax, OOP, ARC, native FFI, build system, packaging, and editor integration | Application & library developers | [Guides Overview](guide/README.md) |
| **Reference Wiki** | Fast dictionary lookups for all keywords, primitive types, CLI commands, and error codes (`E0000`–`R0024`) | Everyday Solix developers & IDE users | [Wiki Overview](wiki/README.md) |
| **Formal Specifications** | Normative lexical, type, ISA, `.slxasm` disassembly, `.slxb`/`.slxp` container, statement, stdlib, and CLI/LSP contracts | Compiler/VM engineers & tool authors | [Specs Overview](spec/README.md) |
| **Architecture Deep Dives** | Internal compiler pipeline phases, binder layout/VTable passes, VirtualHeap allocator, ARC cycle collection, and zero-cost unwinding | Runtime & systems contributors | [Architecture Overview](architecture/README.md) |
| **Engineering Guidelines** | Contributor governance, Rust safety (`GL-0011`), testing matrices, branching, and release automation | Repository maintainers & contributors | [Guidelines Overview](guidelines/README.md) |

---

## Core Language & Runtime Highlights

- **Deterministic Memory Safety Without GC Pauses:** Compile-time ARC tracking, immediate segregated free-list recycling, RAII destructors (`~ClassName()`), `weak` cycle breaking, and a synchronous Bacon & Rajan cycle collector.
- **64-Bit Register Virtual Machine:** Fixed-width 8-byte instructions with sliding register windows, superinstructions, dynamic quickening, and zero-cost table-driven exception unwinding.
- **Native C/C++ & Rust Interoperability:** Safe C-ABI FFI SDKs (`ffi/c` `solix::ffi` and `ffi/rust` `solix-ffi`) with dual-mode CMake/Cargo builds packaged cleanly inside multi-platform `.slxp` packages ([FFI Wiki](wiki/ffi/README.md)).
- **Integrated Toolchain:** A single `solix` CLI handles project scaffolding (`solix new`, `solix new native`), pure-Rust declarative builds (`manifest.yaml`), execution, packaging, LSP 3.17 IDE intelligence, DAP debugging, and this offline documentation portal (`solix wiki`).
