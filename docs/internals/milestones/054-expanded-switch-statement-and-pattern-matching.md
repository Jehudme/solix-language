# Milestone 054: Expanded Switch Statement, Operator Equality & Pattern Matching

## Document Metadata
* **Milestone ID:** MS-054
* **Target Version:** `v2.0.0`
* **Milestone Branch:** `milestone/054-expanded-switch-statement-and-pattern-matching`
* **Status:** COMPLETED
* **Lead Architect:** Core Systems & Compiler Architecture Team
* **Target Delivery Date:** 2026-11-16
* **Related Specifications:** [GL-0001](../../internals/guidelines/authoring-commits.md), [GL-0008](../../internals/guidelines/authoring-milestones.md), [GL-0011](../../internals/guidelines/language-conventions.md), [GL-0013](../../internals/guidelines/documentation-update-protocol.md), [SPEC-0001](../../spec/README.md)

---

## 1. Executive Summary & Objectives

Milestone 054 elevates the Solix `switch` construct from a primitive integer branch into a modern, expressive pattern matching and equality dispatch mechanism.

Key objectives:
1. **Universal Primitive Type Support**:
   - Allow `switch` on all integer types (`int8`, `uint8`, `int16`, `uint16`, `int32`, `uint32`, `int64`, `uint64`), floating-point types (`float32`, `float64`), `bool`, `char`, `enum`, and `string`.
   - String dispatch uses direct equality for small cases ($\le 4$) and precomputed FNV-1a hash dispatch tables for large cases ($> 4$) guaranteeing $O(1)$ routing.
2. **Class Equality Operator Dispatch (`operator ==` / `operator !=`)**:
   - Permit switching on instances of classes that define `operator ==` or `operator !=`.
   - The emitter compiles each case candidate into an operator method call (`discriminant.operator_==(case_val)`).
   - Emit compile-time diagnostic `E0310` if a class does not implement equality overloading.
3. **Tuples & Pattern Matching**:
   - Support switching on tuples (`(T1, T2)`), validating arity and element-wise value equality.
   - Support `instanceof` pattern matching in `case` clauses:
     - `case instanceof NominalClass:`
     - `case instanceof (T1, T2):` (tuple structure & element type checks)
     - `case null:` on nullable reference types.

---

## 2. Priority Blocker Taxonomy & Deliverables

| Priority | Task ID | Subsystem | Description | Release Impact |
|:---|:---|:---|:---|:---|
| **P0** | `MS054-T01` | `parser` | Expand `case` syntax to accept string literals, enum variants, tuple literals, and `instanceof` type expressions. | Syntax support |
| **P0** | `MS054-T02` | `binder` | Typecheck switch discriminant against case values; validate class `operator ==` overloading; validate tuple arity. | Type system soundness |
| **P0** | `MS054-T03` | `emitter` | Emit bytecode for primitive equalities, hash table string dispatch, class operator invocations, and `instanceof` checks. | Bytecode generation |
| **P1** | `MS054-T04` | `runtime` | Verify VM support for `OpCode::EqString`, `OpCode::StringHash`, and tuple dynamic type inspection. | Execution fidelity |
| **P2** | `MS054-T05` | `docs` | Update grammar reference, compiler specification, and wiki tutorials on switch pattern matching. | Living documentation |

---

## 3. New Diagnostics & Error Codes

| Error Code | Subsystem | Name | Description & Template |
|:---|:---|:---|:---|
| **`E0310`** | Compiler (Binder) | `SwitchClassMissingEqualityOperator` | `Class '{class_name}' does not implement 'operator ==' or 'operator !=' and cannot be used in equality switch cases.` |
| **`E0311`** | Compiler (Binder) | `IncompatibleSwitchCaseType` | `Case expression of type '{case_type}' is incompatible with switch discriminant of type '{discriminant_type}'.` |
| **`E0312`** | Compiler (Binder) | `DuplicateSwitchCaseConstant` | `Duplicate case value '{case_value}' detected in switch statement.` |
| **`E0313`** | Compiler (Binder) | `InvalidSwitchPatternTarget` | `Right-hand target of 'instanceof' in switch case must be a class, interface, or tuple type, found '{found}'.` |
| **`E0314`** | Compiler (Binder) | `TupleArityMismatchInCase` | `Tuple case pattern arity {case_arity} does not match switch discriminant tuple arity {disc_arity}.` |

---

## 4. Comprehensive Test Matrix

### A. Positive Conformance Tests
1. `test_switch_all_primitives`: Conformance across `int8` through `uint64`, `float32`, `float64`, `bool`, and `char`.
2. `test_switch_enum_variants`: Switching on `enum Color { Red, Green, Blue }` with exhaustive and non-exhaustive default handling.
3. `test_switch_strings_hash_dispatch`: Switching on 16 distinct string keys correctly routes to matching blocks.
4. `test_switch_class_overloaded_equality`: Switching on custom class `ComplexNumber` calling `operator ==`.
5. `test_switch_tuple_equality`: Switching on `(int32, string)` matches `(1, "start")` and `(2, "stop")`.
6. `test_switch_instanceof_class`: Polymorphic dispatch via `case instanceof Circle:` and `case instanceof Square:`.
7. `test_switch_instanceof_tuple`: Dynamic type query matching `case instanceof (int32, int32):`.

### B. Negative Rejection Tests
1. `test_switch_class_without_equality_rejected`: Emits `E0310` when class lacks `operator ==`.
2. `test_switch_incompatible_case_type_rejected`: Emits `E0311` when matching `string` case against `int32` discriminant.
3. `test_switch_duplicate_case_rejected`: Emits `E0312` on duplicate case constant.
4. `test_switch_tuple_arity_mismatch_rejected`: Emits `E0314` when matching 3-element tuple against 2-element discriminant.

---

## 5. Definition of Done (DoD)
1. All positive and negative switch test suites pass with 100% success.
2. Zero warnings in `cargo clippy --workspace --all-targets -- -D warnings`.
3. Language specifications in `docs/spec/` updated with new switch grammar and semantic rules.

---

## 6. Completed Deliverables & Verification Record

* **Universal Primitive & String Switch:** Full support for `int8`..`int64`, `uint8`..`uint64`, `float32`, `float64`, `bool`, `char`, `string`, and strongly typed `enum` discriminants and cases.
* **Class Equality Operator Dispatch (`operator ==` / `operator !=`):** Implemented validation and calling convention for nominal class equality matching with error code `E0310`.
* **Tuple Value Equality & Tuple Arity Validation:** Implemented multi-element value matching using `OpCode::GetTuple` with error code `E0314`.
* **Pattern Matching (`instanceof`):** Implemented `case instanceof TargetClass:` and `case instanceof (T1, T2):` with error code `E0313`.
* **Duplicate & Incompatible Case Diagnostics:** Enforced `E0311` (incompatible case type) and `E0312` (duplicate case constant).
* **Hermetic Integration Tests:** Added 11 new tests in `test_expanded_switch.rs` verifying 100% test passing across the compiler suite.
* **Living Documentation:** Synchronized `docs/spec/statements/control_flow/switch_statement.md`.
