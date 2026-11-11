# Solix Visual Studio Code Extension Guide

The official Solix Visual Studio Code extension (`editors/vscode`) provides first-class language support, full Language Server Protocol (LSP 3.17) intelligence, 108-token TextMate grammar highlighting, interactive CodeLens execution, virtual bytecode disassembly tabs, and automated task integration for the Solix programming language.

---

## 1. Features

### Language Server Protocol (LSP 3.17) Intelligence
- **Live Push Diagnostics**: Real-time error and warning squiggles updated on buffer edits, including clickable links to online error documentation (`https://solix-lang.org/wiki/errors/E0XXX`).
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
| `solix.runMain` | `Solix: Run Application (solix run)` | Runs `solix run` in an integrated VS Code terminal. Wired to `▶ Run Main` CodeLens. |
| `solix.runTest` | `Solix: Run Test Suite (solix run --test)` | Runs test suite via `solix run --test`. Wired to `▶ Run Test` CodeLens. |
| `solix.buildProject` | `Solix: Build Project (solix build)` | Builds the active workspace project into `.slxarc`. |
| `solix.syncPackages` | `Solix: Sync Installed Packages (solix sync --all)` | Runs `solix sync --all` and refreshes language server indexes. |
| `solix.showDisassembly` | `Solix: Inspect 64-Bit Register VM Disassembly` | Opens a side-by-side virtual read-only tab (`solix-disasm://...`) showing disassembled 64-bit VM bytecode. |
| `solix.showSynthesizedApi` | `Solix: Inspect Exported Public API (.slx)` | Opens a side-by-side virtual read-only tab (`solix-api://...`) showing public `.slx` API headers. |
| `solix.restartServer` | `Solix: Restart Language Server` | Cleanly restarts the LanguageClient and re-analyzes all buffers. |

---

## 3. Installation

### Option A: Install Pre-packaged `.vsix`
To install the pre-built `.vsix` extension file directly into Visual Studio Code:

```bash
code --install-extension editors/vscode/solix-0.1.0.vsix
```

Alternatively, inside Visual Studio Code:
1. Open the Extensions view (`Ctrl+Shift+X` or `Cmd+Shift+X`).
2. Click the `...` (Views and More Actions) menu in the top-right corner.
3. Select **Install from VSIX...**.
4. Browse to `editors/vscode/solix-0.1.0.vsix` and click **Install**.

### Option B: Development Mode (Run from Source)
1. Open the `editors/vscode` folder in VS Code:
   ```bash
   code editors/vscode
   ```
2. Press `F5` to launch an Extension Development Host window.
3. Open any workspace containing `.slx` files.

---

## 4. Configuration Settings

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

## 5. Automated Verification Suite

The extension features a 3-layer headless automated verification suite:

```bash
cd editors/vscode

# Layer 1: TextMate grammar assertions (Runs in pure Node.js in < 1 second)
npm run test:grammar

# Layer 2: Headless JSON-RPC integration test suite (Runs against real solix lsp server)
npm run test:client-rpc

# Combined verification
npm test

# Layer 3: Distribution package validation
npm run package
```
