# Milestone 012: Compiler Pipeline Modularization, Extensible Phase Architecture & Stabilization

## Document Metadata
* **Milestone ID:** MS-012
* **Target Version:** `v0.1.0-alpha.12`
* **Milestone Branch:** `milestone/012-compiler-pipeline-modularization`
* **Status:** COMPLETED
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-11-15
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/milestones/004-compiler-stabilization-and-incremental-architecture.md` (MS-004)
  * `docs/internals/milestones/011-multi-binary-support-and-dynamic-linking.md` (MS-011)

---

## 1. Executive Summary & Purpose

The existing compilation driver couples pipeline execution directly inside `CompilationContext` via procedural methods (`lex_source`, `parse_source`, `emit_context`). This monolithic design limits extensibility and flexibility:
- Tools like the **LSP server** cannot easily run syntax and semantic analysis without also running or manually omitting bytecode emission.
- **Custom compiler plugins**, optimization passes (AST optimization, constant folding, bytecode peephole), and linters cannot be inserted into the compilation lifecycle without modifying compiler internals.
- **Diagnostic handling** is split across disparate error types with redundant mapping logic.

**Milestone 012** refactors the Solix compiler into a modular, extensible **Phase/Pass Architecture**:
1. **Unified Diagnostic System (`solix-common`)**:
   - `CompilationDiagnostic` enum wrapping `LexingError`, `ParsingError`, `BindingError`, `EmissionError`, and extensible `Custom` diagnostics with unified `miette::Diagnostic` support.
2. **Modular `CompilationPhase` Trait (`solix-compilation`)**:
   - Isolated, reusable phases (`LexerPhase`, `ParserPhase`, `BinderPhase`, `EmitterPhase`) implementing execution and lifecycle hooks (`on_success`, `on_failure`) returning flow control decisions (`PhaseDecision::Continue` / `PhaseDecision::Halt`).
3. **Blackboard State `CompilationContext` (`solix-compilation`)**:
   - Decoupled from procedural pipeline execution methods, retaining only source code registries, dependency graphs, phase artifacts, and a generic type blackboard (`context.extensions`) for custom phase data.
4. **Pluggable `CompilationPipeline` Engine (`solix-compilation`)**:
   - Supports pre-configured pipelines (`default()`, `lsp()`, `syntax_only()`) as well as user-customized pipelines where phases can be dynamically added, removed, or reordered.
5. **Tooling & Test Migration (`solix-client`, `solix-lsp`, `solix-tests`)**:
   - CLI diagnostic reporting and LSP semantic diagnostic pipeline.
   - Comprehensive test harness refactoring and new dedicated test suite for custom phases and lifecycle flow control.

---

## 2. Architecture & Design Specification

### 2.1 Unified Diagnostic Hierarchy (`solix-common`)

```rust
#[derive(Debug, Clone, thiserror::Error, miette::Diagnostic)]
pub enum CompilationDiagnostic {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Lex(#[from] LexingError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Parse(#[from] ParsingError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Bind(#[from] BindingError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Emit(#[from] EmissionError),

    #[error("Custom diagnostic [{code}]: {message}")]
    Custom {
        code: String,
        message: String,
        span: Option<Span>,
    },
}
```

Methods on `CompilationDiagnostic`:
- `error_code(&self) -> &str`
- `span(&self) -> Option<Span>`

### 2.2 `CompilationPhase` Trait & Lifecycle Hooks (`solix-compilation`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseDecision {
    Continue,
    Halt,
}

pub trait CompilationPhase: Send + Sync {
    fn name(&self) -> &'static str;
    fn run(&mut self, context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>>;

    fn on_success(&mut self, _context: &mut CompilationContext) -> PhaseDecision {
        PhaseDecision::Continue
    }

    fn on_failure(
        &mut self,
        _context: &mut CompilationContext,
        _diagnostics: &[CompilationDiagnostic],
    ) -> PhaseDecision {
        PhaseDecision::Halt
    }
}
```

Standard phases:
- `LexerPhase`: batch tokenization of all registered sources into `context.tokens`.
- `ParserPhase`: parsing token streams into `context.ast`.
- `BinderPhase`: multi-pass symbol resolution, type hierarchy validation, type checking, and layout generation into `context.binding_artifacts`.
- `EmitterPhase`: register allocation and bytecode generation into `context.emission_artifacts`.

### 2.3 Blackboard `CompilationContext` & Extensions

```rust
pub struct CompilationContext {
    pub options: CompilationOptions,
    pub source_names: Vec<String>,
    pub sources: Vec<String>,
    pub tokens: Vec<Vec<Token>>,
    pub ast: Vec<Option<ProgramAst>>,
    pub dependencies: Vec<Vec<u16>>,
    pub dependents: Vec<Vec<u16>>,
    pub binding_artifacts: Option<BindingArtifacts>,
    pub emission_artifacts: Option<EmissionArtifacts>,
    pub extensions: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl CompilationContext {
    pub fn new(options: CompilationOptions) -> Self;
    pub fn insert_extension<T: Send + Sync + 'static>(&mut self, value: T);
    pub fn get_extension<T: 'static>(&self) -> Option<&T>;
    pub fn get_extension_mut<T: 'static>(&mut self) -> Option<&mut T>;
}
```

### 2.4 `CompilationPipeline` Engine

```rust
pub struct CompilationPipeline {
    phases: Vec<Box<dyn CompilationPhase>>,
}

impl CompilationPipeline {
    pub fn empty() -> Self;
    pub fn add_phase<P: CompilationPhase + 'static>(mut self, phase: P) -> Self;
    pub fn default() -> Self;
    pub fn lsp() -> Self;
    pub fn syntax_only() -> Self;
    pub fn execute(&mut self, context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>>;
}
```

---

## 3. Definition of Done (DoD) Matrix

| Requirement | Acceptance Criteria | Verification Method |
| :--- | :--- | :--- |
| **Unified Diagnostics** | `CompilationDiagnostic` implements `miette::Diagnostic` and wraps all compiler errors. | Unit tests in `solix-common` |
| **Modular Phases** | `LexerPhase`, `ParserPhase`, `BinderPhase`, `EmitterPhase` implement `CompilationPhase`. | Unit tests in `solix-compilation` |
| **Pipeline Presets** | `default()`, `lsp()`, and `syntax_only()` pipelines execute appropriate stages. | Unit & integration tests |
| **Context Blackboard** | `CompilationContext` cleaned of procedural methods; `extensions` allows arbitrary phase metadata. | Unit tests in `solix-compilation` |
| **LSP Integration** | `solix-lsp` consumes `CompilationPipeline::lsp()` without invoking bytecode emission. | Integration tests in `solix-lsp` |
| **Test Suite Migration** | All 425+ conformance tests run cleanly using `CompilationPipeline`. | Conformance tests in `solix-tests` |
| **Pipeline Customization Tests** | Dedicated tests verifying custom phase insertion, lifecycle hooks, and early halting. | New integration test suite |
| **Zero Warnings & Safe Code** | `#![forbid(unsafe_code)]`, `cargo clippy -- -D warnings`, `cargo fmt --check`. | Workspace automated checks |

---

## 4. Phased Roadmap

* [x] **Step 0: Specification Document & Milestone Branch Inception**
* [x] **Phase 1: Unified Diagnostic System (`solix-common`)**
* [x] **Phase 2: `CompilationPhase` Trait, Lifecycle Hooks & Standard Phases (`solix-compilation`)**
* [x] **Phase 3: Blackboard `CompilationContext` & `CompilationPipeline` Engine (`solix-compilation`)**
* [ ] **Phase 4: Client & LSP Integration (`solix-client`, `solix-lsp`)** *(Deferred to standalone milestone)*
* [x] **Phase 5: Conformance Test Suite & Pipeline Customization Migration (`solix-tests`)**
