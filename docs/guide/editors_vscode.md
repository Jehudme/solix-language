# Solix Visual Studio Code Extension Guide

The official Solix Visual Studio Code extension (`editors/vscode`) provides first-class language support, full Language Server Protocol (LSP 3.17) intelligence, 108-token TextMate grammar highlighting, interactive CodeLens execution, virtual bytecode disassembly tabs, and automated task integration for the Solix programming language.

---

## 1. Features

### Language Server Protocol (LSP 3.17) Intelligence
- **Live Push Diagnostics**: Real-time error and warning squiggles updated on buffer edits, linked to error reference documentation in `docs/wiki/errors/` (also bundled offline via `solix wiki`).
- **Code Navigation**:
  - **Go-to-Definition** (`F12`): Jump directly to local variables, methods, fields, and classes across the project and dependencies.
  - **Go-to-Type-Definition**: Jump to declaring class or interface of variable instances.
  - **Go-to-Implementation**: Jump to interface method implementations.
  - **Find All References** (`Shift+F12`): Search across all workspace files for symbol occurrences.
- **Document Highlights**: Distinguishes read usages from mutating write usages.
- **Call & Type Hierarchies**: Incoming/outgoing call trees and subtype/supertype navigation.
- **Autocompletion**:
  - Strict Solix v2.0 `.` (instance) vs `::` (static/enum) auto-rewrites.
  - Postfix completions: `.if`, `.notnull`, `.println`, `.ret`.
  - Contextual keywords and member completions with full markdown documentation resolution.
- **Hover Inspection**: Markdown tooltips displaying symbol kinds, type signatures, `sizeof: XXB` instance byte sizes, field `+XB` offsets, and doc comments.
- **Signature Help**: Inline parameter hints and active parameter tracking during function/method calls (`(` and `,`).
- **Inlay Hints**: Parameter name hints and RAII scope-exit destructor hints (`~dtor()`).
- **Semantic Tokens**: Delta-encoded LSP 3.17 token highlighting for types, keywords, modifiers, parameters, and members.
- **Document Outline & Workspace Symbols**: Hierarchical symbol tree in the VS Code Outline view and fuzzy workspace symbol lookup (`Ctrl+T` / `Cmd+T`).
- **CodeLens Execution**: Interactive `▶ Run Main` and `▶ Run Test` buttons directly above declarations.
- **Code Actions & Quick Fixes**: Automated fixes for `E0226` (`.` to `::`), `E0201` (missing imports), and `E0284` (add `override`).
- **Symbol Renaming**: Safe cross-file identifier refactoring with conflict checking.
- **Code Formatting**: Standardized 4-space indentation preserving inline comments and doc blocks (`Shift+Alt+F`).

### Modern Syntax & Disassembly Highlighting
- **108-Token TextMate Grammar**: Complete syntax highlighting matching Solix v2.0 (`syntaxes/solix.tmLanguage.json`).
- **Register VM Disassembly Syntax**: Dedicated grammar for inspecting 64-bit Register VM instructions, registers (`r0`..`r65535`), offsets, and mnemonics (`syntaxes/solix-disasm.tmLanguage.json`).
- **Smart Doc-Comment Auto-Continuation**: `onEnterRules` automatically continue `/// ` and `/** */` blocks.

---

## 2. Interactive Commands & Virtual Previews

| Command ID | Title | Description |
| :--- | :--- | :--- |
| `solix.runMain` | `Run Application (solix run)` | Runs `solix build --release && solix run` in an integrated VS Code terminal. Wired to `▶ Run Main` CodeLens. |
| `solix.debugMain` | `Debug Application (solix dap)` | Starts interactive debugging session. Wired to `Debug Main` CodeLens and editor title bar button. |
| `solix.runTest` | `Run Test Suite (solix run --test)` | Runs test suite via `solix build --test && solix run --test`. Wired to `▶ Run Test` CodeLens. |
| `solix.debugTest` | `Debug Test Suite (solix dap --test)` | Starts interactive debugging of test runner. Wired to `Debug Test` CodeLens. |
| `solix.showReferences` | `Show References` | Opens VS Code reference peek view for symbol references at target location. |
| `solix.buildProject` | `Build Project (solix build)` | Builds the active workspace project into `.slxarc`. |
| `solix.syncPackages` | `Refresh Packages` | Refreshes local package inventory and language server indexes. |
| `solix.showDisassembly` | `Inspect 64-Bit Register VM Disassembly` | Opens a side-by-side virtual read-only tab (`solix-disasm://...`) showing disassembled 64-bit VM bytecode. |
| `solix.showSynthesizedApi` | `Inspect Exported Public API (.slx)` | Opens a side-by-side virtual read-only tab (`solix-api://...`) showing public `.slx` API headers. |
| `solix.restartServer` | `Restart Language Server` | Cleanly restarts the LanguageClient and re-analyzes all buffers. |

---

## 3. Interactive Debugging (DAP 2.0)

The Solix extension features native zero-config `F5` interactive debugging powered by the integrated Solix Debug Adapter (`solix dap`).

### 3.1 Integrated Terminal DAP Execution (`solix dap --port <port>`)
When launching a debug session from VS Code, the extension spawns `solix dap --port <ephemeral_port>` inside a dedicated VS Code Integrated Terminal titled `"Solix Debug"`.
* **Native Terminal I/O:** Guest standard input (`stdin`), standard output (`stdout`), and standard error (`stderr`) connect directly to the real terminal pseudo-teletype (PTY). Interactive CLI programs utilizing `Console::read_line()` or ANSI terminal escapes function without interception.
* **TCP Protocol Channel:** Debug Adapter Protocol (DAP) JSON-RPC messages travel over a dedicated localhost TCP socket, decoupling debugger control from user terminal I/O.
* **Headless / CI Mode:** Headless and automated integration tests retain support for standard I/O mode via `solix dap --stdio`.

### 3.2 Debugger Capabilities
- **Breakpoints**: Source line breakpoints with automatic instruction snapping.
- **Conditional Breakpoints**: Evaluates expressions in local frame context (e.g. `index == 10`).
- **Hit-Conditional Breakpoints**: Triggers after hit thresholds (e.g. `>= 5`, `== 10`, `% 2 == 0`).
- **Logpoints**: Evaluates `{variable}` interpolation templates and outputs directly to the Debug Console without pausing execution.
- **Function Breakpoints**: Breaks on entry of named methods (`Point::distance_from_origin` or `main`).
- **Call Stack Navigation**: Hierarchical stack frames showing file, line, and column.
- **Variable Inspection & Mutation**: Three-tier scopes (Locals & Parameters, Static & Global Variables, VM Registers & Telemetry), object field expansion, and live in-place mutation.
- **Inline Values Provider**: Displays variable values directly inline in the editor beside source statements while paused.
- **Smart Stepping**: Automatically skips over release-compiled standard library internals while pausing seamlessly inside user callback closures.

---

## 4. Installation

### Option A: One-Command Installation (`solix install --vscode`)
The simplest and recommended method to install the official extension into Visual Studio Code, VSCodium, or Cursor is using the built-in toolchain command:

```bash
solix install --vscode
```

This command inspects your environment, detects any installed editor command-line tools (`code`, `codium`, `cursor`), and automatically installs the bundled extension archive with zero external downloads required.

### Option B: Install Pre-packaged `.vsix` via Editor CLI
You can also install the `.vsix` package manually using your editor's CLI:

```bash
code --install-extension editors/vscode/build/solix-language.vsix --force
```

> [!TIP]
> If you are updating from an earlier version of the Solix extension, reload the editor window after installation (`Ctrl+Shift+P` / `Cmd+Shift+P` -> `Developer: Reload Window`) to refresh registered commands and language server background workers.

Alternatively, inside Visual Studio Code:
1. Open the Extensions view (`Ctrl+Shift+X` or `Cmd+Shift+X`).
2. Click the `...` (Views and More Actions) menu in the top-right corner.
3. Select **Install from VSIX...**.
4. Browse to `editors/vscode/build/solix-language.vsix` and click **Install**.
5. Run `Developer: Reload Window` via the Command Palette.

### Option C: Development Mode (Run from Source)
1. Open the `editors/vscode` folder in VS Code:
   ```bash
   code editors/vscode
   ```
2. Press `F5` to launch an Extension Development Host window.
3. Open any workspace containing `.slx` files.

---

## 5. Configuration Settings

The extension can be configured in your user or workspace `settings.json`:

```json
{
  "solix.lsp.path": "",
  "solix.lsp.arguments": [
    "lsp",
    "--stdio"
  ],
  "solix.trace.server": "off"
}
```

| Setting | Type | Default | Description |
|---|---|---|---|
| `solix.lsp.path` | `string` | `""` | Custom path to the `solix` executable or `solix-lsp` server. If empty, automatically discovers from workspace `target/`, `~/.cargo/bin`, or system `PATH`. |
| `solix.lsp.arguments` | `array` | `["lsp", "--stdio"]` | Arguments passed when launching the Solix language server binary. |
| `solix.trace.server` | `string` | `"off"` | Traces JSON-RPC communication between VS Code and the Solix language server (`"off"`, `"messages"`, `"verbose"`). |

---

## 6. Automated Verification Suite

All compiled bundles, TypeScript build artifacts, and `.vsix` packages are emitted under `editors/vscode/build/` (ignored by Git). The extension features a deterministic headless automated verification suite:

```bash
cd editors/vscode

# Layer 1: Mock VS Code API unit tests (activates commands, task providers, DAP providers)
npm run test:unit

# Layer 2: Headless JSON-RPC LSP & DAP sessions (runs against real solix lsp / solix dap)
npm run test:client-rpc
npm run test:dap-rpc

# Layer 3: Package manifest & TextMate grammar contract checks
npm run test:contract
npm run test:grammar
```

# Combined verification
npm test

# Distribution package validation
npm run package
```
