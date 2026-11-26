# `solix lsp` — Language Server Protocol Service

The `lsp` subcommand starts the embedded Solix Language Server (`solix-lsp`) speaking LSP 3.17 JSON-RPC over standard input/output for IDE integrations.

---

## Synopsis

```bash
solix lsp [path] [options]
```

---

## Positional Arguments

| Argument | Type | Required | Default | Description |
|---|---|---|---|---|
| `[path]` | Path | No | `.` (Current Directory) | Optional project workspace root directory containing `manifest.yaml`. |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--stdio` | | Flag | `true` | Runs the LSP server over standard input and standard output JSON-RPC streams. |
| `--pkg-dir <dir>` | | Path | `~/.solix/packages` | Override directory used by `ClientModuleSolver` to resolve installed dependencies. |
| `--help` | `-h` | Flag | `false` | Prints CLI help summary. |

---

## Related Documentation

* [LSP Wiki Reference](../../../wiki/commands/lsp.md)
* [LSP Architecture & Protocol Spec](../lsp/protocol.md)
