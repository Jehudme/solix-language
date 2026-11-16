# Milestone 055: Advanced Escape Analysis, Scalar Replacement & ARC Elision

## Document Metadata
* **Milestone ID:** MS-055
* **Target Version:** `v2.0.0`
* **Milestone Branch:** `milestone/055-escape-analysis-scalar-replacement-and-arc-elision`
* **Status:** COMPLETED
* **Lead Architect:** Core Systems & Optimization Engineering Team
* **Target Delivery Date:** 2026-11-16
* **Related Specifications:** [GL-0001](../../internals/guidelines/authoring-commits.md), [GL-0008](../../internals/guidelines/authoring-milestones.md), [GL-0011](../../internals/guidelines/language-conventions.md), [GL-0013](../../internals/guidelines/documentation-update-protocol.md), [SPEC-0001](../../spec/README.md)

---

## 1. Executive Summary & Objectives

Milestone 055 introduces flow-sensitive escape analysis, Scalar Replacement of Aggregates (SRA), and inter-procedural ARC elision to significantly reduce heap allocation pressure and eliminate redundant reference counting instructions.

Key objectives:
1. **Flow-Sensitive Escape Analysis**:
   - Classify variable lifetimes into `NoEscape`, `ArgEscape` (borrowed by callee), and `GlobalEscape`.
   - Track all escape paths: returns, static/global field writes, escaping field stores, array writes, escaping closure captures, untrusted calls, and exception throws.
   - Annotate function parameter contracts with `NonEscaping` when parameters are guaranteed not to leak.
2. **Scalar Replacement of Aggregates (SRA)**:
   - For `NoEscape` objects: decompose fields into local VM registers in the call frame.
   - Completely eliminate `OpCode::NewObject` for non-escaping allocations (zero heap overhead).
   - Direct register operations replace field reads and writes.
3. **ARC Elision Pass**:
   - For `NoEscape` objects: eliminate all `OpCode::IncRef` and `OpCode::DecRef` instructions.
   - For `ArgEscape` variables passed to `NonEscaping` parameters: caller emits zero `IncRef` before call, and callee emits zero `DecRef` upon return.
   - Retain counts only on initial allocation and true cross-scope transfers.

---

## 2. Priority Blocker Taxonomy & Deliverables

| Priority | Task ID | Subsystem | Description | Release Impact |
|:---|:---|:---|:---|:---|
| **P0** | `MS055-T01` | `compiler/binder` | Implement flow-sensitive escape analysis pass tracking points-to sets and escape categories. | Core analysis |
| **P0** | `MS055-T02` | `compiler/binder` | Implement inter-procedural function parameter escape contract deduction. | Inter-procedural optimization |
| **P0** | `MS055-T03` | `compiler/emitter` | Implement Scalar Replacement of Aggregates (SRA) transforming non-escaping struct fields to local registers. | Zero-allocation speedup |
| **P0** | `MS055-T04` | `compiler/optimizer` | Implement ARC Elision pass pruning redundant `IncRef`/`DecRef` instructions. | Instruction count reduction |
| **P1** | `MS055-T05` | `benchmarks` | Run full benchmark suite (`python3 benchmarks/runner.py --verify`) measuring speedup and heap reduction. | Performance validation |
| **P2** | `MS055-T06` | `docs` | Document escape analysis rules, parameter annotations, and SRA limitations in compiler specs. | Living documentation |

---

## 3. New Diagnostics & Error Codes

| Error Code | Subsystem | Name | Description & Template |
|:---|:---|:---|:---|
| **`W0320`** | Compiler (Optimization) | `EscapeAnalysisDegraded` | `Variable '{var_name}' could not be scalar-replaced due to dynamic virtual dispatch escaping: {reason}` |
| **`W0321`** | Compiler (Optimization) | `ArcElisionBypassedForNative` | `ARC elision bypassed for argument '{arg_name}' passed to unannotated native function '{func_name}'.` |

---

## 4. Comprehensive Test Matrix

### A. Positive Conformance Tests
1. `test_sra_point_struct_registers`: `Point p = new Point(10, 20); int sum = p.x + p.y;` compiles without emitting `OpCode::NewObject` or heap allocations.
2. `test_arc_elision_local_temporary`: Temporary objects created and consumed locally emit zero `IncRef` or `DecRef` bytecode instructions.
3. `test_arc_elision_borrowed_parameter`: Function `void inspect(Point p)` marked `NonEscaping` induces zero ref count changes at call sites.
4. `test_escape_analysis_array_escape`: Storing into an array correctly escalates variable to `GlobalEscape`.
5. `test_escape_analysis_return_escape`: Returning a variable correctly escalates variable to `GlobalEscape`.

### B. Negative & Regression Tests
1. `test_escape_analysis_aliasing_propagation`: Assigning `y = x` where `y` escapes correctly prevents scalar replacement of `x`.
2. `test_virtual_call_forces_heap_allocation`: Calling interface method passes valid heap address even if object was otherwise local.

---

## 5. Definition of Done (DoD)
1. 100% of benchmark suite passes with bit-for-bit mathematical output matching.
2. Verified zero regressions in memory safety or lifetime management.
3. Zero clippy warnings under `-D warnings`.
