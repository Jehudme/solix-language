# Solix Compiler & Virtual Machine Architecture

This directory provides deep technical specifications and internal implementation blueprints for the Solix language compiler toolchain, modular binder passes, memory management subsystem, and register virtual machine execution engine.

---

## Architecture Documents

- [`pipeline.md`](file:///home/jehud/Projects/solix-language/docs/architecture/pipeline.md) — The complete compilation and execution pipeline: Lexer $\to$ Parser $\to$ Binder $\to$ Optimizer $\to$ Emitter $\to$ Runtime Register VM.
- [`binder_passes.md`](file:///home/jehud/Projects/solix-language/docs/architecture/binder_passes.md) — Detailed mechanics of semantic passes: Symbol Discovery, Hierarchy & Member Layout, Type Checking & ARC Hook Insertion, and Generic Monomorphization.
- [`memory_model.md`](file:///home/jehud/Projects/solix-language/docs/architecture/memory_model.md) — Byte-addressed virtual heap, 8-byte `BlockHeader` packing, natural field alignment, and `BlockTag` metadata.
- [`arc_internals.md`](file:///home/jehud/Projects/solix-language/docs/architecture/arc_internals.md) — Deterministic Automatic Reference Counting (ARC), cycle prevention via `weak` references, and segregated free-list recycling.
- [`exception_unwinding.md`](file:///home/jehud/Projects/solix-language/docs/architecture/exception_unwinding.md) — Table-driven zero-cost exception handling, VTable catch matching, and cleanup trampolines.

---

## Architectural Principles

1. **Deterministic Lifecycle**: Solix eliminates non-deterministic tracing garbage collection pauses in favor of compile-time inserted ARC reference tracking and immediate free-list block recycling.
2. **Fixed-Width Register VM**: Bytecode instructions are uniformly encoded into 64-bit aligned words supporting sliding register windows, eliminate stack thrashing, and optimize dispatch cache locality.
3. **Memory Safety Without Unsafe Rust**: The entire compiler and runtime engine is authored under `#![forbid(unsafe_code)]`, enforcing zero undefined behavior through checked slicing and validated byte encodings.
4. **Zero-Cost Abstractions**: Exceptions incur zero CPU instruction overhead on happy paths through static PC lookup tables, while generic data structures monomorphize into specialized native layouts.
