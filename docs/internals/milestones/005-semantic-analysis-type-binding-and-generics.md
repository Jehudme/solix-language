# Milestone 005: Semantic Analysis, Type Binding & On-the-Fly Generics

## Document Metadata
* **Milestone ID:** MS-005
* **Target Version:** `v0.1.0-alpha.5`
* **Milestone Branch:** `milestone/005-semantic-analysis-and-binding`
* **Status:** COMPLETED
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-10-25
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/spec/types.md` (SPEC: Type System & Static Semantics)
  * `docs/internals/milestones/003-recursive-descent-parser.md` (MS-003)
  * `docs/internals/milestones/004-compiler-stabilization-and-incremental-architecture.md` (MS-004)

---

## 1. Executive Summary & Purpose

With lexical tokenization (MS-002) and recursive-descent parsing (MS-003) fully operational, the compiler possesses a high-fidelity Abstract Syntax Tree (AST) partitioned across source files in `CompilationContext`. However, the raw AST contains unverified identifier strings, unresolved type annotations, unvalidated inheritance chains, and un-instantiated generic blueprints.

The **Semantic Binder** (`phases/binder/`) is the central intelligence engine of the Solix compiler. Drawing from the architecture of the reference C++ compiler while eliminating raw pointer hazards and cyclic mutability through idiomatic Rust dataflow, Milestone 005 accomplishes five critical goals:

1. **Multi-Pass Decoupled Semantic Architecture:**
   Because Solix supports out-of-order declarations, circular type references, and mutual dependencies across files, semantic binding executes across discrete sequential phases:
   * **Pass 1: Symbol Discovery & Package Outline Registration:** Discovers all packages, aliases, and top-level types (classes, interfaces, enums) and populates the global symbol table.
   * **Pass 2: Type Hierarchy, Virtual Tables & Memory Layout:** Resolves superclasses (`extends`) and interfaces (`implements`), detects inheritance cycles, calculates class VTables, ITables, instance memory layouts, and ARC reference field offset bitmasks.
   * **Pass 3: Statement & Expression Type-Checking, Scoping & Overload Resolution:** Validates expressions, checks numeric widening chains, enforces access modifiers (`public`, `private`, `protected`), resolves method and constructor overloads, and manages lexical call-frame slot allocations.
2. **On-the-Fly Generic Monomorphization:**
   Whenever a generic type (e.g. `List<int32>`, `Container<K, V>`) or generic method is encountered during type resolution, the binder dynamically looks up the generic blueprint, synthesizes a concrete AST clone, substitutes type parameters with concrete arguments, and executes a mini-pipeline (Pass 1 -> Pass 2 -> Pass 3) to compile the monomorphized type on the fly.
3. **Partitioned Public Symbol Storage & Global Lookup:**
   To cleanly support both incremental LSP compilation and workspace batch builds, public symbols are cataloged per-source (`public_symbols: Vec<HashMap<String, SymbolId>>`) alongside a unified qualified global index (`global_symbols: HashMap<String, SymbolId>`). Each symbol explicitly retains its originating `source_id: u16`.
4. **Sound Incremental Rebinding with Full Rebind Fallback:**
   Provides both a blazingly fast full rebind (`bind_context`), which rebuilds all valid ASTs in mere milliseconds with zero risk of stale state, and an incremental driver (`bind_incremental`) that queries `get_transitive_dependents(source_id)` to rebind only the affected file and its downstream dependents when internal method bodies change.
5. **Decoupled Diagnostics in `solix-common`:**
   Standardizes all semantic error diagnostics (`BindingError`) with standardized codes `E0201`–`E0260` in `solix-common`, enabling direct consumption by `solix-lsp` for hover documentation, diagnostics, and go-to-definition.

---

## 2. Scope & Crates Affected

### 2.1 `crates/solix-common`
* `src/types.rs`: Canonical compiler type representation:
  * `Type` enum (`Primitive`, `Class`, `Interface`, `Enum`, `Array`, `Tuple`, `FunctionPointer`, `GenericParam`, `Void`, `Error`).
  * Primitive numeric conversion helpers: `is_widening_convertible_to`, `is_narrowing_convertible_to`, `is_assignable_to`.
  * Display and serialization implementations.
* `src/diagnostics/binding.rs`:
  * `BindingError` enum deriving `thiserror::Error` and `miette::Diagnostic` with standardized error codes:
    * `E0201`: Undefined symbol / unresolved type name.
    * `E0202`: Duplicate symbol declaration.
    * `E0203`: Type mismatch in assignment or expression.
    * `E0204`: Cyclic class inheritance detected (`A extends B extends A`).
    * `E0205`: Invalid base class (cannot extend interface, enum, or final class).
    * `E0206`: Unimplemented interface method in concrete class.
    * `E0207`: Method signature override mismatch.
    * `E0208`: Access violation (attempt to access `private` or `protected` member).
    * `E0209`: Cannot assign to `const` variable or field.
    * `E0210`: Variable used before initialization.
    * `E0211`: Return type mismatch in function or method.
    * `E0212`: Method overload ambiguity.
    * `E0213`: Generic arity mismatch (expected $N$ type arguments).
    * `E0214`: Type bound constraint violation.
    * `E0215`: Non-constant case label in switch.
    * `E0216`: Break or continue outside loop or switch.
    * `E0217`: Super constructor call not first statement in constructor.
    * `E0218`: Missing return statement in non-void method.
    * `E0219`: Cannot instantiate abstract class or interface.
* `src/diagnostics/mod.rs` & `src/lib.rs`: Top-level re-exports of `Type` and `BindingError`.

### 2.2 `crates/solix-compilation`
* `src/phases/binder/mod.rs`: Module exports and entrypoints.
* `src/phases/binder/types.rs`: Semantic type representation, primitive table, and type deduction engine.
* `src/phases/binder/symbols.rs`:
  * `SymbolId`, `SymbolKind` (Class, Interface, Enum, Field, Method, Constructor, Parameter, LocalVar, Alias).
  * `SymbolEntry` (name, fully_qualified_name, source_id, span, visibility, type_info).
  * `SymbolTable` managing global table, per-source public exports, and lexical scope stack.
* `src/phases/binder/passes/`:
  * `pass1_symbol_discovery.rs`: Outlines packages, imports, top-level type declarations, aliases, and registers generic blueprints.
  * `pass2_type_hierarchy.rs`: Resolves inheritance, checks cycles, validates interface implementations, generates VTables and ITables, and calculates instance field offsets.
  * `pass3_type_checking.rs`: Traverses AST nodes, validates expressions, assigns local stack slots, resolves overloads, and tracks ARC reference lifecycles.
  * `monomorphizer.rs`: Dynamic generic template clone and on-the-fly monomorphization engine.
* `src/phases/binder/binder.rs`: Driver orchestrating `bind_context`, `bind_source`, and `bind_incremental`.
* `src/CompilationContext.rs`:
  * Added `binding_artifacts: Option<BindingArtifacts>`.
  * `BindingArtifacts` stores:
    * `symbol_table: SymbolTable`
    * `node_types: HashMap<Span, Type>` (powers hover tooltips in LSP)
    * `resolved_symbols: HashMap<Span, SymbolId>` (powers go-to-definition in LSP)
    * `class_layouts: HashMap<SymbolId, ClassLayout>` (instance size, field offsets, vtable)
    * `monomorphized_classes: Vec<ProgramAst>` (ASTs generated on-the-fly for generic instantiations)

### 2.3 `crates/solix-lsp`
* `src/hover.rs`: LSP hover handler querying `context.binding_artifacts.node_types` by cursor position.
* `src/definition.rs`: LSP definition handler querying `context.binding_artifacts.resolved_symbols` to navigate to the exact symbol definition `Span`.

---

## 3. Prioritized Deliverables & Task Matrix

### 3.1 P0: Critical Blockers (Release Gate)

- [x] `[P0]` **Dedicated Milestone Branch (`milestone/005-semantic-analysis-and-binding`):**
  Cut and isolate all binder work in `milestone/005-semantic-analysis-and-binding` conforming to GL-0002.

- [x] `[P0]` **Canonical Type Representation (`crates/solix-common/src/types.rs`):**
  Implement the strongly-typed `Type` model:
  * Primitives: `bool`, `char`, `int8`..`int64`, `uint8`..`uint64`, `float32`, `float64`, `void`, `tuple`.
  * Nominal user types: `Class`, `Interface`, `Enum` parameterized with `SymbolId` and optional type arguments.
  * Structural types: `Array`, `Tuple`, `FunctionPointer`.
  * Conversions: `is_widening_convertible_to`, `is_narrowing_convertible_to`, `is_assignable_to`.

- [x] `[P0]` **Canonical Semantic Diagnostics (`crates/solix-common/src/diagnostics/binding.rs`):**
  Define `BindingError` with standardized diagnostic codes `E0201` through `E0220` with miette source spans and descriptive help messages.

- [x] `[P0]` **Symbol Table & Scoping Model (`crates/solix-compilation/src/phases/binder/symbols.rs`):**
  Implement hierarchical symbol table with `SymbolId`, visibility checking (`public`, `protected`, `private`, `internal`), per-source exports vector (`Vec<HashMap<String, SymbolId>>`), and lexical scope stack for local variables and parameters.

### 3.2 P1: High Priority (Milestone Core)

- [x] `[P1]` **Pass 1: Symbol Discovery & Package Outline Registration:**
  * Traverse AST across all files in `context.valid_asts()`.
  * Process `package` and `import` statements (both explicit and wildcard `.*`).
  * Register all top-level types (Classes, Interfaces, Enums, Aliases) in `global_symbols` under their fully-qualified names (FQNs) and per-source export map.
  * Register generic blueprint ASTs in `GenericBlueprintRegistry`.

- [x] `[P1]` **Pass 2: Type Hierarchy, VTables & Memory Layout:**
  * Resolve superclass (`extends`) and interface (`implements`) annotations.
  * Detect and reject cyclic inheritance graphs (`E0204`).
  * Verify all abstract and interface method contracts are fulfilled by concrete classes (`E0206`).
  * Construct deterministic virtual dispatch tables (VTables) and interface tables (ITables).
  * Calculate class instance layouts, field memory offsets, total byte sizes, and ARC reference field offset lists.

- [x] `[P1]` **Pass 3: Type Checking, Scoping & Expression Validation:**
  * Traverse statements and expressions using the AST Visitor pattern (`AstVisitor`).
  * Push and pop local variable scopes, assigning call-frame `slot_index` offsets.
  * Type-check all expressions against the Solix specification (`docs/spec/types.md`):
    * Numeric widening and narrowing conversions.
    * Arithmetic, logical, relational, and bitwise operations.
    * Member access (`obj.field`, `obj.method(...)`) with access control checks.
    * Array indexing, length checking, and array instantiation.
    * Method and constructor overload resolution with argument type matching.
  * Record expression types into `BindingArtifacts::node_types` indexed by `Span`.
  * Record symbol references into `BindingArtifacts::resolved_symbols` indexed by `Span`.

- [x] `[P1]` **On-the-Fly Generic Monomorphization Engine:**
  * When a parameterized type (e.g. `Box<int32>`, `Map<string, User>`) is referenced:
    * Construct canonical mangled name (e.g. `solix.collections.Box<int32>`).
    * Check if the monomorphized type is already compiled in the symbol table.
    * If absent, clone the blueprint AST, substitute generic type parameters recursively with concrete arguments, and execute Pass 1, Pass 2, and Pass 3 on the synthesized class.
    * Register the instantiated type in `global_symbols`, calculate its dedicated VTable, and append the generated AST to `context.binding_artifacts.monomorphized_classes`.

- [x] `[P1]` **Symmetrical Single-Source & Context Binding APIs:**
  * `bind_context(context: &mut CompilationContext) -> Result<BindingMetaData, Vec<BindingError>>`
  * `bind(context)` direct alias for `bind_context`.
  * `bind_incremental(context: &mut CompilationContext, changed_source_id: u16) -> Result<BindingMetaData, Vec<BindingError>>` utilizing `context.get_transitive_dependents(changed_source_id)`.

### 3.3 P2: Medium Priority (LSP Verification & Test Suite)

- [x] `[P2]` **Master Positive Semantic Fixture Verification:**
  * Bind the master fixture `crates/solix-compilation/tests/fixtures/syntax/positive/master_all_statements.slx`.
  * Assert zero semantic errors, verify all classes, interfaces, enums, lambdas, generics, and operator overloads bind completely.

- [x] `[P2]` **Master Negative Semantic Fixture & Diagnostic Assertions:**
  * Create `tests/fixtures/semantics/negative/master_invalid_semantics.slx` containing targeted semantic errors.
  * Verify that all diagnostic codes (`E0201`–`E0220`) are emitted with accurate source spans.

- [x] `[P2]` **LSP Semantic Bridge & Integration:**
  * Connect `solix-lsp` to `context.binding_artifacts`:
    * Implement hover provider resolving expression type strings at cursor offset.
    * Implement definition provider jumping to definition `Span`.

---

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Compiles with 0 warnings across all targets | `cargo build --workspace --all-targets` | [x] Passed |
| **Automated Test Suite** | 100% pass across all unit and integration tests | `cargo test --workspace --all-targets` | [x] Passed |
| **Clippy Static Analysis** | 0 warnings, strict pass | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Passed |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [x] Passed |
| **Safety Invariants** | `#![forbid(unsafe_code)]`, zero unwrap, zero single-letter variables | Workspace source audit | [x] Passed |
| **Generics Monomorphization**| On-the-fly generic class and method instantiations verified | `cargo test --test binder_tests` | [x] Passed |
| **Semantic Diagnostics** | Error codes `E0201`–`E0225` verified on invalid semantic fixtures | `cargo test --test binder_tests` | [x] Passed |

---

## 5. Architectural Blueprint

```
                      ┌───────────────────────────────────────┐
                      │             solix-common              │
                      │  • Type (Primitive, Class, Array...)  │
                      │  • BindingError (E0201..E0220)        │
                      └───────────────┬───────────────────────┘
                                      │
                                      ▼
                      ┌───────────────────────────────────────┐
                      │    solix-compilation::phases::binder  │
                      └───────┬───────────────────────┬───────┘
                              │                       │
      ┌───────────────────────▼──────┐         ┌──────▼────────────────────────┐
      │   Pass 1: Symbol Discovery   │         │ On-The-Fly Monomorphization   │
      │   • Package outlines         │         │ • Template Blueprint Registry │
      │   • Top-level declarations   │ ◄───────┤ • AST Cloning & Substitution  │
      │   • FQN & Import mapping     │         │ • Sub-pipeline instantiation  │
      └───────────────┬──────────────┘         └───────────────────────────────┘
                      │
      ┌───────────────▼──────────────┐
      │  Pass 2: Type Hierarchy &    │
      │          Memory Layout       │
      │   • Extends & Implements     │
      │   • Inheritance cycle check  │
      │   • VTable / ITable IDs      │
      │   • Field offsets & ARC mask │
      └───────────────┬──────────────┘
                      │
      ┌───────────────▼──────────────┐
      │  Pass 3: Type Checking &     │
      │          Execution Binding   │
      │   • Lexical ScopeStack       │
      │   • Expression typing        │
      │   • Overload resolution      │
      │   • ARC scope exit tracking  │
      └───────────────┬──────────────┘
                      │
                      ▼
      ┌──────────────────────────────┐
      │       BindingArtifacts       │
      │  • symbol_table              │
      │  • node_types: Span -> Type  │ ────────► solix-lsp (Hover & Navigation)
      │  • resolved: Span -> SymbolId│
      │  • class_layouts             │
      │  • monomorphized_classes     │
      └──────────────────────────────┘
```
