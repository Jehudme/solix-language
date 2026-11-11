# Solix Developer Guides & Tutorials

Welcome to the Solix user guides. These guides provide a structured, task-oriented path for learning and building real-world applications with Solix v2.0.

Per [`GL-0012`](file:///home/jehud/Projects/solix-language/docs/internals/guidelines/authoring-features-and-wiki.md), user guides focus on **practical application and idioms**. For formal semantics, refer to the [Formal Specifications](file:///home/jehud/Projects/solix-language/docs/spec/README.md); for quick syntax lookups, refer to the [Atomic Wiki](file:///home/jehud/Projects/solix-language/docs/wiki/README.md).

---

## Recommended Learning Path

```text
  01. Getting Started & Toolchain
                 │
                 ▼
  02. Variables, Primitives & Strings
                 │
                 ▼
  03. Control Flow & Functions
                 │
                 ▼
  04. Classes, OOP & Namespaces
                 │
                 ▼
  05. Memory Management, ARC & RAII
                 │
                 ▼
  06. Error Handling & Assertions
                 │
                 ▼
  07. Generics & Variadics
                 │
                 ▼
  08. Packages, Dependencies & Linking
                 │
                 ▼
  09. Native Rust Extensions
                 │
                 ▼
  10. Standard Library Tour
```

---

## Guide Directory

| # | Guide Title | Core Concepts Covered |
| :--- | :--- | :--- |
| **01** | [**Getting Started & Toolchain**](file:///home/jehud/Projects/solix-language/docs/guide/01-getting-started-and-toolchain.md) | Installing Solix, `solix new`, `build`, `run`, CLI workflows, project structure (`manifest.toml`). |
| **02** | [**Variables, Primitives & Strings**](file:///home/jehud/Projects/solix-language/docs/guide/02-variables-primitives-and-strings.md) | Explicit typing, integer signedness, float precision, `char`, `bool`, arrays, tuples, primitive `string` vs `solix.core.String`. |
| **03** | [**Control Flow & Functions**](file:///home/jehud/Projects/solix-language/docs/guide/03-control-flow-and-functions.md) | `if`/`else`, `while`, `do`/`while`, `for`, `switch`/`case`, free functions, default parameters, function pointers, and lambdas. |
| **04** | [**Classes, OOP & Namespaces**](file:///home/jehud/Projects/solix-language/docs/guide/04-classes-oop-and-namespaces.md) | Classes, single inheritance, interfaces, abstract classes, operator overloading, static members via `::`, and qualified declarations. |
| **05** | [**Memory Management, ARC & RAII**](file:///home/jehud/Projects/solix-language/docs/guide/05-memory-management-arc-and-raii.md) | Deterministic ARC, strong vs weak references, breaking reference cycles, and scope-based destructors (`~ClassName()`). |
| **06** | [**Error Handling & Assertions**](file:///home/jehud/Projects/solix-language/docs/guide/06-error-handling-and-assertions.md) | Structured `try`/`catch`/`finally`, custom exception classes, rethrowing, defensive `assert()`, and process termination (`exit()`). |
| **07** | [**Generics & Variadic Templates**](file:///home/jehud/Projects/solix-language/docs/guide/07-generics-and-variadic-templates.md) | Parameterized classes and methods, interface bounds (`<T : Bound>`), variadic templates (`<args...>`), and monomorphization. |
| **08** | [**Packages, Dependencies & Linking**](file:///home/jehud/Projects/solix-language/docs/guide/08-packages-dependencies-and-linking.md) | Multi-file packages, `extern import`, version resolution via `ModuleSolver`, `.slxb` bytecode, and `.slxarc` archives. |
| **09** | [**Native Rust Extensions**](file:///home/jehud/Projects/solix-language/docs/guide/09-native-rust-extensions.md) | Writing native shared libraries in Rust (`cdylib`), using `SolixVmContext` and `SolixCValue`, and bundling in `natives/`. |
| **10** | [**Standard Library Tour (`solixlib`)**](file:///home/jehud/Projects/solix-language/docs/guide/10-standard-library-tour.md) | Practical recipes covering collections, file I/O, streams, cryptography, date/time, and process execution. |
