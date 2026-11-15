# Milestone 037: Implicit Numeric Literal Coercion & Safe Widening Casts

## Document Metadata
* **Milestone ID:** MS-037
* **Target Version:** `v0.1.0-rc.2`
* **Milestone Branch:** `milestone/037-implicit-numeric-literal-coercion`
* **Status:** VALIDATED
* **Lead Architect:** @solix-lead
* **Target Delivery Date:** 2026-11-18
* **Actual Delivery Date:** 2026-11-15
* **Related Specifications:**
  * `docs/internals/specs/language/type-system.md`
  * `docs/internals/specs/compilation/semantic-binder.md`
  * `docs/internals/specs/compilation/bytecode-emitter.md`

---

## 1. Executive Summary & Objectives

Implement context-typed numeric literal coercion, compile-time integer literal bounds checking, and lossless numeric widening in `solix-compilation`.

Previously, unsuffixed integer literals always defaulted strictly to `int32` and unsuffixed floating-point literals always defaulted strictly to `float64`. Consequently, initializing or passing `uint8`, `int16`, `uint32`, `int64`, or `float32` values required verbose manual casts on every literal—such as `new Color((uint8)18, (uint8)20, (uint8)30, (uint8)255)` or `new Vector2((float32)100.0, (float32)200.0)`.

This milestone upgrades the semantic binder (`type_resolver.rs` and `pass4_body_binding.rs`) and bytecode emitter (`emitter/`) so unsuffixed integer and floating-point literals coerce implicitly to their expected target numeric type whenever the literal fits within the target type's representable range, and enables safe, lossless numeric widening between expressions without requiring explicit casts.

## 2. Scope & Crates Affected
* `crates/solix-common`: Diagnostic error codes for out-of-range integer literal coercion if new error variants are needed.
* `crates/solix-compilation`:
  * `src/phases/binder/type_resolver.rs`: Lossless numeric widening rules and literal bounds validation helpers.
  * `src/phases/binder/passes/pass4_body_binding.rs`: Target-type propagation into declarations, assignments, call/constructor arguments, return statements, array literals, and binary expressions; recording coerced types in `semantic_model.node_types`.
  * `src/phases/emitter/`: Emitting properly typed constants or `ConvIToF` / bit-width conversions for coerced literals and widened expressions.
* `crates/solix-runtime`: Not affected.
* `crates/solix-toolchain`: Not affected.
* `crates/solix-lsp`: Inherits updated binder semantics automatically (eliminating false-positive type mismatch diagnostics on uncast literals).
* `crates/solix-dap`: Not affected.
* `examples/breakout`: Remove redundant `(uint8)` and `(float32)` literal casts from `src/main.slx`.
* `docs/`: Update language type-system specification and guides.

## 3. Prioritized Deliverables & Task Matrix

### 3.1 P0: Critical Blockers (Release Gate)
- [x] `[P0]` **Context-Typed Integer Literal Coercion & Range Validation (`type_resolver.rs`, `pass4_body_binding.rs`):** Allow unsuffixed integer literals (`LiteralValue::Int(val)`) and negated integer literals (`-LiteralValue::Int(val)`) to coerce implicitly to any target integer primitive (`int8`, `uint8`, `int16`, `uint16`, `int32`, `uint32`, `int64`, `uint64`) or floating-point primitive (`float32`, `float64`) when `val` fits within the target integer type's exact bounds:
  * `int8`: $[-128, 127]$
  * `uint8`: $[0, 255]$
  * `int16`: $[-32768, 32767]$
  * `uint16`: $[0, 65535]$
  * `int32`: $[-2147483648, 2147483647]$
  * `uint32`: $[0, 4294967295]$
  * `int64` / `uint64`: 64-bit representable bounds
  Emit a deterministic compile-time semantic error when an integer literal overflows or underflows the target type (e.g., `uint8 r = 300;` or `uint8 neg = -1;`).
- [x] `[P0]` **Context-Typed Float Literal Coercion (`type_resolver.rs`, `pass4_body_binding.rs`):** Allow unsuffixed floating-point literals (`LiteralValue::Float(val)`) and negated float literals (`-LiteralValue::Float(val)`) to coerce implicitly to `float32` whenever the expected target type is `float32` (while still defaulting to `float64` when unconstrained).
- [x] `[P0]` **Emitter Coercion & `node_types` Synchronization (`pass4_body_binding.rs`, `emitter/`):** When a literal or expression is coerced/widened to a target type (e.g., `LiteralValue::Int(100)` coerced to `float32`, `float64`, `int64`, or `uint8`), update its resolved type in `semantic_model.node_types` (or emit `ConvIToF` / target constant encoding in `emitter`) so VM registers and heap field/array writes receive the exact expected runtime `Value` (`Value::F64` for `float32`/`float64`, `Value::I64` for `int64`/`uint64`, `Value::I32` for `int8..uint32`).

### 3.2 P1: High Priority (Milestone Core)
- [x] `[P1]` **Safe Lossless Numeric Widening (`type_resolver.rs`):** Update `TypeResolver::is_assignable_to` to permit implicit lossless widening between non-literal expressions:
  * Signed integer chain: $\text{int8} \to \text{int16} \to \text{int32} \to \text{int64}$
  * Unsigned integer chain: $\text{uint8} \to \text{uint16} \to \text{uint32} \to \text{uint64}$
  * Unsigned-to-larger-signed chain: $\text{uint8} \to \text{int16} \mid \text{int32} \mid \text{int64}$; $\text{uint16} \to \text{int32} \mid \text{int64}$; $\text{uint32} \to \text{int64}$
  * Float widening: $\text{float32} \to \text{float64}$
  * Integer-to-float widening: $\text{int8} \mid \text{uint8} \mid \text{int16} \mid \text{uint16} \to \text{float32} \mid \text{float64}$; $\text{int32} \mid \text{uint32} \to \text{float64}$
  Preserve strict compile-time rejection for narrowing conversions on non-literal variables (e.g., `float64` variable assigned to `float32` variable without an explicit `(float32)` cast).
- [x] `[P1]` **Comprehensive Target-Type Context Propagation (`pass4_body_binding.rs`):** Propagate expected target types across all syntactic contexts:
  1. Local and static variable declarations (`uint8 r = 255;`, `float32 x = 10.5;`, `int64 total = 0;`).
  2. Variable and object field assignments (`pos.x = 100.0;`, `color.a = 255;`).
  3. Function, method, and constructor call arguments (`new Color(18, 20, 30, 255)`, `new Vector2(100.0, 200.0)`).
  4. Return statements (`return 0;` inside `int64`, `uint32`, or `float32` functions).
  5. Array literals and indexed element assignments (`float32[] arr = [1.0, 2.5, 3.0];`, `arr[0] = 4.5;`).
  6. Binary arithmetic and comparison expressions (`+`, `-`, `*`, `/`, `%`, `<`, `<=`, `>`, `>=`, `==`, `!=`) where one operand is a typed variable (e.g., `float32` or `uint8` or `int64`) and the other operand is an unsuffixed numeric literal (`pos.x * 2.0`, `pos.x + 10`, `color.r == 255`).
- [x] `[P1]` **Conformance & Negative Bounds Test Suite (`crates/solix-tests`):** Add unit and integration tests covering all 6 coercion contexts, safe variable widening, and compile-time rejection of out-of-range integer literals (`uint8 x = 256;`, `int8 y = -200;`) and uncast narrowing variables.

### 3.3 P2: Medium Priority (Ergonomics & Polish)
- [x] `[P2]` **Modernize `examples/breakout/src/main.slx`:** Strip obsolete `(uint8)` and `(float32)` casts from numeric literals in `examples/breakout/src/main.slx` and verify both `debug` and `release` builds succeed cleanly.

### 3.4 P3: Low Priority (Opportunistic Cleanups)
- [x] `[P3]` **Diagnostic Hint for Narrowing Variable Assignments:** When a user assigns a `float64` variable `x` to a `float32` target without a cast, include an actionable hint suggesting `(float32)x`.

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Zero warnings with `-D warnings` | `cargo build --workspace --all-targets` | [x] Completed |
| **Automated Test Suite** | 100% pass across all unit, integration, and doc tests | `cargo test --workspace --all-targets` | [x] Completed |
| **Clippy Static Analysis** | Zero warnings, zero `unwrap` / `expect` | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Completed |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [x] Completed |
| **Documentation Sync** | Specs and guides updated to GL-0010 | GL-0010 audit checklist | [x] Completed |

## 5. Risk Assessment & Mitigation Strategies
* **Risk 1:** Coercing an integer literal (e.g., `100`) to `float32` at the binder level without updating `semantic_model.node_types` would cause the bytecode emitter to emit `LoadConst` with `Value::I32(100)` instead of `Value::F64(100.0)`, leading to a runtime `TypeMismatch` in float arithmetic or `SetField`.
  * *Mitigation:* Explicitly update `semantic_model.node_types` for coerced literal AST nodes AND inspect the target `node_types` entry in `emitter` when emitting `ExprKind::Literal` and `ExprKind::Unary(Neg, Literal)` so integer literals coerced to `float32`/`float64` emit `Value::F64(val as f64)` and integer literals coerced to `int64`/`uint64` emit `Value::I64(val)`.
* **Risk 2:** Method overload or constructor resolution ambiguity when multiple overloads accept different numeric widths.
  * *Mitigation:* Prefer exact type matches first (`int32` for `Int`, `float64` for `Float`), then lossless widening matches, then context-typed literal coercion matches.