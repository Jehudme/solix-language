# Milestone 011: Multi-Binary Support, Manifest Metadata Extraction, and Dynamic Linking

## Document Metadata
* **Milestone ID:** MS-011
* **Target Version:** `v0.1.0-alpha.11`
* **Milestone Branch:** `milestone/011-multi-binary-support-and-dynamic-linking`
* **Status:** COMPLETED
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-11-30
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/milestones/008-register-vm-runtime-and-memory-system.md` (MS-008)
  * `docs/internals/milestones/010-dedicated-conformance-test-suite-and-migration.md` (MS-010)

---

## 1. Executive Summary & Purpose

Modern modular programming in Solix requires the ability to compile libraries into independent binary containers (`.slxb`) and link them dynamically during runtime execution. This milestone implements the complete end-to-end multi-binary dynamic linking pipeline:

1. **Self-Describing Binary Metadata (`solix-common`)**:
   - Storing package-level metadata (`package_name`, `package_version`, `package_description`) inside `BinaryProgram`.
   - Recording complete export manifests (`ExportManifest` in binary section 8) containing all declaration modifiers (`public`, `protected`, `private`, `static`, `const`, `virtual`, `override`, `abstract`, `weak`, `inline`), parameter names & types (`Type`), return types, static variable slots & types, field offsets & types, class hierarchies, and uninstantiated generic template sources.
   - Structured JSON API extraction function `extract_api_manifest(&BinaryProgram) -> Result<String, serde_json::Error>` in `solix-common`.

2. **Compiler `extern` Language Support & API Source Transformation (`solix-compilation`)**:
   - Adding the `extern` keyword and modifier across the compiler pipeline (lexer, parser, binder, and emitter).
   - Strict negative diagnostic enforcement with new diagnostic error codes (`E0137`, `E0138`, `E0139`, `E0140`).
   - Transforming API manifests into compilable Solix source code via `manifest_to_source(manifest_json: &str) -> Result<(String, String), DiagnosticError>` using the `<name>@<version>` source naming convention (e.g., `math@1.0.0.slx`).
   - Supporting user-authored `extern` interface declarations as well as auto-generated ones.
   - Preserving generic template sources in the manifest so downstream consumers can monomorphize new generic specializations (e.g. `Box<CustomType>`).

3. **Dynamic Linking, Multi-Binary Relocation & In-Place Quickening (`solix-runtime`)**:
   - Introducing dynamic linking opcodes: `CallExtern` (0x64), `SetStaticGlobal` (0x77), and `AllocObjectExtern` (0x87).
   - Fixing multi-binary relocation in `RuntimeContext::new` by tracking `vtable_id_offset` and relocating `vtable_id`, `base_vtable_id`, `class.vtable`, `class.itable`, and `class.destructor_id`.
   - Implementing in-place instruction quickening via `context.patch_instruction(...)`: patching `CallExtern` to `CallDirect`, `GetStaticGlobal` to `GetStaticDirect`, `SetStaticGlobal` to `SetStaticDirect`, and `AllocObjectExtern` to `AllocObject` on initial dispatch.

4. **Exhaustive Conformance & Test Suite Updates (`solix-tests`)**:
   - Full negative error code coverage for `extern` syntax and binding errors.
   - End-to-end multi-binary dynamic linking execution tests verifying zero memory leaks (`vm.verify_clean_memory() == Ok(())`) and instruction quickening verification.

---

## 2. Architecture & Design Specification

### 2.1 Binary Container & Export Manifest (`solix-common`)

Section 8 in `BinaryProgram` stores `ExportManifest`:
```rust
pub struct ExportManifest {
    pub package_name: String,
    pub package_version: String,
    pub package_description: String,
    pub exported_classes: Vec<ExportedClass>,
    pub exported_functions: Vec<ExportedFunction>,
    pub exported_statics: Vec<ExportedStatic>,
    pub generic_templates: Vec<ExportedGenericTemplate>,
}
```

Modifiers are preserved via `DeclarationModifier`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeclarationModifier {
    Public,
    Protected,
    Private,
    Static,
    Const,
    Virtual,
    Override,
    Abstract,
    Weak,
    Inline,
}
```

### 2.2 Compiler `extern` Support (`solix-compilation`)

- `extern` can qualify functions, methods, constructors, destructors, classes, and global/static variables.
- Function/method marked `extern` must end with a semicolon `;` and cannot have a body `{ ... }` (Error `E0137`).
- Variable marked `extern` cannot have an initializer `= ...` (Error `E0138`).
- `extern` cannot appear inside a local function scope or block (Error `E0139`).
- `extern` cannot be combined with conflicting modifiers: `native`, `inline`, or `abstract` (Error `E0140`).
- The pipeline skips allocating local static slots for `extern` variables, local function IDs for `extern` functions, and local `compiled_classes` for `extern` classes.
- Calls to `extern` functions emit `OpCode::CallExtern`.
- Access to `extern` statics emit `OpCode::GetStaticGlobal` and `OpCode::SetStaticGlobal`.
- Instantiation of `extern` classes (`new Class()`) emits `OpCode::AllocObjectExtern`.

### 2.3 Runtime Relocation & Quickening (`solix-runtime`)

- `RuntimeContext::new` accepts primary and secondary binaries:
  - Maps global function IDs by offsetting local function IDs with `func_offset`.
  - Relocates `class.vtable_id` and `class.base_vtable_id` with `vtable_id_offset`.
  - Relocates all indices in `class.vtable`, `class.itable`, and `class.destructor_id`.
  - Tracks symbol-to-vtable mapping `symbol_to_vtable_id: HashMap<String, (u32, usize)>`.
- Dynamic instruction quickening:
  - `CallExtern` -> `CallDirect` (patches caller instruction at runtime, replacing with target `global_func_id`).
  - `GetStaticGlobal` -> `GetStaticDirect` (patches caller instruction with `global_slot`).
  - `SetStaticGlobal` -> `SetStaticDirect` (patches caller instruction with `global_slot`).
  - `AllocObjectExtern` -> `AllocObject` (patches caller instruction with `vtable_id` and `instance_size`).

---

## 3. Definition of Done (DoD) Matrix

| Requirement | Acceptance Criteria | Verification Method |
| :--- | :--- | :--- |
| **Manifest & Extraction** | `extract_api_manifest` produces JSON representation of all exports and template sources. | Unit tests in `solix-common` |
| **Compiler `extern`** | Lexing, parsing, and binding of `extern` declarations; strict diagnostic error codes `E0137`-`E0140`. | Conformance tests in `solix-tests` |
| **Manifest-to-Source** | `manifest_to_source` converts JSON manifest into compilable `<name>@<version>.slx` source string. | Integration tests in `solix-compilation` & `solix-tests` |
| **Multi-Binary Relocation** | VTables, ITables, static slots, and function IDs relocated cleanly across secondary binaries. | Runtime unit tests in `solix-runtime` |
| **Dynamic Linking Quickening** | `CallExtern`, `GetStaticGlobal`, `SetStaticGlobal`, `AllocObjectExtern` execute and patch instructions in-place to direct forms. | End-to-end integration tests in `solix-tests` |
| **Zero Memory Leaks** | `vm.verify_clean_memory()` returns `Ok(())` on all multi-binary executions. | Conformance assertion across all tests |
| **Test Specification** | `crates/solix-tests/TESTS.md` updated with exhaustive positive & negative test cases. | Documentation review |
| **Zero Warnings / Forbid Unsafe** | `#![forbid(unsafe_code)]`, `cargo clippy -- -D warnings`, `cargo fmt --check`. | Automated workspace verification |

---

## 4. Phased Roadmap

* [x] **Step 0: Specification & Milestone Inception**
* [x] **Phase 1: Manifest Schema, Serialization & API JSON Extraction (`solix-common`)**
* [x] **Phase 2: Compiler `extern` Support, Manifest Emission & Source Generation (`solix-compilation`)**
* [x] **Phase 3: Multi-Binary Relocation, Dynamic Linking & Instruction Quickening (`solix-runtime`)**
* [x] **Phase 4: Conformance Test Suite, `TESTS.md` Expansion & Verification (`solix-tests`)**
