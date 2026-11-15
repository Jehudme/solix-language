# Milestone 045: VS Code VM Disassembly Inspection, Build Bar Actions & Compiler Manifest Auto-Discovery

## Document Metadata
* **Milestone ID:** MS-045
* **Target Version:** `v1.1.2`
* **Milestone Branch:** `milestone/045-vscode-disassembly-inspection-build-bar-actions-and-compiler-manifest-discovery`
* **Status:** ACTIVE
* **Lead Architect:** Core Systems & Toolchain Team
* **Target Delivery Date:** 2026-11-15
* **Related Specifications:** [GL-0001](../../internals/guidelines/authoring-commits.md), [GL-0008](../../internals/guidelines/authoring-milestones.md), [GL-0011](../../internals/guidelines/language-conventions.md), [GL-0013](../../internals/guidelines/documentation-update-protocol.md), [SPEC-0001](../../spec/README.md)

---

## 1. Executive Summary & Objectives

Visual Studio Code integration for the Solix toolchain features commands to inspect compiled bytecode disassembly (`solix.showDisassembly`), examine reconstructed API headers (`solix.showSynthesizedApi`), run, debug, and build applications. Testing on real-world projects such as the Raylib Breakout showcase revealed two significant usability blockers:

1. **VM Disassembly Inspection Failure on Dependency-Bound Projects:** Clicking the "Inspect 64-bit VM Disassembly" editor title button on a project file (such as `examples/breakout/src/main.slx`) failed with compiler errors (`cannot_resolve_import`, `unknown_type_in_declaration`). The extension invoked `solix compile <source> -o /tmp/solix_inspect_*.slxb` without propagating dependencies declared in `manifest.toml` (e.g., `--deps raylib@5.5.0`) or entrypoints (`--entry BreakoutGame::main`), resulting in immediate compilation failure before disassembly could be inspected.
2. **Missing Build Action in Editor Title Bar & Status Bar:** While commands existed for running and debugging, there was no dedicated Build button in the editor title bar (`editor/title`) or the bottom status bar next to the profile switcher. Developers had to use the command palette or terminal to build the project.
3. **Compiler Manifest Auto-Discovery:** When invoking `solix compile` directly on source files within a project directory (without explicitly specifying `--deps` or `--entry`), the CLI compiler should automatically detect parent `manifest.toml` files and inherit declared dependencies and entrypoints when not overridden on the command line.

Milestone 045 eliminates these defects across `solix-toolchain` and `editors/vscode`, delivers robust disassembly and API inspection, and introduces convenient Build button actions across VS Code.

---

## 2. Scope & Crates Affected

* **`crates/solix-toolchain`:**
  * In `commands/compile.rs`: Add upward `manifest.toml` discovery. If explicit `--deps` are not supplied on the CLI and a parent `manifest.toml` exists, automatically parse and load declared package dependencies and default entrypoints.
* **`editors/vscode`:**
  * In `src/commands.ts`: Update `VirtualContentProvider` for disassembly (`--disasm`) and API inspection (`--api`) to parse `manifest.toml` dependencies and entrypoint flags when compiling temporary inspection binaries. If an archive (`build/<profile>/<name>.slxarc`) or staged binary already exists, prioritize inspecting the built artifact. Increase execution timeout from 5 seconds to 15 seconds to allow full disassembly of complex games and libraries.
  * In `package.json`: Add `solix.buildProject` with icon `$(gear)` to `menus.editor/title` (navigation@1) alongside `solix.runMain`, `solix.debugMain`, `solix.runTest`, and `solix.showDisassembly`.
  * In `src/commands.ts`: Add a dedicated Build status bar button (`$(gear) Build`) on the bottom status bar next to the build profile selector.
  * Bump extension version to `1.1.2`.
* **`docs/` & `scripts/`:**
  * Update living documentation in `docs/spec/tools/cli/compile.md`, `docs/wiki/commands/compile.md`, and VS Code extension documentation.
  * Validate zero documentation drift using `python3 scripts/verify_docs.py` and `python3 scripts/doc_sync_check.py`.

---

## 3. Implementation Plan

1. **Phase 1: Compiler Upward Manifest Auto-Discovery**
   - In `crates/solix-toolchain/src/commands/compile.rs`, implement upward directory traversal for `manifest.toml` when `--deps` is empty.
   - Auto-populate missing dependencies from `[dependencies]`, entrypoint from `[entrypoints].main`, and project name from `[info].name`.
   - Add integration test in `crates/solix-toolchain/tests/cli_tests.rs`.

2. **Phase 2: VS Code Virtual Content Provider Robustness**
   - In `editors/vscode/src/commands.ts`, extract dependencies and entrypoint from `manifest.toml` when preparing compilation arguments for `solix_inspect_*.slxb`.
   - Extend process execution timeout to 15 seconds.

3. **Phase 3: VS Code Build Action in Title & Status Bars**
   - Add `solix.buildProject` to `menus.editor/title` in `editors/vscode/package.json` with icon `$(gear)`.
   - Add status bar item `buildStatusBarItem` in `editors/vscode/src/commands.ts` displaying `$(gear) Build`.

4. **Phase 4: Packaging, Testing & Documentation**
   - Recompile and package `solix-1.1.2.vsix`.
   - Install and test extension in VS Code.
   - Run complete test suite and documentation verifications.
