# Milestone 050: Solix-FFI Native Extension SDK, Scaffolding & Crates.io Publication

## Document Metadata
* **Milestone ID:** MS-050
* **Target Version:** `v1.1.4`
* **Milestone Branch:** `milestone/050-solix-ffi-native-extension-sdk-scaffolding-and-crates-io-publication`
* **Status:** ACTIVE
* **Lead Architect:** Core Systems & Native Interoperability Team
* **Target Delivery Date:** 2026-11-15
* **Related Specifications:** [GL-0001](../../internals/guidelines/authoring-commits.md), [GL-0008](../../internals/guidelines/authoring-milestones.md), [GL-0011](../../internals/guidelines/language-conventions.md), [GL-0013](../../internals/guidelines/documentation-update-protocol.md), [SPEC-0001](../../spec/README.md)

---

## 1. Executive Summary & Objectives

Writing high-performance native extensions for Solix currently requires manually replicating low-level C-ABI wrappers, argument unpacking, and VM callback collectors across every native library. Furthermore, crates in the Solix workspace should be structured and packaged cleanly for publication to crates.io:

1. **`solix-ffi` Crate**:
   - Create a dedicated, safe abstraction crate `crates/solix-ffi` providing reusable helpers for native extension authors:
     - Macros for defining and exporting native Solix functions (`solix_fn!`).
     - Safe unpacking and packing helpers for numbers, strings, structs, and arrays.
     - Collector callback registration helpers for linking dynamic libraries with the Solix VM.
2. **`solix new --native` Scaffolding**:
   - Update `solix new` to accept a `--native` flag (or native template option) that generates a ready-to-build project structure with a `natives/` Cargo crate configured to depend on `solix-ffi`.
3. **Crates.io Publication Readiness**:
   - Configure workspace package manifests (`Cargo.toml`) with publication metadata (descriptions, documentation URLs, repository links, license specifications, and keywords).
   - Verify that crates can be published independently (`solix-ffi`, `solix-common`, `solix-compilation`, `solix-runtime`) or via an umbrella crate configuration.
   - Configure GitHub Actions release automation with the `CARGO_REGISTRY_TOKEN` secret to publish crates to crates.io on releases.

---

## 2. Priority Blocker Taxonomy & Deliverables

| Priority | Task ID | Subsystem | Description | Release Impact |
|:---|:---|:---|:---|:---|
| **P0** | `MS050-T01` | `solix-ffi` | Create `crates/solix-ffi` crate with safe FFI bindings, macros, and value marshaling helpers. | Native extension SDK |
| **P1** | `MS050-T02` | `solix-toolchain` | Update `solix new` to support `--native` scaffolding with pre-configured `natives/` Cargo setup. | Developer onboarding |
| **P1** | `MS050-T03` | `workspace` | Configure `Cargo.toml` manifests with publication metadata and publish targets. | Crates.io readiness |
| **P2** | `MS050-T04` | `ci/cd` | Verify automated publication in release workflow using `CARGO_REGISTRY_TOKEN`. | Release automation |
| **P2** | `MS050-T05` | `docs` | Document native extension authoring guide in `docs/guide/native_extensions.md`. | Documentation sync |

---

## 3. Definition of Done (DoD)

1. `crates/solix-ffi` compiles cleanly with `#![forbid(unsafe_code)]` in safe modules and verified `unsafe` FFI boundaries.
2. `solix new <name> --native` creates a project with functional `natives/` Cargo integration.
3. Crates have complete `Cargo.toml` publication metadata.
4. All unit and integration tests pass with zero single-letter variables (GL-0011).
5. `python3 scripts/verify_docs.py` and `python3 scripts/doc_sync_check.py` pass with 0 defects.
