# CLI Reference: `dap`

The `solix dap` subcommand starts the official Solix Debug Adapter Protocol (DAP) service. It powers interactive debugging sessions in Visual Studio Code, Neovim (`nvim-dap`), Helix, and other DAP-compliant development environments.

---

## 1. Synopsis

```bash
solix dap [path] [options]
```

---

## 2. Key Options & Flags

* `[path]`: Optional project root directory [default: `.`].
* `--stdio`: Runs the debug adapter over standard input and standard output streams [default: `true`].
* `--port <PORT>`: Runs the debug adapter listening on a local TCP socket port for terminal debugger connections.
* `-h, --help`: Prints command help information.
* `-V, --version`: Prints version information for the unified `solix` CLI.

---

## 3. Execution Architecture

The Solix Debug Adapter supports two operational modes:

### 3.1 Standard I/O Mode (`--stdio`)
Used for automated testing and editors that communicate with debug adapters via child process stdio pipes. Guest standard output is wrapped into structured DAP output events.

```bash
solix dap --stdio
```

### 3.2 Integrated Terminal Mode (`--port <PORT>`)
Used by the official Visual Studio Code extension. When launching a debug session, the editor starts `solix dap --port <ephemeral_port>` inside an integrated terminal and connects via a local TCP socket, allowing live interactive guest terminal I/O.

```bash
solix dap --port 45678
```

---

## 4. Supported DAP Capabilities

* **Breakpoints**: Line breakpoints with source map coordinate resolution.
* **Execution Control**: Step into (`F11`), step over (`F10`), step out (`Shift+F11`), pause, and resume (`F5`).
* **Variable Inspection**: Scope-aware variable evaluation, local registers, object field inspection, and array element indexing.
* **Call Stack**: Native frame unwinding displaying function names, source paths, and line/column spans.
* **Expression Evaluation**: Interactive watch expressions and debug console command evaluation.
