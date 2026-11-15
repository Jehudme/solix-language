# Milestone 030: Single-Executable Toolchain Consolidation (`solix`), Shared `solix-common` Package Solver & Complete LSP/VS Code Stabilization

## Document Metadata

* **Milestone ID:** MS-030
* **Target Version:** `v0.1.0-alpha.29`
* **Milestone Branch:** `milestone/029-single-executable-and-lsp-stabilization`
* **Status:** DRAFT (Awaiting Architect Approval)
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/guidelines/documentation-update-protocol.md` (GL-0013)
  * `docs/internals/milestones/026_package-manager-qol-solix-sync-rich-inspection-checksum-security.md` (MS-026)
  * `docs/internals/milestones/027-industrial-language-server-protocol-solix-lsp-32-feature-code-intelligence-engine.md` (MS-027)
  * `docs/internals/milestones/028-official-vs-code_extension-template-customization-automated-headless-verification-suite.md` (MS-028)

---

## 1. Executive Summary & Architectural Objectives

Having multiple separate executables (`solix`, `solix-lsp`, `solix-dbg`) across the workspace creates binary bloat, version-mismatch hazards between the compiler and editor tooling, duplicated PATH discovery logic in VS Code, and split package-solver logic. Simultaneously, real-world usage of `solix-lsp` and `editors/vscode` exposed **15 critical logic bugs** (including `Cannot resolve import 'solix.io.Console'`, broken CodeLens `Run` arguments, missing local variable completions, cross-file diagnostic bleed, and formatter operator corruption).

**Milestone 029** executes a clean architectural consolidation and stabilization across four pillars:

1. **Pillar 1: Single-Executable Toolchain Consolidation (`solix` Only):**
   * Removes the `[[bin]]` target (`src/main.rs`) from `crates/solix-lsp` so `solix-lsp` is strictly a modular library crate (`[lib]`).
   * Renames `crates/solix-dbg` $\to$ `crates/solix-dap` and converts it into a library crate (`[lib]`, removing its `[[bin]]` target).
   * **`crates/solix-toolchain` (`solix`) becomes the SOLE executable binary in the entire workspace**, exposing all toolchain capabilities as subcommands:
     * `solix compile`, `solix build`, `solix run`, `solix info`, `solix new`, `solix install`, `solix sync`, `solix uninstall`, `solix list`, `solix fmt`, **`solix lsp`**, and **`solix dap`**.
2. **Pillar 2: Relocating `ClientModuleSolver`, `FilesystemPackageCache` & `PackageRegistry` to `solix-common`:**
   * Previously, `ClientModuleSolver`, `FilesystemPackageCache`, and `PackageRegistry` lived inside `crates/solix-toolchain/src/`, making them inaccessible to `solix-lsp` and `solix-dap` without manual wiring.
   * Moving `ClientModuleSolver`, `FilesystemPackageCache`, `PackageRegistry`, and `resolve_package_store_dir()` into **`crates/solix-common`** ensures that `solix-compilation`, `solix-runtime`, `solix-lsp`, `solix-dap`, and `solix-toolchain` all share the exact same authenticated package store (`~/.solix/packages`) and cache (`~/.solix/cache/packages`) out of the box!
3. **Pillar 3: Remediating All 9 Language Server Bugs (`solix-lsp`):**
   * Wires `ClientModuleSolver`, `ProjectManifest::from_file` (`manifest.toml` `[dependencies]`), `DependencyResolutionPhase`, and `resolve_extern_import` into `WorkspaceManager::analyze_document` so `import solix.io.Console;` and `extern import solix.io.Console;` resolve with **zero false-positive errors**.
   * Indexes local variables (`Statement::VariableDeclaration`) and method/constructor parameters (`Parameter`) into `BoundSemanticSnapshot::local_symbols` so dot-completion (`origin_point.`), Hover, Go-to-Definition (`F12`), Document Highlights, and Rename (`F2`) work accurately on local variables.
   * Filters diagnostics strictly by `error_span.id == target_source_id` so errors from one file never bleed into another open tab.
   * Fixes `solix fmt` (`formatter.rs`) so compound assignments (`+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`), lambda arrows (`=>`), and `} else {` / `} catch {` / `} finally {` blocks format cleanly without syntax corruption.
4. **Pillar 4: Remediating All 6 VS Code Extension Bugs & Adding Deterministic Command Tests (`editors/vscode`):**
   * Simplifies binary discovery in `extension.ts` to locate the single `solix` executable.
   * Fixes `solix.runMain` and `solix.runTest` in `commands.ts` to accept `vscode.Uri | string | undefined` (fixing the CodeLens string-URI bug where `documentUri.fsPath` evaluated to `undefined`!), auto-save dirty buffers, and resolve `manifest.toml` projects vs. standalone `.slx` scripts.
   * Fixes broken `class` and `try` snippets in `snippets/solix.code-snippets`.
   * Adds automated unit and E2E tests that **execute** the VS Code commands and assert the exact CLI invocations.

---

## 2. Complete Forensic Bug Table Remediated in Milestone 029

| Bug ID | Location | Root Cause | Fix in Milestone 029 |
| :--- | :--- | :--- | :--- |
| **BUG-LSP-01** | `solix-lsp/src/workspace.rs:285-338` | `analyze_document` called `tokenize_source` $\to$ `parse_context_resilient` $\to$ `bind_context` directly, skipping `DependencyResolutionPhase` and `resolve_extern_import`. | Run `DependencyResolutionPhase` before lexing and run `resolve_extern_import` on all `extern import` and `solix.*` imports after parsing. |
| **BUG-LSP-02** | `solix-lsp/src/workspace.rs:248-280` | `analyze_document` never parsed `manifest.toml` (`ProjectManifest::from_file`) to extract `[dependencies]`. | Parse `manifest.toml` and populate `options.dependency_specs` from `[dependencies]`. |
| **BUG-LSP-03** | `solix-lsp/src/workspace.rs:195` | `WorkspaceManager::new` defaulted `solver` to `None` when not explicitly passed. | Default to `Arc::new(ClientModuleSolver::with_default_store())` from `solix-common`. |
| **BUG-LSP-04** | `solix-lsp/src/workspace.rs:321-337` | All `parse_errors` and `bind_errors` across all project files were pushed to `target_uri` without checking `error.span().id == target_source_id`. | Filter all errors strictly by `error.span().id == target_source_id`. |
| **BUG-LSP-05** | `solix-lsp/src/workspace.rs:290-308` | `options.codes` used an unordered `HashMap` and re-tokenized `"<builtins>"` at `source_id = 0`. | Sort project source paths deterministically and skip re-tokenizing pre-populated external/builtin sources. |
| **BUG-LSP-06** | `solix-lsp/src/completion.rs:144-168` | `handle_dot_completion` only looked up `receiver` in `symbol_table.symbols` (which lacks local variables and parameters) and fell back to dumping all methods of all classes. | Resolve `this`, local variables, parameters (`local_symbols`), and fields to their exact `Type`, and remove the bogus all-methods fallback. |
| **BUG-LSP-07** | `solix-lsp/src/workspace.rs:58-92` | `symbol_at_offset` ignored local variable declarations and method parameters. | Index `LocalSymbolInfo` (decl span, scope span, type) so Hover, `F12`, Highlights, and Rename work on locals and parameters. |
| **BUG-LSP-08** | `solix-lsp/src/formatter.rs:76, 175` | Formatter split `+=`, `-=`, `*=`, `/=`, `%=`, `=>` (`a + = 1`, `(x) = > x`) and lost indentation on `} else {`. | Preserve multi-character operators (`+=`, `=>`, etc.) and handle mid-line `} else {` / `} catch {` / `} finally {` brace indentation. |
| **BUG-LSP-09** | `solix-lsp/src/navigation.rs:180-198` | `resolve_source_url` fell back to `snapshot.uri` when jumping to an unopened project file. | Maintain `source_id_to_uri` map for all discovered project files in `WorkspaceManager`. |
| **BUG-EXT-01** | `commands.ts:80-89` & `code_lens.rs:30` | `code_lens.rs` passed a JSON `string` URI to `solix.runMain`, while `commands.ts` accessed `.fsPath` on it (`undefined`), running bare `solix run` with no arguments! | Normalize `vscode.Uri \| string \| undefined` via `resolveTargetFilePath()` in `commands.ts`. |
| **BUG-EXT-02** | `commands.ts:80-98` | Running `solix.runMain` from Command Palette (`undefined` arg) never fell back to `activeTextEditor.document.uri`. | Fall back to `vscode.window.activeTextEditor?.document.uri.fsPath` or workspace `manifest.toml`. |
| **BUG-EXT-03** | `commands.ts:92-98` & `tasks.ts:39` | `solix.runTest` ran bare `solix run --test` without building or specifying the `.slxarc` or `.slx` path. | Build and run the project test archive (`solix build --test && solix run --test build/test/<name>.slxarc`) or active test file. |
| **BUG-EXT-04** | `extension.ts:29-125` | Separate `solix` and `solix-lsp` binaries caused `solix-lsp` to be passed to CLI commands (`solix-lsp run`). | Consolidate into a single `solix` binary across the entire workspace! |
| **BUG-EXT-05** | `commands.ts:28-50` | `VirtualContentProvider` (`--disasm` / `--api`) compiled a single `.slx` file without project `--deps`. | If inside a `manifest.toml` project, inspect the built `.slxb` or pass project dependencies automatically. |
| **BUG-EXT-06** | `snippets/solix.code-snippets:24, 106` | `class` snippet used invalid `public constructor()` instead of `public ${1:ClassName}()`, and `try` used legacy `std.Exception`. | Fix `class` constructor syntax and use `Exception` in `try` snippet. |

---

## 3. Target Workspace Crate Topology (Single Binary Architecture)

```text
crates/
├── solix-common/          # [lib] Shared Core: Bytecode, Types, Span, ProjectManifest,
│                          #       SlxarcArchive, ModuleSolver, ClientModuleSolver,
│                          #       FilesystemPackageCache, PackageRegistry
├── solix-compilation/     # [lib] Compiler Pipeline: Dependency, Lexer, Parser,
│                          #       Binder, ApiExport, Inliner, Emitter, Optimizer
├── solix-runtime/         # [lib] Register VM, VirtualHeap, ARC, Native FFI
├── solix-lsp/             # [lib] Language Server Protocol Engine (NO [[bin]])
├── solix-dap/             # [lib] Debug Adapter Protocol Engine (Renamed from solix-dbg, NO [[bin]])
├── solix-toolchain/       # [[bin]] `solix` — The ONLY executable in the workspace!
│                          #         Subcommands: compile, build, run, info, new,
│                          #         install, sync, uninstall, list, fmt, lsp, dap
└── solix-tests/           # [test] End-to-End Conformance & Integration Suite
```

---

## 4. Phased Implementation Plan

### Phase 1: Single-Executable Consolidation & Moving Solver/Cache/Registry to `solix-common`
* [x] `[P0]` **Task 1.1:** Move `ClientModuleSolver`, `FilesystemPackageCache`, `PackageRegistry`, and `resolve_package_store_dir` from `crates/solix-toolchain/src/{solver.rs, cache.rs, registry.rs}` into `crates/solix-common/src/{solver.rs, cache.rs, registry.rs}` and re-export them from `solix-common` (and `solix-toolchain` for backward compatibility).
* [x] `[P0]` **Task 1.2:** Remove `[[bin]]` (`src/main.rs`) from `crates/solix-lsp/Cargo.toml` and delete `crates/solix-lsp/src/main.rs`. Update `crates/solix-lsp/tests/stdio_e2e_tests.rs` (or move stdio E2E test to `crates/solix-toolchain/tests/`) to spawn `solix lsp --stdio`.
* [x] `[P0]` **Task 1.3:** Rename `crates/solix-dbg` to `crates/solix-dap`, configure `crates/solix-dap/Cargo.toml` as a `[lib]` crate (`src/lib.rs`, deleting `src/main.rs`), update root `Cargo.toml` workspace members, and wire `solix dap [--stdio]` into `crates/solix-toolchain/src/cli.rs` and `main.rs`.

### Phase 2: Remediate All 9 Language Server Bugs in `crates/solix-lsp`
* [x] `[P0]` **Task 2.1:** In `WorkspaceManager::new` (`crates/solix-lsp/src/workspace.rs`), automatically initialize `ClientModuleSolver` (backed by `FilesystemPackageCache` from `solix-common`) whenever `solver` is `None`.
* [x] `[P0]` **Task 2.2:** In `WorkspaceManager::analyze_document`:
  - Parse `manifest.toml` via `ProjectManifest::from_file` and populate `options.dependency_specs` from `[dependencies]`.
  - If any source file in the workspace contains `import solix.` or `extern import solix.` and `solix` is not yet in `dependency_specs`, automatically resolve `solix` via `solver.resolve_latest("solix")` so standard library imports work seamlessly in both projects and standalone scripts.
  - Run `DependencyResolutionPhase` on `CompilationContext` before tokenizing user files, and run `DependencyResolutionPhase::resolve_extern_import` on all `extern import` statements after resilient parsing.
* [x] `[P0]` **Task 2.3:** Filter all `LexingError`, `ParsingError`, and `BindingError` diagnostics strictly by `error_span.id == target_source_id`, and store a complete `source_id_to_uri: HashMap<u16, Url>` mapping for all discovered project files.
* [x] `[P0]` **Task 2.4:** Extract `LocalSymbolInfo` (local variables from `Statement::VariableDeclaration` and parameters from `MethodDeclaration`/`ConstructorDeclaration`/`LambdaExpression`, together with their enclosing `scope_span` and resolved `Type`) into `BoundSemanticSnapshot::local_symbols`.
* [x] `[P0]` **Task 2.5:** Update `completion.rs`, `workspace.rs` (`symbol_at_offset`), `hover.rs`, `navigation.rs`, and `rename.rs` to resolve local variables, parameters, `this`, and class fields accurately, and remove the fallback in `handle_dot_completion` that dumped all methods of all classes when a receiver's type was unknown.
* [x] `[P0]` **Task 2.6:** Fix `formatter.rs` so compound assignments (`+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`), lambda arrows (`=>`), and `} else {` / `} catch {` / `} finally {` blocks format cleanly without corrupting tokens or indentation.

### Phase 3: Remediate VS Code Extension Bugs & Simplify Binary Discovery (`editors/vscode`)
* [x] `[P0]` **Task 3.1:** Update `findSolixBinary` in `editors/vscode/src/extension.ts` to search exclusively for the unified `solix` executable (`target/release/solix`, `target/debug/solix`, `~/.local/bin/solix`, `~/.cargo/bin/solix`, `PATH`), removing all `solix-lsp` binary special cases.
* [x] `[P0]` **Task 3.2:** Refactor `editors/vscode/src/commands.ts`:
  - Implement a pure, unit-testable function `resolveExecutionPlan(inputUri, activeEditorUri, workspaceFolders, solixBinaryPath, mode)` that handles `string` URIs (from CodeLens), `vscode.Uri` instances, and `undefined` (from Command Palette).
  - Auto-save modified buffers before executing `run` or `build`.
  - Fix `solix.runMain`, `solix.runTest`, `solix.buildProject`, `solix.showDisassembly`, and `solix.showSynthesizedApi`.
* [x] `[P1]` **Task 3.3:** Fix `SolixTaskProvider` (`editors/vscode/src/tasks.ts`) and `editors/vscode/snippets/solix.code-snippets`.

### Phase 4: Exhaustive Automated Regression Tests & Documentation Sync (`GL-0013`)
* [x] `[P0]` **Task 4.1:** Add regression tests in `crates/solix-lsp/tests/lsp_tests.rs` verifying:
  - `import solix.io.Console;` and `extern import solix.io.Console;` resolve with **0 diagnostics** and provide `Console::println` completions and hover using `InMemoryModuleSolver` / `ClientModuleSolver`.
  - Dot completion on local variable `Point origin_point = new Point(10, 20); origin_point.` returns **only** `Point` members (`coordinate_x`, `coordinate_y`, `distance_from_origin`) and never unrelated class methods (`MathHelper::add`).
  - Cross-file errors in `broken_other.slx` never appear in `main.slx`'s diagnostics.
  - Formatter preserves `+=`, `-=`, `*=`, `/=`, `%=`, `=>`, and `} else {` indentation.
* [x] `[P0]` **Task 4.2:** Expand `editors/vscode/test/client_rpc.test.mjs` and `editors/vscode/test/suite/extension.test.ts` to test `resolveExecutionPlan`, `solix.runMain` (with string URI, `vscode.Uri`, and `undefined`), `solix.runTest`, and `sample_workspace/src/main.slx`.
* [x] `[P1]` **Task 4.3:** Synchronize `docs/wiki/commands/`, `docs/spec/tools/`, and `docs/guide/editors_vscode.md` per `GL-0013` to reflect the unified `solix` executable and `solix dap` subcommand.

---

## 5. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Command | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Single Executable (`solix`)** | Only 1 `[[bin]]` (`solix`) in workspace; `solix-lsp` & `solix-dap` are `[lib]` | `cargo build --workspace --bins` | [x] Completed |
| **Shared Solver in `solix-common`** | `ClientModuleSolver`, `FilesystemPackageCache`, `PackageRegistry` in `solix-common` | `cargo test -p solix-common` | [x] Completed |
| **`solix.io.Console` IntelliSense** | `import solix.io.Console;` & `extern import solix.io.Console;` emit 0 errors in LSP | `cargo test -p solix-lsp` | [x] Completed |
| **Local Variable IntelliSense** | `origin_point.` completes only `Point` members; Hover/F12/Rename work on locals | `cargo test -p solix-lsp` | [x] Completed |
| **Run / Test Commands Verified** | `solix.runMain` & `solix.runTest` tested with string URI, `vscode.Uri`, and `undefined` | `cd editors/vscode && npm test` | [x] Completed |
| **Clippy & Rustfmt Clean** | Zero warnings across all workspace crates | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Completed |