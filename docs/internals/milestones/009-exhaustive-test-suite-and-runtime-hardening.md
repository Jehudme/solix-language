# Milestone 009: Exhaustive Language Test Suite, Runtime Error Codes, and Native Integration Hardening

## Document Metadata
* **Milestone ID:** MS-009
* **Target Version:** `v0.1.0-alpha.9`
* **Milestone Branch:** `milestone/009-exhaustive-test-suite-and-runtime-hardening`
* **Status:** COMPLETED
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-11-16
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/milestones/006-register-bytecode-emitter-and-binary-container.md` (MS-006)
  * `docs/internals/milestones/007-advanced-language-features-and-emitter-completion.md` (MS-007)
  * `docs/internals/milestones/008-register-vm-runtime-and-memory-system.md` (MS-008)

---

## 1. Executive Summary & Purpose

Following the successful completion of the register bytecode emitter (Milestone 006), language feature completion (Milestone 007), and the register VM runtime with virtual memory (Milestone 008), the compiler and virtual machine subsystems must be hardened against real-world, exhaustive language workloads.

The purpose of **Milestone 009** is to construct an exhaustive test suite matching and exceeding the comprehensive 368-case specification from `TESTS.md` in the reference codebase, implement structured numeric runtime error codes with graceful and abrupt process exit codes, enforce canonical entry point signatures (`main()` / `main(char[][] args)` returning `void` / `int32`), enable native host function registration directly via `RuntimeOptions`, and build a monolithic master integration test fixture verifying 100% of Solix language features working together end-to-end.

Milestone 009 is organized across four foundational pillars:

1. **Structured Runtime Error Codes & Abrupt Termination Protocol:**
   - Define a strongly typed `RuntimeErrorCode` enum covering all VM failure modes (`OutOfMemory`, `StackOverflow`, `DivideByZero`, `NullReferenceAccess`, `IndexOutOfBounds`, `InvalidCast`, `TypeMismatch`, `AssertionFailed`, `MemoryLeak`, `InvalidEntryPointSignature`, etc.).
   - Standardize VM execution results: normal completion returns the exit value of `main()` (or `0` for `void`), while abrupt termination returns the specific numeric `RuntimeErrorCode` with full diagnostic error details.

2. **Entry Point Signature Enforcement & CLI Argument Delivery:**
   - Validate that entry points conform strictly to canonical signatures:
     - `public static void main()` $\to$ executes and automatically returns `0`.
     - `public static int32 main()` $\to$ executes and returns user exit code.
     - `public static void main(char[][] args)` or `public static void main(string[] args)` $\to$ passes CLI argument string array.
     - `public static int32 main(char[][] args)` or `public static int32 main(string[] args)` $\to$ passes CLI argument string array and returns user exit code.
   - Rejects non-canonical entry point signatures with `RuntimeErrorCode::InvalidEntryPointSignature`.

3. **`RuntimeOptions` Native Function Registration & Verification Tracing:**
   - Allow host applications and tests to register custom native callbacks directly onto `RuntimeOptions` before VM instantiation.
   - Wire custom native callback sinks (e.g. `println(char[])` / `println(string)`) to capture runtime output logs during integration test execution.

4. **Exhaustive Language Test Suites & Mega-Integration Fixture:**
   - Cross-crate test integration: `solix-compilation` imports `solix-runtime` as dev-dependency to compile and execute test programs directly.
   - Comprehensive individual test suites covering every language feature from `TESTS.md`: generics/templates, class inheritance, abstract classes, interfaces, enums, operator overloads, constructors, destructors, lambdas, function pointers, ternary expressions, sizeof, instanceof, tuple instanceof, arrays, tuples, try/catch/finally, exceptions.
   - A monumental master integration test fixture (`master_all_features_exhaustive.slx`) compiling and executing every single language keyword and construct in a unified program with live `println` trace assertions and zero memory leaks.

---

## 2. Architecture & Design Specification

### 2.1 Runtime Error Codes (`RuntimeErrorCode`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum RuntimeErrorCode {
    Success = 0,
    OutOfMemory = 1,
    StackOverflow = 2,
    DivideByZero = 3,
    IndexOutOfBounds = 4,
    NullReferenceAccess = 5,
    InvalidCast = 6,
    FunctionNotFound = 7,
    EntryPointNotFound = 8,
    InvalidInstruction = 9,
    InvalidOpCode = 10,
    InvalidFieldOffset = 11,
    InvalidStaticOffset = 12,
    BinaryLoadError = 13,
    UserException = 14,
    NativeFunctionError = 15,
    SecurityViolation = 16,
    TypeMismatch = 17,
    AssertionFailed = 18,
    MemoryLeak = 19,
    InvalidEntryPointSignature = 20,
}
```

### 2.2 Entry Point Signature Grammar

| Signature Form | Parameters | Return Type | Runtime Behavior |
| :--- | :---: | :---: | :--- |
| `void main()` | 0 | `void` | Executes with 0 args, returns `0` automatically |
| `int32 main()` | 0 | `int32` | Executes with 0 args, returns evaluated integer |
| `void main(char[][] args)` | 1 | `void` | Allocates `ArrayRef` of `String`s from CLI args, returns `0` |
| `int32 main(char[][] args)` | 1 | `int32` | Allocates `ArrayRef` of `String`s from CLI args, returns integer |
| *Other* | Any | Any | Throws `RuntimeError::InvalidEntryPointSignature` |

---

## 3. Prioritized Deliverables & Implementation Roadmap

### Phase 1: Structured Runtime Error Codes & Abrupt Termination Protocol
- [x] `[P0]` **Task 1.1:** Define `RuntimeErrorCode` enum with fixed discriminants in `crates/solix-runtime/src/exceptions/errors.rs`.
- [x] `[P0]` **Task 1.2:** Implement `RuntimeError::error_code(&self) -> RuntimeErrorCode` and `as_i32()` conversion methods.
- [x] `[P0]` **Task 1.3:** Update `VirtualMachine::run()` to capture abrupt error codes into `vm.exit_code`.
- [x] `[P0]` **Task 1.4:** Write unit tests for all `RuntimeErrorCode` conversions and abrupt exit handling.

### Phase 2: Entry Point Signature Validation & Argument Binding
- [x] `[P0]` **Task 2.1:** Implement signature inspector validating parameter count (0 or 1) and return type (`void` or `int32`).
- [x] `[P0]` **Task 2.2:** Implement automatic `0` exit code synthesis for `void` entry points when returning `Value::Null`.
- [x] `[P0]` **Task 2.3:** Implement CLI arguments array packing: convert `options.args` into heap `ArrayRef` of `String`s and pass as register argument when parameter count is 1.
- [x] `[P0]` **Task 2.4:** Write unit tests for `void` main, integer main, string arguments main, and invalid signature rejection.

### Phase 3: `RuntimeOptions` Native Function Registration & Execution
- [x] `[P0]` **Task 3.1:** Add `with_native_function()` and `native_functions` map to `RuntimeOptions`.
- [x] `[P0]` **Task 3.2:** Wire `VirtualMachine::new()` to auto-register all native callbacks from `RuntimeOptions` into `NativeRegistry`.
- [x] `[P0]` **Task 3.3:** Implement native `println` string capturing callback utility for integration test verification.
- [x] `[P0]` **Task 3.4:** Write unit tests verifying native registration via `RuntimeOptions` and output trace capture.

### Phase 4: Cross-Crate Test Infrastructure & Test Harness
- [x] `[P0]` **Task 4.1:** Add `solix-runtime.workspace = true` to `crates/solix-compilation/Cargo.toml` under `[dev-dependencies]`.
- [x] `[P0]` **Task 4.2:** Build reusable test runner helper compiling source text and executing directly in `VirtualMachine`.

### Phase 5: Comprehensive Individual Feature Test Suites
- [x] `[P0]` **Task 5.1:** Implement test suites for Generics & Templates (`Box<T>`, `Pair<K, V>`, generic methods, interface bounds).
- [x] `[P0]` **Task 5.2:** Implement test suites for OOP features: single & multi-level inheritance, abstract classes, interfaces, method overriding, super calls, access modifiers (`public`, `protected`, `private`).
- [x] `[P0]` **Task 5.3:** Implement test suites for Constructors, Destructors, and ARC lifetime.
- [x] `[P0]` **Task 5.4:** Implement test suites for Operator Overloads (`+`, `-`, `*`, `==`, `!=`, `<`, `>`).
- [x] `[P0]` **Task 5.5:** Implement test suites for Control Flow: if-else, while, do-while, for, switch, try-catch-finally, throw.
- [x] `[P0]` **Task 5.6:** Implement test suites for Expressions: ternary (`? :`), sizeof, instanceof, tuple instanceof, lambdas with captures, function pointers, arrays, tuples.

### Phase 6: Master Mega-Integration Test Fixture
- [x] `[P0]` **Task 6.1:** Author comprehensive `master_all_features_exhaustive.slx` exercising all language keywords, declarations, expressions, and statements.
- [x] `[P0]` **Task 6.2:** Wire integration test to capture live `println` log output and assert exact expected trace.
- [x] `[P0]` **Task 6.3:** Verify clean memory deallocation (`verify_clean_memory() == Ok(())`) upon completion of the master fixture.

### Phase 7: Full Verification, Documentation Sync, and Master Merge
- [x] `[P0]` **Task 7.1:** Full workspace build and test pass: `cargo test --workspace --all-targets`.
- [x] `[P0]` **Task 7.2:** Full clippy check with `-D warnings`: `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] `[P0]` **Task 7.3:** Full rustfmt formatting: `cargo fmt --check`.
- [x] `[P0]` **Task 7.4:** Complete DoD Sign-Off matrix and update milestone status to `COMPLETED`.
- [x] `[P0]` **Task 7.5:** Merge `milestone/009-exhaustive-test-suite-and-runtime-hardening` into `master` via non-fast-forward merge (`--no-ff`).

---

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Command | Sign-Off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Zero warnings, `#![forbid(unsafe_code)]` | `cargo build --workspace --all-targets` | [x] Complete (Verified) |
| **Automated Test Suite** | 100% test pass across all units and integration fixtures | `cargo test --workspace --all-targets` | [x] Complete (Verified) |
| **Clippy Static Analysis** | Zero warnings with `-D warnings` | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Complete (Verified) |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [x] Complete (Verified) |
| **Documentation Sync** | Milestone specifications and architecture documents updated | GL-0010 audit checklist | [x] Complete (Verified) |

---

## 5. Risk Assessment & Engineering Mitigations

1. **Risk: Cyclic Crate Dependencies between `solix-compilation` and `solix-runtime`**
   - *Mitigation:* `solix-runtime` is added exclusively as a `[dev-dependencies]` entry in `solix-compilation/Cargo.toml`. Cargo allows cyclic dev-dependencies between workspace members for integration tests without compromising release compilation topology.

2. **Risk: Stack Overflow or Memory Exhaustion in Monumental Master Fixture**
   - *Mitigation:* The master fixture is cleanly structured with modular test methods and explicit scopes, allowing ARC to collect temporary allocations progressively throughout execution.
