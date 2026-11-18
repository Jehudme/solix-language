verify that all feature wiki keword a# Formal Specification: Compilation Pipeline, Dual-Mode Linking & Dead-Code Elimination

## Document Metadata

* **Specification ID:** SPEC-0003
* **Scope:** Compiler Architecture & Pipeline Stages (`crates/solix-compilation/src/`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/guidelines/authoring-specs.md` (GL-0005)
* **Rust Implementation:** [`crates/solix-compilation/src/pipeline.rs`](../../crates/solix-compilation/src/pipeline.rs)

---

## 1. Architectural Pipeline Overview

The Solix Ahead-of-Time (AOT) compiler transforms UTF-8 source translation units into executable `.slxb` bytecode containers through eight sequential compilation phases:

```text
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 0: DependencyResolutionPhase                                    │
│   • Traverses dependency tree via ModuleSolver                         │
│   • Ingests pre-parsed ASTs directly into CompilationContext          │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 1: LexerPhase                                                   │
│   • Tokenizes source files into 108 strongly typed TokenKinds          │
│   • Attaches absolute file-offset coordinate Spans                     │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 2: ParserPhase                                                  │
│   • Builds ProgramAst using Pratt and Recursive-Descent parsing        │
│   • Resolves dynamically encountered extern import statements          │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 3: BinderPhase                                                  │
│   • Pass 1: Registers class, interface, enum, and global signatures    │
│   • Pass 2: Resolves inheritance, computes VTables, ITables & layouts  │
│   • Pass 3: Strict type-checking, control flow & monomorphization      │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 4: ApiAndAstExportPhase                                         │
│   • Synthesizes external API headers (manifest_to_source)              │
│   • Serializes compact ProgramAst for downstream consumer reuse        │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 5: AstInlinerPhase                                              │
│   • Inlines small functions and inline-annotated methods in-place     │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 6: EmitterPhase                                                 │
│   • Emits 64-bit Register VM bytecode with sliding register windows   │
│   • Encodes ConstantPool, exception handlers, and source mappings      │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 7: BytecodeOptimizerPhase                                       │
│   • Peephole instruction fusion and superinstruction synthesis         │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
                    Executable `.slxb` Container
```

---

## 2. Compilation Context & State Architecture

All phases operate upon a unified [`CompilationContext`](../../crates/solix-compilation/src/CompilationContext.rs):
* **`sources: Vec<String>`**: Raw UTF-8 source strings indexed by `source_id: u16`.
* **`tokens: Vec<Vec<Token>>`**: Token streams for each translation unit.
* **`ast: Vec<Option<ProgramAst>>`**: Parsed ASTs. External dependencies inject pre-parsed ASTs directly into this array, completely bypassing lexical analysis.
* **`symbol_table: SymbolTable`**: Scoped hierarchical environment tracking types, variables, classes, and methods.
* **`dependencies_map: HashMap<String, DependencyRequirement>`**: Records exact SemVer requirements and cryptographic digests.

---

## 3. Dual-Mode Linking & Dead-Code Elimination (DCE)

Solix supports two complementary linking topologies:

### 3.1 Dynamic Linking (Modular Multi-Binary)
* Preserves modular `.slxb` boundaries.
* External calls emit `CallExtern(target_symbol)`.
* Upon VM boot, the dynamic linker locates the target binary and mutates the calling instruction in-place to `CallDirect` (instruction quickening).

### 3.2 Static Linking with Reachability DCE (Hermetic LTO)
When `--static-link` is passed to the compiler:
1. **Dependency Inlining:** All functions, classes, and static variables from resolved `.slxb` dependencies are merged into a single binary container.
2. **Reachability Analysis (DFS):** Starting from the program entry point (`main()`) and static initializers, the reachability graph traverses all function calls, object instantiations, and vtable entries.
3. **Dead-Code Elimination:** Any class, method, or function not in the reachable set is completely stripped from the final binary.
4. **Direct Call Rewriting:** All `CallExtern` instructions targeting statically merged functions are rewritten to local `CallDirect` opcodes.
