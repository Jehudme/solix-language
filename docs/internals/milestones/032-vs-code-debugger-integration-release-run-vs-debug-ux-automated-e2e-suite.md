# Milestone 032: Official VS Code Debugger Integration (`solix dap`), Release-Mode `▶ Run` vs. Debug-Mode `🐞 Debug` Workflow & Automated Headless DAP E2E Suite

## Document Metadata

* **Milestone ID:** MS-032
* **Target Version:** `v0.1.0-alpha.31`
* **Milestone Branch:** `milestone/031-vscode-debugger-and-release-run-ux`
* **Status:** DRAFT (Awaiting Architect Approval)
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/documentation-update-protocol.md` (GL-0013)
  * `docs/guide/editors_vscode.md`
  * `docs/internals/milestones/029-single-executable-consolidation-lsp-remediation-and-common-solver.md` (MS-029)
  * `docs/internals/milestones/030-industrial-debug-adapter-protocol-solix-dap-and-zero-overhead-vm-hooks.md` (MS-030)

---

## 1. Executive Summary & Architectural Vision

With the unified `solix` executable (MS-029) and the `solix dap` Debug Adapter Protocol engine (MS-030) in place, **Milestone 031** integrates first-class interactive debugging and a streamlined **Run (Release) vs. Debug (DAP)** workflow directly into the official Solix Visual Studio Code extension (`editors/vscode`).

### 1.1 Clear Separation Within One Unified Extension (`editors/vscode`)
Rather than forcing users to install and synchronize two separate extensions, `editors/vscode` follows the industry-standard architecture of `rust-analyzer`, Go, and Microsoft's `vscode-mock-debug` by cleanly separating its internal TypeScript modules while shipping as a single unified `.vsix` package:
* `src/extension.ts`: Discovers the single `solix` toolchain binary and coordinates activation.
* `src/commands.ts`: Manages **`▶ Run` (Release Mode)**, **`🐞 Debug` (Debug Profile + DAP)**, **`🧪 Run Test`**, and **`🐞 Debug Test`**.
* `src/debug.ts`: Implements `SolixDebugConfigurationProvider`, `SolixDebugAdapterDescriptorFactory` (spawning `solix dap --stdio`), and `SolixInlineValuesProvider` (showing live variable values inline next to source lines while paused at a breakpoint!).
* `src/tasks.ts`: Provides pre-configured build/run tasks (`solix build --release`, `solix build --debug`, `solix build --test`).

---

## 2. The New `▶ Run` (Release) vs. `🐞 Debug` (Debug + DAP) UX Contract

Every entry point (`main()` and `test*()`) and every `.slx` editor tab now provides two distinct, crystal-clear execution paths:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                 Solix VS Code Run vs. Debug Execution Matrix                     │
├──────────────────────────────────────────────────────────────────────────────────┤
│  1. `▶ Run` (Editor Title Play Button & `▶ Run Main` CodeLens — `solix.runMain`) │
│     • Purpose: Maximum execution speed (Release Optimization Profile).           │
│     • Project Mode (`manifest.toml` present):                                    │
│         Executes: `solix build --release && solix run build/release/<name>.slxarc`│
│     • Standalone Script Mode (single `.slx` file, no `manifest.toml`):           │
│         Executes: `solix run "<file.slx>"` (runs with Release optimizations).    │
├──────────────────────────────────────────────────────────────────────────────────┤
│  2. `🐞 Debug` (Editor Title Bug Button, `🐞 Debug Main` CodeLens, or `F5` —     │
│                 `solix.debugMain`)                                               │
│     • Purpose: Full source-level interactive debugging with breakpoints & watches│
│     • Project Mode (`manifest.toml` present):                                    │
│         1. Runs pre-launch build: `solix build --debug`                          │
│            (preserving all private AST declarations & local variable tables!).   │
│         2. Starts VS Code DAP session (`vscode.debug.startDebugging`) targeting  │
│            `build/debug/<name>.slxarc` via `solix dap --stdio`!                  │
│     • Standalone Script Mode (single `.slx` file, no `manifest.toml`):           │
│         Starts VS Code DAP session targeting `<file.slx>` directly (compiled     │
│         in-memory by `solix dap` using `BuildProfile::Debug`)!                   │
├──────────────────────────────────────────────────────────────────────────────────┤
│  3. `🧪 Run Test` (`solix.runTest`) & `🐞 Debug Test` (`solix.debugTest`)        │
│     • `▶ Run Test`: Runs `solix build --test && solix run --test <archive>`      │
│     • `🐞 Debug Test`: Builds with `--test` and starts DAP with `testMode: true` │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. VS Code Debugger Contributions & Features (`package.json` & `src/debug.ts`)

### 3.1 `contributes.debuggers` & Breakpoint Languages in `package.json`
1. **Debugger Registration (`type: "solix"`):**
   * Supports `launch` and `attach` configurations.
   * Configuration attributes:
     * `program`: Path to `.slx`, `.slxb`, or `.slxarc` (defaults to `${workspaceFolder}` or `${file}`).
     * `args`: Array of guest command-line arguments (`string[]`).
     * `cwd`: Working directory (`${workspaceFolder}`).
     * `stopOnEntry`: Boolean (`false` by default) to pause on the first instruction of `main`.
     * `testMode`: Boolean (`false` by default) to debug the `[entrypoints].test` runner.
   * **Zero-Config `F5` Debugging (`initialConfigurations` & `resolveDebugConfiguration`):**
     * If a user presses `F5` without a `.vscode/launch.json` file, `SolixDebugConfigurationProvider::resolveDebugConfiguration` automatically detects whether the active file is part of a `manifest.toml` project (building `build/debug/<name>.slxarc` first!) or a standalone `.slx` script, and launches the debugger with zero manual setup!
2. **CodeLens Upgrade in `crates/solix-lsp/src/code_lens.rs`:**
   * Above `main()`: emits **`▶ Run Main`** (`solix.runMain`) AND **`🐞 Debug Main`** (`solix.debugMain`)!
   * Above `test*()` functions: emits **`▶ Run Test`** (`solix.runTest`) AND **`🐞 Debug Test`** (`solix.debugTest`)!
3. **Inline Variable Values While Debugging (`SolixInlineValuesProvider`):**
   * Registers `vscode.languages.registerInlineValuesProvider('solix', ...)` so while paused at a breakpoint, VS Code displays the live value of every local variable and parameter right at the end of each source line in the editor!

---

## 4. Phased Implementation Plan

### Phase 1: Upgrade LSP CodeLens for `🐞 Debug Main` and `🐞 Debug Test` (`crates/solix-lsp`)
* [x] `[P0]` **Task 1.1:** Update `code_lenses` in `crates/solix-lsp/src/code_lens.rs` to emit `🐞 Debug Main` (`command: "solix.debugMain"`) alongside `▶ Run Main`, and `🐞 Debug Test` (`command: "solix.debugTest"`) alongside `▶ Run Test`.
* [x] `[P0]` **Task 1.2:** Update `crates/solix-lsp/tests/lsp_tests.rs` to assert that both Run and Debug CodeLenses are generated with valid arguments.

### Phase 2: Implement VS Code Debug Adapter Integration & Inline Values (`editors/vscode/src/debug.ts`)
* [x] `[P0]` **Task 2.1:** Create `editors/vscode/src/debug.ts` (modeled on Microsoft's `vscode-mock-debug` client architecture):
  - `SolixDebugAdapterDescriptorFactory`: Spawns `new vscode.DebugAdapterExecutable(solixBinaryPath, ['dap', '--stdio'])`.
  - `SolixDebugConfigurationProvider`: Implements `provideDebugConfigurations` and `resolveDebugConfigurationWithSubstitutedVariables`. When launching a `manifest.toml` project in debug mode, automatically compiles the debug archive (`solix build --debug`) before starting the DAP session so debug symbols are always fresh!
  - `SolixInlineValuesProvider`: Scans identifiers on visible lines up to the current paused line inside the active function and emits `vscode.InlineValueVariableLookup` items.
* [x] `[P0]` **Task 2.2:** Register `SolixDebugAdapterDescriptorFactory`, `SolixDebugConfigurationProvider`, and `SolixInlineValuesProvider` in `editors/vscode/src/extension.ts`.

### Phase 3: Update `▶ Run` (Release Mode) and `🐞 Debug` (Debug Mode + DAP) Commands & UI (`editors/vscode`)
* [x] `[P0]` **Task 3.1:** Update `solix.runMain` in `editors/vscode/src/commands.ts` so:
  - In a `manifest.toml` project, it builds and executes the **Release profile** (`solix build --release && solix run build/release/<name>.slxarc`).
  - On a standalone `.slx` script, it executes `solix run "<file.slx>"`.
* [x] `[P0]` **Task 3.2:** Implement `solix.debugMain` and `solix.debugTest` in `editors/vscode/src/commands.ts`, invoking `vscode.debug.startDebugging` with the resolved target URI/project.
* [x] `[P0]` **Task 3.3:** Update `editors/vscode/package.json` with:
  - `contributes.debuggers` (`type: "solix"`, `label: "Solix Debugger"`, configuration snippets, schema properties).
  - `contributes.breakpoints` (`language: "solix"`).
  - `contributes.menus` (`editor/title` and `editor/context` buttons for `$(play) Run Solix (Release)` and `$(debug-alt) Debug Solix`).

### Phase 4: 100% Automated Headless DAP & Extension E2E Verification Suite (`editors/vscode/test/`)
* [x] `[P0]` **Task 4.1:** Create `editors/vscode/test/dap_rpc.test.mjs` — a pure Node.js headless DAP client integration test that spawns `solix dap --stdio`, sends framed DAP JSON-RPC requests (`initialize` $\to$ `launch` on `sample_workspace/src/main.slx` $\to$ `setBreakpoints` on line 46 $\to$ `configurationDone`), and asserts:
  1. The debugger emits a `stopped` event (`reason: "breakpoint"`) at line 46!
  2. `stackTrace` returns `main` at `sample_workspace/src/main.slx:46`!
  3. `scopes` and `variables` return `origin_point` (`Point`) and expand its fields!
  4. `evaluate` on `"10 + 20"` returns `"30"`!
  5. `continue` streams guest `println` output and emits `terminated` + `exited` (`exitCode: 0`)!
* [x] `[P0]` **Task 4.2:** Update `editors/vscode/test/suite/extension.test.ts` to test `solix.runMain` (verifying `--release` build/run command generation), `solix.debugMain`, and `SolixDebugConfigurationProvider`.
* [x] `[P1]` **Task 4.3:** Package the updated `.vsix` (`npm run package`) and update `docs/guide/editors_vscode.md` per `GL-0013`.

---

## 5. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Command | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **`▶ Run` Uses Release Mode** | `solix.runMain` builds & runs `release` profile on projects | `cd editors/vscode && npm test` | [x] Completed |
| **`🐞 Debug` Uses Debug + DAP** | `solix.debugMain` & `F5` build `debug` profile and launch `solix dap --stdio` | `cd editors/vscode && npm test` | [x] Completed |
| **Headless Node.js DAP E2E Test** | Verifies breakpoint stop, stackTrace, Object field expansion, evaluate, stepOver, stdout | `cd editors/vscode && npm run test:dap-rpc` | [x] Completed |
| **Inline Values & CodeLens** | `▶ Run Main` + `🐞 Debug Main` CodeLenses & `SolixInlineValuesProvider` active | `cargo test -p solix-lsp && cd editors/vscode && npm test` | [x] Completed |
| **VSIX Package & Docs Sync** | Clean `solix-0.1.0.vsix` package and updated `docs/guide/editors_vscode.md` | `cd editors/vscode && npm run package` | [x] Completed |