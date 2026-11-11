# Solix CLI Commands Reference

The Solix toolchain provides a unified command-line executable (`solix`) for project scaffolding, compilation, building, execution, container inspection, and package store management across Linux, macOS, and Windows.

---

## Subcommand Directory

| Subcommand | Purpose | Wiki Article | Formal Specification |
|---|---|---|---|
| `new` | Scaffolds application or library project workspaces | [Package Commands](file:///home/jehud/Projects/solix-language/docs/wiki/commands/package-commands.md#1-solix-new) | [`docs/spec/tools/cli/new.md`](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/new.md) |
| `compile` | Compiles `.slx` files into `.slxb` containers | [Compile & Build](file:///home/jehud/Projects/solix-language/docs/wiki/commands/compile-and-build.md#1-solix-compile) | [`docs/spec/tools/cli/compile.md`](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/compile.md) |
| `build` | Builds multi-file projects into `.slxarc` archives | [Compile & Build](file:///home/jehud/Projects/solix-language/docs/wiki/commands/compile-and-build.md#2-solix-build) | [`docs/spec/tools/cli/build.md`](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/build.md) |
| `run` | Executes `.slxb`, `.slxarc`, or `.slx` source files | [Run & Info](file:///home/jehud/Projects/solix-language/docs/wiki/commands/run-and-info.md#1-solix-run) | [`docs/spec/tools/cli/run.md`](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/run.md) |
| `info` | Inspects container metadata, disassembly, or APIs | [Run & Info](file:///home/jehud/Projects/solix-language/docs/wiki/commands/run-and-info.md#2-solix-info) | [`docs/spec/tools/cli/info.md`](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/info.md) |
| `install` | Installs `.slxb` or `.slxarc` into local store | [Package Commands](file:///home/jehud/Projects/solix-language/docs/wiki/commands/package-commands.md#2-solix-install) | [`docs/spec/tools/cli/install.md`](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/install.md) |
| `uninstall` | Removes packages or versions from local store | [Package Commands](file:///home/jehud/Projects/solix-language/docs/wiki/commands/package-commands.md#3-solix-uninstall) | [`docs/spec/tools/cli/uninstall.md`](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/uninstall.md) |
| `list` | Lists installed packages and versions | [Package Commands](file:///home/jehud/Projects/solix-language/docs/wiki/commands/package-commands.md#4-solix-list) | [`docs/spec/tools/cli/list.md`](file:///home/jehud/Projects/solix-language/docs/spec/tools/cli/list.md) |
| `lsp` | Starts the 32-feature Solix Language Server | [LSP Reference](file:///home/jehud/Projects/solix-language/docs/wiki/commands/lsp.md) | [`docs/spec/tools/lsp/protocol.md`](file:///home/jehud/Projects/solix-language/docs/spec/tools/lsp/protocol.md) |
| `fmt` | Formats Solix source code to specification | [Formatter Reference](file:///home/jehud/Projects/solix-language/docs/wiki/commands/fmt.md) | [`docs/spec/tools/lsp/protocol.md`](file:///home/jehud/Projects/solix-language/docs/spec/tools/lsp/protocol.md) |

---

## Command Reference Sections

* [**`compile` & `build`**](file:///home/jehud/Projects/solix-language/docs/wiki/commands/compile-and-build.md): Standalone file compilation, optimization profiles, and project archive build orchestration (`manifest.toml`).
* [**`run` & `info`**](file:///home/jehud/Projects/solix-language/docs/wiki/commands/run-and-info.md): Executing bytecode containers, instant source execution, guest argument passing, and container inspection/disassembly.
* [**Package Management Commands**](file:///home/jehud/Projects/solix-language/docs/wiki/commands/package-commands.md): `solix new`, `solix install`, `solix uninstall`, and `solix list`.
* [**Language Server (`lsp`)**](file:///home/jehud/Projects/solix-language/docs/wiki/commands/lsp.md): Official LSP 3.17 stdio server integration with editor engines.
* [**Code Formatter (`fmt`)**](file:///home/jehud/Projects/solix-language/docs/wiki/commands/fmt.md): Standardized 4-space indentation and formatting checker.
