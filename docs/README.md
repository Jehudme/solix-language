# Solix Language Master Documentation Hub

Welcome to the authoritative documentation suite for the **Solix Programming Language (v2.0)**, an expressive, memory-safe, statically typed systems and application language featuring an Ahead-of-Time AST compiler and a high-throughput 64-bit register-based bytecode virtual machine.

---

## 1. Tripartite Documentation Architecture

To guarantee long-term maintainability and prevent documentation drift, the Solix documentation is strictly partitioned into three specialized tiers per [`GL-0012`](file:///home/jehud/Projects/solix-language/docs/internals/guidelines/authoring-features-and-wiki.md):

```text
docs/
├── guide/      # User Guides ("How to Use It") — task tutorials, quick starts, examples
├── spec/       # Formal & Internal Specifications ("How It Works Under the Hood")
└── wiki/       # Atomic Reference Wiki ("Fast Dictionary Reference") — errors, keywords, CLI
```

| Documentation Tier | Target Audience | Primary Focus | Content Style |
| :--- | :--- | :--- | :--- |
| [**User Guides** (`docs/guide/`)](file:///home/jehud/Projects/solix-language/docs/guide/README.md) | Software engineers building Solix applications | Practical usage, task tutorials, idioms, standard library recipes | Narrative tutorials, code snippets, links to specs & wiki |
| [**Formal Specifications** (`docs/spec/`)](file:///home/jehud/Projects/solix-language/docs/spec/README.md) | Compiler engineers, language contributors, systems architects | Normative grammar, AST nodes, type binding, VM opcodes, ARC layout | Deep technical precision, EBNF, instruction formats, memory blocks |
| [**Atomic Wiki** (`docs/wiki/`)](file:///home/jehud/Projects/solix-language/docs/wiki/README.md) | Developers looking up specific errors, keywords, or CLI flags | Error codes (`E0000`–`R0023`), keywords, primitives, CLI commands | Concise dictionary lookups, minimal reproduction code, cheat sheets |

---

## 2. Core Modern Solix v2.0 Architectural Invariants

Solix v2.0 represents a modern, ground-up rewrite in safe Rust (Edition 2024), departing fundamentally from legacy prototype versions:

* **64-Bit Register-Based Virtual Machine:** Fixed 64-bit little-endian instructions, sliding register frames, and dynamic superinstruction fusing.
* **Scope Resolution Operator (`::`):** Static methods, static fields, and enum variants **MUST** be accessed using `::` (`Math::sin`, `Console::println`, `Color::Red`). Access via dot (`.`) is rejected with compile error `E0226`.
* **Standardized Package Containers:** Source projects use `manifest.toml`. Compilations output `.slxb` (bytecode + pre-parsed AST) or `.slxarc` (archive containing bytecode, native shared libraries, and resources).
* **Deterministic ARC & RAII:** Segregated free-list virtual heap with zero-cost RAII destructors (`~ClassName()`) executing automatically at variable scope boundaries.
* **Dual-Mode Linking & Dead-Code Elimination:** Dynamic module linking quickening extern calls in-place to direct calls, and static LTO pruning unreferenced classes and functions.
* **Granular Diagnostic Codes:** Standardized diagnostics spanning lexical (`E0000`–`E0015`), syntactic (`E0101`–`E0200`), semantic binding (`E0201`–`E0290`), and runtime execution (`R0001`–`R0023`).

---

## 3. Quick Navigation Index

### 3.1 Developer Learning Path ([User Guides](file:///home/jehud/Projects/solix-language/docs/guide/README.md))
1. [01. Getting Started & Toolchain](file:///home/jehud/Projects/solix-language/docs/guide/01-getting-started-and-toolchain.md)
2. [02. Variables, Primitives & Strings](file:///home/jehud/Projects/solix-language/docs/guide/02-variables-primitives-and-strings.md)
3. [03. Control Flow & Functions](file:///home/jehud/Projects/solix-language/docs/guide/03-control-flow-and-functions.md)
4. [04. Classes, OOP & Namespaces](file:///home/jehud/Projects/solix-language/docs/guide/04-classes-oop-and-namespaces.md)
5. [05. Memory Management, ARC & RAII](file:///home/jehud/Projects/solix-language/docs/guide/05-memory-management-arc-and-raii.md)
6. [06. Error Handling & Assertions](file:///home/jehud/Projects/solix-language/docs/guide/06-error-handling-and-assertions.md)
7. [07. Generics & Variadic Templates](file:///home/jehud/Projects/solix-language/docs/guide/07-generics-and-variadic-templates.md)
8. [08. Packages, Dependencies & Linking](file:///home/jehud/Projects/solix-language/docs/guide/08-packages-dependencies-and-linking.md)
9. [09. Native Rust Extensions](file:///home/jehud/Projects/solix-language/docs/guide/09-native-rust-extensions.md)
10. [10. Standard Library Tour (`solixlib`)](file:///home/jehud/Projects/solix-language/docs/guide/10-standard-library-tour.md)

### 3.2 Formal Specifications ([Specifications](file:///home/jehud/Projects/solix-language/docs/spec/README.md))
* [Lexical Grammar & Tokenization](file:///home/jehud/Projects/solix-language/docs/spec/lexical-grammar.md)
* [Type System, Hierarchy & Semantics](file:///home/jehud/Projects/solix-language/docs/spec/type-system-and-semantics.md)
* [Compilation Pipeline & Linking Passes](file:///home/jehud/Projects/solix-language/docs/spec/compilation-pipeline.md)
* [Binder, VTables, ITables & Monomorphization](file:///home/jehud/Projects/solix-language/docs/spec/binder-and-monomorphization.md)
* [Optimizer, Inliner & Superinstructions](file:///home/jehud/Projects/solix-language/docs/spec/optimizer-and-inliner.md)
* [64-Bit Register VM ISA & Opcode Matrix](file:///home/jehud/Projects/solix-language/docs/spec/vm-isa-and-execution.md)
* [Deterministic Memory Model, VirtualHeap & ARC](file:///home/jehud/Projects/solix-language/docs/spec/memory-model-and-arc.md)
* [Zero-Cost Exception Handling & Unwinding](file:///home/jehud/Projects/solix-language/docs/spec/exception-unwinding.md)
* [Binary Formats (`.slxb` & `.slxarc`)](file:///home/jehud/Projects/solix-language/docs/spec/binary-formats-slxb-and-slxarc.md)
* [Package Solver, SemVer & Dual-Mode Linking](file:///home/jehud/Projects/solix-language/docs/spec/package-solver-and-linking.md)
* [Native C-FFI ABI & Dynamic Extensions](file:///home/jehud/Projects/solix-language/docs/spec/native-ffi-abi.md)
* [Statement & Expression Compilation Specifications](file:///home/jehud/Projects/solix-language/docs/spec/statements/README.md)
* [Standard Library v2.0 Specifications](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/README.md)

### 3.3 Atomic Wiki References ([Wiki](file:///home/jehud/Projects/solix-language/docs/wiki/README.md))
* [Diagnostic Error Codes (`E0000`–`E0350`, `R0001`–`R0023`)](file:///home/jehud/Projects/solix-language/docs/wiki/errors/README.md)
* [Built-in Primitive Types & Aliases](file:///home/jehud/Projects/solix-language/docs/wiki/primitives/README.md)
* [CLI Commands Reference](file:///home/jehud/Projects/solix-language/docs/wiki/commands/README.md)
* [Language Keywords Dictionary](file:///home/jehud/Projects/solix-language/docs/wiki/keywords/README.md)

### 3.4 Internal Engineering Standards ([Guidelines](file:///home/jehud/Projects/solix-language/docs/internals/guidelines/README.md))
* Core engineering standards, conventional commits (`GL-0001`), branching topology (`GL-0002`), military Rust safety (`GL-0011`), tripartite architecture (`GL-0012`), and the mandatory pre-merge audit protocol (`GL-0013`).
