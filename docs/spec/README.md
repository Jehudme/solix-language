# Solix Formal Specifications Index

This directory contains the authoritative, normative technical specifications for the Solix language compiler, 64-bit register-based virtual machine, memory runtime, standard library, and package toolchain.

Per [`GL-0012`](file:///home/jehud/Projects/solix-language/docs/internals/guidelines/authoring-features-and-wiki.md), specifications serve as the normative source of truth for systems engineers and toolchain contributors.

---

## 1. Core Language & Architecture Specifications

* [**Lexical Grammar & Tokenization**](lexical.md): UTF-8 encoding, comments, string escapes, number literals (hex, binary, underscore separators), and complete token dictionary.
* [**Type System, Hierarchy & Semantics**](types.md): Primitive types, reference classes, structural tuples, function pointers, nominal subtyping, and operator overloading rules.
* [**Diagnostics & Error Reporting**](diagnostics.md): Canonical error formatting, line and column spans, diagnostic severity levels, and category codes.
* [**64-Bit Register VM ISA & Opcode Matrix**](vm_isa.md): Register execution model, Instruction Formats A/B/C/Call, sliding register window ABI, and opcode taxonomy.
* [**Native Interoperability & C-FFI ABI**](native_interop.md): C-ABI bridge, plugin loader, data layout conversions, and dynamic shared library integration.

---

## 2. Statement & Expression Compilation Specifications ([`statements/`](statements/README.md))

Detailed specifications defining the AST representations, binder type rules, and 64-bit Register VM lowering for all 44 language constructs:

### 2.1. Compilation Units & Modules ([`statements/modules/`](statements/modules/package_statement.md))
* [`package_statement.md`](statements/modules/package_statement.md) — Namespace boundaries & compilation unit isolation
* [`import_statement.md`](statements/modules/import_statement.md) — Selective & wildcard symbol imports
* [`alias_statement.md`](statements/modules/alias_statement.md) — Type synonyms & generic alias substitutions

### 2.2. Declarations ([`statements/declarations/`](statements/declarations/class_declaration.md))
* [`class_declaration.md`](statements/declarations/class_declaration.md) — Reference types, single inheritance, and VTables
* [`interface_declaration.md`](statements/declarations/interface_declaration.md) — Abstract contracts & multiple interface conformance
* [`enum_declaration.md`](statements/declarations/enum_declaration.md) — Strongly typed discrete enumerations
* [`field_declaration.md`](statements/declarations/field_declaration.md) — Static, instance, and `weak` cycle-breaking fields
* [`constructor_declaration.md`](statements/declarations/constructor_declaration.md) — Initializer lists, `super()` chaining, and heap allocation
* [`destructors.md`](statements/declarations/destructors.md) — Deterministic RAII cleanup, subclass-to-base inheritance chaining, and exception stack unwinding
* [`method_declaration.md`](statements/declarations/method_declaration.md) — Static, virtual, and abstract member functions
* [`operator_declaration.md`](statements/declarations/operator_declaration.md) — Operator overloading for `+`, `-`, `*`, `/`, `=`
* [`generic_declaration.md`](statements/declarations/generic_declaration.md) — Template metaprogramming, classes, methods, and monomorphization
* [`tuple_primitive.md`](statements/declarations/tuple_primitive.md) — First-class `tuple` primitive, variadic packs `<args...>`, and deterministic RAII cleanup

### 2.3. Control Flow ([`statements/control_flow/`](statements/control_flow/while_statement.md))
* [`block_statement.md`](statements/control_flow/block_statement.md) — Lexical scope boundaries & ARC scope-exit cleanup
* [`variable_declaration.md`](statements/control_flow/variable_declaration.md) — Register allocation, primitives vs. reference types
* [`expression_statement.md`](statements/control_flow/expression_statement.md) — Side-effect evaluation & temporary reference `DEC_REF`
* [`if_statement.md`](statements/control_flow/if_statement.md) — Boolean predicate branching & short-circuit jumps
* [`while_statement.md`](statements/control_flow/while_statement.md) — Pre-test loop iteration & loop context labels
* [`do_while_statement.md`](statements/control_flow/do_while_statement.md) — Post-test single-pass guarantee
* [`for_statement.md`](statements/control_flow/for_statement.md) — Induction variable scoping, step expressions, and iteration
* [`switch_statement.md`](statements/control_flow/switch_statement.md) — Multi-way jump tables, case matching, and fall-through
* [`break_statement.md`](statements/control_flow/break_statement.md) — Non-local escape & intermediate block ARC unwinding
* [`continue_statement.md`](statements/control_flow/continue_statement.md) — Iteration advancement & intermediate block cleanup
* [`return_statement.md`](statements/control_flow/return_statement.md) — Frame termination, return values, receiver/parameter cleanup
* [`throw_statement.md`](statements/control_flow/throw_statement.md) — Exception hierarchy enforcement & zero-cost unwinding
* [`try_catch_finally_statement.md`](statements/control_flow/try_catch_finally_statement.md) — Exception matching tables, parameter binding, `finally` guarantees
* [`intrinsics.md`](statements/control_flow/intrinsics.md) — Language intrinsics `assert` invariant validation and `exit` process termination

### 2.4. Expressions ([`statements/expressions/`](statements/expressions/assignment_expression.md))
* 16 granular expression specifications including [`assignment_expression.md`](statements/expressions/assignment_expression.md), [`binary_expression.md`](statements/expressions/binary_expression.md), [`cast_expression.md`](statements/expressions/cast_expression.md), [`instanceof_expression.md`](statements/expressions/instanceof_expression.md), [`lambda_expression.md`](statements/expressions/lambda_expression.md), and [`new_instance_expression.md`](statements/expressions/new_instance_expression.md).

---

## 3. Standard Library Component Specifications ([`libraries/solixlib/`](libraries/solixlib/string.md))

Normative contracts for all 17 components of the standard library:

* [`string.md`](libraries/solixlib/string.md) — `String`, `StringBuilder`, UTF-8 manipulation, formatting
* [`list.md`](libraries/solixlib/list.md) — Resizable `List<T>` dynamic array container
* [`map.md`](libraries/solixlib/map.md) — Hash-bucket `Map<K, V>` associative container
* [`set.md`](libraries/solixlib/set.md) — Hash-based `Set<T>` unique element set
* [`streams.md`](libraries/solixlib/streams.md) — Abstract streams, byte streams, memory buffers
* [`filesystem.md`](libraries/solixlib/filesystem.md) — `File`, `Directory`, `Path`, filesystem operations
* [`crypto.md`](libraries/solixlib/crypto.md) — Cryptographic hashes (SHA-256, MD5) and secure random numbers
* [`time.md`](libraries/solixlib/time.md) — Timestamps, duration, clock intervals, calendar dates
* [`math.md`](libraries/solixlib/math.md) — Trigonometric functions, exponentials, roots, constants
* [`console.md`](libraries/solixlib/console.md) — Console I/O, terminal styling, color formatting
* [`environment.md`](libraries/solixlib/environment.md) — Environment variables, command-line arguments, process info
* [`exceptions.md`](libraries/solixlib/exceptions.md) — Standard exception hierarchy, stack traces, unwinding
* [`linear_collections.md`](libraries/solixlib/linear_collections.md) — Stack, Queue, Deque, linear buffers
* [`collections_core.md`](libraries/solixlib/collections_core.md) — Collection interfaces, iterators, enumeration contracts
* [`primitives.md`](libraries/solixlib/primitives.md) — Primitive boxing wrappers, parsing, bit-level inspection
* [`resource.md`](libraries/solixlib/resource.md) — Embedded asset extraction from `.slxarc` containers
* [`scaffolding.md`](libraries/solixlib/scaffolding.md) — Project templates, boilerplate scaffolding, test harnesses

---

## 4. CLI Toolchain & Language Server Specifications ([`tools/`](tools/cli/README.md))

* [**CLI Specifications (`tools/cli/`)**](tools/cli/README.md): [`build.md`](tools/cli/build.md), [`compile.md`](tools/cli/compile.md), [`info.md`](tools/cli/info.md), [`install.md`](tools/cli/install.md), [`list.md`](tools/cli/list.md), [`manifest.md`](tools/cli/manifest.md), [`new.md`](tools/cli/new.md), [`run.md`](tools/cli/run.md), [`uninstall.md`](tools/cli/uninstall.md)
* [**Language Server Specifications (`tools/lsp/`)**](tools/lsp/protocol.md): [`protocol.md`](tools/lsp/protocol.md), [`navigation.md`](tools/lsp/navigation.md), [`intelligence.md`](tools/lsp/intelligence.md)
