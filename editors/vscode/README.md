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
  - Code actions and Quick Fixes for error codes (`E0201`, `E0212`, `E0226`, `E0227`, `E0265`, `E0282`, `E0306`).
  - Dynamic build profile selection (`Solix: Select Active Build Profile`) parsed from `manifest.yaml` / `manifest.yml` / `manifest.toml` (including `vars:` YAML anchors), routing Build, Run, Debug, and Test through `--profile <selected>` and `.slxp` artifacts.
  - Symbol rename and 4-space code formatting preserving comments.
- **Syntax Highlighting**: Complete Solix v3 TextMate grammar with scope resolution and destructor highlighting.
- **Bytecode Disassembly Preview**: Inspect 64-bit Register VM bytecode side-by-side (`Solix: Inspect 64-Bit Register VM Disassembly`).
- **Exported API Header Preview**: Reconstruct public `.slx` API headers side-by-side (`Solix: Inspect Exported Public API`).
- **Snippets**: Solix v3 idiomatic snippets.

## Requirements

The extension requires the Solix toolchain (`solix` or `solix-lsp`) installed and available via user settings (`solix.lsp.path`, `solix.toolchain.path`), `~/.local/bin`, `~/.cargo/bin`, `~/.solix/bin`, or system `PATH` (with `.exe` discovery on Windows). Workspace build artifacts (`target/`) are never automatically executed as language servers; developers testing local builds can explicitly set `"solix.lsp.path": "${workspaceFolder}/target/release/solix"`.

## Extension Settings

* `solix.lsp.path`: Custom path to the `solix` or `solix-lsp` executable.
* `solix.lsp.arguments`: Arguments passed to the server [default: `["lsp", "--stdio"]`].
* `solix.toolchain.path`: Custom path to the `solix` toolchain CLI executable.
* `solix.toolchain.compatibleMajor`: Target major version of the Solix toolchain [default: `3`].
* `solix.toolchain.minVersion`: Minimum required SemVer version for the Solix toolchain [default: `"3.0.0"`].
* `solix.trace.server`: Traces JSON-RPC communication between VS Code and the Solix Language Server.
