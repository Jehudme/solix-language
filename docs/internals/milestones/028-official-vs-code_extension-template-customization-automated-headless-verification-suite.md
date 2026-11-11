# Milestone 028: Official Solix VS Code Extension (`editors/vscode`), Template-Driven `vscode-languageclient` Architecture, Debug/Run CodeLens & 100% Automated Headless Verification Suite

## Document Metadata

* **Milestone ID:** MS-028
* **Target Version:** `v0.1.0-alpha.28`
* **Milestone Branch:** `milestone/028-vscode-extension-and-headless-tests`
* **Status:** DRAFT (Awaiting Architect Approval)
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/documentation-update-protocol.md` (GL-0013)
  * `docs/guide/editors_vscode.md`
  * `docs/internals/milestones/027-language-server-protocol-and-code-intelligence.md` (MS-027)

---

## 1. Executive Summary & Architectural Strategy

Building a VS Code extension from scratch with custom ad-hoc RPC glue or untested configuration files is brittle and error-prone. Worse, if an extension can only be tested by a human manually pressing `F5` on a desktop GUI, regressions inevitably slip in.

**Milestone 028** solves both problems permanently by adopting two proven industry practices:

1. **Proven Upstream Boilerplate Foundation (Zero Custom RPC Reinvention):**
   * Instead of writing custom IPC or custom LSP client wrappers from scratch, `editors/vscode/` is built directly upon the battle-tested **Microsoft `vscode-extension-samples/lsp-sample` + `IWANABETHATGUY/tower-lsp-boilerplate` architecture** using the official `vscode-languageclient` (`v9.0.1`) library.
   * Because `vscode-languageclient` natively handles 100% of the 32 LSP 3.17 features implemented in Milestone 027 (`solix lsp`), the TypeScript extension code stays **under 250 lines of clean, rock-solid code**—focused strictly on:
     1. Locating the `solix` (or `solix-lsp`) binary automatically (workspace `target/release/solix`, `target/debug/solix`, `~/.cargo/bin/solix`, or system `PATH`).
     2. Launching `LanguageClient` with `solix lsp --stdio`.
     3. Registering client-side commands for the **CodeLens `▶ Run Main` / `▶ Run Test` buttons**, **`Solix: Restart Language Server`**, **`Solix: Build Project`**, and **`Solix: Show Disassembly / API Preview`**.
     4. Providing a **Status Bar Item** showing server health, active Solix version, and one-click server restart.

2. **Three-Layer 100% Automated Headless Verification Suite (No Manual GUI Needed!):**
   To ensure that an AI agent or CI server can test **every single feature of the VS Code extension from a headless Linux terminal**, Milestone 028 implements a **3-Layer Automated Test Suite**:
   * **Layer 1: Automated TextMate Grammar Snapshot & Scope Tests (`vscode-tmgrammar-test`):**
     * Runs in pure Node.js in $< 1\text{ second}$ without needing a display server!
     * Tokenizes real `.slx` fixtures against `syntaxes/solix.tmLanguage.json` and asserts that every Solix v2.0 construct (`package`, `extern import`, `class a::b::C`, `Math::sqrt`, `~Destructor()`, `weak`, `<args...>`, `0xDEAD_BEEF`, string escapes) receives the exact expected TextMate scopes.
   * **Layer 2: Headless VS Code Extension Host E2E Tests (`@vscode/test-cli` + `@vscode/test-electron` + `xvfb-run`):**
     * Downloads a hermetic headless VS Code instance (or runs under `xvfb-run -a` on Linux), activates the Solix extension, opens a real Solix test workspace, waits for `solix lsp` to initialize, and programmatically invokes VS Code's actual editor providers (`vscode.executeHoverProvider`, `vscode.executeCompletionItemProvider`, `vscode.executeDefinitionProvider`, `vscode.executeDocumentSymbolProvider`, `vscode.executeCodeActionProvider`, `vscode.executeFormatDocumentProvider`, `vscode.languages.getDiagnostics`) and asserts on the results!
   * **Layer 3: Schema & Packaging Validation (`vsce package --no-dependencies`):**
     * Validates `package.json` contribution points, `language-configuration.json`, `snippets/solix.code-snippets`, and packages the distributable `solix-vscode-<version>.vsix` artifact automatically.

---

## 2. Complete Extension Feature & Contribution Architecture

```text
editors/vscode/
├── package.json                        # Manifest based on tower-lsp-boilerplate & lsp-sample
├── tsconfig.json                       # Strict ES2022 TypeScript configuration
├── .vscode-test.mjs                    # Official @vscode/test-cli headless configuration
├── language-configuration.json         # Brackets, auto-closing, folding, indent rules, onEnterRules
├── syntaxes/
│   ├── solix.tmLanguage.json           # Modern Solix v2.0 TextMate grammar (::, extern import, etc.)
│   └── solix-disasm.tmLanguage.json    # Syntax highlighting for 64-bit Register VM disassembly!
├── snippets/
│   └── solix.code-snippets             # Idiomatic v2.0 snippets (class, main, test, dtor, try, match)
├── src/
│   ├── extension.ts                    # LanguageClient bootstrap, binary locator, status bar
│   ├── commands.ts                     # Run Main, Run Test, Build, Sync, Inspect API & Disasm
│   └── tasks.ts                        # Automatic VS Code TaskProvider for `solix build` & `solix run`
└── test/
    ├── grammar/                        # Layer 1: vscode-tmgrammar-test TextMate unit tests
    │   ├── syntax_v2.slx               # Assertions for ::, extern import, ~dtor, primitives, generics
    │   └── expressions.slx             # Assertions for operators, lambdas, tuples, string escapes
    ├── fixtures/                       # Sample Solix project opened by Headless VS Code E2E tests
    │   ├── manifest.toml
    │   └── src/
    │       ├── main.slx
    │       └── helper.slx
    └── suite/                          # Layer 2: @vscode/test-electron E2E integration tests
        ├── index.ts
        ├── extension.test.ts           # Verifies activation, LanguageClient ready state & commands
        ├── lsp_features.test.ts        # Verifies Diagnostics, Hover, Completion, Def, CodeAction, Format
        └── tasks_and_codelens.test.ts  # Verifies CodeLens (`Run Main`, `Run Test`) & TaskProvider
```

---

### 2.1 Modern Solix v2.0 TextMate Grammar (`syntaxes/solix.tmLanguage.json`)
The legacy grammar in `docs/spec/tools/lsp/protocol.md` referenced obsolete keywords (`match`, `defer`, `f32`, `f64`, `auto`). The new `syntaxes/solix.tmLanguage.json` **MUST** match the exact **108 `TokenKind`s** of Solix v2.0:
1. **Declaration Keywords (`keyword.declaration.solix`):** `package`, `import`, `alias`, `class`, `interface`, `enum`, `operator`.
2. **Modifiers (`storage.modifier.solix`):** `public`, `private`, `protected`, `internal`, `static`, `inline`, `native`, `const`, `virtual`, `override`, `weak`, `abstract`, `extern`.
3. **Control Flow Keywords (`keyword.control.solix`):** `if`, `else`, `while`, `do`, `for`, `switch`, `case`, `default`, `break`, `continue`, `return`, `try`, `catch`, `finally`, `throw`, `assert`, `exit`.
4. **Expression Keywords (`keyword.operator.word.solix`):** `new`, `super`, `this`, `instanceof`, `sizeof`, `cast`, `extends`, `implements`.
5. **14 Built-in Primitives & Friendly Aliases (`storage.type.primitive.solix`):**
   * `int8`, `int16`, `int32`, `int64`, `uint8`, `uint16`, `uint32`, `uint64`, `float32`, `float64`, `bool`, `char`, `string`, `tuple`, `void`.
   * Friendly aliases: `sbyte`, `short`, `int`, `long`, `byte`, `ushort`, `uint`, `ulong`, `float`, `double`, `boolean`.
6. **Scope Resolution & Member Calls:**
   * `([A-Z][A-Za-z0-9_]*)\s*(::)\s*([a-zA-Z_][A-Za-z0-9_]*)`: Highlights `ClassName` as `entity.name.type.class.solix`, `::` as `punctuation.accessor.colon.solix`, and the target method/constant appropriately!
   * Destructors `~([A-Z][A-Za-z0-9_]*)`: Highlights `~ClassName` as `entity.name.function.destructor.solix`.
7. **Numeric Literals with Underscores & Suffixes (`constant.numeric.solix`):**
   * Hex `0[xX][0-9a-fA-F_]+[Ll]?`, Binary `0[bB][01_]+[Ll]?`, Float `[0-9][0-9_]*\.[0-9_]*([eE][+-]?[0-9_]+)?[fFdD]?`, Integer `[0-9][0-9_]*[Ll]?`.

---

### 2.2 Smart `language-configuration.json` & Doc-Comment Auto-Continuation
* Configures bracket pairs (`{}`, `[]`, `()`), surrounding pairs, and angle brackets (`<>` in type contexts).
* Configures **`onEnterRules`** so pressing `Enter` inside a `///` doc-comment block automatically inserts `/// ` on the next line, and pressing `Enter` after `/**` automatically inserts ` * ` and closes with ` */`!
* Configures `wordPattern` and indentation rules (`increaseIndentPattern`, `decreaseIndentPattern`).

---

### 2.3 Rich Client Commands, Virtual Documents & CodeLens Actions (`src/commands.ts`)
In addition to standard LSP features, the extension registers these interactive commands in the VS Code Command Palette (`Ctrl+Shift+P`) and wires them to the LSP `CodeLens` buttons:

| Command ID | Title | Behavior |
| :--- | :--- | :--- |
| `solix.runMain` | `Solix: Run Application (solix run)` | Invoked by clicking `▶ Run Main` CodeLens above `main()` or from the editor title bar play button; runs `solix run` in an integrated VS Code terminal. |
| `solix.runTest` | `Solix: Run Tests (solix build --test && solix run --test)` | Invoked by clicking `▶ Run Test` CodeLens above `test()`; builds and runs the test suite in the integrated terminal. |
| `solix.buildProject` | `Solix: Build Project (Debug / Release)` | Prompts or builds the current `manifest.toml` project via `solix build`. |
| `solix.syncPackages` | `Solix: Sync Installed Packages (solix sync --all)` | Runs `solix sync --all` and refreshes the language server workspace. |
| `solix.showDisassembly` | `Solix: Inspect 64-Bit Register VM Disassembly` | Compiles the active `.slx` file or project in memory and opens a side-by-side virtual read-only tab (`solix-disasm://...`) showing the disassembled 64-bit Register VM bytecode (`solix info --disasm`)! |
| `solix.showSynthesizedApi` | `Solix: Inspect Exported Public API (.slx)` | Opens a side-by-side virtual read-only tab (`solix-api://...`) showing the reconstructed `.slx` API header (`solix info --api`)! |
| `solix.restartServer` | `Solix: Restart Language Server` | Cleanly restarts the `solix lsp` client and re-indexes the workspace. |

---

## 3. How the 3-Layer Automated Testing Pipeline Works

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│           Automated Verification Pipeline for `editors/vscode`                   │
├──────────────────────────────────────────────────────────────────────────────────┤
│  Step 1: `npm run lint` & `npm run compile`                                      │
│    • Type-checks all TypeScript (`tsc --noEmit`) & validates JSON schemas.       │
├──────────────────────────────────────────────────────────────────────────────────┤
│  Step 2: `npm run test:grammar` (Pure Node.js — Runs in < 1 second, No GUI!)     │
│    • Uses `vscode-tmgrammar-test` to assert exact TextMate scopes on             │
│      `test/grammar/syntax_v2.slx` and `test/grammar/expressions.slx`.            │
├──────────────────────────────────────────────────────────────────────────────────┤
│  Step 3: `npm run test:e2e` (`xvfb-run -a npx vscode-test` or Headless Electron) │
│    • Launches headless VS Code Extension Host with `editors/vscode` loaded.      │
│    • Automatically locates `../../target/debug/solix` (or `release/solix`).      │
│    • Opens `test/fixtures/src/main.slx` and asserts:                             │
│        1. Extension activates and `LanguageClient` reaches `State.Running`.      │
│        2. `vscode.executeHoverProvider` returns Markdown with `sizeof` & types.  │
│        3. `vscode.executeCompletionItemProvider` on `Helper::` returns statics.  │
│        4. `vscode.executeDefinitionProvider` jumps from `Helper::compute` to     │
│           `test/fixtures/src/helper.slx` at the exact line & column!             │
│        5. Introducing `Helper.compute()` triggers diagnostic `E0226` AND         │
│           `vscode.executeCodeActionProvider` returns the `.` -> `::` Quick Fix!  │
│        6. `vscode.executeFormatDocumentProvider` formats unindented code!        │
├──────────────────────────────────────────────────────────────────────────────────┤
│  Step 4: `npm run package` (`npx @vscode/vsce package`)                          │
│    • Produces the installable `editors/vscode/solix-0.1.0.vsix` package!         │
└──────────────────────────────────────────────────────────────────────────────────┘
```

* **Fallback / Dual-Runner Guarantee:** Even in container environments where Electron download is restricted, we also include a **Direct Node.js `vscode-jsonrpc` Client Integration Test (`npm run test:client-rpc`)** that runs the exact TypeScript client configuration against `solix lsp --stdio` in pure Node.js without needing X11/Wayland, PLUS the full `@vscode/test-cli` Electron suite when Xvfb/display is available!

---

## 4. Phased Implementation Plan

### Phase 1: Scaffold `editors/vscode` from `tower-lsp-boilerplate` / `lsp-sample` Architecture
* [ ] `[P0]` **Task 1.1:** Create `editors/vscode/package.json`, `tsconfig.json`, and `.vscodeignore` using `vscode-languageclient` (`^9.0.1`), `@vscode/vsce`, `vscode-tmgrammar-test`, and `@vscode/test-cli`.
* [ ] `[P0]` **Task 1.2:** Author `editors/vscode/language-configuration.json` with bracket matching, comment toggling, folding markers, and `///` / `/** */` doc-comment `onEnterRules`.
* [ ] `[P0]` **Task 1.3:** Author `editors/vscode/snippets/solix.code-snippets` using strict Solix v2.0 syntax (`::` scope resolution, zero single-letter variables per `GL-0011`, `~ClassName()` destructors, `extern import`, `manifest.toml` snippets).

### Phase 2: Modern Solix v2.0 TextMate Grammar & Automated Grammar Scope Tests
* [ ] `[P0]` **Task 2.1:** Author `editors/vscode/syntaxes/solix.tmLanguage.json` covering all 108 Solix v2.0 tokens, `::` scope resolution, qualified declarations (`class a::b::C`), destructors (`~Name()`), all 14 primitives + friendly aliases, and numeric literals with underscores.
* [ ] `[P1]` **Task 2.2:** Author `editors/vscode/syntaxes/solix-disasm.tmLanguage.json` for syntax-highlighting 64-bit Register VM disassembly output (`LoadI32`, `AddI32Imm`, `CallDirect`, `r0`..`r65535`).
* [ ] `[P0]` **Task 2.3:** Author `test/grammar/syntax_v2.slx` and `test/grammar/expressions.slx` and verify 100% pass via `npx vscode-tmgrammar-test`.

### Phase 3: TypeScript `LanguageClient`, Auto-Binary Discovery, Virtual Previews & Commands
* [ ] `[P0]` **Task 3.1:** Implement `editors/vscode/src/extension.ts`:
  - Automatic discovery of the `solix` binary (`solix.lsp.path` setting $\to$ workspace `target/release/solix` $\to$ `target/debug/solix` $\to$ `~/.cargo/bin/solix` $\to$ `PATH`).
  - Initializes `LanguageClient` spawning `solix lsp --stdio` (with file watcher for `**/*.slx` and `**/manifest.toml`).
  - Status bar item displaying `$(check) Solix LSP` (or `$(sync~spin)` / `$(error)`).
* [ ] `[P1]` **Task 3.2:** Implement `editors/vscode/src/commands.ts` and `src/tasks.ts`:
  - Client commands for CodeLens (`solix.runMain`, `solix.runTest`), `solix.buildProject`, `solix.syncPackages`, `solix.restartServer`.
  - Virtual `TextDocumentContentProvider`s for `solix.showDisassembly` (`solix info --disasm`) and `solix.showSynthesizedApi` (`solix info --api`).
  - Automatic VS Code `TaskProvider` for `solix build --debug`, `solix build --release`, and `solix build --test`.

### Phase 4: Automated Headless E2E Verification Suite & `.vsix` Packaging
* [ ] `[P0]` **Task 4.1:** Implement the Node.js JSON-RPC Client E2E test (`test/client_rpc.test.mjs`) and the `@vscode/test-cli` headless VS Code Extension Host test suite (`test/suite/lsp_features.test.ts`) verifying Diagnostics, Hover, Completion (`.` and `::`), Go-to-Definition, Document Symbols, Code Actions (`E0226` `.` $\to$ `::`), Inlay Hints, Rename, and Formatting.
* [ ] `[P0]` **Task 4.2:** Package the extension into `editors/vscode/solix-0.1.0.vsix` via `npm run package` and verify zero warnings or packaging errors.
* [ ] `[P1]` **Task 4.3:** Update `docs/guide/editors_vscode.md` with all new settings, commands, keyboard shortcuts, and troubleshooting steps per `GL-0013`.

---

## 5. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Command | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **TypeScript & Schema Compile** | Zero TypeScript or JSON schema errors | `cd editors/vscode && npm run compile` | [ ] Pending |
| **Layer 1: TextMate Grammar Tests** | 100% of v2.0 grammar scope assertions pass | `cd editors/vscode && npm run test:grammar` | [ ] Pending |
| **Layer 2: Automated Client/E2E Tests** | Hover, `.` / `::` Completion, Def, QuickFix, Rename, Format verified | `cd editors/vscode && npm test` | [ ] Pending |
| **Layer 3: VSIX Package Build** | Builds `editors/vscode/solix-0.1.0.vsix` cleanly | `cd editors/vscode && npm run package` | [ ] Pending |
| **Documentation Sync (`GL-0013`)** | `docs/guide/editors_vscode.md` updated with all features | `python3 scripts/doc_sync_check.py` | [ ] Pending |