# Milestone 049: Hierarchical Profile Inheritance & Manifest Compilation Options

## Document Metadata
* **Milestone ID:** MS-049
* **Target Version:** `v1.1.4`
* **Milestone Branch:** `milestone/049-hierarchical-profile-inheritance-and-manifest-compilation-options`
* **Status:** ACTIVE
* **Lead Architect:** Core Systems & Toolchain Team
* **Target Delivery Date:** 2026-11-15
* **Related Specifications:** [GL-0001](../../internals/guidelines/authoring-commits.md), [GL-0008](../../internals/guidelines/authoring-milestones.md), [GL-0011](../../internals/guidelines/language-conventions.md), [GL-0013](../../internals/guidelines/documentation-update-protocol.md), [SPEC-0001](../../spec/README.md)

---

## 1. Executive Summary & Objectives

Project configuration via `manifest.toml` should offer complete, granular control over all compilation, optimization, and packaging options. This milestone redesigns the `manifest.toml` profile and dependency configuration into a comprehensive, hierarchical inheritance system:

1. **Rich Compilation Options in Manifest Profiles**:
   - Allow any compilation option (e.g., `optimization_level`, `inline_threshold`, `debug_symbols`, `verify_checksums`, `entry_point`, `link_mode`, `emit_disassembly`, `warnings_as_errors`, `memory_limit_mb`) to be declared directly in `manifest.toml`.
2. **Hierarchical Profile Inheritance System**:
   - Establish a baseline `default` profile containing standard compiler defaults.
   - Built-in profiles `debug`, `release`, and `test` inherit from `default` and override specific options (e.g. `release` sets `optimization_level = 3`, `inline_threshold = 8`, `debug_symbols = false`; `debug` sets `optimization_level = 0`, `debug_symbols = true`).
   - Allow user-defined custom profiles (e.g., `[profile.bench]`, `[profile.ci]`) that explicitly inherit from `default`, `release`, `debug`, or any other parent profile via `inherits = "<parent>"`.
   - The profile resolver resolves the inheritance hierarchy depth-first, applying overrides cleanly.
3. **Profile-Aware Dependency Declarations**:
   - In `[dependencies]`, support specifying the target profile for each dependency:
     - String format: `raylib-release = "5.5.0"`, `raylib-debug = "5.5.0"`
     - Table format: `raylib = { version = "5.5.0", profile = "debug" }`
     - When profile is omitted, default automatically to `release`.
4. **Toolchain CLI Integration**:
   - Update `solix build --profile <name>`, `solix compile`, and `solix run` to consume the resolved hierarchical options from `manifest.toml`.
   - Update project template generators in `crates/solix-toolchain/assets/templates/` to generate `manifest.toml` files with the modern profile configuration.

---

## 2. Priority Blocker Taxonomy & Deliverables

| Priority | Task ID | Subsystem | Description | Release Impact |
|:---|:---|:---|:---|:---|
| **P0** | `MS049-T01` | `solix-common` | Extend `ProjectManifest` schema with `ProfileConfig` and hierarchical `inherits` resolution in `project.rs`. | Core manifest schema |
| **P0** | `MS049-T02` | `solix-common` | Support profile-aware dependency specifications (string suffix and table formats) with release fallback. | Dependency resolution |
| **P1** | `MS049-T03` | `solix-toolchain` | Wire `solix build`, `solix compile`, and `solix run` to apply resolved profile compilation options. | Toolchain execution |
| **P1** | `MS049-T04` | `templates` | Update default project templates (`untitled-rust`, scaffold scripts) with standard profile configuration. | Project scaffolding |
| **P2** | `MS049-T05` | `docs` | Update `docs/spec/tools/project/manifest.md`, `docs/guide/02_project_structure.md`, and wiki entries. | Documentation sync |

---

## 3. Definition of Done (DoD)

1. `manifest.toml` supports `[profile.<name>]` with `inherits` and granular compilation options.
2. User-defined custom profiles resolve inheritance chains and correctly override parent settings.
3. Dependencies can be declared with target profiles (`name = { version = "...", profile = "..." }` or `name-profile = "..."`).
4. Unit and integration tests verify manifest deserialization, profile inheritance, and option propagation.
5. All tests pass with zero single-letter variables (GL-0011).
6. `python3 scripts/verify_docs.py` and `python3 scripts/doc_sync_check.py` pass with 0 defects.
