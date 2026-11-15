# Milestone 029: Solix LSP & VS Code Extension Stabilization, Full Pipeline Dependency Resolution (`extern import` / `solixlib`), Local Variable Intelligence & Editor UX Overhaul

## Document Metadata

* **Milestone ID:** MS-029
* **Target Version:** `v0.1.0-alpha.29`
* **Milestone Branch:** `milestone/029-lsp-and-vscode-stabilization`
* **Status:** DRAFT (Awaiting Architect Approval)
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/guidelines/documentation-update-protocol.md` (GL-0013)
  * `docs/internals/milestones/027-language-server-protocol-and-code-intelligence.md` (MS-027)
  * `docs/internals/milestones/028-vscode-extension-and-automated-e2e-testing.md` (MS-028)

---

## 1. Executive Summary & Forensic Root-Cause Analysis

Following real-world interactive testing of the Solix Language Server (`solix-lsp`) and the Visual Studio Code extension (`editors/vscode`), **15 critical bugs and UX gaps** were identified across the language server and editor client:

### 1.1 Summary of All 15 Bugs to Remediate

| Bug ID | Subsystem & Location | Root Cause | User-Facing Symptom |
| :--- | :--- | :--- | :--- |
| **BUG-LSP-01** | `solix-lsp/src/workspace.rs:285-338` | `analyze_document` manually calls `tokenize_source` $\to$ `parse_context_resilient` $\to$ `bind_context`, completely bypassing `DependencyResolutionPhase` and `ParserPhase`'s `resolve_extern_import` worklist! | `extern import solix.io.Console;` and `import solix.io.Console;` fail in the editor with `E0232: Cannot resolve import 'solix.io.Console'`, even though `solix compile` succeeds! |
| **BUG-LSP-02** | `solix-lsp/src/workspace.rs:248-280` | `analyze_document` locates `manifest.toml` to find the project root directory, but **never parses `manifest.toml`** (`ProjectManifest::from_file`) and never populates `options.dependency_specs` from `[dependencies]`! | Projects declaring `solix = "2.0.0"` in `manifest.toml` `[dependencies]` fail to resolve standard library or external package imports in the IDE! |
| **BUG-LSP-03** | `solix-lsp/src/main.rs:6` & `workspace.rs:195` | Standalone `solix-lsp` passes `solver = None`, and `WorkspaceManager` has no default filesystem `ModuleSolver` fallback when `solver` is `None`. | Running `solix-lsp` directly has no package solver to locate `~/.solix/packages` (`solix@2.0.0.slxarc`). |
| **BUG-LSP-04** | `solix-lsp/src/workspace.rs:321-337` | `analyze_document` converts all `parse_errors` and `bind_errors` in `context` into diagnostics for `target_uri` **without filtering by `error_span.id == target_source_id`**! | Errors from other files in the workspace (or synthetic headers) are falsely rendered at random offsets in the currently active editor tab! |
| **BUG-LSP-05** | `solix-lsp/src/workspace.rs:290-308` | `CompilationContext::new` inserts `"<builtins>"` (`blt.io`) at `source_id = 0` and iterates `options.codes` (`HashMap`) in non-deterministic order; `analyze_document` re-lexes `"<builtins>"` and mismaps default spans (`id == 0`). | Unstable `source_id` ordering and diagnostic span mismatches in multi-file workspaces. |
| **BUG-LSP-06** | `solix-lsp/src/completion.rs:144-168` | `handle_dot_completion` looks up `receiver` only in `sem.symbol_table.symbols`. However, local variables and parameters are popped from `symbol_table.scopes` at the end of Pass 3 and are NOT in `symbol_table.symbols`! | Typing `my_local_var.` inside a method fails to find `my_local_var`'s class type and dumps all methods of all classes instead of `my_local_var`'s members! |
| **BUG-LSP-07** | `solix-lsp/src/workspace.rs:58-92` & `navigation.rs` | `symbol_at_offset` only checks `resolved_symbols` and `symbol_table.symbols`, ignoring local variable declarations (`Statement::VariableDeclaration`), `for`-loop initializers, `catch` variables, and method parameters (`Parameter`). | Hover, Go-to-Definition (`F12`), Find References, Document Highlight, and Rename (`F2`) fail when the cursor is on a local variable or parameter declaration! |
| **BUG-LSP-08** | `solix-lsp/src/formatter.rs:76, 175-184` | 1. Formatter inserts spaces inside compound operators (`+=` $\to$ `+ =`, `!=` $\to$ `! =`, `=>` $\to$ `= >`).<br>2. Lines like `} else {` or `} catch (...) {` decrement `indent_level` without re-incrementing for `{`. | Running `Format Document` (`Shift+Alt+F` or `solix fmt`) breaks compound assignments, lambdas, and `if/else` / `try/catch` indentation! |
| **BUG-EXT-01** | `solix-lsp/src/code_lens.rs:30` & `editors/vscode/src/commands.ts:83` | `code_lens.rs` passes `snapshot.uri.to_string()` (a **string**) as argument 0 to `solix.runMain`, while `commands.ts` expects `documentUri?: vscode.Uri` and reads `documentUri.fsPath` (`undefined` on a string!). | Clicking `▶ Run Main` CodeLens sends bare `solix run` (with no file path!) to the terminal, which immediately fails! |
| **BUG-EXT-02** | `editors/vscode/src/commands.ts:83-105` | `solix.runMain` and `solix.runTest` do not fall back to `vscode.window.activeTextEditor?.document.uri` when invoked from the Command Palette or keybinding, and do not detect `manifest.toml` projects vs standalone `.slx` scripts! | Running `Solix: Run Application` from Command Palette or clicking `▶ Run Test` fails with missing arguments! |
| **BUG-EXT-03** | `editors/vscode/src/extension.ts:44-118, 174` | `findSolixBinary` returns a single `binaryPath`. If `solix-lsp` is selected (`isToolchain == false`), `extension.ts` passes `solix-lsp` into `registerSolixCommands` and `SolixTaskProvider`! | CLI commands (`run`, `build`, `sync`, `info`) attempt to execute `solix-lsp run` / `solix-lsp build` and fail! |
| **BUG-EXT-04** | `editors/vscode/src/commands.ts:31-50` | `VirtualContentProvider` (`showDisassembly` / `showSynthesizedApi`) compiles only `sourceFilePath` without passing project `src/` or `--pkg-dir` / manifest dependencies, and on Windows `documentUri.path` has a leading `/C:/...` slash! | `Solix: Inspect 64-Bit Register VM Disassembly` and `Inspect Exported Public API` fail on any file that imports another project file or `solixlib`! |
| **BUG-EXT-05** | `editors/vscode/src/tasks.ts:39` | `createTasks()` defines `this.buildTask('run', [])` with empty arguments, and does not set `cwd` on `ProcessExecution`. | Running the `solix run` task from VS Code's `Terminal -> Run Task` menu always fails! |
| **BUG-EXT-06** | `editors/vscode/snippets/solix.code-snippets:24, 106` | Class snippet uses invalid `public constructor()` (instead of `public ${1:ClassName}()`), and `try` snippet uses legacy `std.Exception` (instead of `Exception`). | Built-in code snippets insert syntax errors! |
| **BUG-TEST-01** | `editors/vscode/test/suite/extension.test.ts` | Extension tests only checked `registeredCommands.includes(...)` without executing the commands, without testing CodeLens argument types, and without testing `extern import solix.io.Console;`! | Broken commands and broken stdlib imports escaped automated testing! |

---

## 2. Architectural Solutions & Editor UX Improvements

### 2.1 Unifying `solix-lsp` with the Real Compiler Dependency & `extern import` Pipeline

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│          Fixed `WorkspaceManager::analyze_document` Pipeline (`solix-lsp`)       │
├──────────────────────────────────────────────────────────────────────────────────┤
│  1. Built-in Default `ModuleSolver` in `solix-common` / `solix-lsp`:             │
│     • Move `FilesystemModuleSolver` & `FilesystemPackageCache` into              │
│       `solix-common` (or initialize default solver in `WorkspaceManager::new`)   │
│       so BOTH `solix lsp` AND standalone `solix-lsp` always resolve              │
│       `~/.solix/packages` (`solix@2.0.0.slxarc`)!                                │
├──────────────────────────────────────────────────────────────────────────────────┤
│  2. Full `manifest.toml` Ingestion:                                              │
│     • When `find_project_manifest` locates `manifest.toml`:                      │
│       - Parse `ProjectManifest::from_file(&manifest_path)`.                      │
│       - Convert `[dependencies]` (`solix = "2.0.0"`, etc.) into                  │
│         `DependencyRequirement`s on `CompilationOptions`!                        │
│       - Run `DependencyResolutionPhase` BEFORE lexing/parsing!                   │
├──────────────────────────────────────────────────────────────────────────────────┤
│  3. Full `extern import` Resolution in `parse_context_resilient`:                │
│     • After parsing user ASTs resiliently, scan all ASTs for                     │
│       `import` / `extern import` statements and invoke the exact same            │
│       `resolve_extern_import` / `DependencyResolutionPhase` logic used by        │
│       `ParserPhase` so `extern import solix.io.Console;` and                     │
│       `import solix.io.Console;` (when `solix` is in `manifest.toml` or          │
│       auto-resolved from `~/.solix/packages`) work 100% identically to CLI!      │
├──────────────────────────────────────────────────────────────────────────────────┤
│  4. Strict Per-File Diagnostic Filtering (`span.id == target_source_id`):        │
│     • Only attach `LexingError`, `ParsingError`, and `BindingError` to           │
│       `target_uri` when `error.span().id == target_source_id`!                   │
└──────────────────────────────────────────────────────────────────────────────────┘
```

#### Smart Implicit Standard Library Resolution for Single-File Scripts & Projects:
Notice what you experienced: in your file you wrote `import solix.io.Console;` or `extern import solix.io.Console;`.
* In `analyze_document`, if an import begins with `solix.` (e.g. `import solix.io.Console;` or `extern import solix.io.Console;`) and `solix` is installed in `~/.solix/packages`, the language server (and compiler when `solix` is installed) will automatically resolve `solix` via the `ModuleSolver` so `solix.io.Console`, `solix.collections.List`, `solix.core.String`, etc. are **always** recognized by IntelliSense, Hover, Completion, and Go-to-Definition!

---

### 2.2 Complete Local Variable & Parameter Indexing in `solix-lsp`

Why did `origin_point.` fail to autocomplete `Point`'s members in `main.slx`? Because `origin_point` is a local variable inside `void main()`, and `Pass 3` pops local scopes when `visit_method_declaration` finishes!

#### The Clean Fix (`LocalSymbolTable` / Preserving Local Variable & Parameter Symbols):
1. **In `crates/solix-compilation/src/phases/binder/passes/pass3_type_checking.rs` (or `AstLocalCollector` in `solix-lsp`):**
   * Whenever `symbol_table.define_local(name, ty, span, is_const)` is called in Pass 3 (for method parameters, local `VariableDeclaration`s, `for`-loop variables, and `catch` variables), **also record a `LocalSymbolEntry { name, symbol_type, decl_span, scope_span, source_id, is_const }` in `SymbolTable::local_symbols: Vec<LocalSymbolEntry>`** (and map its `decl_span` in `resolved_symbols`)!
2. **In `AnalysisSnapshot::symbol_at_offset(offset)` and `completion.rs`:**
   * When resolving the receiver of `origin_point.` at `offset`:
     1. First check `local_symbols` whose enclosing `scope_span` contains `offset` and whose `name == "origin_point"`!
     2. Also check `node_types` and AST `VariableDeclaration` / `Parameter` nodes in the enclosing method!
     3. Then check class fields (`this.field` or implicit `field` inside an instance method of the enclosing class) and global symbols!
   * **Result:** Typing `origin_point.` inside `main()` immediately knows `origin_point` has type `Point` (`sample_workspace::Point`) and suggests **only** `coordinate_x`, `coordinate_y`, and `distance_from_origin()`!

---

### 2.3 Fixing `solix.runMain`, `solix.runTest`, Top-Right Play Button & Profile Selector in VS Code

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                VS Code Extension Execution & UX Architecture                     │
├──────────────────────────────────────────────────────────────────────────────────┤
│  1. Separate `toolchainBinaryPath` (`solix`) & `lspBinaryPath`:                  │
│     • Always resolves `solix` for `run`, `build`, `sync`, `info` commands!       │
├──────────────────────────────────────────────────────────────────────────────────┤
│  2. Top-Right Editor Title Bar Buttons (`package.json` `menus.editor/title`):    │
│     • `$(play) Run Solix` button (`solix.runMain`) visible on every `.slx` tab!  │
│     • `$(beaker) Run Solix Tests` (`solix.runTest`) & `$(file-code) Disassembly` │
├──────────────────────────────────────────────────────────────────────────────────┤
│  3. Smart Context-Aware `solix.runMain` & `solix.runTest`:                       │
│     • Accepts `string | vscode.Uri | undefined` (fixes CodeLens string URI bug!).│
│     • Falls back to `vscode.window.activeTextEditor?.document.uri`.              │
│     • Auto-saves modified files before running (`workspace.saveAll(false)`).     │
│     • Walks upward from target `.slx` file to check for `manifest.toml`:         │
│       - If `manifest.toml` exists:                                               │
│         * `runMain`: builds project (`solix build <root> --<profile>`) and runs  │
│           `solix run "<root>/build/<profile>/<name>.slxarc"`!                    │
│         * `runTest`: builds test archive (`solix build <root> --test`) and runs  │
│           `solix run --test "<root>/build/test/<name>.slxarc"`!                  │
│       - If standalone `.slx` script (no `manifest.toml`):                        │
│         * `runMain`: runs `solix run "<file.slx>"` directly!                     │
├──────────────────────────────────────────────────────────────────────────────────┤
│  4. Interactive Build Profile Status Bar Item (`$(tools) Solix: debug`):         │
│     • Click to switch between `debug`, `release`, and `test` profiles!           │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Phased Implementation Plan

### Phase 1: Fix `solix-lsp` Dependency Resolution (`extern import` & `manifest.toml`) and Diagnostic Filtering
* [x] `[P0]` **Task 1.1:** Move/expose `ClientModuleSolver` and `FilesystemPackageCache` (or a shared `DefaultFilesystemSolver` in `solix-common` / `solix-lsp`) so `WorkspaceManager::new` always has a working `ModuleSolver` pointing to `~/.solix/packages` even when started via standalone `solix-lsp`.
* [x] `[P0]` **Task 1.2:** Update `WorkspaceManager::analyze_document` (`crates/solix-lsp/src/workspace.rs`):
  - Insert the target file first in deterministic order (`BTreeMap` or `Vec`) instead of random `HashMap` iteration.
  - If `manifest.toml` is found, parse `ProjectManifest::from_file(&manifest)` and register all `[dependencies]` into `CompilationOptions::dependency_specs`.
  - Run `DependencyResolutionPhase` before lexing/parsing, and after `parse_context_resilient`, resolve all `extern import` statements (and `import solix.*` when `solix` is available in the package store) using `DependencyResolutionPhase::resolve_extern_import_for_lsp`!
* [x] `[P0]` **Task 1.3:** Filter all `LexingError`, `ParsingError`, and `BindingError` diagnostics in `analyze_document` by `error.span().id == target_source_id` so errors from other files or synthetic headers never bleed into the active file.

### Phase 2: Fix Local Variable & Parameter Intelligence (Completion, Hover, Definition, Rename)
* [x] `[P0]` **Task 2.1:** Record scoped local variables and parameters (`LocalSymbolEntry { name, symbol_type, decl_span, scope_span, source_id, is_const }`) in `SymbolTable` during `Pass 3` (`pass3_type_checking.rs`) and register their declaration and usage spans in `resolved_symbols` and `node_types`.
* [x] `[P0]` **Task 2.2:** Update `AnalysisSnapshot::symbol_at_offset` (`workspace.rs`) and `handle_dot_completion` (`completion.rs`) to resolve local variables, parameters, and enclosing class `this` fields using `local_symbols` and `node_types` at the cursor offset.
* [x] `[P0]` **Task 2.3:** Update `handle_scope_resolution_completion` (`completion.rs`) and `find_definition` (`navigation.rs`) to support imported and namespace-qualified classes (`solix::io::Console`, `Console::println`, `MathHelper::add`) across both local project files and `extern import` packages (`solixlib`).

### Phase 3: Fix the Solix Code Formatter (`formatter.rs`)
* [x] `[P0]` **Task 3.1:** Fix `format_line_content` (`crates/solix-lsp/src/formatter.rs`) so compound operators (`+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`), relational/equality operators (`==`, `!=`, `<=`, `>=`), and lambda arrows (`=>`) are **never** split with spaces!
* [x] `[P0]` **Task 3.2:** Fix brace indentation calculation in `format_source` for mid-line blocks (`} else {`, `} else if (...) {`, `} catch (...) {`, `} finally {`) so `indent_level` decrements for the leading `}` and increments for the trailing `{`.

### Phase 4: Fix All VS Code Extension Commands, CodeLens, Snippets, Tasks & Top-Right Play Button (`editors/vscode/`)
* [x] `[P0]` **Task 4.1:** Update `findSolixBinary` in `editors/vscode/src/extension.ts` to separately resolve both `toolchainBinaryPath` (`solix`) and `lspBinaryPath` (`solix` or `solix-lsp`), ensuring `registerSolixCommands` and `SolixTaskProvider` **always** receive the `solix` CLI binary!
* [x] `[P0]` **Task 4.2:** Fix `solix.runMain`, `solix.runTest`, `solix.buildProject`, `solix.showDisassembly`, and `solix.showSynthesizedApi` in `editors/vscode/src/commands.ts`:
  - Handle `string | vscode.Uri | undefined` arguments in `solix.runMain` and `solix.runTest` (fixing the CodeLens string argument bug) and fall back to `vscode.window.activeTextEditor?.document.uri`.
  - Automatically save modified buffers before running/building.
  - Detect whether the target `.slx` file is inside a `manifest.toml` project or is a standalone `.slx` script, executing the appropriate `solix build` + `solix run` or direct `solix run "<file.slx>"` command!
  - Fix `VirtualContentProvider` (`showDisassembly` & `showSynthesizedApi`) to use `documentUri.fsPath` and compile with project context when inside a `manifest.toml` workspace.
* [x] `[P1]` **Task 4.3:** Add Top-Right Editor Title Bar Play/Test/Disassembly buttons (`menus.editor/title`) and a clickable Build Profile Status Bar selector (`Solix Profile: debug | release | test`), and fix `snippets/solix.code-snippets` (`public ${1:ClassName}()` and `Exception`).

### Phase 5: Exhaustive End-to-End Automated Tests for Commands, `solixlib` Imports & Local Variable Completion
* [x] `[P0]` **Task 5.1:** Add integration tests in `crates/solix-lsp/tests/lsp_tests.rs` verifying:
  - `extern import solix.io.Console;` and `import solix.io.Console;` (with `solix@2.0.0` in package store) produce **zero false-positive `E0232` errors** and provide `Console::println` completions and hover!
  - Local variable dot completion (`Point p = new Point(1, 2); p.`) returns **only** `Point`'s members (`coordinate_x`, `coordinate_y`, `distance_from_origin`) and **never** leaks unrelated classes' methods!
  - Multi-file project error isolation (an error in `helper.slx` is never reported on `main.slx`).
  - Formatter preservation of `+=`, `-=`, `=>`, and `} else {`.
* [x] `[P0]` **Task 5.2:** Upgrade `editors/vscode/test/client_rpc.test.mjs` and `editors/vscode/test/suite/extension.test.ts` to actually **execute** the CodeLens commands (`solix.runMain` with string URI, `vscode.Uri`, and `undefined`), verify `sample_workspace` diagnostics are clean (`0 errors`), and verify `extern import` and local variable completions end-to-end!

---

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Command | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **`solix.io.Console` Import in LSP** | Zero `E0232` errors on `extern import solix.io.Console;` or `manifest.toml` deps | `cargo test -p solix-lsp -- test_stdlib_import` | [x] Complete |
| **Local Variable `.` Completion** | `origin_point.` suggests only `Point` fields & methods | `cargo test -p solix-lsp -- test_local_var_dot_completion` | [x] Complete |
| **CodeLens & Play Button `Run`** | Clicking `▶ Run Main`, `▶ Run Test`, or top-right Play button runs cleanly | `cd editors/vscode && npm test` | [x] Complete |
| **Multi-File Diagnostic Isolation** | Diagnostics only attach to `error.span().id == target_source_id` | `cargo test -p solix-lsp -- test_multifile_diagnostic_isolation` | [x] Complete |
| **Formatter Compound Op & Else Fix** | `+=`, `=>`, and `} else {` format with zero syntax corruption | `cargo test -p solix-lsp -- test_formatter` | [x] Complete |
| **VSIX Re-Packaged & Verified** | `editors/vscode/solix-0.1.0.vsix` built and verified | `cd editors/vscode && npm run package` | [x] Complete |
| **Clippy & Rustfmt Clean** | Zero warnings across all workspace crates | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Complete |