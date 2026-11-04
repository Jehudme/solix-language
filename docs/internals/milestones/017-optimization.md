# Milestone 017: Unboxed Register VM, Two-Stage Optimization Pipeline (Auto-Inlining & Bytecode Peephole) & Lua-Competitive Performance

## Document Metadata

* **Milestone ID:** MS-017
* **Target Version:** `v0.1.0-alpha.17`
* **Milestone Branch:** `milestone/017-optimization`
* **Status:** ACTIVE
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/milestones/012-compiler-pipeline-modularization-and-stabilization.md` (MS-012)
  * `docs/internals/milestones/014-high-performance-vm-engine-and-fast-execution-mode.md` (MS-014)
  * `docs/internals/milestones/015-compiler-and-runtime-hardening-and-diagnostic-precision.md` (MS-015)
  * `docs/internals/milestones/016-benchmarking-strategy-evaluation-dimensions.md` (MS-016)
  * `BENCHMARK_RESULTS.md`

---

## 1. Executive Summary & Forensic Codebase Bottleneck Analysis

In Milestone 016, we established a 16-workload cross-language benchmark suite (`benchmarks/runner.py`) comparing Solix against **CPython 3.12**, **Lua 5.4**, **LuaJIT 2.1**, and **GCC -O3**. While Solix already outperforms Python on 6 workloads (`10_matrix_multiply_2d` at **2.06x**, `03_nbody_float64` at **1.98x**, `14_exception_zero_cost` at **1.76x**, `02_tight_loop_int` at **1.48x**, `12_closures_capture` at **1.35x**, and `16_multi_binary_link` at **1.16x**), it currently trails Python on 10 workloads and remains $4\times$ to $17\times$ slower than standard Lua 5.4.

A line-by-line forensic audit of `crates/solix-runtime` and `crates/solix-compilation` reveals **seven severe architectural bottlenecks** in the current implementation that artificially throttle Solix's execution speed:

### 1.1 The 7 Root-Cause Bottlenecks in the Current Codebase

| Bottleneck ID | Exact Location in Codebase | What the Code Currently Does | Performance Catastrophe |
| :--- | :--- | :--- | :--- |
| **BOTTLENECK-01**<br>*(Per-Call String Splitting & Hash Lookups)* | `solix-runtime/src/vm/fast.rs:11-23, 444`<br>`solix-runtime/src/vm/fast.rs:41-47` | On **every single `CallDirect` instruction**, `run_fast` calls `self.is_native_function(global_func_id)`, which splits `func.symbol` on `'('`, rsplits on `['.', ':']`, and performs up to **3 `HashMap` lookups** in `native_registry`! Then `push_frame_fast` runs `.iter().position(...)` over `loaded_binaries`! | In `01_fib_recursive` (`fib(28)` = 1,028,457 calls) and `04_ackermann_stack`, the VM executes **over 3 million string splits and `HashMap` probes** just to call a normal Solix function! |
| **BOTTLENECK-02**<br>*(Uninlined Heap, Array, Call & ARC Opcodes in `run_fast`)* | `solix-runtime/src/vm/fast.rs:512-527` | `run_fast` **only** handles basic arithmetic, `Jump`, and `CallDirect`. Crucial opcodes—`LoadConst`, `AllocObject`, `GetField`, `SetField`, `AllocArray`, `GetArray`, `SetArray`, `ArrayLength`, `CallVirtual`, `CallInterface`, `CallCallable`, and `DecRef`—all hit the `_ =>` fallback arm, writing `pc` back to `CallFrame`, calling `dispatch_instruction()`, and running `reload_frame!()`! | In `09_sieve_bool_array`, `05_object_churn_pod`, `07_virtual_dispatch`, and `08_interface_dispatch`, the VM bounces out of `run_fast` and reloads frame pointers on almost every instruction! |
| **BOTTLENECK-03**<br>*(Heap Side-Table `HashMap`s & $O(N)$ Deallocation Scan)* | `solix-runtime/src/vm/engine.rs:54-60, 146-159`<br>`solix-runtime/src/vm/dispatch.rs:607-755` | `VirtualMachine` stores fields, array elements, and closure captures in `object_fields: HashMap<(u32, usize), Value>`, `array_elements: HashMap<(u32, usize), Value>`, and `closure_captures: HashMap<(u32, usize), Value>`. Worse: when an object's refcount hits 0 in `release_reference` (`engine.rs:147`), it runs `self.object_fields.keys().filter(|(obj_addr, _)| *obj_addr == addr).collect()`—**scanning the entire global `HashMap` of all live object fields in the VM**! | In `06_binary_trees_arc` (depth 10 = 2,047 nodes) and `05_object_churn_pod`, deallocating a tree is $O(N^2)$ because every single freed node scans all remaining nodes' fields in `object_fields`! |
| **BOTTLENECK-04**<br>*(String Constant Re-Allocation on Every `LoadConst`)* | `solix-runtime/src/vm/dispatch.rs:54-61` | Whenever `OpCode::LoadConst` loads a `ConstantPoolValue::String(s)`, it calls `self.heap.allocate(bytes.len(), BlockTag::String)` and copies the string bytes into a brand-new heap block! | In `15_string_processing`, `"alpha"`, `"beta"`, and `"gamma"` are loaded inside a 100,000-iteration loop, causing **300,000 redundant heap allocations and deallocations** for immutable string literals! |
| **BOTTLENECK-05**<br>*(16-Byte Boxed `Value` Enums & Frame Zero-Filling)* | `solix-runtime/src/value/model.rs:10`<br>`solix-runtime/src/vm/fast.rs:61-65` | Every register is a 16-byte `enum Value`. Every arithmetic instruction in `run_fast` branches on `as_i32()?` / `as_f64()?`, and every `push_frame_fast` resizes `register_stack` with `Value::Null` and copies arguments via `copy_within`. | Wastes 50% of L1 data cache bandwidth, adds branch mispredictions on statically known types, and copies registers that are already in a contiguous argument window. |
| **BOTTLENECK-06**<br>*(`BTreeMap` Allocator & `live_allocations` Overhead)* | `solix-runtime/src/memory/heap.rs:101-175` | Every `heap.allocate()` searches a `BTreeMap` (`FreeListManager`), zeroes memory, and inserts into `live_allocations: BTreeMap<u32, LiveAllocationInfo>`. Every `heap.deallocate()` removes from `live_allocations` and coalesces in `BTreeMap`. | Millions of host heap `BTreeMap` node allocations and tree rebalancing operations during object churn (`05_object_churn_pod`, `11_generic_containers`). |
| **BOTTLENECK-07**<br>*(No Pre-Emission Automatic Function Inlining or Post-Emission Bytecode Optimizer)* | `solix-compilation/src/pipeline.rs:47-53`<br>`solix-compilation/src/phases/emitter/emitter.rs` | `CompilationPipeline::standard()` goes straight from `BinderPhase` to `EmitterPhase`. Tiny functions ($\le 8$ statements) like `Box::get_val()`, `Point::constructor()`, `Calculator::compute()`, and `StrProcApp::process()` emit full `CallDirect` frames, and loop bodies reload constants (`LoadI32`) on every iteration. | Billions of avoidable instructions and millions of avoidable call frames across `02_tight_loop_int`, `05_object_churn_pod`, `11_generic_containers`, `15_string_processing`, and `16_multi_binary_link`. |

---

## 1.2 MS-016 Baseline vs. MS-017 Performance Targets

The objective of Milestone 017 is to **obliterate Python 3.12 across all 16 workloads (achieving 3x to 15x speedups over Python)** and **close the gap to (or beat) PUC-Rio Lua 5.4** across the entire benchmark suite:

| Workload | MS-016 Solix Fast (ms) | Python 3.12 (ms) | Lua 5.4 (ms) | MS-017 Target (ms) | Target vs. Python | Target vs. Lua 5.4 |
| :--- | ---: | ---: | ---: | ---: | :---: | :---: |
| `01_fib_recursive` | 289.07 | 103.54 | 49.41 | **< 42.00 ms** | **> 2.4x faster** | **Beats Lua 5.4** |
| `02_tight_loop_int` | 220.35 | 326.58 | 34.88 | **< 32.00 ms** | **> 10.0x faster** | **Beats Lua 5.4** |
| `03_nbody_float64` | 34.59 | 68.43 | 5.03 | **< 6.50 ms** | **> 10.5x faster** | **~1.2x of Lua** |
| `04_ackermann_stack` | 63.58 | 42.94 | 7.45 | **< 8.50 ms** | **> 5.0x faster** | **~1.1x of Lua** |
| `05_object_churn_pod` | 91.55 | 62.86 | 23.66 | **< 12.00 ms** | **> 5.2x faster** | **Beats Lua 5.4 (2x)** |
| `06_binary_trees_arc` | 86.61 | 44.16 | 16.01 | **< 15.00 ms** | **> 2.9x faster** | **Beats Lua 5.4** |
| `07_virtual_dispatch` | 94.72 | 72.36 | 12.70 | **< 14.00 ms** | **> 5.1x faster** | **~1.1x of Lua** |
| `08_interface_dispatch` | 85.36 | 75.22 | 20.14 | **< 15.00 ms** | **> 5.0x faster** | **Beats Lua 5.4** |
| `09_sieve_bool_array` | 134.45 | 56.40 | 7.87 | **< 9.00 ms** | **> 6.2x faster** | **~1.1x of Lua** |
| `10_matrix_multiply_2d` | 13.85 | 28.55 | 4.66 | **< 4.50 ms** | **> 6.3x faster** | **Beats Lua 5.4** |
| `11_generic_containers` | 169.53 | 90.83 | 45.75 | **< 14.00 ms** | **> 6.4x faster** | **Beats Lua 5.4 (3x)** |
| `12_closures_capture` | 32.23 | 43.54 | 4.75 | **< 6.50 ms** | **> 6.7x faster** | **~1.3x of Lua** |
| `13_function_pointers` | 118.73 | 82.08 | 18.41 | **< 16.00 ms** | **> 5.1x faster** | **Beats Lua 5.4** |
| `14_exception_zero_cost` | 23.10 | 40.62 | 26.32 | **< 6.00 ms** | **> 6.7x faster** | **Beats Lua 5.4 (4x)** |
| `15_string_processing` | 215.45 | 78.04 | 22.47 | **< 14.00 ms** | **> 5.5x faster** | **Beats Lua 5.4** |
| `16_multi_binary_link` | 41.07 | 47.77 | 5.26 | **< 6.00 ms** | **> 7.9x faster** | **~1.1x of Lua** |

---

## 2. Normative Architectural Upgrades (The 6 Pillars of MS-017)

### 2.1 Pillar 1: Pre-Computed Load-Time Metadata, String Interning & Zero-Lookup Calls
1. **Eliminate Per-Call String Splitting (`BOTTLENECK-01`):**
   * Extend `CompiledFunction` (or a runtime-side `ResolvedFunctionMeta` table in `RuntimeContext`) with pre-resolved fields computed **once** inside `RuntimeContext::new()`:
     * `binary_id: u16`: Pre-computed index of the owning `LoadedBinary` (eliminates `.iter().position(...)` in `push_frame_fast`).
     * `native_fn: Option<NativeFunction>`: Resolved once at VM initialization (eliminates `is_native_function()` string splitting and `HashMap` lookups on every `CallDirect` and `CallVirtual`).
     * `has_references: bool`: `true` if `!reference_registers.is_empty()`.
2. **Direct VTable Indexing (`O(1)` Array Lookup Instead of `.iter().find()`):**
   * Currently, `CallVirtual`, `CallInterface`, and `release_reference` run `self.context.all_classes.iter().find(|c| c.vtable_id == vtable_id)` on every virtual call and every object drop!
   * In `RuntimeContext::new()`, build a direct `vtable_lookup: Vec<Option<CompiledClass>>` indexed by `vtable_id as usize` so class lookup is a 1-cycle array index `self.context.vtable_lookup[vtable_id as usize]`!
3. **Load-Time String Constant Interning (`BOTTLENECK-04`):**
   * During `VirtualMachine::new()`, pre-allocate all `ConstantPoolValue::String` entries in `VirtualHeap` as immortal/static-lifetime string blocks (`ref_count = MAX_REF_COUNT / 2`) and store their `Value::Ref(addr)` in a flat `const_pool_cache: Vec<Vec<Value>>`.
   * Executing `OpCode::LoadConst` or `OpCode::LoadF64` becomes a single array read `self.const_pool_cache[binary_id][idx]` with **zero runtime heap allocations**!

### 2.2 Pillar 2: Complete Elimination of `HashMap` Side-Tables & Direct `VirtualHeap` Addressing
1. **Remove `object_fields`, `array_elements`, and `closure_captures` `HashMap`s (`BOTTLENECK-03`):**
   * All object fields, array elements, tuple slots, and closure captures **MUST** be stored directly in `VirtualHeap.buffer` (alongside a compact type-tag byte or raw 64-bit representation).
   * **`GetField` / `SetField`:** Read and write directly at `obj_addr + offset`.
   * **`GetArray` / `SetArray`:** Read and write directly at `arr_addr + 8 + idx * elem_size`.
   * **`PackClosure` / `UnpackCaptures`:** Read and write directly at `closure_addr + 8 + idx * 8`.
2. **$O(1)$ Reference Field Cascade in `release_reference`:**
   * Instead of scanning a global `HashMap`, when an `Object` is freed (`rc == 0`), iterate over `class.reference_field_offsets` (typically 0 to 4 offsets) and read the 32-bit child address directly from `self.heap.read_u64(addr + offset as u32) as u32`.
   * If `class.destructor_id.is_none() && class.reference_field_offsets.is_empty()` (`is_pod_class == true`), `release_reference` skips `active_destructors` hash set insertion and immediately frees the block!

### 2.3 Pillar 3: $O(1)$ Intrusive Size-Class Quick-Lists & Fast Allocator in `VirtualHeap`
1. **Bypass `BTreeMap` in Fast Mode (`BOTTLENECK-06`):**
   * Add `quick_lists: [u32; 32]` to `VirtualHeap` indexing 8-byte size classes from 8 bytes to 256 bytes, plus a `bump_offset: usize` cursor.
   * In fast mode (`!safe_mode`), `VirtualHeap::allocate` first checks `quick_lists[size_class]`; if non-zero, it pops the block in 3 CPU instructions. Otherwise, it bumps `bump_offset` in 3 CPU instructions.
   * In fast mode, skip `live_allocations: BTreeMap` insertions on each allocation; instead, store `allocated_block_size` in the block header and track `active_live_objects: usize` with a simple integer counter. (Full `BTreeMap` leak reporting remains active in `--safe` mode or when `verify_clean_memory()` is invoked).

### 2.4 Pillar 4: Full Opcode Coverage & Sliding Register Windows in `run_fast`
1. **Inline All Hot Opcodes Directly in `run_fast` (`BOTTLENECK-02`):**
   * Inline `LoadConst`, `AllocObject`, `GetField`, `SetField`, `AllocArray`, `GetArray`, `SetArray`, `ArrayLength`, `CallVirtual`, `CallInterface`, `CallCallable`, `PackClosure`, `UnpackCaptures`, `Instanceof`, `CastCheck`, `EqString`, `NeqString`, and `DecRef` directly inside `run_fast` so none of the 16 benchmarks ever fall back to `dispatch_instruction`.
2. **Fast Frame Transitions & Primitive-Only Leaf Fast Path:**
   * When `func.reference_registers.is_empty()`, skip zero-filling local registers on call entry and skip reference cleanup loops on return.
   * Pre-allocate `register_stack` to `65536` slots at VM startup so `push_frame_fast` and `ReturnValue` never reallocate or call `Vec::resize` / `Vec::truncate` during execution!

### 2.5 Pillar 5: Pre-Emission Automatic AST Inliner ($\le 8$ Statements) & Threshold Benchmarking
1. **Automatic Pre-Emission `AstInlinerPhase` (`crates/solix-compilation/src/phases/optimizer/ast_inliner.rs`):**
   * Runs automatically between `BinderPhase` and `EmitterPhase` in `CompilationPipeline::standard()`.
   * **Automatic Eligibility Rule:** Any function, static method, non-virtual/final instance method, or constructor is **automatically** eligible for inlining (even without the `inline` keyword) if:
     1. Its body contains **$\le N$ statements** (where default `DEFAULT_INLINE_STATEMENT_LIMIT = 8`, configurable via `CompilationOptions::with_inline_threshold(usize)`).
     2. It is **not** recursive (does not call itself directly or mutually).
     3. It is **not** `virtual`, `override`, `abstract`, `native`, or `extern`.
     4. It does not contain `try/catch/finally` blocks.
   * **AST Transformation Mechanics:**
     * **Single-Return Expression Functions (`return <expr>;`):** When a call `callee(arg1, arg2)` targets a function whose body is a single `ReturnStatement(Some(expr))` and arguments are side-effect-free (literals or local identifiers), substitute the `MethodCall` expression directly with `expr` (with parameters replaced by argument expressions).
     * **Multi-Statement Functions & Constructors ($\le 8$ statements):** Lower the call statement into a scoped `BlockStatement` in the caller: bind each parameter to a uniquely mangled local variable (`__inline_<id>_<param>`), rewrite `this` field assignments (`this.x = ...` $\to$ `target_obj.x = ...`), and replace the final `return expr;` with assignment to the call destination variable!
2. **Empirical Inlining Threshold Benchmark Experiment (`N = 0, 2, 4, 6, 8, 12, 16`):**
   * Include a dedicated benchmark experiment script/test measuring compilation time, `.slxb` binary size, and runtime execution speed across all 16 workloads for thresholds $N \in \{0, 2, 4, 6, 8, 12, 16\}$ to empirically prove whether $N = 8$ is the optimal default threshold.

### 2.6 Pillar 6: Post-Emission `BytecodeOptimizerPhase` (Peephole & Loop Constant Hoisting)
1. **Post-Emission `BytecodeOptimizerPhase` (`crates/solix-compilation/src/phases/optimizer/bytecode_optimizer.rs`):**
   * Runs immediately after `EmitterPhase` on every `CompiledFunction::instructions` stream:
     1. **Redundant Move Elimination:** Fuses `Instruction(OpX, dst=r_temp, ...)` followed immediately by `Mov(dst=r_local, src1=r_temp)` (where `r_temp` is a scratch register) into `Instruction(OpX, dst=r_local, ...)`, eliminating millions of temporary `Mov` instructions.
     2. **Loop-Invariant Constant Hoisting:** Detects `LoadI32` / `LoadI64` / `LoadF64` / `LoadBool` instructions inside backward-jump loops (`PC_header .. PC_backjump`) that write to a scratch register not modified elsewhere, and hoists them before `PC_header` into a dedicated constant register.
     3. **Superinstruction Fusion:**
        * `AddI32Imm` (`0xD0`) & `SubI32Imm` (`0xD1`): Fuses `LoadI32 r_tmp, imm16` + `AddI32 r_dst, r_src, r_tmp` into a single 1-cycle instruction `r[dst] = r[src] + imm16_signed`!
        * `JumpIfLtI32` (`0xD2`), `JumpIfLteI32` (`0xD3`), `JumpIfGtI32` (`0xD4`), `JumpIfGteI32` (`0xD5`), `JumpIfEqI32` (`0xD6`), `JumpIfNeqI32` (`0xD7`): Fuses integer comparison (`LtI32 r_cmp, r_a, r_b`) + conditional branch (`JumpIfFalse r_cmp, offset`) into a single compare-and-branch instruction!
     4. **Jump Offset & Exception Table Recalculation:** Safely remaps all relative jump offsets and `ExceptionHandlerEntry` PC ranges (`try_start_pc`, `try_end_pc`, `handler_pc`, `finally_pc`) whenever instructions are compacted.

---

## 3. Scope & Crates Affected

* **`crates/solix-common`**:
  * `src/bytecode/opcodes.rs`: Add superinstructions `AddI32Imm = 0xD0`, `SubI32Imm = 0xD1`, `JumpIfLtI32 = 0xD2`, `JumpIfLteI32 = 0xD3`, `JumpIfGtI32 = 0xD4`, `JumpIfGteI32 = 0xD5`, `JumpIfEqI32 = 0xD6`, `JumpIfNeqI32 = 0xD7`, and `CallNativeDirect = 0x69`.
  * `src/bytecode/class.rs`: Add helper `is_pod(&self) -> bool`.
* **`crates/solix-compilation`**:
  * `src/phases/optimizer/mod.rs`: New optimizer module exporting `AstInlinerPhase` and `BytecodeOptimizerPhase`.
  * `src/phases/optimizer/ast_inliner.rs`: Pre-emission AST inlining pass for functions/methods/constructors with $\le 8$ statements.
  * `src/phases/optimizer/bytecode_optimizer.rs`: Post-emission linear bytecode optimizer (constant hoisting, `Mov` coalescing, superinstruction fusion, PC jump remapping).
  * `src/CompilationContext.rs` & `src/pipeline.rs`: Add `inline_threshold: usize` (default `8`) to `CompilationOptions` and register `AstInlinerPhase` and `BytecodeOptimizerPhase` in `CompilationPipeline::standard()`.
  * `src/phases/emitter/disassembler.rs`: Disassemble all new `0xD0..0xD7` and `0x69` superinstructions.
* **`crates/solix-runtime`**:
  * `src/context.rs`: Pre-compute `vtable_lookup: Vec<Option<CompiledClass>>` and per-function `binary_id` / `native_fn` metadata at load time; rewrite intra-module `GetStatic`/`SetStatic` to `GetStaticDirect`/`SetStaticDirect`.
  * `src/memory/heap.rs`: Add $O(1)$ intrusive `quick_lists: [u32; 32]` and bump-pointer allocation; eliminate `BTreeMap` overhead in fast mode while preserving `verify_clean_memory()`.
  * `src/vm/engine.rs` & `src/vm/dispatch.rs`: Eliminate `object_fields`, `array_elements`, and `closure_captures` `HashMap`s; store all fields, elements, and captures directly in `VirtualHeap.buffer`; intern string constants at startup.
  * `src/vm/fast.rs`: Pre-allocate `register_stack`; inline all object, array, virtual/interface call, string, ARC, and superinstruction opcodes into `run_fast`.
* **`crates/solix-tests`**:
  * Add unit and integration tests for `AstInlinerPhase` ($\le 8$ statements vs $> 8$ statements), `BytecodeOptimizerPhase` (superinstructions, constant hoisting, exception PC remapping), and safe-vs-fast equivalence across all 16 benchmark workloads.

---

## 4. Phased Implementation Roadmap

Milestone 017 is divided into **6 sequential, independently verifiable phases**. Every phase **MUST** pass `cargo test --workspace --all-targets` and `python3 benchmarks/runner.py --verify` before advancing to the next phase.

```
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 1: Direct VirtualHeap Storage, String Interning & Zero HashMaps  │
│          └─► Gate: object_fields/array_elements/closure_captures gone  │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 2: O(1) Intrusive Quick-Lists, Bump Allocator & Fast VTable Map  │
│          └─► Gate: 0 BTreeMap ops in hot loops + clean memory audit    │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 3: Load-Time Relocation, Pre-Resolved Call Meta & Full run_fast  │
│          └─► Gate: Zero string splits on CallDirect; all opcodes inline│
├────────────────────────────────────────────────────────────────────────┤
│ Phase 4: Pre-Emission Automatic AST Inliner Phase (<= 8 Statements)    │
│          └─► Gate: Auto-inlines functions/methods <= 8 stmts + tests   │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 5: Post-Emission BytecodeOptimizerPhase & Superinstructions      │
│          └─► Gate: Loop constant hoisting + 0xD0..0xD7 fusion + tests  │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 6: Inlining Threshold Sweep (N=0..16) & Benchmark Verification   │
│          └─► Gate: Obliterates Python on 16/16 workloads & rivals Lua  │
└────────────────────────────────────────────────────────────────────────┘
```

### Phase 1: Direct `VirtualHeap` Storage, Load-Time String Interning & Elimination of `HashMap` Side-Tables
- [ ] `[P0]` **Task 1.1:** Remove `object_fields`, `array_elements`, and `closure_captures` `HashMap`s from `VirtualMachine` (`crates/solix-runtime/src/vm/engine.rs`).
- [ ] `[P0]` **Task 1.2:** Store and read object fields (`GetField`, `SetField`), array elements (`GetArray`, `SetArray`), tuple elements (`AllocTuple`, `GetTuple`, `TupleInstanceof`), and closure captures (`PackClosure`, `UnpackCaptures`) directly inside `VirtualHeap.buffer`.
- [ ] `[P0]` **Task 1.3:** Refactor `VirtualMachine::release_reference` to traverse `CompiledClass::reference_field_offsets` directly in `VirtualHeap` in $O(\text{ref\_fields})$ time instead of scanning `object_fields.keys()`.
- [ ] `[P0]` **Task 1.4:** Intern all `ConstantPoolValue::String` literals into `VirtualHeap` once during `VirtualMachine::new()` so `OpCode::LoadConst` returns the pre-allocated string reference in $O(1)$ without allocating on every iteration (`15_string_processing`).
- [ ] `[P0]` **Task 1.5:** Verify all existing unit and conformance tests pass with zero memory leaks (`verify_clean_memory()`).

### Phase 2: $O(1)$ Intrusive Size-Class Quick-Lists, Bump Allocator & Direct VTable Lookup Table
- [ ] `[P1]` **Task 2.1:** Add `quick_lists: [u32; 32]` (for 8-byte size classes $8\text{ B} \dots 256\text{ B}$) and `bump_cursor: usize` to `VirtualHeap` (`crates/solix-runtime/src/memory/heap.rs`).
- [ ] `[P1]` **Task 2.2:** Implement $O(1)$ pop from `quick_lists` and bump allocation in `VirtualHeap::allocate`, and $O(1)$ intrusive push to `quick_lists` in `VirtualHeap::deallocate`.
- [ ] `[P1]` **Task 2.3:** Update `VirtualHeap::verify_clean_memory()` to flush `quick_lists` and the bump tail into `FreeListManager` before auditing so leak detection remains 100% exact.
- [ ] `[P1]` **Task 2.4:** Replace `.all_classes.iter().find(|c| c.vtable_id == vtable_id)` with a flat `vtable_lookup: Vec<Option<CompiledClass>>` indexed directly by `vtable_id` in `RuntimeContext`.

### Phase 3: Load-Time Relocation, Pre-Resolved Call Metadata & Complete `run_fast` Inlining
- [ ] `[P0]` **Task 3.1:** Pre-compute `FunctionRuntimeMeta { binary_id: usize, native_fn: Option<NativeFunction>, has_references: bool }` for every function in `RuntimeContext::new()`, and rewrite intra-module `GetStatic`/`SetStatic` to `GetStaticDirect`/`SetStaticDirect` at load time.
- [ ] `[P0]` **Task 3.2:** Remove `is_native_function()` string splitting and `.loaded_binaries.iter().position()` from `run_fast` and `push_frame_fast` (`crates/solix-runtime/src/vm/fast.rs`).
- [ ] `[P0]` **Task 3.3:** Pre-allocate `register_stack` to capacity at VM creation; in `push_frame_fast`, skip register `Value::Null` zero-filling and return ARC cleanup when `has_references == false`.
- [ ] `[P1]` **Task 3.4:** Inline `LoadConst`, `AllocObject`, `GetField`, `SetField`, `AllocArray`, `GetArray`, `SetArray`, `ArrayLength`, `CallVirtual`, `CallInterface`, `CallCallable`, `PackClosure`, `UnpackCaptures`, `EqString`, `NeqString`, and `DecRef` directly inside `run_fast`.
- [ ] `[P1]` **Task 3.5:** Implement `OpCode::CallNativeDirect = 0x69` and patch `CallSymbol` in-place on first execution.

### Phase 4: Pre-Emission Automatic AST Inliner Phase ($\le 8$ Statements)
- [ ] `[P1]` **Task 4.1:** Add `inline_threshold: usize` (default `8`) and `.with_inline_threshold(usize)` to `CompilationOptions` (`crates/solix-compilation/src/CompilationContext.rs`).
- [ ] `[P1]` **Task 4.2:** Implement `AstInlinerPhase` (`crates/solix-compilation/src/phases/optimizer/ast_inliner.rs`) implementing `CompilationPhase`:
  * Automatically catalogs all non-virtual, non-recursive, non-extern, non-native functions, methods, and constructors whose body statement count is $\le \text{inline\_threshold}$ (default `8`), regardless of whether the `inline` keyword was written.
  * Inlines single-return expression functions (`return expr;`) directly into caller expressions.
  * Inlines multi-statement functions and constructors ($\le 8$ statements) by hygienically renaming callee parameters/locals (`__inl_<call_id>_<name>`) and expanding statements into the caller block.
- [ ] `[P1]` **Task 4.3:** Register `AstInlinerPhase` between `BinderPhase` and `EmitterPhase` in `CompilationPipeline::standard()`.
- [ ] `[P1]` **Task 4.4:** Write comprehensive unit and conformance tests in `crates/solix-compilation/tests/optimizer_inliner_tests.rs` testing:
  * Automatic inlining of functions with 1, 4, and 8 statements.
  * Non-inlining of functions with 9+ statements (unless explicitly marked `inline`).
  * Non-inlining of recursive (`fib`, `ack`) and `virtual` / `override` methods.
  * Hygienic variable renaming avoiding variable shadowing collisions in the caller.

### Phase 5: Post-Emission `BytecodeOptimizerPhase` & Superinstruction Fusion
- [ ] `[P1]` **Task 5.1:** Add superinstructions `AddI32Imm` (`0xD0`), `SubI32Imm` (`0xD1`), `JumpIfLtI32` (`0xD2`), `JumpIfLteI32` (`0xD3`), `JumpIfGtI32` (`0xD4`), `JumpIfGteI32` (`0xD5`), `JumpIfEqI32` (`0xD6`), and `JumpIfNeqI32` (`0xD7`) to `OpCode` in `solix-common`, `disassembler.rs` in `solix-compilation`, and `dispatch.rs` + `fast.rs` in `solix-runtime`.
- [ ] `[P1]` **Task 5.2:** Implement `BytecodeOptimizerPhase` (`crates/solix-compilation/src/phases/optimizer/bytecode_optimizer.rs`) running immediately after `EmitterPhase`:
  * Peephole `Mov` elimination (`Op dst=r_tmp` + `Mov r_local, r_tmp` $\to$ `Op dst=r_local`).
  * Loop-invariant `LoadI32` / `LoadF64` constant hoisting out of backward-jump loop bodies.
  * Peephole fusion of `LoadI32` + `AddI32`/`SubI32` into `AddI32Imm`/`SubI32Imm`.
  * Peephole fusion of `LtI32`/`LteI32`/`GtI32`/`GteI32`/`EqI32`/`NeqI32` + `JumpIfFalse` into `JumpIf*I32`.
  * Exact jump target and `ExceptionHandlerEntry` PC range recalculation.
- [ ] `[P1]` **Task 5.3:** Write unit and integration tests in `crates/solix-compilation/tests/optimizer_bytecode_tests.rs` asserting instruction count reductions and exception table accuracy.

### Phase 6: Empirical Inlining Threshold Sweep ($N = 0 \dots 16$) & Full Benchmark Verification
- [ ] `[P1]` **Task 6.1:** Run an empirical benchmark sweep across `inline_threshold` values $N \in \{0, 2, 4, 6, 8, 12, 16\}$ across all 16 workloads, measuring execution time, instruction count, and `.slxb` binary size, and lock in the optimal default threshold in `CompilationOptions`.
- [ ] `[P0]` **Task 6.2:** Execute `python3 benchmarks/runner.py --verify` and `python3 benchmarks/runner.py --mode parity` to prove 100% mathematical checksum equivalence across Solix Fast, Solix `--safe`, Python, Lua, LuaJIT, and C.
- [ ] `[P1]` **Task 6.3:** Update `BENCHMARK_RESULTS.md` and `benchmarks/baselines/ms017_baseline.json` with the new Milestone 017 performance metrics.

---

## 5. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Priority | Sign-off Status |
| :--- | :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Compiles with 0 warnings across all targets | `cargo build --workspace --all-targets` | `P0` | [ ] Pending |
| **Release Build** | Clean optimized release build | `cargo build --workspace --release` | `P0` | [ ] Pending |
| **Automated Test Suite** | 100% pass across all 700+ unit & conformance tests | `cargo test --workspace --all-targets` | `P0` | [ ] Pending |
| **Benchmark Correctness Gate** | 16/16 workloads match checksums in Fast and `--safe` modes | `python3 benchmarks/runner.py --verify` | `P0` | [ ] Pending |
| **Auto-Inlining ($\le 8$ Stmts)** | Functions/constructors $\le 8$ statements automatically inlined | `cargo test --test optimizer_inliner_tests` | `P1` | [ ] Pending |
| **Bytecode Optimizer Phase** | Loop constants hoisted & superinstructions fused | `cargo test --test optimizer_bytecode_tests` | `P1` | [ ] Pending |
| **Performance vs. Python 3.12** | Beats Python 3.12 on all 16/16 workloads (3x–15x faster) | `python3 benchmarks/runner.py --markdown` | `P1` | [ ] Pending |
| **Clippy & Rustfmt Clean** | Zero warnings and zero formatting violations | `cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --check` | `P0` | [ ] Pending |
| **Documentation Sync** | `BENCHMARK_RESULTS.md` and MS-017 updated | GL-0010 audit checklist | `P1` | [ ] Pending |

---

## 6. Risk Assessment & Mitigation Strategies

* **Risk 1: AST Inlining Variable Name Collisions or Early-Return Control Flow Bugs**
  * *Mitigation:* Every inlined invocation generates a unique monotonic call counter (`__inl_<call_id>_<var>`) for all parameters and local variables declared inside the callee. For multi-statement functions ($\le 8$ statements), automatic AST inlining is restricted to functions where `return` appears only as the final statement of the body (no mid-block conditional early returns), guaranteeing 100% control-flow soundness without complex goto lowering.
* **Risk 2: Bytecode Compaction Invalidating Relative Jump Offsets or `ExceptionHandlerEntry` PCs**
  * *Mitigation:* `BytecodeOptimizerPhase` constructs an explicit `old_pc_to_new_pc: Vec<u32>` mapping table whenever instructions are fused or hoisted. Every `FormatC` branch offset (`target_old_pc = (old_pc as i32) + old_offset`) and every `ExceptionHandlerEntry` (`try_start_pc`, `try_end_pc`, `handler_pc`, `finally_pc`) is deterministically translated through `old_pc_to_new_pc`.
* **Risk 3: Intrusive Quick-Lists Masking Memory Leaks in `verify_clean_memory()`**
  * *Mitigation:* `VirtualHeap::verify_clean_memory()` drains all blocks from `quick_lists` and the unallocated bump region back into `FreeListManager` before checking `active_live_objects == 0 && free_list.is_fully_coalesced()`. Interned constant pool strings are tracked separately in `interned_string_addrs` and released during VM teardown prior to leak verification.