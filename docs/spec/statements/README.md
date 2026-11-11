# Statement & Expression Compilation Specifications

This directory contains deep-dive normative specifications detailing the AST representations, binder type rules, and 64-bit Register VM lowering for every Solix language construct:

* [**Modules, Imports & Qualified Namespaces (`modules.md`)**](file:///home/jehud/Projects/solix-language/docs/spec/statements/modules.md): `package`, `import`, `extern import`, `alias`, and inline qualified `::` declarations.
* [**Declarations & OOP Members (`declarations.md`)**](file:///home/jehud/Projects/solix-language/docs/spec/statements/declarations.md): `class`, `interface`, `enum`, constructors, `~destructors`, methods, fields, and operator overloads.
* [**Generics & Variadic Templates (`generics-and-variadics.md`)**](file:///home/jehud/Projects/solix-language/docs/spec/statements/generics-and-variadics.md): Type parameter bounds (`<T : Bound>`), variadic packs (`<args...>`), tuple lowering, and monomorphization blueprints.
* [**Control Flow Statements (`control-flow.md`)**](file:///home/jehud/Projects/solix-language/docs/spec/statements/control-flow.md): `if`/`else`, `while`, `do`/`while`, `for`, `switch`/`case`, `break`, `continue`, and `return`.
* [**Exceptions & Runtime Intrinsics (`exceptions-and-intrinsics.md`)**](file:///home/jehud/Projects/solix-language/docs/spec/statements/exceptions-and-intrinsics.md): `try`/`catch`/`finally`, `throw`, `assert`, `exit`, and `print`/`println`.
* [**Expressions & Operators (`expressions.md`)**](file:///home/jehud/Projects/solix-language/docs/spec/statements/expressions.md): Binary/unary operators, ternary operator, casts (`cast(e, T)` & `(T)e`), `instanceof`, `sizeof`, `default`, `new`, indexing `[]`, and closures `=>`.
