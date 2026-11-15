# Milestone 027: Industrial Language Server Protocol (`solix-lsp` / `solix lsp`), Resilient AST Engine, Formatter & 32-Feature Code Intelligence Suite

## Document Metadata

* **Milestone ID:** MS-027
* **Target Version:** `v0.1.0-alpha.27`
* **Milestone Branch:** `milestone/027-language-server-protocol-and-intelligence`
* **Status:** DRAFT (Awaiting Architect Approval)
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/guidelines/documentation-update-protocol.md` (GL-0013)
  * `docs/spec/tools/lsp/protocol.md`
  * `docs/spec/tools/lsp/navigation.md`
  * `docs/spec/tools/lsp/intelligence.md`
  * `docs/internals/milestones/026_package-manager-qol-solix-sync-rich-inspection-checksum-security.md` (MS-026)

---

## 1. Executive Summary & Architectural Vision

Currently, `crates/solix-lsp` contains only a basic coordinate/diagnostic conversion stub (`src/diagnostics.rs`) and an empty `fn main() {}`. To provide a modern, industrial-grade IDE experience on par with `rust-analyzer`, TypeScript, and C# OmniSharp, **Milestone 027** implements the complete **Solix Language Server (`solix-lsp`)** powered by `tower-lsp = "0.20"` and `tokio = { version = "1", features = ["full"] }`.

### 1.1 Dual Library + Toolchain Subcommand Architecture (`solix lsp` & `solix-lsp`)

To maximize modularity, code reuse, and testability, the Language Server is architected as a **Library-First Engine**:
1. **`crates/solix-lsp/src/lib.rs` (Core Library):**
   * Encapsulates the complete `SolixLanguageServer` backend, `WorkspaceManager`, `ResilientAstIndex`, `CodeActionEngine`, `SemanticTokenizer`, `InlayHintEngine`, and `SolixFormatter`.
   * Exposes `build_lsp_service()` (returning `(LspService<SolixLanguageServer>, ClientSocket)`) for **zero-process, in-memory automated testing** and `run_stdio_server()` for stdio transport.
2. **`solix lsp` Subcommand (`crates/solix-toolchain`):**
   * Adds `solix lsp [--stdio]` directly into the unified `solix` CLI binary (and `solix fmt` exposing the built-in formatter on the command line!).
   * Automatically wires `ClientModuleSolver` and `FilesystemPackageCache` (`~/.solix/cache/packages/`) into the language server so `extern import solix.io.Console;` and `manifest.toml` `[dependencies]` resolve in $< 5\text{ ms}$!
3. **`crates/solix-lsp/src/main.rs` (Standalone Binary Alias `solix-lsp`):**
   * A minimal 10-line entrypoint invoking `solix_lsp::run_stdio_server().await` so external editors (Neovim `nvim-lspconfig`, Helix, Zed, Emacs `eglot`) expecting a standalone `solix-lsp` executable work out of the box.

---

## 2. The Complete 32-Feature Modern LSP Matrix

Every single feature below **MUST** be implemented in `crates/solix-lsp` and verified by automated unit and integration tests:

| # | LSP Method / Capability | Feature Name | Solix-Specific Intelligence & Behavior |
| :-: | :--- | :--- | :--- |
| **1** | `initialize` / `initialized` / `shutdown` | Lifecycle & Capability Handshake | Advertises all 32 server capabilities, negotiates UTF-16/UTF-8 offsets, discovers workspace root. |
| **2** | `textDocument/didOpen`, `didChange`, `didSave`, `didClose` | In-Memory Overlay `DocumentStore` | Maintains live unsaved editor buffers in memory (`DashMap` / `RwLock`), taking precedence over disk files during multi-file compilation. |
| **3** | Upward `manifest.toml` & Store Discovery | Project & Dependency Auto-Mount | Searches upward from any opened `.slx` file for `manifest.toml`; mounts all `src/` and `tests/` files plus `[dependencies]` and `solix` stdlib from `PackageCache`. |
| **4** | `workspace/didChangeWatchedFiles` | Live File & Manifest Watcher | Re-indexes workspace when `.slx` or `manifest.toml` files are created, modified, or deleted on disk. |
| **5** | Resilient Last-Known-Good AST Cache | Fault-Tolerant Live Typing Snapshot | Preserves the last successfully bound `SymbolTable`, `ClassLayout`s, and `ProgramAst` per file when mid-keystroke syntax is incomplete (e.g., trailing `.` or `::`). |
| **6** | `textDocument/publishDiagnostics` | Real-Time Push Diagnostics | Publishes Lexer (`E0000`–`E0015`), Parser (`E0101`–`E0200`), and Binder (`E0201`–`E0291`) diagnostics with exact multi-character token spans. |
| **7** | `textDocument/diagnostic` & `workspace/diagnostic` | LSP 3.17 Pull Diagnostics | Supports on-demand pull diagnostics for active files and workspace-wide health checks. |
| **8** | `Diagnostic.codeDescription.href` | Clickable Error Wiki Links | Links every diagnostic code (`E0226`, `E0201`, etc.) to its canonical `docs/wiki/errors/` reference URL/URI. |
| **9** | `DiagnosticTag::Unnecessary` & `Deprecated` | Unused Symbol & Dead-Code Fading | Marks unused local variables, unused `import` statements, and unreachable statements after `return`/`break`/`throw` with `DiagnosticTag::Unnecessary`. |
| **10** | `DiagnosticRelatedInformation` | Secondary Spans for Collisions | Attaches clickable secondary locations for duplicate declarations (`E0202`) and mismatched method overrides (`E0207`). |
| **11** | `textDocument/definition` | Go to Definition (with Chaining) | Jumps to variables, parameters, fields, methods, constructors, classes, interfaces, enums, and aliases across project & `extern` packages. Invoking on a declaration chains to its type! |
| **12** | `textDocument/typeDefinition` | Go to Type Definition | Jumps directly from any variable, field, parameter, or expression to its declaring `class`, `interface`, or `enum`. |
| **13** | `textDocument/implementation` | Go to Implementation | From an `interface` or `abstract`/`virtual` method, returns all implementing classes and `override` methods across the workspace. |
| **14** | `textDocument/references` | Find All References | Locates all usages of a class, interface, enum, method, field, parameter, or local variable across the workspace. |
| **15** | `textDocument/documentHighlight` | Read vs. Write Symbol Highlighting | Highlights all occurrences of the symbol under cursor in the current file, marking mutations (`=`, `+=`, `++`) as `Write` and reads as `Read`. |
| **16** | `textDocument/prepareCallHierarchy`, `incomingCalls`, `outgoingCalls` | Call Hierarchy Explorer | Builds caller (`incomingCalls`) and callee (`outgoingCalls`) trees for methods, constructors, and free functions. |
| **17** | `textDocument/prepareTypeHierarchy`, `supertypes`, `subtypes` | Type Hierarchy Explorer | Builds inheritance trees showing base classes + implemented interfaces (`supertypes`) and derived subclasses (`subtypes`). |
| **18** | `textDocument/completion` (`.` & `::`) | Dot & Scope-Resolution Completion | **`.`**: Instance fields & methods (with generic `T` substituted!). **`::`**: Static methods, static `const` fields, and `enum` variants (never suggests instance members on `::` or statics on `.`!). |
| **19** | `textDocument/completion` (Grammar & Postfix) | Context-Aware & Postfix Completions | Smart completions after `package`, `import`, `extern import`, `extends`, `implements`, `new`, `case`, `catch`, `alias`, plus postfix `.if`, `.notnull`, `.println`, `.ret`. |
| **20** | `completionItem/resolve` | Auto-Import & Doc Resolution | Lazily populates Markdown documentation and attaches `additionalTextEdits` to auto-insert `import pkg.ClassName;` when completing an unimported type! |
| **21** | `textDocument/hover` | Rich Layout, ARC & Doc Hover | Displays Markdown code block with full signature, `///` doc comments, **instance byte size (`sizeof: 24B`)**, **field byte offset (`offset: +8B`)**, and **ARC representation (`ObjectPod` vs `Object`)**. |
| **22** | `textDocument/signatureHelp` | Overload-Aware Signature Help | Triggered on `(`, `,`, and `<`; lists all matching method/constructor overloads and highlights the active parameter (`activeParameter`). |
| **23** | `textDocument/inlayHint` | Parameter, Generic & RAII Inlay Hints | Displays inline parameter names (`width:`, `height:`), deduced generic type arguments (`<int32>`), and optional RAII `~Destructor()` scope-exit markers. |
| **24** | `textDocument/semanticTokens/full` & `range` | Binder-Driven Semantic Highlighting | Emits semantic tokens for classes, interfaces, enums, enum members, methods, static methods, fields, `const` vars, parameters, `weak` fields, and generic type parameters. |
| **25** | `textDocument/codeLens` | Executable & Reference Code Lenses | Emits `▶ Run` / `▶ Run Test` lenses on `main()` / `test()` entrypoints and `N references` / `M implementations` lenses on types and methods. |
| **26** | `textDocument/documentSymbol` | Hierarchical Outline & Breadcrumbs | Returns nested `DocumentSymbol` trees (`Package` $\to$ `Class`/`Interface`/`Enum` $\to$ `Field`/`Constructor`/`~Destructor`/`Method`/`Operator`). |
| **27** | `workspace/symbol` | Fuzzy Workspace Symbol Search | Searches all classes, interfaces, enums, functions, and methods across the workspace by prefix/subsequence (`Ctrl+T`). |
| **28** | `textDocument/foldingRange` | AST Block & Import Folding | Folds contiguous `import` blocks, multi-line `/* ... */` comments, classes, interfaces, enums, methods, and control-flow blocks. |
| **29** | `textDocument/selectionRange` | Smart Structural AST Selection | Expands/shrinks selection along AST parent chains (token $\to$ expr $\to$ stmt $\to$ block $\to$ method $\to$ class $\to$ file). |
| **30** | `textDocument/documentLink` | Clickable Imports & `res://` Links | Turns `import a.b.C;` paths and `"res://..."` asset strings into clickable editor links that open the target file. |
| **31** | `textDocument/codeAction` | Quick Fixes & Code Generators | **1)** Replace `.` with `::` on `E0226`. **2)** Auto-import missing type on `E0201`. **3)** "Did you mean?" typo fix on `E0201`. **4)** Implement missing interface/abstract methods on `E0206`. **5)** Add `override` on `E0284`. **6)** Generate constructor/getters/setters. |
| **32** | `textDocument/rename`, `prepareRename`, & `formatting` | Safe Workspace Rename & Formatter | Multi-file symbol rename (rejecting keywords/invalid identifiers) + built-in Solix code formatter (`textDocument/formatting`, `rangeFormatting`, and `solix fmt`). |

---

## 3. Deep Architectural Blueprint

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                        Solix Language Server (`solix-lsp`)                       │
├──────────────────────────────────────────────────────────────────────────────────┤
│  JSON-RPC 2.0 Transport (`tower-lsp` 0.20 + `tokio` async runtime)               │
│    • Stdio (`solix lsp --stdio` / `solix-lsp`)                                   │
│    • In-Memory `tower::Service` (`LspTestClient` for deterministic unit tests)   │
├──────────────────────────────────────────────────────────────────────────────────┤
│  Workspace & Document State (`WorkspaceState`)                                   │
│    • `open_buffers: DashMap<Url, VersionedDocument>` (Unsaved live text)         │
│    • `project_roots: Vec<ProjectWorkspace>` (Discovered `manifest.toml` roots)   │
│    • `solver: Arc<dyn ModuleSolver>` + `PackageCache` (Sub-5ms stdlib & deps)    │
│    • `unit_snapshots: DashMap<Url, AnalysisSnapshot>`                            │
│        ├── `latest_tokens: Vec<Token>`                                           │
│        ├── `latest_diagnostics: Vec<Diagnostic>`                                 │
│        ├── `current_ast: Option<ProgramAst>`                                     │
│        └── `last_good_semantic: Option<Arc<BoundSemanticSnapshot>>`              │
├──────────────────────────────────────────────────────────────────────────────────┤
│  Spatial & Semantic Query Engine (`SpatialAstIndex`)                             │
│    • Interval lookup: `(source_id, byte_offset) -> Smallest Enclosing AST Node`  │
│    • Symbol table & scope chain resolver (supports generic templates `<T>`)      │
│    • Cross-file reference & implementation inverted index                        │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 3.1 The Resilient Live-Typing Snapshot Engine (Solving the `"Console."` / `"Console::"` Problem)
When a developer types `Console::` or `my_list.` on line 15:
1. The file temporarily has a syntax error (`E0102: Expected identifier` after `::` or `.`).
2. A naive compiler discards the AST when parsing fails, breaking autocompletion right when the user needs it most!
3. **Our Solution (`ResilientParser & Last-Known-Good Snapshot`):**
   * First, when `ParserState` encounters a trailing `.` or `::` followed by a newline, `;`, `)`, or `}`, it records the `ParsingError` for diagnostics **and still emits an `Expression::MemberAccess` with a synthetic empty `member_span` at the cursor offset** so the AST remains intact!
   * Second, `WorkspaceState` always retains `last_good_semantic: Option<Arc<BoundSemanticSnapshot>>` containing all class layouts, imported symbols, and method local variables from the most recent bind pass.
   * When completion or signature help triggers on an incomplete line, it queries the receiver expression (`Console` or `my_list`) against `current_ast` + `last_good_semantic`, delivering instant 100% accurate completions on every keystroke!

### 3.2 Strict Solix v2.0 `::` vs `.` Completion & Quick-Fix Rules
Because Solix v2.0 strictly separates instance access (`.`) from static/enum/namespace access (`::`):
* **When triggered by `.` (`receiver.`):**
  * If `receiver` is an **instance variable/expression** (e.g. `list.`), suggest **only** accessible instance methods and instance fields (plus postfix `.if`, `.println`, `.notnull`, `.ret`).
  * If `receiver` is a **type name** (e.g. `Console.`), suggest static members **with an automatic `additionalTextEdit` that rewrites `.` to `::`** (so typing `Console.println` automatically corrects to `Console::println`!), and provide a 1-click Quick Fix (`CodeAction`) on diagnostic `E0226`!
* **When triggered by `:` / `::` (`TypeOrPkg::`):**
  * If `TypeOrPkg` is a `class`, suggest **only** accessible `static` methods and `static` / `const` fields.
  * If `TypeOrPkg` is an `enum`, suggest **only** its enum variants (`Color::RED`, `Color::GREEN`).
  * If `TypeOrPkg` is a package namespace (`solix::io::`), suggest types and functions in that package.

### 3.3 Doc Comment Attachment (`///` and `/** ... */`)
* Extend the Lexer/Parser trivia collector (or a lightweight `DocCommentMap` extractor in `solix-lsp`) that associates contiguous `///` or `/** ... */` comments immediately preceding a `class`, `interface`, `enum`, `method`, `constructor`, `field`, or `alias` declaration with its `SymbolId`.
* Display these Markdown doc comments inside `textDocument/hover`, `textDocument/completion`, and `textDocument/signatureHelp`!

---

## 4. Phased Implementation Plan

### Phase 1: Crate Setup, `solix lsp` Subcommand & Workspace/Manifest Discovery (`solix-lsp` & `solix-toolchain`)
* [ ] `[P0]` **Task 1.1:** Configure `crates/solix-lsp/Cargo.toml` with `tower-lsp = "0.20"`, `tokio = { version = "1", features = ["full"] }`, `serde`, `serde_json`, `url`, and `dashmap` (or `tokio::sync::RwLock`).
* [ ] `[P0]` **Task 1.2:** Structure `crates/solix-lsp` with `src/lib.rs` (public server & test harness API) and `src/main.rs` (standalone stdio binary), and add `solix lsp` + `solix fmt` subcommands to `crates/solix-toolchain/src/cli.rs`.
* [ ] `[P0]` **Task 1.3:** Implement `WorkspaceManager` (`crates/solix-lsp/src/workspace.rs`) with upward `manifest.toml` discovery, automatic loading of `src/` and `tests/` files, `ClientModuleSolver` + `PackageCache` integration for `extern import` and `[dependencies]`, and in-memory buffer overrides (`didOpen`, `didChange`, `didSave`, `didClose`, `didChangeWatchedFiles`).

### Phase 2: Push/Pull Diagnostics, Error Wiki Links, Unused Warnings & Resilient AST (`solix-lsp`)
* [ ] `[P0]` **Task 2.1:** Upgrade parser recovery on trailing `.` and `::` so incomplete member access expressions still produce an AST node for cursor queries, and maintain `last_good_semantic` fallback snapshots in `WorkspaceManager`.
* [ ] `[P0]` **Task 2.2:** Implement real-time push diagnostics (`publishDiagnostics`) and LSP 3.17 pull diagnostics (`textDocument/diagnostic`, `workspace/diagnostic`) with `CodeDescription.href` pointing to `docs/wiki/errors/`, `DiagnosticRelatedInformation` for duplicate/override errors, and `DiagnosticTag::Unnecessary` for unused imports/variables and unreachable code.

### Phase 3: Spatial AST Index & Deep Navigation (`definition`, `typeDefinition`, `implementation`, `references`, `highlight`, `callHierarchy`, `typeHierarchy`, `documentLink`)
* [ ] `[P0]` **Task 3.1:** Implement `SpatialAstIndex` (`crates/solix-lsp/src/index.rs`) mapping byte spans across local files and imported dependency API headers (`solixlib`) to symbols, types, and declarations.
* [ ] `[P0]` **Task 3.2:** Implement `textDocument/definition` (including Definition-to-Type chaining and `override`-to-base jumping), `textDocument/typeDefinition`, and `textDocument/implementation`.
* [ ] `[P1]` **Task 3.3:** Implement `textDocument/references` and `textDocument/documentHighlight` (distinguishing `Read` vs `Write` accesses).
* [ ] `[P1]` **Task 3.4:** Implement Call Hierarchy (`prepareCallHierarchy`, `incomingCalls`, `outgoingCalls`), Type Hierarchy (`prepareTypeHierarchy`, `supertypes`, `subtypes`), and `textDocument/documentLink` (for `import` statements and `"res://..."` strings).

### Phase 4: Context-Aware Completions (`.`, `::`, Grammar, Postfix, Auto-Import), Hover, Signature Help & Inlay Hints
* [ ] `[P0]` **Task 4.1:** Implement `textDocument/completion` and `completionItem/resolve` (`crates/solix-lsp/src/completion.rs`):
  - Dot `.` instance member completion with generic type parameter substitution (`Box<int32>::get() -> int32`) and auto-`::` rewrite if called on a class name.
  - Scope resolution `::` completion for static methods, `const` fields, and `enum` variants.
  - Context-aware grammar completions (`package`, `import`, `extern import`, `extends`, `implements`, `new`, `case`, `catch`, `alias`).
  - Postfix completions (`.if`, `.notnull`, `.println`, `.ret`) and automatic `import` insertion (`additionalTextEdits`) when completing unimported workspace/stdlib types.
* [ ] `[P0]` **Task 4.2:** Implement `textDocument/hover` (`crates/solix-lsp/src/hover.rs`) rendering Markdown signatures, doc comments (`///`), class `sizeof` byte size, field byte offset (`+8B`), and ARC Pod status (`ObjectPod` vs `Object`).
* [ ] `[P1]` **Task 4.3:** Implement `textDocument/signatureHelp` (`(`, `,`, `<`) with overload resolution and active parameter tracking, and `textDocument/inlayHint` (parameter name hints, deduced generic `<T>` hints, and RAII destructor hints).

### Phase 5: Semantic Tokens, Code Lens, Outline, Workspace Symbols, Folding & Selection Range
* [ ] `[P1]` **Task 5.1:** Implement `textDocument/semanticTokens/full` and `range` (`crates/solix-lsp/src/semantic_tokens.rs`) using Binder symbol classifications.
* [ ] `[P1]` **Task 5.2:** Implement `textDocument/documentSymbol` (hierarchical outline), `workspace/symbol` (fuzzy search), `textDocument/codeLens` (`Run Main`, `Run Test`, `N references`, `M implementations`), `textDocument/foldingRange`, and `textDocument/selectionRange`.

### Phase 6: Code Actions (Quick Fixes & Refactorings), Workspace Rename & Solix Formatter (`solix fmt`)
* [ ] `[P0]` **Task 6.1:** Implement `textDocument/codeAction` (`crates/solix-lsp/src/code_actions.rs`):
  - Quick Fix for `E0226`: Replace `.` with `::`.
  - Quick Fix for `E0201`: Levenshtein `"Did you mean ...?"` replacement and `"Import '...'"` insertion.
  - Quick Fix for `E0206`: Generate missing interface / abstract method stubs.
  - Quick Fix for `E0284`: Insert missing `override` keyword.
  - Refactorings: Generate constructor from fields, generate getters/setters, organize imports.
* [ ] `[P1]` **Task 6.2:** Implement `textDocument/prepareRename` and `textDocument/rename` across all workspace files with identifier and keyword collision checks.
* [ ] `[P1]` **Task 6.3:** Implement the comment-preserving Solix Code Formatter (`crates/solix-lsp/src/formatter.rs`) powering `textDocument/formatting`, `textDocument/rangeFormatting`, `textDocument/onTypeFormatting`, and `solix fmt <files...> [--check]`.

### Phase 7: Exhaustive Automated In-Memory & Stdio Test Suite (`crates/solix-lsp/tests/`)
* [ ] `[P0]` **Task 7.1:** Build `LspTestClient` wrapping `tower_lsp::LspService` in `crates/solix-lsp/tests/` and write **60+ automated tests** covering every single one of the 32 LSP features (both positive and negative/broken-syntax resilience cases), plus an end-to-end stdio subprocess test of `solix lsp`.
* [ ] `[P1]` **Task 7.2:** Synchronize `docs/spec/tools/lsp/protocol.md`, `navigation.md`, `intelligence.md`, and `docs/wiki/commands/` per `GL-0013`.

---

## 5. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Command | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Hybrid Library + Binary** | `solix-lsp` library + `solix lsp` CLI subcommand + `solix fmt` | `cargo build --workspace --all-targets` | [ ] Pending |
| **Resilient Live Typing** | Completions & hover work on `Console::` and `list.` with syntax errors | `cargo test -p solix-lsp -- test_resilient_completion` | [ ] Pending |
| **All 32 LSP Features** | Diagnostics, Nav, `.`/`::` Completion, Hover, Inlay, Semantic Tokens, Quick Fixes, Rename, Format | `cargo test -p solix-lsp` (60+ tests) | [ ] Pending |
| **`solixlib` & Manifest Integration** | `extern import solix.io.Console;` resolves hover, completion & definition in $< 10\text{ ms}$ | `cargo test -p solix-lsp -- test_stdlib_resolution` | [ ] Pending |
| **Documentation Sync (`GL-0013`)** | `docs/spec/tools/lsp/*.md` & `docs/wiki/commands/` updated | `python3 scripts/doc_sync_check.py` | [ ] Pending |
| **Clippy & Formatting Clean** | Zero warnings, `#![forbid(unsafe_code)]`, zero single-letter vars (`GL-0011`) | `cargo clippy --workspace --all-targets -- -D warnings` | [ ] Pending |