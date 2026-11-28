# Developer Guides — Index

Welcome to the Solix developer guides. These documents are written for developers building applications, libraries, and native extensions in Solix. They cover core language mechanics, type semantics, memory architecture, package distribution, tooling, and ecosystem integration.

---

## Guides Directory

| # | Guide | Primary Topics |
|---|---|---|
| 1 | [`01_getting_started.md`](01_getting_started.md) | Installation, Cargo build, `solix new`, `solix new native`, `solix build`, `solix run`, `manifest.yaml` |
| 2 | [`02_variables_and_types.md`](02_variables_and_types.md) | All 13 primitive types, declarations, `const` immutability, arrays, `String`, `null` semantics |
| 3 | [`03_control_flow.md`](03_control_flow.md) | `if/else`, `while`, `do-while`, `for`, `switch`, `break`, `continue`, `return` |
| 4 | [`04_classes_and_oop.md`](04_classes_and_oop.md) | Classes, constructors, destructors (`~ClassName()`), inheritance, interfaces, `virtual`/`override`, operators |
| 5 | [`05_memory_management_and_arc.md`](05_memory_management_and_arc.md) | ARC, 8-byte headers, strong/weak references, cycle breaking, deterministic LIFO destruction |
| 6 | [`06_error_handling.md`](06_error_handling.md) | Exception hierarchy, `try/catch/finally`, zero-cost table unwinding, custom exceptions |
| 7 | [`07_functions_and_lambdas.md`](07_functions_and_lambdas.md) | Static methods, function pointers `ret(*)(params)`, lambdas `=>`, closures, ARC captures |
| 8 | [`08_generics_and_templates.md`](08_generics_and_templates.md) | Generic classes, multiple type parameters, deduction, monomorphization pipeline |
| 9 | [`09_native_plugins.md`](09_native_plugins.md) | Dual-mode C, C++ (`solix::ffi`), and Rust (`solix-ffi`) extensions, ABI conventions, dynamic loader (`.so`/`.dll`/`.dylib`) |
| 10 | [`packaging_and_resources.md`](packaging_and_resources.md) | Single-file `.slxp` package containers, manifest specification, native discovery, `Resource` API |
| 11 | [`editors_vscode.md`](editors_vscode.md) | Visual Studio Code syntax highlighting, Language Server Protocol (`solix-lsp`), configuration |
| 12 | [`release.md`](release.md) | Automated build pipelines, cross-compilation, packaging, and release automation |
| 13 | [`build_system.md`](build_system.md) | Pure-Rust `solix build` architecture, `manifest.yaml` profiles/hooks/natives, and cross-compilation |
| 14 | [`embedding_rust_api.md`](embedding_rust_api.md) | In-memory compilation and VM embedding guide using `solix-compilation` and `solix-runtime` Rust APIs |

---

## Technical Specifications & Architecture

- [Architecture Deep Dives](../architecture/README.md) — Compiler pipeline, 64-bit Register VM, byte-addressed heap, and binder passes
- [Language Specifications](../spec/README.md) — Comprehensive grammar, statement, expression, and standard library specifications
- [Solix Wiki](../wiki/README.md) — Keyword reference, diagnostics catalog, and primitive guides
