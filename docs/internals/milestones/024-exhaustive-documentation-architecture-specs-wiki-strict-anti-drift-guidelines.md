# Milestone 024: Exhaustive Documentation Overhaul, Tripartite Architecture (`spec` / `guide` / `wiki`) & Strict Anti-Drift Governance

## Document Metadata

* **Milestone ID:** MS-024
* **Target Version:** `v0.1.0-alpha.24`
* **Milestone Branch:** `milestone/024-documentation-specs-wiki-and-guidelines`
* **Status:** DRAFT (Awaiting Architect Approval)
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/milestones/023-compiler-stabilization-qualified-declarations-and-dual-linking.md` (MS-023)

---

## 1. Executive Summary & Architectural Vision

In the legacy C++ Solix v1.x repository, documentation suffered from two recurring problems:
1. **Role Confusion & Duplication:** User guides (`docs/guide/`), formal specifications (`docs/spec/`), and wiki pages (`docs/wiki/`) duplicated explanations of how features worked internally, making maintenance painful.
2. **Documentation Drift:** When a milestone modified a compiler phase, CLI command, or error code, agents updated one document (or none) and forgot to synchronize the corresponding guide, wiki index, or specification. Furthermore, major architectural differences now exist between Legacy Solix v1.x (stack-based VM, `solix.json`, `.slxbin`, word-addressed memory, C++ plugins) and **Modern Rust Solix v2.0** (register-based VM, 64-bit fixed instructions, superinstructions, `manifest.toml`, `.slxb` / `.slxarc`, `extern import`, namespace-qualified `::` declarations, dual static/dynamic linking, and `E0000`–`E0290` / `R0001`–`R0023` error codes).

**Milestone 024** establishes a permanent, drift-proof documentation architecture before any further feature milestones are merged:

1. **Strict Tripartite Separation of Concerns (`guide/` vs. `spec/` vs. `wiki/`):**
   * **`docs/guide/` (User Guides — *"How to Use It"*):** Practical, task-oriented tutorials grouped by domain. Guides **NEVER** explain deep compiler/VM internals or list exhaustive tables of every flag/error code; instead, they link directly to `docs/spec/` (for how it works under the hood) and `docs/wiki/` (for atomic keyword, primitive, command, and error references).
   * **`docs/spec/` (Formal & Internal Specifications — *"How It Works Under the Hood"*):** Deep, normative engineering specifications for compiler/runtime developers and advanced users. Covers the exact Rust compilation phases (`DependencyResolutionPhase` $\to$ `LexerPhase` $\to$ `ParserPhase` $\to$ `BinderPhase` $\to$ `ApiAndAstExportPhase` $\to$ `AstInlinerPhase` $\to$ `EmitterPhase` $\to$ `BytecodeOptimizerPhase`), the 64-bit Register VM ISA, `VirtualHeap` & ARC mechanics, `.slxb` and `.slxarc` binary layouts, `ModuleSolver`, and ASTlowering for every statement/expression.
   * **`docs/wiki/` (Atomic Reference Lookup — *"Fast Dictionary Reference"*):** Self-contained reference pages for atomic items that are too granular to be full architectural specs:
     * **Error Codes (`docs/wiki/errors/`):** Every Lexer (`E0000`–`E0015`), Parser (`E0101`–`E0200`), Binder (`E0201`–`E0290`), Emission (`E0300`+), and Runtime (`R0001`–`R0023`) error code with cause, bad code example, and fixed code example.
     * **Keywords (`docs/wiki/keywords/`):** All reserved keywords (`control_flow/`, `declarations/`, `modifiers/`, `expressions/`).
     * **Primitives (`docs/wiki/primitives/`):** All 14 built-in primitive types (`int8`..`uint64`, `float32`, `float64`, `bool`, `char`, `string`, `tuple`, `void`) and friendly aliases.
     * **CLI Commands (`docs/wiki/commands/`):** Quick-reference cheat sheets for `compile`, `run`, `build`, `new`, `install`, `uninstall`, `list`, `info`.
2. **Mandatory Pre-Merge Git-Diff Documentation Update Guideline (`GL-0012`):**
   * Before merging **any** milestone branch back to `main`, the agent **MUST** execute `git diff main...HEAD --name-only` and `git diff main...HEAD`, evaluate the **Strict Documentation Trigger Matrix**, and audit the entire `docs/` tree to guarantee zero drift.

---

## 2. Key Differences Between Legacy v1.x Docs and Modern Solix v2.0 (Mandatory Audit Rules)

Every document authored in Milestone 024 **MUST** reflect the modern Rust Solix v2.0 implementation and **MUST NOT** copy obsolete v1.x assumptions:

| Feature / Subsystem | Legacy Solix v1.x (OBSOLETE — DO NOT USE) | Modern Rust Solix v2.0 (NORMATIVE — MUST DOCUMENT) |
| :--- | :--- | :--- |
| **Virtual Machine Model** | Stack-based VM (`PUSH_CONST_I32`, `ADD_I64`, `POP`) | **64-Bit Register-Based VM** (`LoadI32`, `AddI32`, `AddI32Imm`, `JumpIfLtI32`, `CallDirect` sliding register windows) |
| **Instruction Encoding** | Variable-length big-endian byte stream | **Fixed 64-bit little-endian `Instruction(u64)`** (`Format A`, `Format B`, `Format Call`) |
| **Binary Extensions** | `.slxbin` (bytecode) & `.slxpac` (bundle) | **`.slxb`** (Bytecode + `ExportedAst` container) & **`.slxarc`** (Project archive with `bin/`, `lib/`, `resources/`) |
| **Project Manifest** | `solix.json` / `manifest.json` | **`manifest.toml`** (`[info]`, `[entrypoints]`, `[dependencies]`) |
| **Static Member & Enum Access** | Dot `.` (`Math.add`, `Color.RED`) | **Scope resolution `::`** (`Math::add`, `Color::RED`) — accessing static members via `.` is compile error `E0226`! |
| **Namespace Declarations** | Only file-top `package a.b;` | Both `package a.b;` AND inline **qualified declarations** (`class a::b::Console`, `void a::b::fn()`) |
| **External Imports** | Implicit via `solix.json` | **`extern import pkg.Symbol;`** / **`extern import pkg.*;`** + `--deps pkg@ver` + `ModuleSolver` |
| **String Type** | Only `char[]` literal wrapped in `new String(...)` | Both built-in primitive **`string`** (UTF-8 heap string literal) AND standard library class **`solix.core.String`** |
| **Type Casting Syntax** | Only `(Type)expr` | Both C-style **`(Type)expr`** AND intrinsic **`cast(expr, Type)`** + **`default(Type)`** + **`sizeof(Type)`** |
| **Diagnostic Error Codes** | Generic `[E_LEX]`, `[E_PARSE]`, `[E_BIND]`, `[E_ASM]` | Granular codes: **`E0000`–`E0015`** (Lex), **`E0101`–`E0200`** (Parse), **`E0201`–`E0290`** (Bind), **`R0001`–`R0023`** (Runtime) |
| **Standard Library Packages** | `solix.systems.Console`, `solix.exceptions` | **`solix.io.Console`**, **`solix.core`** (includes Exceptions), `solix.collections`, `solix.math`, `solix.time`, `solix.system`, `solix.crypto` |
| **Native Interop (FFI)** | C++ `NativeFunctionPtr(RuntimeContext&, ...)` | **`SolixCValue` & `SolixVmContext`** `#[repr(C)]` C-ABI for Rust `cdylib` (`natives/`) |

---

## 3. Complete `docs/` Directory File Tree to Implement

```text
docs/
├── README.md                                              # Master Documentation Hub & Navigation Index
├── internals/
│   ├── guidelines/
│   │   ├── authoring-docs.md                              # (GL-0010) Updated with Tripartite Architecture Rules
│   │   ├── authoring-features-and-wiki.md                 # (GL-0012 - NEW) How to Author & Update Specs, Guides & Wiki
│   │   └── documentation-update-protocol.md               # (GL-0013 - NEW) Mandatory Pre-Merge Git-Diff Audit Protocol
│   └── milestones/
│       └── ...
├── spec/                                                  # FORMAL & INTERNAL SPECIFICATIONS ("How It Works")
│   ├── README.md                                          # Complete Index of all Specifications
│   ├── lexical-grammar.md                                 # UTF-8, comments, literals, 108 tokens, underscore numbers
│   ├── type-system-and-semantics.md                       # Primitives, classes, interfaces, generics, tuples, fn pointers
│   ├── compilation-pipeline.md                            # All 8 compiler phases, Dual-Mode Linking & Reachability DCE
│   ├── binder-and-monomorphization.md                     # Passes 1, 2, 3, SymbolTable, VTables, ITables, Generic Blueprints
│   ├── optimizer-and-inliner.md                           # AstInlinerPhase & BytecodeOptimizerPhase (Superinstructions)
│   ├── vm-isa-and-execution.md                            # 64-bit Register VM ISA, Instruction Formats A/B/Call, Opcode Table
│   ├── memory-model-and-arc.md                            # VirtualHeap, BlockHeader, Field Alignment, Deterministic ARC & RAII
│   ├── exception-unwinding.md                             # Zero-cost ExceptionHandlerEntry tables & try/catch/finally RAII
│   ├── binary-formats-slxb-and-slxarc.md                  # .slxb binary layout, ExportedAst, and .slxarc archive layout
│   ├── package-solver-and-linking.md                      # ModuleSolver, SemVer compatibility law, dynamic quickening & static LTO
│   ├── native-ffi-abi.md                                  # SolixCValue, SolixVmContext, libloading & Rust cdylib ABI
│   ├── statements/                                        # Statement & Expression Compilation Mechanics (with Register Bytecode!)
│   │   ├── README.md
│   │   ├── modules.md                                     # package, import, extern import, alias, qualified :: declarations
│   │   ├── declarations.md                                # class, interface, enum, field, constructor, ~destructor, method, operator
│   │   ├── generics-and-variadics.md                      # <T : Bound>, <args...>, tuple lowering, monomorphization
│   │   ├── control-flow.md                                # block, var, if/else, while, do-while, for, switch, break, continue, return
│   │   ├── exceptions-and-intrinsics.md                   # try/catch/finally, throw, assert, exit, print/println
│   │   └── expressions.md                                 # binary, unary, assign, ternary, cast, instanceof, sizeof, default, new, [], =>
│   └── solixlib/                                          # Standard Library v2.0.0 Formal Specifications
│       ├── README.md
│       ├── core.md                                        # Interfaces, Primitives, String, StringBuilder, Optional, Result, Exceptions
│       ├── collections.md                                 # List, LinkedList, Deque, Queue, Stack, CircularBuffer, HashMap, TreeMap, BitSet, Algorithms
│       ├── io.md                                          # Console, FileMode, SeekOrigin, IStream, MemoryStream, FileStream, Readers/Writers, Path, File, Dir
│       ├── math.md                                        # Math & xoshiro256** Random
│       ├── time.md                                        # Duration, Instant, DateTime, Stopwatch
│       ├── system.md                                      # Environment & Process
│       └── crypto.md                                      # SHA-256, SHA-1, MD5, Base64, Hex
├── guide/                                                 # DEVELOPER GUIDES ("How to Use It" — Links to spec/ and wiki/)
│   ├── README.md                                          # Guide Index & Learning Path
│   ├── 01-getting-started-and-toolchain.md                # Installing, solix new, build, compile, run, package management
│   ├── 02-variables-primitives-and-strings.md             # Explicit typing, primitives, const, arrays, tuple, string vs String
│   ├── 03-control-flow-and-functions.md                   # Branching, loops, free functions, default params, fn pointers & lambdas
│   ├── 04-classes-oop-and-namespaces.md                   # Classes, :: access, inheritance, interfaces, enums, operators, qualified decls
│   ├── 05-memory-management-arc-and-raii.md               # ARC lifecycle, weak references, ~ClassName() destructors
│   ├── 06-error-handling-and-assertions.md                # try/catch/finally, throwing exceptions, assert & exit
│   ├── 07-generics-and-variadic-templates.md              # Generic classes/methods, type bounds, <args...> variadics
│   ├── 08-packages-dependencies-and-linking.md            # manifest.toml, extern import, ModuleSolver, static vs dynamic linking
│   ├── 09-native-rust-extensions.md                       # Writing Rust cdylib plugins in natives/ with SolixVmContext
│   └── 10-standard-library-tour.md                        # Practical tour of solixlib v2.0.0 with links to spec/solixlib/
└── wiki/                                                  # ATOMIC REFERENCE WIKI (Errors, Keywords, Primitives, CLI Commands)
    ├── README.md                                          # Wiki Hub Index
    ├── errors/                                            # Complete Diagnostic Error Code Reference
    │   ├── README.md                                      # Master Index of E0000..E0350 & R0001..R0023
    │   ├── lexer-errors.md                                # E0000 – E0015
    │   ├── parser-errors.md                               # E0101 – E0200
    │   ├── binder-errors.md                               # E0201 – E0290
    │   ├── emission-errors.md                             # E0300 – E0350
    │   └── runtime-errors.md                              # R0001 – R0023 (Exit codes & VM faults)
    ├── primitives/
    │   ├── README.md                                      # Overview & Conversion Matrix
    │   ├── integers.md                                    # int8, int16, int32, int64, uint8, uint16, uint32, uint64 & aliases
    │   ├── floats.md                                      # float32, float64 & aliases
    │   ├── booleans-and-chars.md                          # bool, char
    │   └── strings-tuples-and-void.md                     # string, tuple, void
    ├── commands/
    │   ├── README.md                                      # CLI Subcommands & Flags Cheat Sheet
    │   ├── compile-and-build.md                           # solix compile, solix build
    │   ├── run-and-info.md                                # solix run, solix info
    │   └── package-commands.md                            # solix new, solix install, solix uninstall, solix list
    └── keywords/
        ├── README.md                                      # All 45+ Keywords Alphabetical & Category Index
        ├── control_flow/                                  # if, else, while, do, for, switch, case, default, break, continue, return, try, catch, finally, throw, assert, exit
        ├── declarations/                                  # class, interface, enum, package, import, alias
        ├── modifiers/                                     # public, private, protected, internal, static, inline, native, const, virtual, override, weak, abstract, extern
        └── expressions/                                   # new, super, this, instanceof, sizeof, default, cast, operator, extends, implements, null, true, false
```

---

## 4. Mandatory Rules in `documentation-update-protocol.md` (`GL-0013`)

`docs/internals/guidelines/documentation-update-protocol.md` (`GL-0013`) **MUST** codify the following **Pre-Merge Git-Diff Enforcement Protocol** and **Cross-Update Trigger Matrix**:

### 4.1 The Pre-Merge Git-Diff Audit Procedure (Mandatory Before Every Merge to `main`)
Before concluding any milestone or merging any feature branch into `main`, the engineer/agent **MUST** execute:
1. `git diff main...HEAD --stat` and `git diff main...HEAD -- crates/ libraries/` to inventory every modified Rust module, AST node, token, error enum, opcode, CLI flag, and standard library class.
2. Cross-reference every code change against the **Documentation Trigger Matrix** below.
3. Scan `docs/README.md`, `docs/spec/README.md`, `docs/guide/README.md`, and `docs/wiki/README.md` to verify all links resolve and no broken relative paths exist.

### 4.2 The Strict Cross-Update Trigger Matrix

| If `git diff main...HEAD` Modifies... | You MUST Inspect & Update in `docs/wiki/` | You MUST Inspect & Update in `docs/spec/` | You MUST Inspect & Update in `docs/guide/` |
| :--- | :--- | :--- | :--- |
| **`TokenKind` / Keywords (`tokens.rs`, `lexer.rs`)** | `wiki/keywords/` + `wiki/primitives/` + `wiki/errors/lexer-errors.md` | `spec/lexical-grammar.md` | Relevant guide in `guide/02..07` |
| **`ParsingError` / Grammar (`parsing.rs`, `parser.rs`)** | `wiki/errors/parser-errors.md` (`E0101`–`E0200`) | `spec/statements/*.md` | Relevant guide if syntax changed |
| **`BindingError` / Type Rules (`binding.rs`, `binder/`)** | `wiki/errors/binder-errors.md` (`E0201`–`E0290`) | `spec/type-system-and-semantics.md` & `spec/binder-and-monomorphization.md` | Relevant guide in `guide/02..08` |
| **`OpCode` / VM Execution (`opcodes.rs`, `vm/fast.rs`)** | `wiki/errors/runtime-errors.md` (`R0001`–`R0023`) | `spec/vm-isa-and-execution.md` & `spec/statements/*.md` bytecode listings | `guide/05-memory-management-arc-and-raii.md` (if ARC/memory changed) |
| **Compiler Phases / `.slxb` / `.slxarc` (`pipeline.rs`, `binary.rs`)** | `wiki/errors/emission-errors.md` | `spec/compilation-pipeline.md` & `spec/binary-formats-slxb-and-slxarc.md` | `guide/08-packages-dependencies-and-linking.md` |
| **CLI Commands / Flags (`cli.rs`, `commands/*.rs`)** | `wiki/commands/*.md` | `spec/package-solver-and-linking.md` | `guide/01-getting-started-and-toolchain.md` |
| **Standard Library (`libraries/solixlib/src/`)** | N/A | `spec/solixlib/*.md` | `guide/10-standard-library-tour.md` |
| **Workflow / Repo Conventions (`crates/` layout)** | N/A | `docs/internals/guidelines/*.md` | `guide/01-getting-started-and-toolchain.md` |

---

## 5. Phased Implementation Plan

```
┌────────────────────────────────────────────────────────────────────────────┐
│ Phase 1: Author Documentation & Pre-Merge Audit Guidelines (GL-0010/12/13) │
│          └─► `authoring-features-and-wiki.md` & `documentation-update...`  │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 2: Author Master Indexes (`docs/README.md`, `spec/`, `guide/`, `wiki`)│
│          └─► Establish full directory tree & verified cross-links          │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 3: Author All Formal Specifications (`docs/spec/**`)                 │
│          └─► Core architecture specs + Register bytecode statement specs   │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 4: Author All Standard Library v2.0.0 Specifications (`spec/solixlib`)│
│          └─► `core`, `collections`, `io`, `math`, `time`, `system`, `crypto`│
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 5: Author Complete Atomic Wiki (`docs/wiki/**`)                      │
│          └─► All error codes (`E0000`–`R0023`), keywords, primitives, CLI  │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 6: Author Concise, Link-Driven User Guides (`docs/guide/01..10`)     │
│          └─► Practical tutorials linking to `spec/` and `wiki/`            │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 7: Automated Link & Code-Snippet Verification Audit                  │
│          └─► Verify 0 broken links and 0 legacy v1.x artefacts remain      │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Artifact | Verification Method | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Guidelines Authored First** | `GL-0010`, `GL-0012`, `GL-0013` with Git-Diff Trigger Matrix | File inspection in `docs/internals/guidelines/` | [ ] Pending |
| **Master Index & File Tree** | `docs/README.md`, `spec/README.md`, `guide/README.md`, `wiki/README.md` | All links resolve to real `.md` files | [ ] Pending |
| **100% v2.0 Accuracy** | Register VM opcodes, `::` syntax, `.slxb`/`.slxarc`, `manifest.toml` | Zero references to `solix.json`, `.slxbin`, or stack opcodes | [ ] Pending |
| **Complete Error Code Wiki** | Every `E0000`–`E0290` and `R0001`–`R0023` documented in `docs/wiki/errors/` | Cross-check against Rust error enums | [ ] Pending |
| **Complete `solixlib` v2.0 Specs** | All 7 packages documented in `docs/spec/solixlib/` | Cross-check against `libraries/solixlib/src/` | [ ] Pending |
| **Link-Driven User Guides** | `docs/guide/01..10` explain usage & link to `spec/` and `wiki/` | Manual & automated link audit | [ ] Pending |