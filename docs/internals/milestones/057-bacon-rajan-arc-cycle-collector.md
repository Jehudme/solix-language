# Milestone 057: Bacon & Rajan ARC Cycle Collector with Zero-Cost Opt-Out

## Document Metadata
* **Milestone ID:** MS-057
* **Target Version:** `v2.0.0`
* **Milestone Branch:** `milestone/057-bacon-rajan-arc-cycle-collector`
* **Status:** COMPLETED
* **Lead Architect:** Runtime Virtual Machine & Memory Management Team
* **Target Delivery Date:** 2026-11-16
* **Related Specifications:** [GL-0001](../../internals/guidelines/authoring-commits.md), [GL-0008](../../internals/guidelines/authoring-milestones.md), [GL-0011](../../internals/guidelines/language-conventions.md), [GL-0013](../../internals/guidelines/documentation-update-protocol.md), [SPEC-0001](../../spec/README.md)

---

## 1. Executive Summary & Objectives

Milestone 057 implements the industry-standard Bacon & Rajan (2001) synchronous cycle collection algorithm for the Solix Automatic Reference Counting (ARC) runtime, while providing a zero-cost runtime opt-out flag for maximum raw execution throughput.

Key objectives:
1. **Bacon & Rajan (2001) Four-Color State Machine**:
   - Implement header color tags: `Black` (in use or free), `Purple` (possible cycle root), `Gray` (trial deletion), `White` (confirmed unreachable garbage cycle).
   - Only objects whose reference count is decremented to a **non-zero** value are suspected and appended to the preallocated `purple_buffer`. Zero-ref objects continue to be deallocated instantly in $O(1)$.
2. **Three-Phase Trial Deletion & Reclamation**:
   - `MarkGray`: Simulates freeing the candidate subgraph by decrementing internal reference counts.
   - `Scan`: Restores external counts ($> 0$) painting nodes `Black`; confirms true isolated cycles painting nodes `White`.
   - `CollectWhite`: Sweeps and returns memory of unreachable cycles to `VirtualHeap`.
3. **Low-Intrusion Triggers & Bounded Execution**:
   - Only runs when `purple_buffer.len() >= cycle_threshold` (default: 1,024 candidates) or heap memory reaches $> 80\%$ watermark.
   - Bounded time slicing via `--cycle-budget-us <N>` prevents latency spikes in game loops and interactive applications.
   - API in Solix: `Vm::collect_cycles()`, `Vm::set_cycle_threshold(N)`.
4. **Zero-Cost Opt-Out**:
   - CLI flag `--cycle-collector <on|off>` and `RuntimeOptions::with_cycle_collector(bool)`.
   - When disabled (`off`), `dec_ref` never tracks purple roots, executing pure reference counting with zero cycle overhead.

---

## 2. Priority Blocker Taxonomy & Deliverables

| Priority | Task ID | Subsystem | Description | Release Impact |
|:---|:---|:---|:---|:---|
| **P0** | `MS057-T01` | `runtime/arc` | Add 4-color state tags to heap block headers and preallocated candidate `purple_buffer`. | Memory layout |
| **P0** | `MS057-T02` | `runtime/collector` | Implement Bacon & Rajan trial deletion (`MarkGray`, `Scan`, `CollectWhite`). | Cycle reclamation |
| **P0** | `MS057-T03` | `runtime/options` | Implement `--cycle-collector <on|off>`, `--cycle-threshold`, and zero-cost opt-out branching. | Performance control |
| **P1** | `MS057-T04` | `runtime/vm` | Expose cycle collector telemetry and explicit collection hook `Vm::collect_cycles()`. | Introspection |
| **P2** | `MS057-T05` | `docs` | Document cycle collector mechanics, algorithms, and opt-out benchmarks in runtime specs. | Living documentation |

---

## 3. New Diagnostics & Error Codes

| Error Code | Subsystem | Name | Description & Template |
|:---|:---|:---|:---|
| **`W0340`** | Runtime (Memory) | `CycleCollectorBudgetExceeded` | `Cycle collector exceeded time slice budget of {budget_us} us; deferred {remaining} roots to next cycle.` |
| **`E0341`** | Runtime (Configuration) | `InvalidCycleCollectorConfig` | `Invalid cycle collector configuration parameter: {reason}` |

---

## 4. Comprehensive Test Matrix

### A. Positive Conformance Tests
1. `test_self_referential_cycle_collected`: Node pointing to itself is correctly reclaimed when outer reference drops.
2. `test_mutual_binary_tree_cycle_collected`: Two nodes pointing to each other are reclaimed.
3. `test_cyclic_graph_complex_collected`: Multi-node directed graph with cycles is freed without memory leaks.
4. `test_acyclic_objects_fast_path`: Acyclic linked lists are freed immediately upon count reaching 0 without triggering cycle collector.
5. `test_cycle_collector_opt_out_speed`: Disabling cycle collector bypasses purple root buffering and matches pure ARC baseline performance.
6. `test_manual_vm_collect_cycles`: Explicitly calling `Vm::collect_cycles()` sweeps candidate buffer and returns reclaimed count.

### B. Negative Rejection Tests
1. `test_invalid_threshold_fails`: Specifying `--cycle-threshold 0` fails with `E0341`.

---

## 5. Definition of Done (DoD)
1. 100% of cycle collection stress tests pass with zero leaks and zero double-frees.
2. Clippy clean (`cargo clippy --workspace --all-targets -- -D warnings`).
3. Runtime memory specifications in `docs/spec/` updated.
