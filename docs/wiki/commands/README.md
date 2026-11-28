# Solix CLI Commands Reference

The Solix toolchain provides a unified command-line executable (`solix`) for project scaffolding, compilation, building, execution, container inspection, and package store management across Linux, macOS, and Windows.

---

## Subcommand Directory

| Subcommand | Purpose | Wiki Article | Formal Specification |
|---|---|---|---|
| `new` | Scaffolds application or library project workspaces | [Package Commands](package-commands.md#1-solix-new) | [`docs/spec/tools/cli/new.md`](../../spec/tools/cli/new.md) |
| `init` | Initializes `manifest.yaml` in an existing directory | [Utility Commands](utility-commands.md) | [`docs/spec/tools/cli/init.md`](../../spec/tools/cli/init.md) |
| `add` | Adds a package dependency to `manifest.yaml` | [Utility Commands](utility-commands.md) | [`docs/spec/tools/cli/add.md`](../../spec/tools/cli/add.md) |
| `compile` | Compiles `.slx` files into `.slxb` containers | [Compile & Build](compile-and-build.md#1-solix-compile) | [`docs/spec/tools/cli/compile.md`](../../spec/tools/cli/compile.md) |
| `build` | Builds multi-file projects into `.slxp` packages | [Compile & Build](compile-and-build.md#2-solix-build) | [`docs/spec/tools/cli/build.md`](../../spec/tools/cli/build.md) |
| `check` | Type-checks a Solix project without emitting bytecode | [Utility Commands](utility-commands.md#1b-solix-check--solix-test) | [`docs/spec/tools/cli/check.md`](../../spec/tools/cli/check.md) |
| `test` | Compiles and runs `test_*` functions or `TestRunner::test` | [Utility Commands](utility-commands.md#1b-solix-check--solix-test) | [`docs/spec/tools/cli/test.md`](../../spec/tools/cli/test.md) |
| `run` | Executes `.slxb`, `.slxp`, or `.slx` source files | [Run & Info](run-and-info.md#1-solix-run) | [`docs/spec/tools/cli/run.md`](../../spec/tools/cli/run.md) |
| `info` | Inspects container metadata, disassembly, or APIs | [Run & Info](run-and-info.md#2-solix-info) | [`docs/spec/tools/cli/info.md`](../../spec/tools/cli/info.md) |
| `install` | Installs `.slxb` or `.slxp` into local store | [Package Commands](package-commands.md#2-solix-install) | [`docs/spec/tools/cli/install.md`](../../spec/tools/cli/install.md) |
| `export` | Exports packages or compiled binaries for distribution | [Package Commands](package-commands.md) | [`docs/spec/tools/cli/export.md`](../../spec/tools/cli/export.md) |
| `uninstall` | Removes packages or versions from local store | [Package Commands](package-commands.md#3-solix-uninstall) | [`docs/spec/tools/cli/uninstall.md`](../../spec/tools/cli/uninstall.md) |
| `list` | Lists installed packages and versions | [Package Commands](package-commands.md#4-solix-list) | [`docs/spec/tools/cli/list.md`](../../spec/tools/cli/list.md) |
| `clean` | Removes project build artifacts and intermediate caches | [Utility Commands](utility-commands.md#1-solix-clean) | [`docs/spec/tools/cli/clean.md`](../../spec/tools/cli/clean.md) |
| `lsp` | Starts the 32-feature Solix Language Server | [LSP Reference](lsp.md) | [`docs/spec/tools/cli/lsp.md`](../../spec/tools/cli/lsp.md) |
| `dap` | Starts the Solix Debug Adapter Protocol service | [DAP Reference](dap.md) | [`docs/spec/tools/cli/dap.md`](../../spec/tools/cli/dap.md) |
| `fmt` | Formats Solix source code to specification | [Formatter Reference](fmt.md) | [`docs/spec/tools/cli/fmt.md`](../../spec/tools/cli/fmt.md) |
| `doc` | Generates HTML or Markdown API docs from source | [Utility Commands](utility-commands.md#2-solix-doc) | [`docs/spec/tools/cli/doc.md`](../../spec/tools/cli/doc.md) |
| `wiki` | Opens the bundled versioned offline docs portal | [Utility Commands](utility-commands.md#3-solix-wiki) | [`docs/spec/tools/cli/wiki.md`](../../spec/tools/cli/wiki.md) |

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
