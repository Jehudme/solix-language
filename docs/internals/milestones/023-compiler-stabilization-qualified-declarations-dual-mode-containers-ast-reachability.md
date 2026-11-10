# Milestone 023: Compiler Stabilization, Namespace-Qualified Declarations (`::`), Dual-Mode Binary Containers (`ExportedAst` + Bytecode) & Whole-Program Reachability

## Document Metadata

* **Milestone ID:** MS-023
* **Target Version:** `v0.1.0-alpha.23`
* **Milestone Branch:** `milestone/023-stabilization-qualified-declarations-and-dual-linking`
* **Status:** DRAFT (Awaiting Architect Approval)
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/milestones/020-package-manager-solver-and-dependency-resolution.md` (MS-020)
  * `docs/internals/milestones/021-comprehensive-codebase-remediation-and-conformance.md` (MS-021)
  * `docs/internals/milestones/022-standard-library-modernization-and-project-scaffolding.md` (MS-022)

---

## 1. Executive Summary & Architectural Vision

During Milestone 022 (`solixlib` v2.0.0 integration), downstream consumer compilation (`extern import solix.io.Console;`) exposed fundamental architectural limitations in how multi-package libraries export their APIs, how the compiler ingests external modules, and how generic/tuple casts are type-checked and emitted:

1. **Multi-Package Header Flattening vs. Namespace Collision:**
   * A single `.slx` source file only allows a single `package <name>;` declaration at the top of the file. When a multi-package library like `solixlib` (`solix.core`, `solix.io`, `solix.collections`, `solix.math`, `solix.time`, `solix.system`, `solix.crypto`) was synthesized into a single header file (`solix@2.0.0.slx`), all classes were flattened under `package solix;`.
   * Attempting to work around this in Pass 1 via import normalization (`solix.io.Console` $\to$ `solix::Console`) is an unacceptable hack that causes symbol collisions when different subpackages define identically named types.
2. **Cold-Start Compilation Latency & Transitive API Re-Export Pollution:**
   * Reconstructing `.slx` source text from JSON (`ExportManifest`) and re-lexing, re-parsing, re-binding, and running `AstInlinerPhase` across 3,500+ lines of uninstantiated standard library collection templates on every 4-line user script causes massive cold-start compilation latency.
   * Worse, `emitter/pipeline.rs` iterated over all ASTs in `context.ast` (including imported dependency ASTs), causing `test.slxb` to re-export the entire `solixlib` API inside `test.slxb`!
3. **`solixlib` Cast & Generic Method Monomorphization Bugs:**
   * Indexing a `tuple` (`TuplePrimitive`) in Pass 3 (`pass3_type_checking.rs:403`) unconditionally returned `Type::Primitive(PrimitiveKind::Int32)`, causing `(T)args[i]` or `(String)args[i]` casts to fail or emit broken `CastCheck` instructions.
   * Downcasting from an `interface` (`IStream`, `IIterable<T>`) to a concrete class was rejected by `check_is_subtype`.
   * Generic static methods on non-generic classes (`Collections::to_string<T>`, `Primitives::hash_any<T>`, `String::from_any<T>`, `Algorithms::sort<T>`) were neither preserved in API headers nor monomorphized per call-site type argument.

**Milestone 023** resolves all of these architectural issues cleanly and permanently through **Five Core Pillars**:

1. **Pillar 1: Namespace-Qualified Declarations (`class solix::io::Console`, `void solix::math::sin(...)`):**
   * Completely removes the Pass 1 import-normalization hack.
   * Extends the Parser and Binder so `class`, `interface`, `enum`, `alias`, and top-level `method`/`field` declarations accept `::`-qualified namespace paths (e.g. `extern public class solix::io::Console { ... }`).
   * Allows a single file (or generated `solix info --api` output) with no `package` header to declare symbols across arbitrary subpackages with **zero collisions**.
2. **Pillar 2: Pre-Generated Local-Only API Header & Pruned `ExportedAst` in `.slxb`:**
   * Replaces the bloated structural `ExportManifest` JSON AST dump with a post-binding **`ApiAndAstExportPhase`** that produces:
     1. A clean, pre-generated `api_header: String` (using `class pkg::sub::Name` syntax) for instant `solix info --api` inspection.
     2. A bincode-serialized, pruned **`exported_ast: Vec<u8>`** (`ExportedModuleAst`) stored in the `.slxb` binary container so downstream compilations deserialize pre-parsed AST nodes in $< 1\text{ ms}$ **without lexing or parsing library text**!
   * **Strict Local-Only & Public-Only Invariant:**
     * Any declaration marked `extern` (imported from a dependency) is **strictly excluded** from the current module's exported API and AST, preventing transitive pollution (`test.slxb` only exports `test`'s own public symbols).
     * Private classes, private functions, private aliases, private static fields, and private methods are **omitted**.
     * **Exception for Memory Layout Integrity:** `private` and `protected` **instance fields** inside a `public class` **MUST** be retained in the exported class declaration so downstream subclasses and `sizeof(Class)` compute 100% identical field byte offsets and `instance_size_bytes`.
3. **Pillar 3: Dual-Mode Linking (`--link dynamic` vs. `--link static`) & Whole-Program Reachability (DCE):**
   * Every library `.slxb` / `.slxarc` is a **Dual-Mode Container** holding both compiled bytecode (for dynamic linking) and the serialized AST (for static linking and generic monomorphization).
   * **`--link dynamic` (Default):** Deserializes only public signatures and `extern` generic templates from `exported_ast`, skips Pass 3 body checking and `AstInlinerPhase` on non-local declarations, and emits `OpCode::CallExtern` for library calls. Cold-start compilation drops to $< 10\text{ ms}$.
   * **`--link static`:** Merges the full library AST into the executable compilation unit, runs a **Call-Graph Reachability Pass (Dead-Code Elimination)** starting from `entrypoints.main`, and emits + optimizes only the reachable classes and functions into a single standalone `.slxb` binary.
4. **Pillar 4: Complete Fix for `solixlib` Casts, Interface Downcasts & Generic Method Monomorphization:**
   * Fixes `TuplePrimitive` subscripting (`t[i]`) and `(T)t[i]` / `cast(t[i], T)` in Pass 3 and Emitter (`OpCode::TupleCast` / `OpCode::GetTuple`).
   * Allows explicit downcasting from `Type::Interface` to `Type::Class` or sub-interfaces via runtime `OpCode::CastCheck`.
   * Implements full AST specialization for generic methods on non-generic classes (`Collections::to_string<T>`, `Primitives::hash_any<T>`, `String::from_any<T>`, `Algorithms::sort<T>`).
5. **Pillar 5: Binder & Pipeline Performance Hardening:**
   * Replaces $O(N \times M)$ linear string scans over `global_names` in Pass 1 import validation with an $O(1)$ `known_packages: HashSet<String>` index.

---

## 2. Detailed Architectural Specifications

### 2.1 Pillar 1: Namespace-Qualified Declarations (`a::b::Name`)

```
┌────────────────────────────────────────────────────────────────────────────┐
│                Namespace-Qualified Declaration Grammar                     │
├────────────────────────────────────────────────────────────────────────────┤
│  QualifiedIdentifier ::= Identifier ( '::' Identifier )*                   │
│                                                                            │
│  ClassDeclaration     ::= Modifier* 'class' QualifiedIdentifier ...        │
│  InterfaceDeclaration ::= Modifier* 'interface' QualifiedIdentifier ...    │
│  EnumDeclaration      ::= Modifier* 'enum' QualifiedIdentifier ...         │
│  AliasStatement       ::= 'alias' QualifiedIdentifier ('<' ... '>')? '='   │
│  MethodDeclaration    ::= Modifier* Type QualifiedIdentifier '(' ... ')'   │
│  FieldDeclaration     ::= Modifier* Type QualifiedIdentifier ('=' ...)? ';'│
└────────────────────────────────────────────────────────────────────────────┘
```

#### Semantics & Symbol Resolution Rules:
1. **AST Representation (`crates/solix-compilation/src/phases/parser/statements/`):**
   * Add `pub namespace_path: Vec<Span>` (or `qualified_name_spans: Vec<Span>`) to `ClassDeclaration`, `InterfaceDeclaration`, `EnumDeclaration`, `MethodDeclaration`, `FieldDeclaration`, and `AliasStatement`.
   * `name_span: Span` continues to point to the final segment (`Console` in `solix::io::Console`), preserving 100% compatibility with constructor name matching (`Console(...)`) and destructor name matching (`~Console()`).
2. **Pass 1 Canonical FQN Calculation (`pass1_symbol_discovery.rs`):**
   * Let `file_pkg` be the `package` statement of the current file (empty `""` if omitted).
   * Let `decl_ns` be the leading `::` segments of the declaration (`"solix::io"` in `class solix::io::Console`, or `""` in `class Console`).
   * The effective package prefix of the declaration is:
     $$\text{effective\_pkg} = \begin{cases} \text{decl\_ns} & \text{if } \text{file\_pkg is empty} \\ \text{file\_pkg} + \text{"::"} + \text{decl\_ns} & \text{if both non-empty and } \neg\text{decl\_ns.starts\_with(file\_pkg)} \\ \text{decl\_ns} & \text{if } \text{decl\_ns.starts\_with(file\_pkg)} \end{cases}$$
   * Register `effective_pkg` (and all parent package prefixes only as package namespaces where appropriate) into `symbol_table.known_packages`.
   * Register the symbol in `symbol_table` with:
     * `name`: `"Console"`
     * `fully_qualified_name`: `"solix::io::Console"`
   * Inside the body of `class solix::io::Console`, sibling types in `solix::io` are automatically in scope because the class's enclosing package context is `"solix::io"`.
3. **Complete Removal of Pass 1 Import Normalization:**
   * Remove Step 4.5 from `pass1_symbol_discovery.rs`. Imports (`import solix.io.Console;`, `extern import solix.io.Console;`, `import solix.io.*;`) resolve directly against `"solix::io::Console"` and package `"solix::io"`.

---

### 2.2 Pillar 2: `ApiAndAstExportPhase`, Visibility Filtering & Binary AST Format

#### 1. What Gets Exported vs. Pruned (`ApiAndAstExportPhase`)

| Construct | `private` / `internal` Top-Level | `public` Top-Level (Concrete) | `public` Top-Level (Generic Template) | Imported (`extern` from Dependency) |
| :--- | :---: | :---: | :---: | :---: |
| **Class / Interface** | **Omitted** | Exported as `extern public class pkg::Name` | Exported as `extern public class pkg::Name<T>` with method bodies | **Omitted (Never Re-Exported)** |
| **Instance Fields (`public`, `protected`, `private`)** | N/A | **ALL Kept** (Required for exact byte offset & `sizeof` layout) | **ALL Kept** | **Omitted** |
| **Static Fields** | **Omitted** | Kept ONLY if `public` or `protected` (`private static` omitted) | Kept if referenced by template methods | **Omitted** |
| **Methods & Constructors** | **Omitted** | Kept ONLY if `public` or `protected` (body stripped to `None` unless generic/inline) | Kept (with body for monomorphization) | **Omitted** |
| **Destructors (`~Class()`)** | **Omitted** | **Omitted** from header (handled via VTable `destructor_ip` at runtime) | Kept if template class defines `~Class()` | **Omitted** |
| **Enums & Aliases** | **Omitted** | Exported as `public enum pkg::Name` / `alias pkg::Name = ...` | Exported with `<T>` | **Omitted** |

#### 2. Self-Contained Source-Independent AST Serialization (`ExportedModuleAst`)
Because AST nodes normally store `Span { source_id, start, end }` indexing into a `source_text: String`, serializing `ProgramAst` alongside its compact interned source string table (`ExportedUnit { source_name: String, source_text: String, ast: ProgramAst }`) allows bincode to serialize and deserialize the pruned `ProgramAst` in **under $500\ \mu\text{s}$** with zero lexing and zero parsing!

```rust
/// Pre-parsed, pruned interface & template AST stored inside `.slxb` containers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExportedModuleAst {
    /// Compact generated Solix API declaration source (using `pkg::Class` syntax).
    pub api_header_source: String,
    /// Pre-parsed AST of `api_header_source` (and full generic/static ASTs when available),
    /// ready to be injected directly into `CompilationContext::ast` without lexing or parsing!
    pub parsed_api_ast_json_or_bincode: Vec<u8>,
}
```

---

### 2.3 Pillar 3: Dual-Mode Linking (`--link dynamic` vs. `--link static`) & Reachability Pass

```
┌────────────────────────────────────────────────────────────────────────────┐
│                   Dual-Mode Compilation Pipeline                           │
├────────────────────────────────────────────────────────────────────────────┤
│ 1. DependencyResolutionPhase                                               │
│    • Fetches SolvedPackage via ModuleSolver.                               │
│    • Deserializes `ExportedModuleAst` directly into `context.sources` &    │
│      `context.ast` (marks source as `is_external_dependency = true`).      │
│    • ZERO Lexing and ZERO Parsing of dependency libraries!                 │
├────────────────────────────────────────────────────────────────────────────┤
│ 2. LexerPhase & ParserPhase                                                │
│    • Skips any `source_id` where `context.ast[source_id].is_some()`!       │
│    • Only lexes and parses the user's own `.slx` files!                    │
├────────────────────────────────────────────────────────────────────────────┤
│ 3. BinderPhase (Pass 1, Pass 2, Pass 3)                                    │
│    • Pass 1 & Pass 2: Index all symbols and layouts across user + deps.    │
│    • Pass 3:                                                               │
│      - In `Dynamic` link mode: skips type-checking method bodies of        │
│        external dependency sources (only type-checks monomorphized         │
│        specializations instantiated by the user!).                         │
│      - In `Static` link mode: runs Reachability Analysis from `main`       │
│        and type-checks + un-marks `extern` on reachable library functions. │
├────────────────────────────────────────────────────────────────────────────┤
│ 4. ApiAndAstExportPhase (New!)                                             │
│    • Generates `api_header` and `exported_ast` ONLY from local user        │
│      sources (`!is_external_dependency` and `!was_imported_extern`).       │
├────────────────────────────────────────────────────────────────────────────┤
│ 5. AstInlinerPhase -> EmitterPhase -> BytecodeOptimizerPhase               │
│    • Dynamic mode: emits bytecode only for local functions + instantiated  │
│      templates; external calls emit `OpCode::CallExtern`.                  │
│    • Static mode: inlines across module boundaries and emits all reachable │
│      functions into a single self-contained `.slxb` binary!                │
└────────────────────────────────────────────────────────────────────────────┘
```

---

### 2.4 Pillar 4: Fixing All `solixlib` Cast & Generic Method Bugs

1. **Bug Fix 1 — `TuplePrimitive` Subscripting & Casting (`pass3_type_checking.rs` & `emitter.rs`):**
   * In `pass3_type_checking.rs`:
     * Add `Type::TupleElement` (or treat `TuplePrimitive` subscripting `t[i]` as dynamically castable to **any** primitive, class, interface, or generic type parameter `T`!).
     * In `Expression::Cast { target_type, expression }` and `Expression::FunctionCall` (`cast(expr, Type)`): if `src_type` is `TupleElement` / `TuplePrimitive` (or an array access on a `TuplePrimitive`), allow casting to any target type `tgt_type` without emitting `E0207` or `E0251`!
   * In `emitter.rs`:
     * When emitting `(T)t[i]` or `cast(t[i], T)` where `t` has type `TuplePrimitive`:
       * Emit `OpCode::GetTuple` (or `OpCode::TupleCast` with the resolved runtime type tag of `T`), which extracts the tagged `Value` from the tuple payload on the heap!
2. **Bug Fix 2 — Interface-to-Class & Interface-to-Interface Downcasting (`pass3_type_checking.rs` & `emitter.rs`):**
   * In `pass3_type_checking.rs:781`:
     * Currently, casting between `Type::Interface` and `Type::Class` (e.g. `(FileStream)stream` or `(IStringable)obj`) is not permitted if neither is a static subtype of the other.
     * In nominal OOP with interfaces, **any non-final `Type::Class` may dynamically implement a `Type::Interface`, and any `Type::Interface` reference may dynamically hold an instance of a `Type::Class` that implements it**!
     * Update `Expression::Cast` in `pass3_type_checking.rs` so casting from `Type::Interface` to `Type::Class` (when the class implements the interface or a subclass could implement it) or `Type::Class` to `Type::Interface` is valid at compile time and emits `OpCode::CastCheck` at runtime!
3. **Bug Fix 3 — Generic Method Monomorphization Across Modules (`monomorphizer.rs`, `pass3_type_checking.rs`, `emitter/pipeline.rs`):**
   * Register generic methods (`!method.generic_params.is_empty()`) in `GenericBlueprintRegistry::method_blueprints`.
   * In `ApiAndAstExportPhase`: when a `public class` (such as `Collections`, `Primitives`, `String`, `Algorithms`, `Math`, `Console`, `Path`) contains `public` generic or variadic methods, preserve those generic/variadic method AST bodies inside the exported `extern public class` in `api_header` / `exported_ast`!
   * When `Pass 3` resolves a call to a generic method (e.g., `Collections::to_string<T>(this)`, `Primitives::hash_any<K>(key)`, `String::from_any<T>(val)`, `Algorithms::sort<int32>(list, cmp)` with concrete type arguments), clone and specialize the method AST into a concrete monomorphized method (`Collections::to_string<int32>`) on the class and emit its bytecode in the consuming module!

---

## 3. Phased Implementation Plan

### Phase 1: Namespace-Qualified Declarations (`a::b::Name`) & Revert Normalization (`solix-compilation`)
* [ ] `[P0]` **Task 1.1:** Update `ClassDeclaration`, `InterfaceDeclaration`, `EnumDeclaration`, `MethodDeclaration`, `FieldDeclaration`, and `AliasStatement` in `crates/solix-compilation/src/phases/parser/statements/` to store `namespace_path: Vec<Span>`.
* [ ] `[P0]` **Task 1.2:** Update `ParserState` (`crates/solix-compilation/src/phases/parser/parser.rs`) to parse `::`-separated qualified identifiers in `parse_class_declaration`, `parse_interface_declaration`, `parse_enum_declaration`, `parse_alias_statement`, and top-level `parse_method_or_field_declaration`.
* [ ] `[P0]` **Task 1.3:** Update `execute_pass1_symbol_discovery` (`pass1_symbol_discovery.rs`), `execute_pass2_type_hierarchy` (`pass2_type_hierarchy.rs`), `execute_pass3_type_checking` (`pass3_type_checking.rs`), and `EmitterPhase` (`emitter/pipeline.rs`) to compute canonical FQNs from `file_package` + `namespace_path` + `name_span`, and switch the active package lookup context inside a qualified declaration to its `effective_package`.
* [ ] `[P0]` **Task 1.4:** Revert the Step 4.5 import-normalization hack in `pass1_symbol_discovery.rs` and replace the $O(N \times M)$ package scan in Step 6 with an $O(1)$ `known_packages: HashSet<String>` lookup on `SymbolTable`.

### Phase 2: Pre-Generated API Headers, Pruned `ExportedAst`, & Pre-Parsed Dependency Injection (`solix-common` & `solix-compilation`)
* [ ] `[P0]` **Task 2.1:** Add `serde::Serialize` and `serde::Deserialize` derives to all AST types in `crates/solix-compilation/src/phases/parser/statements/` and `Span`.
* [ ] `[P0]` **Task 2.2:** Add `api_header: String` and `exported_ast_bytes: Vec<u8>` to `ExportManifest` / `BinaryProgram` in `crates/solix-common/src/bytecode/`.
* [ ] `[P0]` **Task 2.3:** Implement `ApiAndAstExportPhase` (`crates/solix-compilation/src/phases/export_api.rs`):
  - Emits only non-`extern`, `public` declarations from local compilation sources (`!is_external_dependency`).
  - Uses namespace-qualified names (`extern public class solix::io::Console { ... }`) so a single header file preserves all subpackage namespaces (`solix::core`, `solix::io`, `solix::collections`, `solix::math`, `solix::time`, `solix::system`, `solix::crypto`) with zero package collisions.
  - Keeps `private` and `protected` instance fields on exported `public` classes for exact memory layout compatibility, while stripping `private` methods, `private` statics, and non-generic method bodies.
  - Preserves generic/variadic method bodies on exported `public` classes (`Collections::to_string<T>`, `Primitives::hash_any<T>`, `String::from_any<T>`, `Algorithms::sort<T>`, `Console::printf<args...>`) so downstream callers can monomorphize them.
  - Serializes the pre-parsed AST of the generated header into `exported_ast_bytes`.
* [ ] `[P0]` **Task 2.4:** Update `DependencyResolutionPhase` and `resolve_extern_import` to deserialize `exported_ast_bytes` directly into `context.ast` (skipping `LexerPhase` and `ParserPhase` for dependencies!) and mark those `source_id`s in `context.external_source_ids: HashSet<u16>`.

### Phase 3: Dual-Mode Linking (`--link dynamic|static`) & Whole-Program Reachability (`solix-compilation` & `solix-client`)
* [ ] `[P1]` **Task 3.1:** Add `LinkMode` (`Dynamic`, `Static`) to `CompilationOptions` and `--link <dynamic|static>` (plus `--static`) to `solix compile` and `solix build`.
* [ ] `[P1]` **Task 3.2:** In `LinkMode::Dynamic` (default), skip `Pass 3` statement/body type-checking and `AstInlinerPhase` for all `source_id`s in `context.external_source_ids` (only type-checking and emitting templates that are monomorphized by local code).
* [ ] `[P1]` **Task 3.3:** In `LinkMode::Static`, implement `ReachabilityAnalysis` starting from `entry_point`, marking all transitively called functions, methods, constructors, destructors, and classes, and emitting them directly into the output binary with `OpCode::CallDirect`.

### Phase 4: Fix `solixlib` Casts, Interface Downcasts & Generic Method Monomorphization (`solix-compilation` & `solix-runtime`)
* [ ] `[P0]` **Task 4.1:** Fix `TuplePrimitive` subscripting (`args[i]`) in `pass3_type_checking.rs` and `emitter.rs` so `(T)args[i]` and `cast(args[i], T)` type-check cleanly for any `T` (primitive, class, interface, or generic parameter) and emit `OpCode::GetTuple` / `OpCode::TupleCast`.
* [ ] `[P0]` **Task 4.2:** Fix `Expression::Cast` in `pass3_type_checking.rs` and `emitter.rs` to permit downcasting from `Type::Interface` to `Type::Class` (and vice versa) with runtime `OpCode::CastCheck` verification (extending `OpCode::CastCheck` in `fast.rs` to also check `itables` when casting to an interface!).
* [ ] `[P0]` **Task 4.3:** Implement full method-level monomorphization for generic methods (`fn<T>(...)`) and variadic generic methods (`fn<T, args...>(...)`) across both local and imported `extern` classes.

### Phase 5: Rebuild `solixlib` v2.0.0 & End-to-End Consumer Conformance Tests (`libraries/solixlib` & `solix-tests`)
* [ ] `[P0]` **Task 5.1:** Rebuild `libraries/solixlib` (`solix build libraries/solixlib --release` and `--test`) and re-install `solix@2.0.0.slxarc`.
* [ ] `[P0]` **Task 5.2:** Verify `solix info libraries/solixlib/build/release/solix.slxarc --api` outputs namespace-qualified declarations (`extern public class solix::io::Console`, `class solix::collections::List<T>`, etc.) and that `solix info test.slxb --api` **only** outputs `test.slxb`'s own symbols!
* [ ] `[P0]` **Task 5.3:** Add comprehensive tests for `class a::b::C`, dual-mode linking (`--link dynamic` and `--link static`), tuple/interface casts, and `extern import solix.io.Console;` cold-start compilation speed ($< 25\text{ ms}$).

---

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Command | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Qualified Declarations (`a::b::C`)** | `class a::b::C`, `interface a::b::I`, `enum a::b::E`, `void a::b::fn()` parse & bind | `cargo test -p solix-tests` | [ ] Pending |
| **Zero Import Normalization Hack** | Step 4.5 removed; `extern import solix.io.Console;` resolves `solix::io::Console` | `cargo test -p solix-tests` | [ ] Pending |
| **No Transitive API Pollution** | `solix info test.slxb --api` does NOT include `solixlib` classes | `cargo test -p solix-client` | [ ] Pending |
| **Pre-Parsed `ExportedAst` Injection** | Dependencies bypass `LexerPhase` & `ParserPhase`; cold-start $< 25\text{ ms}$ | `cargo test -p solix-compilation` | [ ] Pending |
| **Dual-Mode Linking (`dynamic` / `static`)** | Both `--link dynamic` and `--link static` compile & execute `solixlib` consumers | `cargo test -p solix-client` | [ ] Pending |
| **`solixlib` Cast & Generic Method Fixes** | All `solixlib` modules & consumer casts compile and run with 0 errors | `solix build libraries/solixlib --test` | [ ] Pending |
| **Clippy & Rustfmt Clean** | Zero warnings across all workspace crates | `cargo clippy --workspace --all-targets -- -D warnings` | [ ] Pending |