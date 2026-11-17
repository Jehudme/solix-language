# Solix Language Master Documentation Hub

Welcome to the authoritative documentation suite for the **Solix Programming Language (v2.0)**, an expressive, memory-safe, statically typed systems and application language featuring an Ahead-of-Time AST compiler and a high-throughput 64-bit register-based bytecode virtual machine.

---

## 1. Documentation Taxonomy

To guarantee clarity, complete reference coverage, and prevent documentation drift, the Solix documentation is structured into specialized directories:

```text
docs/
├── architecture/   # Deep technical architecture (Pipeline, ARC, Memory Model, Binder, Exceptions)
├── guide/          # User & Developer Guides (14 step-by-step guides from Getting Started to Native Extensions)
├── spec/           # Formal Language Specifications
│   ├── statements/ # 44 granular specs for every statement, declaration, and expression
│   ├── libraries/  # 17 component-level specifications for the standard library (solixlib)
│   └── tools/      # CLI toolchain and Language Server Protocol specifications
├── wiki/           # Atomic Reference Wiki (41 keywords, diagnostics encyclopedia, primitives, CLI)
└── internals/      # Development milestones, engineering guidelines, and protocols
```

| Documentation Tier | Target Audience | Primary Focus | Content Style |
| :--- | :--- | :--- | :--- |
| [**Architecture Deep Dives** (`docs/architecture/`)](architecture/README.md) | Systems architects, runtime engineers, core contributors | Virtual machine execution model, byte-addressed memory, ARC lifecycle, zero-cost exception tables | Rigorous technical deep dives, ASCII memory bitmasks, flowcharts |
| [**User Guides** (`docs/guide/`)](guide/README.md) | Software engineers building Solix applications | Practical usage, task tutorials, idioms, standard library recipes | Narrative tutorials, code snippets, links to specs & wiki |
| [**Formal Specifications** (`docs/spec/`)](spec/README.md) | Compiler engineers, language contributors, systems architects | Normative grammar, AST nodes, type binding, 64-bit Register ISA, ARC layout | Deep technical precision, EBNF, instruction formats, memory blocks |
| [**Atomic Wiki** (`docs/wiki/`)](wiki/README.md) | Developers looking up specific errors, keywords, or CLI flags | Error codes (`E0000`–`R0023`), 41 keywords, primitives, CLI commands | Concise dictionary lookups, minimal reproduction code, cheat sheets |
| [**Engineering Guidelines** (`docs/internals/guidelines/`)](internals/guidelines/README.md) | Core repository developers & contributors | Commit format, GL-0011 zero single-letter variables, anti-drift protocols | Normative engineering standards & verification protocols |

---

## 2. Core Modern Solix v2.0 Architectural Invariants

Solix v2.0 represents a modern, ground-up rewrite in safe Rust under `#![forbid(unsafe_code)]`:

* **64-Bit Register-Based Virtual Machine:** Uniform 64-bit little-endian instructions (`Format A`, `Format B`, `Format C`, `Format Call`), sliding register windows, and register allocation via linear scan.
* **Scope Resolution Operator (`::`):** Static methods, static fields, and enum variants **MUST** be accessed using `::` (`Math::sqrt`, `Console::println`, `Color::Red`). Access via dot (`.`) is rejected with compile error `E0226`.
* **Standardized Package Containers:** Source projects use `manifest.toml`. Compilations output `.slxb` (bytecode + pre-parsed AST) or `.slxarc` (archive containing bytecode, native shared libraries, and resources).
* **Deterministic ARC & RAII:** Segregated free-list virtual heap with 8-byte packed headers, immediate free-block recycling, and zero-cost RAII destructors (`~ClassName()`) executing automatically at variable scope boundaries.
* **Dual-Mode Linking & Dead-Code Elimination:** Dynamic module linking quickening extern calls in-place to direct calls, and static LTO pruning unreferenced classes and functions.
* **Granular Diagnostic Codes:** Standardized diagnostics spanning lexical (`E0000`–`E0015`), syntactic (`E0101`–`E0200`), semantic binding (`E0201`–`E0290`), and runtime execution (`R0001`–`R0023`).
* **Zero Single-Letter Variables (`GL-0011`):** All code snippets in both Solix and Rust use self-documenting multi-letter identifiers (`index`, `counter`, `number`, `item`, `element`, `message`, `error`).

---

## 3. Quick Navigation Index

### 3.1 Architecture Deep Dives ([Architecture](architecture/README.md))
- [`pipeline.md`](architecture/pipeline.md) — The 7-phase compilation and execution pipeline
- [`binder_passes.md`](architecture/binder_passes.md) — Binder Pass 1a, 1b, Pass 2, and Generic Monomorphization
- [`memory_model.md`](architecture/memory_model.md) — Byte-addressed virtual heap, 8-byte headers, natural alignment
- [`arc_internals.md`](architecture/arc_internals.md) — Deterministic ARC, cycle prevention with `weak`, free lists
- [`exception_unwinding.md`](architecture/exception_unwinding.md) — Table-driven zero-cost exception handling

### 3.2 Developer Learning Path ([User Guides](guide/README.md))
1. [`01_getting_started.md`](guide/01_getting_started.md) — Installation, Cargo build, `solix new`, `solix build`, `solix run`
2. [`02_variables_and_types.md`](guide/02_variables_and_types.md) — All 13 primitive types, declarations, `const`, arrays, `String`
3. [`03_control_flow.md`](guide/03_control_flow.md) — `if/else`, `while`, `do-while`, `for`, `switch`, `break`, `continue`
4. [`04_classes_and_oop.md`](guide/04_classes_and_oop.md) — Classes, constructors, destructors, inheritance, interfaces, operators
5. [`05_memory_management_and_arc.md`](guide/05_memory_management_and_arc.md) — ARC mechanics, strong/weak references, cycle breaking
6. [`06_error_handling.md`](guide/06_error_handling.md) — Exception hierarchy, `try/catch/finally`, zero-cost table unwinding
7. [`07_functions_and_lambdas.md`](guide/07_functions_and_lambdas.md) — Function pointers `ret(*)(params)`, lambdas `=>`, closures
8. [`08_generics_and_templates.md`](guide/08_generics_and_templates.md) — Generic classes, multiple type parameters, monomorphization
9. [`09_native_plugins.md`](guide/09_native_plugins.md) — Native Rust/C extensions, ABI conventions, shared libraries
10. [`packaging_and_resources.md`](guide/packaging_and_resources.md) — Single-file `.slxarc` archives, manifests, `Resource` API
11. [`editors_vscode.md`](guide/editors_vscode.md) — VS Code extension, Language Server Protocol (`solix-lsp`)
12. [`release.md`](guide/release.md) — Release automation, cross-compilation, packaging
13. [`solix_raylib_plan.md`](guide/solix_raylib_plan.md) — Raylib game engine bindings and game loop design
14. [`build-system.md`](features/build-system.md) — Hermetic Python build engine (`solix_build.py`), cache isolation, and Zig/CMake compilation

### 3.3 Formal Specifications ([Specifications](spec/README.md))
* [Lexical Grammar & Tokenization](spec/lexical.md)
* [Type System, Hierarchy & Semantics](spec/types.md)
* [Diagnostics & Error Reporting Specification](spec/diagnostics.md)
* [64-Bit Register VM ISA & Opcode Matrix](spec/vm_isa.md)
* [Native Interoperability & C-FFI ABI](spec/native_interop.md)
* [Statement & Expression Specifications (44 granular specs)](spec/statements/README.md)
* [Standard Library Specifications (17 component specs)](spec/libraries/solixlib/string.md)
* [CLI & Language Server Protocol Tooling](spec/tools/cli/README.md)

### 3.4 Atomic Wiki References ([Wiki](wiki/README.md))
* [Language Keywords Dictionary (41 keywords)](wiki/keywords/README.md)
* [Diagnostic Error Codes (`E0000`–`E0350`, `R0001`–`R0023`)](wiki/errors/README.md)
* [Built-in Primitive Types & Aliases](wiki/primitives/README.md)
* [CLI Commands Reference](wiki/commands/README.md)

### 3.5 Internal Engineering Standards ([Guidelines](internals/guidelines/README.md))
* [Commit Conventions (`GL-0001`)](internals/guidelines/authoring-commits.md)
* [Branching Conventions (`GL-0002`)](internals/guidelines/branching-conventions.md)
* [Rust Military Safety Standards (`GL-0011`)](internals/guidelines/language-conventions.md)
* [Tripartite Documentation Guidelines (`GL-0012`)](internals/guidelines/authoring-features-and-wiki.md)
* [Documentation Update Protocol (`GL-0013`)](internals/guidelines/documentation-update-protocol.md)
* [Engineering Milestones Index (`MS-001` to `MS-062`)](internals/milestones/README.md)
* [Milestone 061 Cross-Language Benchmarks](BENCHMARKS.md)
