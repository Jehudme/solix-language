# CLI Reference: `lsp`

The `solix lsp` subcommand starts the official Solix Language Server Protocol (LSP 3.17) process over standard I/O (`stdin`/`stdout`). It powers editor integrations including the official Visual Studio Code extension, Neovim, Emacs, and Helix.

---

## 1. Synopsis

```bash
solix lsp [options]
```

> [!NOTE]
> As of Milestone 030, all standalone executables are consolidated into the single unified `solix` executable (`solix lsp --stdio` and `solix dap --stdio`). Standalone invocation is unified under the `solix` command line interface.

---

## 2. Key Options & Flags

* `--stdio`: Runs the Language Server over standard input and standard output streams [default: `true`].
* `-h, --help`: Prints command help information.
* `-V, --version`: Prints version information for `solix-lsp`.

---

## 3. Editor Setup & Integration

### Visual Studio Code
The official Solix extension automatically discovers `solix` or `solix-lsp` in system `PATH` and starts the server with:
```json
{
    "solix.server.path": "solix",
    "solix.server.arguments": ["lsp", "--stdio"]
}
```

### Neovim (via `nvim-lspconfig`)
```lua
local lspconfig = require("lspconfig")
local configs = require("lspconfig.configs")

if not configs.solix then
    configs.solix = {
        default_config = {
            cmd = { "solix", "lsp", "--stdio" },
            filetypes = { "solix" },
            root_dir = lspconfig.util.root_pattern("manifest.toml", ".git"),
            settings = {},
        },
    }
end
lspconfig.solix.setup({})
```

### Helix (`languages.toml`)
```toml
[[language]]
name = "solix"
scope = "source.solix"
injection-regex = "solix"
file-types = ["slx"]
roots = ["manifest.toml"]
language-servers = ["solix-lsp"]

[language-server.solix-lsp]
command = "solix"
args = ["lsp", "--stdio"]
```

---

## 4. Supported Features Matrix

The Solix Language Server supports 32 language intelligence features:

1. **Protocol & Lifecycle**: LSP 3.17 initialization, shutdown, exit handshake.
2. **Document Overlay**: Live in-memory buffer synchronization (`textDocument/didOpen`, `textDocument/didChange`, `textDocument/didClose`).
3. **Workspace Discovery**: Multi-file project awareness with upward `manifest.toml` discovery, unified `[configuration]` and `[profiles]` profile-aware dependency resolution (supporting table notation `name = { version = "...", profile = "..." }` and suffix notation `name-profile = "..."`), and `ClientModuleSolver` dependency resolution (`~/.solix/packages`).
4. **Resilient Compilation**: AST and semantic cache retention during partial typing or syntax errors.
5. **Push Diagnostics**: Real-time error and warning squiggles strictly isolated by `source_id` with strict compiler import resolution parity (exempting only `std::*`), linked to online error documentation (`https://solix-lang.org/wiki/errors/E0XXX`).
6. **Code Navigation**: Go-to Definition (`gd`), Type Definition (`gy`), Implementation (`gi`), and Find All References (`gr`).
7. **Document Highlights**: Distinction between read usages and write/mutating usages.
8. **Call & Type Hierarchies**: Incoming/outgoing call trees and subtype/supertype navigation.
9. **Intelligent Completion**: Receiver-type-aware instance member completion on local variables and parameters, LSP 3.17 `label_details` displaying full parameter signatures and return types on completion items, hierarchical solver-backed `import` and `extern import` completion discovering installed packages via `ModuleSolver::list_packages()` alongside subpackage trees and public symbols, strict Solix v2.0 `.` (instance) vs `::` (static/enum) auto-rewrites, postfix snippets (`.if`, `.notnull`, `.println`, `.ret`).
10. **Rich Hover Cards**: Complete signature, `sizeof: XXB` instance byte size, struct field `+XB` offsets, and doc comments for functions, classes, fields, parameters, and local variables.
11. **Signature Help**: Active parameter tracking with declared formal parameter names, overload switching, and lexical fallback during active typing of unclosed invocations.
12. **Inlay Hints**: True parameter names (e.g., `brick_rect:`, `active_color:` instead of generic `arg0:`) across multi-file and library declarations, interactive hover tooltips exposing full parameter type annotations (e.g., `Rectangle rec`, `Color color`), direct AST callee symbol binding via `resolved_symbols`, live document buffer overlay querying, and RAII scope-exit destructor hints (`~dtor()`).
13. **Semantic Tokens**: LSP 3.17 delta-encoded token highlighting with UTF-16 coordinate precision, CRLF column alignment, multi-line span splitting, overlap deduplication, strict variable vs nominal class type classification, and syntax-error drift resilience with call-site token classification.
14. **Document Outline & Workspace Symbols**: Hierarchical symbol outline and fuzzy workspace symbol search.
15. **Code Lens**: Interactive dual-action lenses: `▶ Run Main` / `Debug Main` for application entrypoints configured in `manifest.toml` (or `"main"` fallback) and `▶ Run Test` / `Debug Test` for test suites, plus reference count lenses linked to `solix.showReferences`.
16. **Code Actions & Quick Fixes**: Automated fixes for `E0226` (`.` to `::`), `E0201` (missing imports), `E0284` (add `override`), and `source.organizeImports` filtered dynamically by client request kinds (`params.context.only`).
17. **Refactoring**: Safe symbol renaming across files.
18. **Code Formatting**: 4-space indent formatter preserving inline comments, compound operators (`+=`, `-=`, `*=`, `/=`, `%=`), relational operators (`==`, `!=`, `<=`, `>=`), lambda arrows (`=>`), and mid-line blocks (`} else {`).
19. **Zero-Panic Implementation Contract**: LSP request handlers (`code_actions`, `completion`, `folding`, `inlay_hints`, `rename`, `workspace`) propagate recoverable failures cleanly with zero production `unwrap`/`expect`/`panic` calls and strict GL-0011 naming conformance.

---

## 5. Related Documentation

* [Formal LSP Specification](../../spec/tools/lsp/protocol.md)
* [LSP Navigation Specification](../../spec/tools/lsp/navigation.md)
* [LSP Intelligence Specification](../../spec/tools/lsp/intelligence.md)
* [Code Formatter Command Reference](fmt.md)
