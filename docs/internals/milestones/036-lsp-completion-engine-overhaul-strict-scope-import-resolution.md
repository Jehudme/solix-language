# Milestone 036: LSP Completion Engine Overhaul & Strict Scope/Import Resolution

## Document Metadata
* **Milestone ID:** MS-036
* **Target Version:** `v0.1.0-rc.1`
* **Milestone Branch:** `milestone/036-lsp-completion-engine-overhaul`
* **Status:** ACTIVE
* **Lead Architect:** @solix-lead
* **Target Delivery Date:** 2026-11-16
* **Related Specifications:**
  * `docs/internals/specs/tooling/lsp-architecture.md`
  * `docs/internals/specs/compilation/export-manifest.md`
  * `docs/internals/specs/language/packages-and-imports.md`

---

## 1. Executive Summary & Objectives

Overhaul the Solix Language Server (`solix-lsp`) completion engine and package symbol export metadata (`solix-compilation`) to eliminate first-keystroke debounce lag, enforce strict hierarchical namespace traversal on `import` statements, and isolate `.` (instance member) and `::` (static/namespace member) completion contexts from global symbol pollution.

Previously, `textDocument/didChange` updated the live document buffer immediately while debouncing semantic analysis (`analyze_document_change`) by 250ms before updating `AnalysisSnapshot`. When an IDE client immediately issued `textDocument/completion` upon the user typing `.` or `::`, `server.rs` queried a stale `AnalysisSnapshot.source_text` that lacked the newly typed trigger character, forcing the user to type `.` twice. Furthermore, typing characters after `.` or `::` (e.g., `Console::p` or `vec.x`) caused `line_prefix.ends_with('.')` to evaluate to `false`, falling through to `handle_general_completion()` and polluting the completion list with every symbol and keyword in the workspace. Finally, `ExportedClass.symbol` in `ExportManifest` omitted full package prefixes, causing `import solix.` to dump all leaf classes (`List`, `Console`) instead of only immediate subpackages (`collections`, `core`, `io`, `math`, `system`, `time`, `crypto`).

This milestone resolves all three architectural defects and establishes deterministic, prefix-aware, zero-lag completion across the entire language server.

## 2. Scope & Crates Affected
* `crates/solix-common`: Package symbol qualification helpers if shared across compilation and tooling.
* `crates/solix-compilation`: `src/phases/emitter/mod.rs` and `src/export_api.rs` — ensure `ExportedClass`, `ExportedEnum`, `ExportedFunction`, and `ExportedStatic` retain fully qualified package namespaces (`solix::collections::List`) and subpackage metadata.
* `crates/solix-runtime`: Not affected.
* `crates/solix-toolchain`: Not affected.
* `crates/solix-lsp`: `src/server.rs`, `src/workspace.rs`, and `src/completion.rs` — live buffer overlay (`get_live_snapshot`), prefix-preserving `.` / `::` context parser, strict hierarchical `import` segment completion, and strict instance vs. static member filtering.
* `crates/solix-dap`: Not affected.
* `docs/`: Update LSP feature documentation and milestone tracking.

## 3. Prioritized Deliverables & Task Matrix

### 3.1 P0: Critical Blockers (Release Gate)
- [ ] `[P0]` **Live Buffer Overlay for Zero-Lag Triggering (`workspace.rs`, `server.rs`):** Implement `WorkspaceManager::get_live_snapshot(&self, uri: &Url) -> Option<AnalysisSnapshot>`. When `self.documents.get(uri)` contains newer `content` than `self.snapshots.get(uri)?.source_text`, overlay the live `content` into `snapshot.source_text` and re-lex `snapshot.tokens` synchronously while preserving the latest semantic `SymbolTable`, `AstArena`, and `package_index`. If no snapshot exists yet for `uri`, execute `analyze_document_change` synchronously. Route `completion()`, `signature_help()`, and `hover()` in `crates/solix-lsp/src/server.rs` through `get_live_snapshot(&uri)` so pressing `.` or `::` triggers completions on the very first keystroke.
- [ ] `[P0]` **Prefix-Preserving Member Access Parser (`completion.rs`):** Replace brittle `line_prefix.ends_with('.')` and `line_prefix.ends_with("::")` checks with `parse_member_access_context(line_prefix: &str) -> Option<(String, AccessDelimiter, String)>`, where `AccessDelimiter` is `Dot` (`.`) or `DoubleColon` (`::`), returning `(receiver_expr, delimiter, partial_member_prefix)`. Whenever `parse_member_access_context` returns `Some`, route strictly to `handle_dot_completion` or `handle_static_completion`, filter candidates case-insensitively by `partial_member_prefix`, and **never** fall through to `handle_general_completion()`.
- [ ] `[P0]` **Hierarchical Package & Subpackage `import` Resolution (`export_api.rs`, `emitter/mod.rs`, `workspace.rs`, `completion.rs`):** Ensure exported symbols in `.slxb` / `.slxarc` (`ExportManifest` and `WorkspaceManager::resolve_package_index`) preserve their exact per-file `package` namespace (e.g., `solix.collections`, `solix.io`, `solix.math`). Update `handle_import_completion` so:
  1. `import ` suggests root packages (`solix`, `raylib`, and local workspace root packages).
  2. `import solix.` suggests **only** immediate child namespace segments (`collections`, `core`, `crypto`, `io`, `math`, `system`, `time`), `*`, and symbols declared directly in `package solix;` (`Solix`)—never leaf classes from child subpackages like `List` or `Console`.
  3. `import solix.collections.` suggests immediate child namespaces (if any), `*`, and public classes/enums/functions declared directly in `package solix.collections;` (`List`, `HashMap`, `HashSet`, `Queue`, `Stack`, `Deque`).
  4. Partial segments after the last `.` (e.g., `import solix.col` or `import solix.collections.Li`) remain strictly inside `handle_import_completion` and filter immediate segment candidates by prefix.

### 3.2 P1: High Priority (Milestone Core)
- [ ] `[P1]` **Strict Instance `.` Member Filtering (`completion.rs`):** In `handle_dot_completion`, resolve the receiver expression type across `this`, local variables, parameters, fields (`this.field`), chained method calls (`obj.get_pos().`), array indexing (`items[i].`), and imported classes. Emit **only** non-static instance fields (`SymbolKind::Field`) and non-static instance methods (`SymbolKind::Method`, excluding constructors) belonging to that class and its superclass hierarchy, plus valid postfix templates (`.var`, `.null`, `.nn`, `.return`). Exclude all static methods and unrelated global symbols.
- [ ] `[P1]` **Strict Static & Namespace `::` Filtering (`completion.rs`):** In `handle_static_completion`:
  1. When the receiver resolves to a class (`Console::`, `Math::`, `Color::`), emit **only** `StaticMethod` and `StaticField` / `Const` symbols whose `parent_symbol == Some(class_id)` (or registered in `class_symbol.members`).
  2. When the receiver resolves to an enum (`Key::`, `GameState::`), emit **only** `EnumVariant` symbols belonging to that enum.
  3. When the receiver resolves to a package namespace (`solix::io::`), emit **only** immediate child namespaces and public types/functions in that namespace.
- [ ] `[P1]` **LSP Completion Integration Test Suite (`crates/solix-lsp/tests/`):** Add automated tests verifying:
  1. First-keystroke `.` and `::` completion immediately after `did_change` without waiting for the 250ms debounce timer.
  2. Partial member typing after `.` (`player.x`) and `::` (`Console::pri`) returns only matching members and zero global symbols or keywords.
  3. `import solix.` returns subpackage segments (`collections`, `io`, `math`, etc.) and excludes `List`/`Console`, while `import solix.collections.` returns `List`, `HashMap`, etc.
  4. `.` excludes static methods and `::` excludes instance fields/methods.

### 3.3 P2: Medium Priority (Ergonomics & Polish)
- [ ] `[P2]` **Completion Item Sort Order & Detail Formatting:** Assign deterministic `sort_text` priorities so direct class members rank above inherited superclass members, and inherited members rank above postfix snippets (`.var`, `.null`). Populate `detail` with concise type signatures.
- [ ] `[P2]` **Rebuild Default Packages with Qualified Export Manifests:** Rebuild `libraries/solixlib` and `libraries/raylib` so installed `.slxarc` archives contain fully qualified subpackage metadata for immediate LSP consumption.

### 3.4 P3: Low Priority (Opportunistic Cleanups)
- [ ] `[P3]` **Deduplicate Receiver Expression Tokenizer Helpers:** Consolidate line-prefix tokenization helpers shared between `completion.rs` and `signature_help.rs`.

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Zero warnings with `-D warnings` | `cargo build --workspace --all-targets` | [ ] Pending |
| **Automated Test Suite** | 100% pass across all unit, integration, and doc tests | `cargo test --workspace --all-targets` | [ ] Pending |
| **Clippy Static Analysis** | Zero warnings, zero `unwrap` / `expect` | `cargo clippy --workspace --all-targets -- -D warnings` | [ ] Pending |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [ ] Pending |
| **Documentation Sync** | Specs and guides updated to GL-0010 | GL-0010 audit checklist | [ ] Pending |

## 5. Risk Assessment & Mitigation Strategies
* **Risk 1:** Typing `.` or `::` on a line makes that line temporarily syntactically incomplete (`Console::` has a trailing operator), which could cause full AST re-parsing to lose local variable bindings inside the current method body.
  * *Mitigation:* `get_live_snapshot` overlays the live text and re-lexes tokens for line-prefix inspection while retaining the last valid semantic `SymbolTable` and `AstArena` from the snapshot, ensuring local variable types and class symbols remain intact while typing incomplete member access expressions.
* **Risk 2:** Existing `.slxarc` packages installed in `~/.solix/packages` prior to MS-036 may still have short symbol names in `ExportManifest` until rebuilt.
  * *Mitigation:* In `WorkspaceManager::resolve_package_index`, also inspect `exported_ast_bytes` / `api_header` `package` declarations as a fallback when `ExportedClass.symbol` lacks `::`, and rebuild `solixlib` and `raylib` as part of milestone validation.