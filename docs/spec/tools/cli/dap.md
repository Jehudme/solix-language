# `solix dap` — Debug Adapter Protocol Service

The `dap` subcommand starts the interactive bytecode debugger (`solix-dap`) implementing Microsoft Debug Adapter Protocol over `--stdio` or optional TCP sockets.

---

## Synopsis

```bash
solix dap [path] [options]
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
| `--stdio` | | Flag | `true` | Runs the debug adapter over standard input and standard output streams. |
| `--port <port>` | | Integer | None | Optional TCP port to listen on for remote debugger attachments. |
| `--help` | `-h` | Flag | `false` | Prints CLI help summary. |

---

## Related Documentation

* [DAP Wiki Reference](../../../wiki/commands/dap.md)
* [VS Code Extension Guide](../../../guide/editors_vscode.md)
