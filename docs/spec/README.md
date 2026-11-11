# Solix Formal Specifications Index

This directory contains the authoritative, normative technical specifications for the Solix language compiler, 64-bit register-based virtual machine, memory runtime, standard library, and package toolchain.

Per [`GL-0012`](file:///home/jehud/Projects/solix-language/docs/internals/guidelines/authoring-features-and-wiki.md), specifications serve as the normative source of truth for systems engineers and toolchain contributors.

---

## 1. Core Language & Compiler Architecture Specifications

* [**Lexical Grammar & Tokenization**](file:///home/jehud/Projects/solix-language/docs/spec/lexical-grammar.md): UTF-8 encoding, comments, string escapes, number literals (hex, binary, underscore separators), and complete 108 token dictionary.
* [**Type System, Hierarchy & Semantics**](file:///home/jehud/Projects/solix-language/docs/spec/type-system-and-semantics.md): Primitive types, reference classes, structural tuples, function pointers, nominal subtyping, and operator overloading rules.
* [**Compilation Pipeline & Linking Passes**](file:///home/jehud/Projects/solix-language/docs/spec/compilation-pipeline.md): Comprehensive 8-phase pipeline (`DependencyResolutionPhase` $\to$ `LexerPhase` $\to$ `ParserPhase` $\to$ `BinderPhase` $\to$ `ApiAndAstExportPhase` $\to$ `AstInlinerPhase` $\to$ `EmitterPhase` $\to$ `BytecodeOptimizerPhase`), dual static/dynamic linking, and reachability Dead Code Elimination (DCE).
* [**Binder, VTables, ITables & Monomorphization**](file:///home/jehud/Projects/solix-language/docs/spec/binder-and-monomorphization.md): Three-pass semantic analysis, symbol table layout, class hierarchy construction, vtable/itable offsets, and template instantiation.
* [**Optimizer, Inliner & Superinstructions**](file:///home/jehud/Projects/solix-language/docs/spec/optimizer-and-inliner.md): AST function and method inlining, constant folding, bytecode peephole optimization, and superinstruction fusing.
* [**64-Bit Register VM ISA & Opcode Matrix**](file:///home/jehud/Projects/solix-language/docs/spec/vm-isa-and-execution.md): Register execution model, Instruction Formats A/B/Call, sliding register window ABI, and full 256-opcode instruction set architecture.
* [**Deterministic Memory Model, VirtualHeap & ARC**](file:///home/jehud/Projects/solix-language/docs/spec/memory-model-and-arc.md): Segregated free-list memory allocator, 24-byte `BlockHeader`, field alignment, strong/weak reference counting, and deterministic RAII destructors.
* [**Zero-Cost Exception Handling & Unwinding**](file:///home/jehud/Projects/solix-language/docs/spec/exception-unwinding.md): Table-driven `ExceptionHandlerEntry` structures, try/catch/finally unwinding, and stack frame restoration.
* [**Binary Formats (`.slxb` & `.slxarc`)**](file:///home/jehud/Projects/solix-language/docs/spec/binary-formats-slxb-and-slxarc.md): Bytecode container specification, magic header (`b"SLXB"`), section tables, `ExportManifest`, pre-parsed AST, and compressed `.slxarc` archive layout.
* [**Package Solver, SemVer & Dual-Mode Linking**](file:///home/jehud/Projects/solix-language/docs/spec/package-solver-and-linking.md): Pluggable `ModuleSolver` trait, SemVer 2.0 compatibility rules, `extern import` resolution, in-place dynamic call quickening, and static LTO.
* [**Native C-FFI ABI & Dynamic Extensions**](file:///home/jehud/Projects/solix-language/docs/spec/native-ffi-abi.md): `SolixCValue`, `SolixVmContext`, `#[repr(C)]` plugin bridge, and dynamic shared library loading via `libloading`.

---

## 2. Statement & Expression Compilation Specifications ([`statements/`](file:///home/jehud/Projects/solix-language/docs/spec/statements/README.md))

Detailed specifications defining the AST representations, binder type rules, and 64-bit Register VM lowering for every language construct:

* [**Modules, Imports & Qualified Namespaces**](file:///home/jehud/Projects/solix-language/docs/spec/statements/modules.md): `package`, `import`, `extern import`, `alias`, and qualified `::` declarations.
* [**Declarations & OOP Members**](file:///home/jehud/Projects/solix-language/docs/spec/statements/declarations.md): `class`, `interface`, `enum`, constructors, `~destructors`, methods, fields, and operator overloads.
* [**Generics & Variadic Templates**](file:///home/jehud/Projects/solix-language/docs/spec/statements/generics-and-variadics.md): Type parameter bounds (`<T : Bound>`), variadic packs (`<args...>`), tuple lowering, and monomorphization blueprints.
* [**Control Flow Statements**](file:///home/jehud/Projects/solix-language/docs/spec/statements/control-flow.md): `if`/`else`, `while`, `do`/`while`, `for`, `switch`/`case`, `break`, `continue`, and `return`.
* [**Exceptions & Runtime Intrinsics**](file:///home/jehud/Projects/solix-language/docs/spec/statements/exceptions-and-intrinsics.md): `try`/`catch`/`finally`, `throw`, `assert`, `exit`, and `print`/`println`.
* [**Expressions & Operators**](file:///home/jehud/Projects/solix-language/docs/spec/statements/expressions.md): Binary/unary operators, ternary operator, casts (`cast(e, T)` & `(T)e`), `instanceof`, `sizeof`, `default`, `new`, indexing `[]`, and closures `=>`.

---

## 3. Standard Library v2.0 Specifications ([`solixlib/`](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/README.md))

Normative contracts for all 7 standard library packages in `libraries/solixlib/`:

* [**Core (`solix.core`)**](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/core.md): Standard interfaces, primitive wrappers, `String`, `StringBuilder`, `Optional<T>`, `Result<T, E>`, and standard exception hierarchy.
* [**Collections (`solix.collections`)**](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/collections.md): Generic data structures (`List`, `LinkedList`, `Deque`, `Queue`, `Stack`, `CircularBuffer`, `HashMap`, `TreeMap`, `BitSet`) and generic `Algorithms`.
* [**I/O & File System (`solix.io`)**](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/io.md): `Console`, streams (`IStream`, `MemoryStream`, `FileStream`), stream readers/writers, `Path`, `File`, and `Dir`.
* [**Mathematics & Randomness (`solix.math`)**](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/math.md): Transcendental functions, numeric bounds, and deterministic `xoshiro256**` PRNG (`Random`).
* [**Time & Date (`solix.time`)**](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/time.md): Monotonic `Instant`, wall-clock `DateTime`, `Duration`, and high-resolution `Stopwatch`.
* [**System & Process (`solix.system`)**](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/system.md): `Environment` variables, process arguments, exit codes, and process spawning (`Process`).
* [**Cryptography & Encoding (`solix.crypto`)**](file:///home/jehud/Projects/solix-language/docs/spec/solixlib/crypto.md): SHA-256, SHA-1, MD5, Base64 encoding/decoding, and Hex utilities.
