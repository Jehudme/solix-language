# Milestone 051: Unified Configuration Manifest Schema, Profile-Scoped Sources & Automated Crates.io CI/CD

## Document Metadata
* **Milestone ID:** MS-051
* **Target Version:** `v1.2.0`
* **Milestone Branch:** `milestone/051-unified-configuration-manifest-and-profile-scoped-sources`
* **Status:** COMPLETED
* **Lead Architect:** Core Systems & Toolchain Infrastructure Team
* **Target Delivery Date:** 2026-11-16
* **Related Specifications:** [GL-0001](../../internals/guidelines/authoring-commits.md), [GL-0008](../../internals/guidelines/authoring-milestones.md), [GL-0011](../../internals/guidelines/language-conventions.md), [GL-0013](../../internals/guidelines/documentation-update-protocol.md), [SPEC-0001](../../spec/README.md)

---

## 1. Executive Summary & Objectives

The existing Solix project manifest schema relies on fragmented top-level sections (`[info]`, `[entrypoints]`, `[build]`, `[profile.<name>]`, and root `[dependencies]`). This structure causes configuration duplication across build profiles, prevents profile-specific source discovery (e.g. compiling unit tests without bundling them into release artifacts), and complicates dependency management across target environments.

Milestone 051 executes a comprehensive, breaking refactor that deprecates and removes the legacy sections in favor of a modern, unified manifest architecture:

1. **Unified `[configuration]` Table**:
   - Replaces `[info]` with `[configuration]` containing project metadata (`name`, `version`, `description`, `license`, `authors`).
2. **First-Class `[profiles]` Architecture**:
   - The only mandatory profile is `[profiles.release]`.
   - Any secondary profile (`debug`, `test`, custom) defaults to inheriting from `release` if `inherits` is omitted.
   - Profile-scoped properties:
     - `entrypoint`: Profile-specific entry function (e.g. `Main::main`, `Test::test`).
     - `optimization_level` (alias `opt_level`): Integer optimization level (0-3).
     - `inline_threshold`: Function inlining size limit.
     - `debug_symbols`: Boolean flag controlling DWARF / debug information.
     - `static`: Boolean flag configuring static vs dynamic link mode.
     - `sources`: Array of directory paths relative to project root (e.g. `["src"]`, `["src", "tests"]`) recursively scanned for Solix source files (`.slx`).
     - `dependencies`: Inline table specifying profile-scoped dependencies with table notation (`{ version = "...", profile = "..." }`).
3. **Scaffolding Overhaul (`solix new`)**:
   - Project generator produces modern manifests with `release`, `debug`, and `test` profiles pre-configured.
   - The `--native` flag automatically provisions a Rust bridge library in `natives/` configured with a workspace dependency on `solix-ffi`.
4. **CI/CD Automated Publishing to Crates.io**:
   - Configures GitHub Actions release workflows to automatically publish public library crates (`solix-ffi`, `solix-common`, `solix-runtime`, `solix-compilation`) using `CARGO_REGISTRY_TOKEN`.
   - Enforces `publish = false` on internal tooling crates (`solix-toolchain`, `solix-lsp`, `solix-dap`, `solix-tests`).

---

## 2. Priority Blocker Taxonomy & Deliverables

| Priority | Task ID | Subsystem | Description | Release Impact |
|:---|:---|:---|:---|:---|
| **P0** | `MS051-T01` | `solix-common` | Overhaul `ProjectManifest` schema: introduce `[configuration]` and `[profiles]`, removing legacy sections. | Core schema migration |
| **P0** | `MS051-T02` | `solix-common` | Implement profile inheritance resolution: mandatory `release`, implicit parent defaulting, `sources` accumulation, and `dependencies` overrides. | Manifest solver |
| **P1** | `MS051-T03` | `solix-toolchain` | Update `compile`, `build`, and `solix_build.py` to compile resolved `sources` and profile settings. | Build engine integration |
| **P1** | `MS051-T04` | `scaffolding` | Modernize `solix_scaffold.py` and `--native` flag to generate bridge crate depending on `solix-ffi`. | Scaffolding UX |
| **P1** | `MS051-T05` | `projects` | Migrate all project manifests (`breakout`, `solixlib`, `raylib`, `sample_workspace`) and verify tests. | Library conformance |
| **P1** | `MS051-T06` | `ci/cd` | Configure automated Crates.io publishing on release in GitHub Actions workflows. | Release pipeline |
| **P2** | `MS051-T07` | `docs` | Update CLI manifest specifications and synchronization tests. | Documentation sync |

---

## 3. Definition of Done (DoD)

1. The workspace compiles cleanly with zero warnings (`cargo build --workspace --all-targets`).
2. 100% test pass rate across all workspace crates and integration suites (`cargo test --workspace`).
3. Legacy manifest syntax (`[info]`, `[build]`, `[entrypoints]`, root `[dependencies]`) is completely removed and rejected with descriptive diagnostics.
4. `solixlib` test suite runs and passes using the modern `[profiles.test]` configuration.
5. `solix new <project> --native` generates a working project with `natives/` Cargo setup referencing `solix-ffi`.
6. `scripts/verify_docs.py` and `scripts/doc_sync_check.py` pass with 0 defects and zero single-letter variables.
