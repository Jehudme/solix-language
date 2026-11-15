# Milestone 048: VS Code Tooling, Automated VSIX Packaging & Built-in Install Command

## Document Metadata
* **Milestone ID:** MS-048
* **Target Version:** `v1.1.4`
* **Milestone Branch:** `milestone/048-vscode-tooling-automated-vsix-packaging-and-builtin-install-command`
* **Status:** ACTIVE
* **Lead Architect:** Developer Tooling & Distribution Team
* **Target Delivery Date:** 2026-11-15
* **Related Specifications:** [GL-0001](../../internals/guidelines/authoring-commits.md), [GL-0008](../../internals/guidelines/authoring-milestones.md), [GL-0011](../../internals/guidelines/language-conventions.md), [GL-0013](../../internals/guidelines/documentation-update-protocol.md), [SPEC-0001](../../spec/README.md)

---

## 1. Executive Summary & Objectives

To maximize ease of adoption and eliminate manual extension installation steps, this milestone implements automated `.vsix` distribution in CI/CD and adds a built-in `solix install --vscode` command to the official toolchain:

1. **Built-in `solix install --vscode` Command**:
   - Provide a first-class CLI command that inspects the environment, detects installed editor command-line tools (`code`, `codium`, `cursor`), and automatically installs the official Solix Visual Studio Code extension.
   - Bundle the pre-built `.vsix` archive into `<exe_dir>/assets/vscode/` during release builds so the command functions seamlessly offline without network dependencies.
   - Provide clear, actionable instructions if no editor CLI is discovered in the user's `PATH`.
2. **Automated VSIX Packaging in CI/CD Release Pipeline**:
   - Update `.github/workflows/release.yml` to automatically compile and package `editors/vscode/solix-<version>.vsix` on every release tag.
   - Attach the resulting `.vsix` file directly to the GitHub Release assets alongside binary distributions and standard library archives.
3. **Distribution Asset Bundling**:
   - Configure `crates/solix-toolchain/build.rs` and release packaging scripts to include `assets/vscode/` in binary distribution archives.

---

## 2. Priority Blocker Taxonomy & Deliverables

| Priority | Task ID | Subsystem | Description | Release Impact |
|:---|:---|:---|:---|:---|
| **P0** | `MS048-T01` | `solix-toolchain` | Add `--vscode` flag to `InstallArgs` and implement `execute_install_vscode` in `pkg.rs`. | Core CLI feature |
| **P0** | `MS048-T02` | `solix-toolchain` | Implement `resolve_bundled_vscode_extension_path` in `assets.rs` to locate bundled `.vsix`. | Asset resolution |
| **P1** | `MS048-T03` | `ci/cd` | Add automated `.vsix` build and release asset attachment job to `.github/workflows/release.yml`. | Distribution pipeline |
| **P1** | `MS048-T04` | `solix-toolchain` | Stage `assets/vscode/` and verify `build.rs` packaging into `<target_dir>/assets/vscode/`. | Offline packaging |
| **P2** | `MS048-T05` | `docs` | Document `solix install --vscode` in `README.md`, `docs/wiki/commands/package-commands.md`, and `docs/guide/editors_vscode.md`. | Documentation sync |

---

## 3. Definition of Done (DoD)

1. `solix install --vscode` successfully detects `code` / `codium` / `cursor` in `PATH` and installs the extension.
2. If no editor CLI is found, `solix install --vscode` outputs the absolute path to the `.vsix` file with manual installation instructions.
3. `.github/workflows/release.yml` builds and attaches `solix-<version>.vsix` to GitHub Releases.
4. All unit and CLI integration tests pass cleanly with zero single-letter variables (GL-0011).
5. Documentation audits via `verify_docs.py` and `doc_sync_check.py` pass with 0 defects.
