# Solix for Visual Studio Code

The official Visual Studio Code extension for the **Solix** programming language.

## Features

- **Language Server Intelligence**: Powered by `solix lsp` with 32 features:
  - Real-time diagnostic squiggles with clickable links to Solix documentation.
  - Go-to Definition (`gd`), Type Definition (`gy`), Implementation (`gi`), and Find All References (`gr`).
  - Document Highlights distinguishing read and mutating write usages.
  - Call and Type Hierarchies.
  - Intelligent completion with strict `.` (instance) vs `::` (static/enum) auto-rewrites and postfix templates (`.if`, `.notnull`, `.println`, `.ret`).
  - Rich hover cards with complete signatures, `sizeof` byte calculations, and struct field byte offsets.
  - Inlay hints for parameter names and RAII destructor invocations.
  - Semantic tokens highlighting.
  - CodeLens buttons for `▶ Run Main` and `▶ Run Test`.
  - Code actions and Quick Fixes for error codes (`E0226`, `E0201`, `E0284`).
  - Symbol rename and 4-space code formatting preserving comments.
- **Syntax Highlighting**: Complete 108-token Solix v2.0 TextMate grammar with scope resolution and destructor highlighting.
- **Bytecode Disassembly Preview**: Inspect 64-bit Register VM bytecode side-by-side (`Solix: Inspect 64-Bit Register VM Disassembly`).
- **Exported API Header Preview**: Reconstruct public `.slx` API headers side-by-side (`Solix: Inspect Exported Public API`).
- **Snippets**: Solix v2.0 idiomatic snippets.

## Requirements

The extension requires the Solix toolchain (`solix` or `solix-lsp`) installed and available in system `PATH`, `~/.cargo/bin`, or workspace `target/` directories.

## Extension Settings

* `solix.lsp.path`: Custom path to the `solix` or `solix-lsp` executable.
* `solix.lsp.arguments`: Arguments passed to the server [default: `["lsp", "--stdio"]`].
* `solix.trace.server`: Traces JSON-RPC communication between VS Code and the Solix Language Server.
