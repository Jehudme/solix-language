# Milestone 014: High-Performance VM Engine & Fast Execution Mode

## Document Metadata
* **Milestone ID:** MS-014
* **Target Version:** `v0.1.0-alpha.14`
* **Milestone Branch:** `milestone/014-high-performance-vm-engine`
* **Status:** ACTIVE
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
  * `docs/internals/milestones/013-solix-cli-client-tooling.md` (MS-013)

---

## 1. Executive Summary & Purpose

In Milestone 013, the Solix toolchain gained a unified command-line client (`solix compile`, `solix run`, `solix info`), automated dependency resolution, and rich diagnostic reporting. However, comparative benchmarks against legacy C++ Solix (v2.1.0), Python 3.15, and Lua 5.5 revealed a significant execution latency deficit:
* **Recursive Fibonacci (`fib(32)`)**: New Solix (3,732 ms) vs Old Solix (778 ms) — ~4.79x slower.
* **Tight Loop (20M iterations)**: New Solix (13,871 ms) vs Old Solix (2,956 ms) — ~4.69x slower.
* **Object Allocation (1M objects)**: New Solix (2,696 ms) vs Old Solix (556 ms) — ~4.84x slower.
* **Sieve of Eratosthenes (200k)**: New Solix (464 ms) vs Old Solix (121 ms) — ~3.84x slower.

Profiling reveals that this latency stems directly from three interpreter architectural bottlenecks:
1. **Per-Instruction Dispatch Overhead**: Every single bytecode step executes `self.step()`, performing 3 `call_frames` lookups, 1 `get_function()` lookup, and 2 boundary function calls.
2. **Bounds-Checked Register Access (`#![forbid(unsafe_code)]`)**: Every operand read (`read_reg`) and write (`write_reg`) performs dynamic slice bounds checks on `register_stack`.
3. **HashMap Lookups for Heap Storage**: Array elements and object fields are mapped through `HashMap<(u32, usize), Value>`, introducing hashing and table traversal overhead on every single field read/write.

### Core Objectives of Milestone 014
1. **Unsafe Fast Mode by Default**: Lift `#![forbid(unsafe_code)]` in `solix-runtime` (replacing it with `#![deny(unsafe_op_in_unsafe_fn)]`) to enable raw pointer caching and unchecked register access in hot interpreter loops.
2. **Dual Execution Engine Architecture**:
   - **Fast Mode (Default)**: High-performance inlined dispatch loop using raw pointer register indexing (`unsafe { *reg_base.add(r) }`) and contiguous instruction pointer traversal (`inst_ptr`).
   - **Checked Mode (`--safe`)**: Defensive execution using bounds-checked slices, invariant assertions, and complete memory leak verification.
3. **Direct Indexed Object and Array Storage**: Eliminate `HashMap<(u32, usize), Value>` from hot paths by storing object fields and array elements in contiguous vectors indexed directly by `(addr, offset)`.
4. **CLI Integration**: Add the `--safe` flag to `solix run` (defaulting to fast mode).
5. **Benchmark Parity Verification**: Re-run the `/tmp/solix_benchmark` suite to verify that New Solix matches or exceeds legacy C++ Solix execution performance.

---

## 2. Technical Architecture & Design Specification

### 2.1 The Inlined Local Register Window Engine (`run_fast`)

Rather than invoking `self.step()` on every instruction, `run_fast` executes a tight inlined loop with cached raw pointers:

```rust
// Local register window and instruction pointer caching
let mut pc: usize = frame.pc as usize;
let inst_slice = &func.instructions;
let inst_len = inst_slice.len();
let mut inst_ptr = inst_slice.as_ptr();
let mut reg_base = self.register_stack.as_mut_ptr().add(frame.window_offset);

while pc < inst_len {
    let inst = unsafe { *inst_ptr.add(pc) };
    pc += 1;

    match inst.opcode_u8() {
        OP_ADD_I32 => {
            let src1 = unsafe { *reg_base.add(inst.src1() as usize) };
            let src2 = unsafe { *reg_base.add(inst.src2() as usize) };
            if let (Value::I32(a), Value::I32(b)) = (src1, src2) {
                unsafe { *reg_base.add(inst.dst() as usize) = Value::I32(a.wrapping_add(b)); }
            }
        }
        OP_JUMP => {
            pc = inst.jump_target() as usize;
        }
        OP_CALL | OP_CALL_DIRECT | OP_CALL_VIRTUAL => {
            // Synchronize PC and transition frames
            self.current_frame_mut()?.pc = pc as u32;
            self.push_frame(...)?;
            // Reload cached pointers for the new frame
            ...
        }
        ...
    }
}
```

### 2.2 Direct-Indexed Memory Model for Objects & Arrays

Instead of hashing `(payload_addr, offset)` into a `HashMap`:
```rust
// Current (slow):
pub object_fields: HashMap<(u32, usize), Value>,
pub array_elements: HashMap<(u32, usize), Value>,

// Optimized:
// Pre-allocated flat buffers or direct memory payload embedding:
pub heap_values: Vec<Value>, // indexed by payload_addr + offset
```
Eliminating hashing on array and field access accelerates tight numerical and array manipulation loops (e.g. Sieve of Eratosthenes and Object Allocation benchmarks).

### 2.3 Command-Line Interface (`--safe`)

```bash
# High-performance fast execution mode (default)
solix run app.slxb

# Safe / checked execution mode (bounds-checked, defensive invariants)
solix run --safe app.slxb
```

---

## 3. Definition of Done (DoD) Matrix

| Requirement | Acceptance Criteria | Verification Method |
| :--- | :--- | :--- |
| **Fast Execution Engine** | Inlined dispatch loop caching raw instruction and register window pointers; zero per-instruction function call overhead. | Conformance & performance benchmarks |
| **`--safe` CLI Flag** | Toggles between fast unchecked engine and defensive bounds-checked engine. | CLI integration tests |
| **Direct Indexed Storage** | Eliminates `HashMap` lookups for array indexing and object field access. | Microbenchmarks |
| **Zero Regressions** | 100% of existing tests in `solix-tests`, `solix-runtime`, and `solix-client` pass in both fast and safe modes. | Automated workspace tests |
| **Performance Parity** | Execution time approaches or surpasses legacy C++ Solix across all benchmarks in `/tmp/solix_benchmark/`. | Automated runner script |
| **Zero Warnings & Quality** | `#![deny(unsafe_op_in_unsafe_fn)]`, clean `cargo clippy --workspace --all-targets -- -D warnings`, clean `cargo fmt --check`. | Workspace automated checks |

---

## 4. Phased Roadmap

* [x] **Step 0: Specification Document & Milestone Branch Inception**
* [x] **Phase 1: Dual Execution Mode Model & CLI Integration (`--safe`)**
  - Add `is_safe_mode: bool` to `RuntimeOptions` and `VirtualMachine`.
  - Add `--safe` flag to `solix run` in `solix-client`.
* [x] **Phase 2: Lift Strict Forbid & Author Safety Invariant Protocols**
  - Lift `#![forbid(unsafe_code)]` in `solix-runtime`.
  - Enforce `#![deny(unsafe_op_in_unsafe_fn)]`.
* [ ] **Phase 3: High-Performance Inlined Instruction Dispatcher (`run_fast`)**
  - Implement local pointer-cached execution loop for arithmetic, logic, and jumps.
  - Implement frame synchronization on calls, returns, and exceptions.
* [ ] **Phase 4: Direct-Indexed Heap Storage for Arrays and Objects**
  - Optimize array elements and object fields access, eliminating `HashMap` hashing from hot loops.
* [ ] **Phase 5: Conformance Testing & Benchmark Parity Verification**
  - Verify all 71 runtime tests pass in both `--safe` and fast modes.
  - Re-run `/tmp/solix_benchmark/runner.py` and publish comparative performance report.
* [ ] **Phase 6: Master Merge & Milestone Closure**
