# Milestone 010: Dedicated Conformance Test Suite Project and Full Specification Migration

## Document Metadata
* **Milestone ID:** MS-010
* **Target Version:** `v0.1.0-alpha.10`
* **Milestone Branch:** `milestone/010-dedicated-conformance-test-suite-and-migration`
* **Status:** COMPLETED
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-11-23
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/milestones/008-register-vm-runtime-and-memory-system.md` (MS-008)
  * `docs/internals/milestones/009-exhaustive-test-suite-and-runtime-hardening.md` (MS-009)

---

## 1. Executive Summary & Purpose

In Milestone 009, reciprocal `[dev-dependencies]` were introduced between `solix-compilation` and `solix-runtime` to allow integration testing of emitted bytecode against the virtual machine. While functional, co-locating end-to-end multi-hundred-case integration suites directly inside library crates causes tight compilation coupling, increases build times, and splits language verification across multiple disparate crates.

The purpose of **Milestone 010** is to architect a dedicated, standalone integration and conformance testing project: **`crates/solix-tests`**. This crate centralizes 100% of the 300+ test specifications from `TESTS.md` in the reference Solix codebase, organized into structured subdirectories matching the original language specification:

1. **`compilation/`**: End-to-end compiler pipeline and execution suites. Every language feature (declarations, expressions, statements, modules, generics, variadics, tuples, closures, exceptions, etc.) is compiled, executed in the VM, checked for expected output and exit codes, and verified for clean heap deallocation (`vm.verify_clean_memory() == Ok(())`).
2. **`runtime/`**: Dedicated virtual machine and memory subsystem invariant suites (virtual heap allocation, block headers, free-list coalescing, dynamic VTables/ITables, error code mappings, abrupt exit codes, canonical entry point signatures, and native FFI callbacks).
3. **Negative Test Error Code Assertions**: All negative tests explicitly document and assert their compiler error codes (e.g. `E0136`, `E0204`, etc.) or runtime error codes (`RuntimeErrorCode`).
4. **Decoupled Architecture & Workspace Cleanup**: Library crates (`solix-compilation` and `solix-runtime`) retain only unit-level tests for their internal modules, removing reciprocal `[dev-dependencies]` and leaving a clean, decoupled workspace topology.

---

## 2. Architecture & Design Specification

### 2.1 Workspace Layout & Crate Hierarchy

```
crates/
├── solix-common/          # Fundamental AST, Bytecode ISA, Token, Diagnostics
├── solix-compilation/     # Lexer, Parser, Binder, Emitter (unit tests only)
├── solix-runtime/         # VirtualHeap, ARC, Dispatch, Registry (unit tests only)
├── solix-client/          # CLI Driver & Packaging
├── solix-lsp/             # Language Server Protocol Daemon
├── solix-dbg/             # Debug Adapter Protocol Engine
└── solix-tests/           # Dedicated Conformance & End-to-End Test Suite
    ├── Cargo.toml         # Depends on solix-common, solix-compilation, solix-runtime
    ├── TESTS.md           # Canonical specification copied from reference codebase
    └── tests/
        ├── common/        # Test runner helpers (compile_and_run, clean memory checks)
        ├── compilation/   # End-to-end language conformance test suites
        │   ├── modules/
        │   ├── declarations/
        │   ├── control_flow/
        │   ├── expressions/
        │   └── integration/
        └── runtime/       # Virtual machine and memory subsystem invariants
            ├── test_virtual_heap.rs
            ├── test_object_model.rs
            ├── test_error_codes_and_entry_point.rs
            ├── test_exceptions.rs
            ├── test_call_and_returns.rs
            ├── test_closures_lambdas.rs
            ├── test_options_and_context.rs
            └── test_native_ffi.rs
```

### 2.2 Test Harness Abstractions (`crates/solix-tests/tests/common/mod.rs`)

```rust
pub fn compile_and_run(source: &str) -> (Value, VirtualMachine) {
    // 1. Lex & Parse
    // 2. Semantic Bind
    // 3. Emit Bytecode
    // 4. Instantiate VM with RuntimeOptions
    // 5. Execute entry point
    // 6. Assert verify_clean_memory() == Ok(())
    // 7. Return (result, vm)
}

pub fn compile_and_run_multi(sources: &[(&str, &str)]) -> (Value, VirtualMachine);

pub fn assert_compile_error(source: &str, expected_code: &str);

pub fn assert_runtime_error(source: &str, expected_code: RuntimeErrorCode);
```

---

## 3. Prioritized Deliverables & Implementation Roadmap

### Phase 1: Test Crate Scaffolding & Infrastructure
- [x] `[P0]` **Task 1.1:** Add `crates/solix-tests` to workspace members in root `Cargo.toml`.
- [x] `[P0]` **Task 1.2:** Create `crates/solix-tests/Cargo.toml` with dependencies on `solix-common`, `solix-compilation`, `solix-runtime`, `solix-client`, `tempfile`.
- [x] `[P0]` **Task 1.3:** Copy canonical `TESTS.md` from `/home/jehud/Projects/solix/tests/statements/TESTS.md` into `crates/solix-tests/TESTS.md`.
- [x] `[P0]` **Task 1.4:** Implement `crates/solix-tests/tests/common/mod.rs` test harness with automatic memory leak verification and diagnostic error assertions.

### Phase 2: Runtime Invariant Suites Migration
- [x] `[P0]` **Task 2.1:** Migrate VirtualHeap tests (headers, allocation, free-list coalescing) to `tests/runtime/test_virtual_heap.rs`.
- [x] `[P0]` **Task 2.2:** Migrate ObjectModel tests (vtables, itables, weak refs, instanceof) to `tests/runtime/test_object_model.rs`.
- [x] `[P0]` **Task 2.3:** Migrate ErrorCodes & EntryPoint tests (signature validation, args array, stdout) to `tests/runtime/test_error_codes_and_entry_point.rs`.
- [x] `[P0]` **Task 2.4:** Migrate Exceptions & Unwinding tests to `tests/runtime/test_exceptions.rs`.
- [x] `[P0]` **Task 2.5:** Migrate Calls, Recursion & Relocation tests to `tests/runtime/test_call_and_returns.rs`.
- [x] `[P0]` **Task 2.6:** Migrate Closures & Lambda unpacking tests to `tests/runtime/test_closures_lambdas.rs`.
- [x] `[P0]` **Task 2.7:** Migrate Options & Multi-binary Context tests to `tests/runtime/test_options_and_context.rs`.
- [x] `[P0]` **Task 2.8:** Migrate Native FFI tests to `tests/runtime/test_native_ffi.rs`.

### Phase 3: Compilation Modules & Declarations Suites
- [x] `[P0]` **Task 3.1:** Implement `tests/compilation/modules/`: `test_package_statement.rs`, `test_alias_statement.rs`, `test_import_statement.rs` (with multi-file cross-module imports).
- [x] `[P0]` **Task 3.2:** Implement `tests/compilation/declarations/`: `test_class_declaration.rs`, `test_constructor_declaration.rs`, `test_destructor_declaration.rs`.
- [x] `[P0]` **Task 3.3:** Implement `tests/compilation/declarations/`: `test_enum_declaration.rs`, `test_field_declaration.rs`, `test_interface_declaration.rs`, `test_method_declaration.rs`, `test_operator_declaration.rs`.
- [x] `[P0]` **Task 3.4:** Implement `tests/compilation/declarations/test_generics.rs` (single & multi-parameter generic classes, interface bounds, nested specialization).
- [x] `[P0]` **Task 3.5:** Implement `tests/compilation/declarations/test_tuple_primitive.rs` (tuple primitive, element indexing, tuple element instanceof, and variadic template parameter packs `<args...>` and `args...`).

### Phase 4: Compilation Control Flow & Expressions Suites
- [x] `[P0]` **Task 4.1:** Implement `tests/compilation/control_flow/`: `test_block_statement.rs`, `test_break_statement.rs`, `test_continue_statement.rs`, `test_do_while_statement.rs`, `test_expression_statement.rs`.
- [x] `[P0]` **Task 4.2:** Implement `tests/compilation/control_flow/`: `test_for_statement.rs`, `test_if_statement.rs`, `test_return_statement.rs`, `test_switch_statement.rs`, `test_throw_statement.rs`, `test_try_catch_finally_statement.rs`, `test_variable_declaration_statement.rs`, `test_while_statement.rs`.
- [x] `[P0]` **Task 4.3:** Implement `tests/compilation/control_flow/`: `test_diagnostics_consistency.rs`, `test_call_frame_allocation.rs`.
- [x] `[P0]` **Task 4.4:** Implement `tests/compilation/expressions/`: `test_array_access_expression.rs`, `test_array_creation_expression.rs`, `test_array_literal_expression.rs`, `test_assignment_expression.rs`, `test_binary_expression.rs`, `test_cast_expression.rs`.
- [x] `[P0]` **Task 4.5:** Implement `tests/compilation/expressions/`: `test_default_expression.rs`, `test_function_pointers.rs`, `test_identifier_node.rs`, `test_instanceof_expression.rs`, `test_intrinsics.rs`, `test_lambdas.rs`, `test_literal_expression.rs`.
- [x] `[P0]` **Task 4.6:** Implement `tests/compilation/expressions/`: `test_member_access_expression.rs`, `test_method_call_expression.rs`, `test_new_instance_expression.rs`, `test_sizeof_expression.rs`, `test_ternary_expression.rs`, `test_unary_expression.rs`.
- [x] `[P0]` **Task 4.7:** Implement `tests/compilation/integration/master_all_features.rs` executing the master all-features exhaustive fixture.

### Phase 5: Variadic Generics `<args...>` Enhancements (if required)
- [x] `[P0]` **Task 5.1:** Verify and support variadic template parameter pack `<args...>` in generic parser.
- [x] `[P0]` **Task 5.2:** Verify parameter list lowering of `args...` to tuple and negative validation (error `E0136`).

### Phase 6: Decouple Library Crates & Clean Up Redundant Tests
- [x] `[P0]` **Task 6.1:** Remove reciprocal dev-dependencies between `solix-compilation` and `solix-runtime`.
- [x] `[P0]` **Task 6.2:** Remove duplicated integration/spec tests from `crates/solix-compilation/tests` and `crates/solix-runtime/tests`.
- [x] `[P0]` **Task 6.3:** Verify clean separation: `solix-compilation` tests only compiler modules; `solix-runtime` tests only runtime modules; `solix-tests` tests the entire end-to-end language.

### Phase 7: Verification, Documentation Sync, and Master Merge
- [x] `[P0]` **Task 7.1:** Full workspace build and test pass: `cargo test --workspace --all-targets`.
- [x] `[P0]` **Task 7.2:** Full clippy check with `-D warnings`: `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] `[P0]` **Task 7.3:** Full rustfmt formatting: `cargo fmt --check`.
- [x] `[P0]` **Task 7.4:** Complete DoD Sign-Off matrix and update milestone status to `COMPLETED`.
- [x] `[P0]` **Task 7.5:** Merge `milestone/010-dedicated-conformance-test-suite-and-migration` into `master` via non-fast-forward merge (`--no-ff`).

---

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Command | Sign-Off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Zero warnings, `#![forbid(unsafe_code)]` | `cargo build --workspace --all-targets` | [x] Passed |
| **Automated Test Suite** | 100% test pass across all units and integration fixtures | `cargo test --workspace --all-targets` | [x] Passed |
| **Clippy Static Analysis** | Zero warnings with `-D warnings` | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Passed |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [x] Passed |
| **Documentation Sync** | Milestone specifications and architecture documents updated | GL-0010 audit checklist | [x] Passed |

---

## 5. Risk Assessment & Engineering Mitigations

1. **Risk: Missing Test Coverage during Migration**
   - *Mitigation:* `crates/solix-tests/TESTS.md` serves as the authoritative source of truth. Every suite in `TESTS.md` corresponds directly to a test file in `tests/compilation/` or `tests/runtime/`.
2. **Risk: Memory Leaks in Conformance Tests**
   - *Mitigation:* The `compile_and_run` harness strictly asserts `vm.verify_clean_memory() == Ok(())` after every test execution, catching any leaks immediately.
