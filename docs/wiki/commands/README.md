# Solix CLI Commands Reference

The Solix toolchain provides a unified command-line executable (`solix`) for project scaffolding, compilation, building, execution, container inspection, and package store management across Linux, macOS, and Windows.

---

## Subcommand Directory

| Subcommand | Purpose | Wiki Article | Formal Specification |
|---|---|---|---|
| `new` | Scaffolds application or library project workspaces | [Package Commands](package-commands.md#1-solix-new) | [`docs/spec/tools/cli/new.md`](../../spec/tools/cli/new.md) |
| `compile` | Compiles `.slx` files into `.slxb` containers | [Compile & Build](compile-and-build.md#1-solix-compile) | [`docs/spec/tools/cli/compile.md`](../../spec/tools/cli/compile.md) |
| `build` | Builds multi-file projects into `.slxarc` archives | [Compile & Build](compile-and-build.md#2-solix-build) | [`docs/spec/tools/cli/build.md`](../../spec/tools/cli/build.md) |
| `run` | Executes `.slxb`, `.slxarc`, or `.slx` source files | [Run & Info](run-and-info.md#1-solix-run) | [`docs/spec/tools/cli/run.md`](../../spec/tools/cli/run.md) |
| `info` | Inspects container metadata, disassembly, or APIs | [Run & Info](run-and-info.md#2-solix-info) | [`docs/spec/tools/cli/info.md`](../../spec/tools/cli/info.md) |
| `install` | Installs `.slxb` or `.slxarc` into local store | [Package Commands](package-commands.md#2-solix-install) | [`docs/spec/tools/cli/install.md`](../../spec/tools/cli/install.md) |
| `reinstall` | Reinstalls packages from recorded source directories | [Package Commands](package-commands.md) | [`docs/spec/tools/cli/reinstall.md`](../../spec/tools/cli/reinstall.md) |
| `uninstall` | Removes packages or versions from local store | [Package Commands](package-commands.md#3-solix-uninstall) | [`docs/spec/tools/cli/uninstall.md`](../../spec/tools/cli/uninstall.md) |
| `list` | Lists installed packages and versions | [Package Commands](package-commands.md#4-solix-list) | [`docs/spec/tools/cli/list.md`](../../spec/tools/cli/list.md) |
| `clean` | Removes project build artifacts and intermediate caches | [Clean Command](../../spec/tools/cli/clean.md) | [`docs/spec/tools/cli/clean.md`](../../spec/tools/cli/clean.md) |
| `lsp` | Starts the 32-feature Solix Language Server | [LSP Reference](lsp.md) | [`docs/spec/tools/lsp/protocol.md`](../../spec/tools/lsp/protocol.md) |
| `dap` | Starts the Solix Debug Adapter Protocol service | [DAP Reference](dap.md) | [`docs/wiki/commands/dap.md`](dap.md) |
| `fmt` | Formats Solix source code to specification | [Formatter Reference](fmt.md) | [`docs/spec/tools/lsp/protocol.md`](../../spec/tools/lsp/protocol.md) |
| `doc` | Generates HTML or Markdown API docs from source | [Doc Generator Spec](../../spec/tools/cli/doc.md) | [`docs/spec/tools/cli/doc.md`](../../spec/tools/cli/doc.md) |
| `wiki` | Opens the bundled versioned offline docs portal | [Offline Wiki Spec](../../spec/tools/cli/wiki.md) | [`docs/spec/tools/cli/wiki.md`](../../spec/tools/cli/wiki.md) |

---

## Command Reference Sections

* [**`compile` & `build`**](compile-and-build.md): Standalone file compilation, optimization profiles, and declarative `manifest.yaml` build orchestration.
* [**`run` & `info`**](run-and-info.md): Executing bytecode containers, instant source execution, guest argument passing, and container inspection/disassembly.
* [**Package & Scaffolding Commands**](package-commands.md): `solix new`, `solix new native`, `solix install`, `solix uninstall`, and `solix list`.
* [**`manifest.yaml` Wiki**](../manifest.md): Complete guide to YAML anchors, self-contained profiles, `platforms`, hooks, and `natives`.
* [**FFI SDK Wiki**](../ffi/README.md): C/C++ (`solix::ffi`) and Rust (`solix-ffi`) native bridge APIs.
* [**`clean` Command**](../../spec/tools/cli/clean.md): Cleaning project build artifacts (`build/`) and intermediate caches (`build/.cache/`).
* [**Language Server (`lsp`)**](lsp.md): Official LSP 3.17 stdio server integration with editor engines.
* [**Debug Adapter (`dap`)**](dap.md): Official Debug Adapter Protocol integration for interactive debugging.
* [**Code Formatter (`fmt`)**](fmt.md): Standardized 4-space indentation and formatting checker.
