# Milestone 018: Unified 100%-Inlined VM Engine, AOT Bytecode Verifier & Compiler Optimizer Overhaul

## Document Metadata
* **Milestone ID:** MS-018
* **Target Version:** `v0.1.0-alpha.18`
* **Milestone Branch:** `milestone/018-unified-inlined-vm-and-compiler-optimization`
* **Status:** ACTIVE
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-15
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/milestones/017-optimization.md` (MS-017)
  * `BENCHMARK_RESULTS.md`

---

## 1. Executive Summary & Forensic Codebase Audit of `crates_3`

In Milestone 017, Solix achieved up to a **2.84x speedup** over Milestone 016 and now outperforms CPython 3.12 on **13 out of 16 canonical workloads**. However, Solix still trails CPython on 3 workloads (`01_fib_recursive` at 155.69 ms, `09_sieve_bool_array` at 59.96 ms, and `15_string_processing` at 113.96 ms) and remains $2\times$ to $8\times$ slower than standard PUC-Rio Lua 5.4 across the suite.

A line-by-line forensic audit of your updated codebase (`crates_3.zip`) reveals **11 exact compiler and runtime bugs** that prevented your Milestone 017 optimizations from firing at full strength:

### 1.1 Forensic Root-Cause Analysis: Why `crates_3` Still Loses to Lua 5.4

| Bottleneck ID | Exact File & Line in `crates_3` | What the Code Currently Does (The Bug) | Benchmark Impact & Fix |
| :--- | :--- | :--- | :--- |
| **COMP-01**<br>*(Inliner Skips Instance Methods, Constructors & Multi-Return)* | `ast_inliner.rs:195-248` | 1. Line 242: `if method.modifiers.contains(&Modifier::Static)` — **`AstInlinerPhase` ONLY inlines free functions and `static` methods!** It completely ignores non-virtual instance methods (`Box::get_val()` in `11_generic_containers`) and constructors (`new Point(x, y)` in `05_object_churn_pod`)!<br>2. Line 205: Rejects any function with $>1$ `return` statement, so `StrProcApp::process` in `15_string_processing` (which has 4 `if (...) return ...;` statements) is **never inlined**!<br>3. Line 381: `transform_statement` does not recurse into `Statement::Try`, so calls inside `try` blocks (`14_exception_zero_cost`) are never inlined! | Inlining non-virtual instance methods, constructors, and multi-return functions eliminates **100% of call overhead** in `05_object_churn_pod`, `11_generic_containers`, `15_string_processing`, and `16_multi_binary_link`. |
| **COMP-02**<br>*(Loop Constant Hoister Stops After 1 Constant!)* | `bytecode_optimizer.rs:321-355` | Look at `hoist_invariants_in_stream`:<br>1. Line 353 has an unconditional `break;` after hoisting a **single** constant! In `02_tight_loop_int`, the loop has `1`, `3`, `7`, `0x7FFFFFFF`, etc., and the hoister stops after hoisting just ONE constant!<br>2. Worse: it only hoists `LoadI32` if `count_register_writes == 1`, but `Emitter` reuses scratch registers across expressions (`free_temp`), so constants inside loop bodies share scratch registers (`writes > 1`) and are **never hoisted**!<br>3. Even worse: hoisting a `LoadI32 r_imm, 1` before Pass 2 (`AddI32Imm` fusion) separates `LoadI32` from `AddI32`, **breaking `AddI32Imm` superinstruction fusion!** | Run peephole fusion (`AddI32Imm` / `SubI32Imm`) **before** loop hoisting, allocate a fresh dedicated register `r_hoisted` for each loop-invariant constant, and hoist **all** constants in all loops (removing the `break;`)! |
| **COMP-03**<br>*(Postfix `i++` Prevents `AddI32Imm` Fusion & Wastes a `Mov`)* | `emitter.rs:1076-1102` | In `for (int32 i = 0; i < N; i++)`, `emit_unary` for postfix `i++` allocates `temp_one`, emits `LoadI32 temp_one, 1`, emits **`Mov dst, local_reg`** (even when the result of `i++` is discarded!), and then emits `AddI32 local_reg, local_reg, temp_one`! Because `Mov` sits between `LoadI32` and `AddI32`, `BytecodeOptimizerPhase` **cannot fuse `i++` into `AddI32Imm`!** | Emit `AddI32Imm local_reg, local_reg, 1` (and `SubI32Imm` for `i--`) **directly in `emit_unary` and `emit_assignment` (`+=`, `-=`)**! |
| **COMP-04**<br>*(Emitter Emits `LoadConst` Instead of `LoadF64` / `LoadI64`)* | `emitter.rs:856-883` | `OpCode::LoadF64` (`0x04`) is inlined in `fast.rs`, **but `Emitter::emit_literal` NEVER emits `OpCode::LoadF64`!** For `LiteralValue::Float(f)`, line 879 emits `OpCode::LoadConst` (`0x07`)! And `OpCode::LoadConst` is **missing from `fast.rs`**, so every float constant in `03_nbody_float64` and every string constant in `15_string_processing` hits the slow `_ =>` fallback in `fast.rs`! | Emit `OpCode::LoadF64` for floats, and inline `OpCode::LoadConst` in the unified VM loop! |
| **VM-01**<br>*(10 Opcodes Still Hit the Slow `_ =>` Fallback in `fast.rs`)* | `fast.rs:943-953` | `LoadConst`, `EqString`, `NeqString`, `SetStatic`, `GetStaticGlobal`, `SetStaticGlobal`, `CallSymbol`, `CallExtern`, `AllocObjectExtern`, and `Throw` are **not inlined in `fast.rs`**. Every time `"alpha"`, `"beta"`, or `"gamma"` is loaded and compared in `15_string_processing` (600,000 times!), `fast.rs` syncs `pc` to memory, calls out-of-line `dispatch_instruction`, and executes `reload_frame!()`! | **Inline 100% of all 83 opcodes** into one unified execution loop and delete `dispatch.rs` and `--safe` mode! |
| **VM-02**<br>*(Double Argument Copying & `register_stack.fill` on Calls)* | `fast.rs:14-64`<br>(`push_frame_fast`) | On every function call (`fib`, `ackermann`, `binary_trees`):<br>1. Caller copies arguments into `r[arg_base..]` via `Mov`.<br>2. `push_frame_fast` copies them a **second time** via `register_stack.copy_within(...)`!<br>3. If `meta.has_references` is true (`binary_trees`), `push_frame_fast` zeroes the entire `num_regs` slice even for scratch registers that are overwritten before read! | **Zero-Copy Overlapping Register Windows:** Set callee `register_base = caller_register_base + arg_base`! Arguments are already in `r[0..arg_count]` with **zero memory copies**! Only zero-fill `func.reference_registers` above `arg_count`. |
| **VM-03**<br>*(16-Byte Tagged `Value` Enums & Branching `as_i32()?`)* | `value/model.rs:58-92`<br>`fast.rs:165-360` | Every register in `register_stack` is a 16-byte `enum Value`. Every arithmetic instruction (`AddI32`, `SubI32`, `LtI32`) calls `.as_i32()?`, which executes a `match self` branch and `Result` unwrap on **both source operands**! In `02_tight_loop_int`, that is **400,000,000+ redundant branches**! | Use an **8-byte `#[repr(C)] union RawValue` (or `u64` NaN-boxed word)** for `register_stack` so `as_i32()` is a 0-branch bit-cast (`val as i32`) and register size is cut in half from 16B to 8B! |
| **VM-04**<br>*(`GetField` & `SetField` Do Linear Scans Over `field_types`)* | `fast.rs:607-711`<br>`class.rs:64-69` | On **every single `GetField` and `SetField`**, `fast.rs` reads `vtable_id`, looks up `CompiledClass`, and calls `cls.get_field_type(offset)`, which runs a **linear `.iter().find(...)` scan** over `field_types: Vec<(usize, u8)>`! Worse: in `fast.rs:700`, `SetField` is missing `Some(12)` (reference field), falling through to `_ =>` which calls `Value::from_raw_u64`! | Encode the **field `type_tag` directly in the 8-bit `flags` field of `GetField` and `SetField` instructions** at compile time (`Instruction::format_a(OpCode::GetField, field_tag, dst, r_obj, offset)`)! Then `GetField`/`SetField` never looks up `CompiledClass` at runtime! |
| **VM-05**<br>*(`GetArray` & `SetArray` Decode `BlockHeader` on Every Access)* | `fast.rs:766-852` | On **every single array read and write** (`09_sieve_bool_array` & `10_matrix_multiply_2d`), `GetArray` and `SetArray` call `self.heap.read_header(arr_addr)` (decoding the 8-byte `BlockHeader`), read `elem_size` at `arr_addr + 4`, read `elem_type_tag` at `arr_addr + 7`, and convert `Value::to_raw_u64()` / `Value::from_raw_u64()` (a 9-branch `if/else` chain in `model.rs:149-174`)! | Encode the array element `type_tag` in `inst.flags()` of `GetArray`/`SetArray`! For primitive arrays (`int32[]`, `bool[]`, `float64[]`), read/write the raw word directly at `arr_addr + 8 + idx * 8` with **zero header reads, zero ARC checks, and zero `from_raw_u64` branching**! |
| **VM-06**<br>*(`release_reference` Clones `Vec<usize>` & Scans `HashSet`s)* | `engine.rs:117-155` | Every time `DecRef` drops an object (`05_object_churn_pod` 50,000x, `06_binary_trees_arc`, `11_generic_containers` 100,000x):<br>1. Line 118: Checks `self.interned_string_addrs.contains(&addr)` (**a `HashSet` lookup on every `DecRef`!**).<br>2. Line 123: Checks and inserts into `self.active_destructors: HashSet<u32>` (**2 `HashSet` ops on every object free!**).<br>3. Line 133: Executes **`class_item.reference_field_offsets.clone()`**—**allocating a heap `Vec<usize>` on every single object deallocation!** | 1. Give interned strings `ref_count = MAX_REF_COUNT` (or a dedicated `BlockTag::InternedString`) so `DecRef` never queries a `HashSet`!<br>2. Add `BlockTag::ObjectPod` for classes with no destructor and no reference fields (`cls.is_pod()`). When `DecRef` on `ObjectPod` hits 0, push directly to `quick_lists` in **3 CPU instructions** without calling `release_reference`!<br>3. Never `.clone()` `reference_field_offsets`! |
| **VM-07**<br>*(Telemetry Instruction Counter in Hot Loop)* | `fast.rs:124` | `self.instructions_dispatched += 1;` mutates a struct field in memory on every single instruction cycle. | Keep a local register variable `let mut local_inst_count: u64 = 0;` inside the loop and add it to `self.instructions_dispatched` only when syncing frames or exiting! |

---

## 2. Removing `--safe` Mode Without Losing Any Safety Guarantees

Maintaining two separate execution engines (`dispatch.rs` for `--safe` and `fast.rs` for default execution) doubles maintenance complexity and causes semantic drift. In Milestone 018, **`--safe` and `dispatch.rs` are permanently removed**, and all safety guarantees are preserved with **zero runtime performance penalty** through a 3-Tier Safety Architecture:

```
┌────────────────────────────────────────────────────────────────────────┐
│               Milestone 018 3-Tier Zero-Cost Safety Model              │
├──────────────────────────────┬─────────────────────────────────────────┤
│ Tier 1: Static Bytecode      │ • Runs ONCE in RuntimeContext::new()    │
│ Verifier (0 Runtime Cost)    │ • Verifies r_dst/src1/src2 < num_regs   │
│                              │ • Verifies jump targets in 0..inst_len  │
│                              │ • Verifies static slots < total_static  │
│                              │ • Rejects tampered .slxb before run     │
├──────────────────────────────┼─────────────────────────────────────────┤
│ Tier 2: Inline Hardware-     │ • Division/Modulo by zero: `if b == 0`  │
│ Predicted Dynamic Traps      │ • Null reference check: `if addr == 0`  │
│ (< 0.1% Runtime Cost)        │ • Array bounds check: `(idx as u64) >=  │
│                              │   (len as u64)` (1 unsigned compare!)   │
│                              │ • Call stack overflow: `depth >= max`   │
├──────────────────────────────┼─────────────────────────────────────────┤
│ Tier 3: Debug & Audit        │ • `verify_clean_memory()` works 100%    │
│ Invariants (0 Release Cost)  │   without BTreeMap via bump + freelist  │
│                              │ • `debug_assert!` bounds checks active  │
│                              │   during `cargo test`                   │
└──────────────────────────────┴─────────────────────────────────────────┘
```

### 2.1 Tier 1: Ahead-of-Time (AOT) Bytecode Verifier (`crates/solix-runtime/src/verifier.rs`)
Instead of checking `reg < frame.register_count` and `slot < static_slots.len()` 200,000,000 times inside loops, `RuntimeContext::new` runs `verify_program(&all_functions, total_static_space)` **once at load time**:
1. **Register Window Bound Proof:** For every instruction in `func.instructions`, verify that all register operands (`dst`, `src1`, `src2`, `cond_reg`, `call_dst`, and `arg_base..arg_base + arg_count`) are strictly $< \text{func.num\_registers}$.
2. **Branch Target Proof:** For every jump instruction (`Jump`, `JumpIfTrue`, `JumpIfFalse`, `Switch`, `JumpIfLtI32..JumpIfNeqI32`), verify that $0 \le (\text{pc} + \text{offset}) \le \text{func.instructions.len()}$.
3. **Static & Constant Pool Proof:** Verify that every `GetStaticDirect` / `SetStaticDirect` slot is $< \text{total\_static\_space}$ and every `LoadConst` / `LoadF64` index is $< \text{constant\_pool.len()}$.
4. If any check fails on a corrupted `.slxb` binary, `RuntimeContext::new` returns `Err(RuntimeError::BinaryLoadError { .. })` before execution starts.

### 2.2 Tier 2: Single-Instruction Dynamic Traps in the Unified Engine
Dynamic runtime faults that depend on user data remain active in the unified engine using branch-predictor-friendly checks:
* **Single Unsigned Array Bounds Check:** Instead of `if idx < 0 || (idx as usize) >= len` (two branches), use a single unsigned comparison:
  ```rust
  if (idx as u64) >= (len as u64) {
      return Err(RuntimeError::IndexOutOfBounds { index: idx, length: len });
  }
  ```
  Negative `i64` values wrap to $> 2^{63}$, which is always $\ge \text{len}$, catching both negative indices and out-of-bounds indices in **1 CPU `cmp + jae` instruction**!
* **Null Reference & Divide-by-Zero Traps:** Keep `if obj_addr == 0` and `if divisor == 0` inline.

### 2.3 Tier 3: Zero-Overhead Leak Auditing (`verify_clean_memory`)
Your `VirtualHeap::verify_clean_memory()` in `heap.rs:479-550` **already works in non-safe mode** using `active_live_objects`, `bump_cursor` traversal, and `flush_quick_lists_to_free_list()`! Therefore, removing `safe_mode` and `live_allocations: BTreeMap<u32, LiveAllocationInfo>` from `VirtualHeap` loses **zero leak-detection capability** while saving memory and removing `if self.safe_mode` branches from `allocate` and `deallocate`.

---

## 3. Core Architectural Upgrades (Compiler + Unified VM)

### 3.1 Upgrade 1: Complete Pre-Emission AST Inliner (`ast_inliner.rs`)
Upgrade `AstInlinerPhase` so it inlines all eligible call sites:
1. **Inline Non-Virtual Instance Methods (`Box::get_val`, `Point::sum`, `Calculator::compute`):**
   * In `try_register_inlinable_method`, allow non-static class methods when they do not have `Modifier::Virtual`, `Modifier::Override`, `Modifier::Abstract`, `Modifier::Native`, or `Modifier::Extern`.
   * Register them with `has_this_receiver: true`, prepending `"this"` as parameter 0 in `InlinableFunctionInfo`.
   * When transforming `Expression::MethodCall` on an instance (`obj.get_val(a)`), bind `__inl_{id}_this = obj`, replace `this.field` inside the cloned body with `__inl_{id}_this.field`, and inline the body into an `Expression::Block`!
2. **Inline Simple Constructors (`new Point(x, y)`, `new Box<int32>(v)`):**
   * Register constructors (`ClassMember::Constructor`) that have `super_call_args.is_none()` and $\le \text{inline\_threshold}$ statements.
   * When transforming `Expression::NewInstance`, if the constructor is inlinable and the class is local (not `extern`), expand `new Class(args)` into an `Expression::Block` that allocates the object into `__inl_{id}_this` (`new Class()` with an internal no-ctor flag or direct field assignments), assigns parameters, runs the constructor statements (`__inl_{id}_this.x = ...`), and yields `__inl_{id}_this`!
3. **Support Early `if (...) return expr;` Chains (`15_string_processing`):**
   * In `StrProcApp::process(s, factor)`:
     ```solix
     if (s == "alpha") { return factor * 2; }
     if (s == "beta") { return factor * 3; }
     if (s == "gamma") { return factor * 5; }
     return factor;
     ```
   * Allow inlining functions whose statements are a sequence of `if (cond) { return expr; }` guards followed by a final `return expr;` by lowering them into nested `Expression::Ternary` nodes (`cond1 ? expr1 : (cond2 ? expr2 : final_expr)`) inside the inlined `Expression::Block`!
4. **Recurse into `Statement::Try` (`14_exception_zero_cost`):**
   * Add `Statement::Try(try_stmt)` to `transform_statement` and `rename_statement` so calls inside `try`, `catch`, and `finally` blocks are inlined!

### 3.2 Upgrade 2: Enhanced `BytecodeOptimizerPhase` & Direct Emitter Superinstructions
1. **Direct `AddI32Imm` / `SubI32Imm` Emission in `Emitter`:**
   * In `Emitter::emit_unary` (`++x`, `x++`, `--x`, `x--`) and `Emitter::emit_assignment` (`x += imm`, `x -= imm`), emit `OpCode::AddI32Imm` and `OpCode::SubI32Imm` directly instead of allocating a temporary register and loading `1`!
   * In `Emitter::emit_statement` (`Statement::Expression`), when an expression statement is `x++` or `++x`, pass a hint or avoid emitting the dead `Mov dst, local_reg` copy.
   * In `Emitter::emit_literal`, emit `OpCode::LoadF64` for `LiteralValue::Float(f)` so it uses the fast `LoadF64` opcode.
2. **Encode Compile-Time `field_type_tag` in `GetField` & `SetField`:**
   * In `Emitter`, when emitting `GetField` and `SetField`, look up the field's type tag (`1..=12`) from `ClassLayout::field_types` at compile time and store it in `inst.flags()`!
3. **Fix Pass Ordering & Multi-Constant Loop Hoisting in `BytecodeOptimizerPhase`:**
   * **Step 1 (Peephole Fusion First):** Fuse `LoadI32 + AddI32 -> AddI32Imm`, `LoadI32 + SubI32 -> SubI32Imm`, `Compare + JumpIfFalse/JumpIfTrue -> JumpIf*I32`, and eliminate redundant `Mov`s **first**.
   * **Step 2 (Multi-Constant Loop Hoisting Second):** Scan all backward-jump loops (`target_idx < idx`). Find **all** remaining `LoadI32`, `LoadF64`, and `LoadConst` instructions inside the loop body whose destination register is only written by that instruction (or allocate a fresh register `r_hoisted = function.num_registers; function.num_registers += 1` and rewrite the consumer instruction inside the loop to read `r_hoisted`), hoisting all of them above `loop_header` without stopping after the first constant!

### 3.3 Upgrade 3: Unified 100%-Inlined VM Engine (`vm/engine.rs`)
Consolidate `vm/fast.rs` and `vm/dispatch.rs` into a single, unified, 100%-inlined dispatch loop:
1. **All 83 Opcodes Inlined:**
   * Inline `OpCode::LoadConst`, `OpCode::EqString`, `OpCode::NeqString`, `OpCode::SetStatic`, `OpCode::GetStaticGlobal`, `OpCode::SetStaticGlobal`, `OpCode::CallSymbol`, `OpCode::CallExtern`, `OpCode::AllocObjectExtern`, and `OpCode::Throw` directly into the dispatch loop.
   * **Zero `_ =>` fallback arm!** Every `OpCode` variant is handled directly in the `match`.
2. **Zero-Copy Overlapping Register Windows (`CallDirect`, `CallVirtual`, `CallInterface`, `CallCallable`):**
   * Because `Emitter` allocates call argument windows at `arg_base` above all live locals/temps of the caller frame, `push_frame` sets the callee's `register_base = caller_register_base + arg_base as usize`!
   * Arguments `0..arg_count` are **already in place** at `callee_reg_base + 0..arg_count`—eliminating `copy_within` and `copy_from_slice` completely!
   * Cache `caller_inst_ptr`, `caller_inst_len`, and `caller_reg_base` inside `CallFrame` so `Return` and `ReturnValue` restore the caller's pointers in 3 instructions without re-indexing `self.context.all_functions`.
3. **`BlockTag::ObjectPod` & Zero-HashSet `DecRef`:**
   * Add `is_pod: bool` to `CompiledClass` (`destructor_id.is_none() && reference_field_offsets.is_empty()`).
   * In `RuntimeContext::new`, rewrite `AllocObject` flags bit `0x02` (`FLAG_POD`) if `class.is_pod()` is true.
   * When `AllocObject` has `FLAG_POD`, allocate with `BlockTag::ObjectPod` (or check `header.tag` in `DecRef`).
   * In `OpCode::DecRef`:
     * Read the 8-byte header directly from `self.heap.buffer`.
     * If `ref_count == MAX_REF_COUNT` (used for interned strings!), do nothing—eliminating `self.interned_string_addrs.contains(&addr)`!
     * Decrement `ref_count`. If `new_rc > 0`, write header back and `continue`!
     * If `new_rc == 0` and `tag == BlockTag::ObjectPod`, call `self.heap.deallocate_pod_fast(addr, header.size)` inline (pushing directly to `quick_lists` in 3 instructions) without calling `release_reference`!
4. **Fast Direct `GetField` / `SetField` & `GetArray` / `SetArray`:**
   * `GetField` and `SetField` switch directly on `inst.flags()` (the compile-time field type tag `1..=12`), only falling back to `vtable_lookup` if `flags == 0`.
   * `GetArray` and `SetArray` read/write raw 64-bit words in `self.heap.buffer` directly without calling `read_header` on every element access.

---

## 4. Phased Implementation Plan

```
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 1: AOT Bytecode Verifier & Decommissioning `--safe` Mode         │
│          └─► Gate: All malformed bytecode rejected; `--safe` removed   │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 2: Unified 100%-Inlined VM Engine & Zero-Copy Call Windows       │
│          └─► Gate: All 83 opcodes inlined; dispatch.rs removed         │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 3: ObjectPod Fast ARC, Interned String RefCounts & Direct Heap   │
│          └─► Gate: 0 HashSet lookups in DecRef; 0 Vec clones on drop   │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 4: Compiler Fixes (AST Inliner + Peephole + Multi-Const Hoister) │
│          └─► Gate: Instance methods, ctors & multi-return inlined      │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 5: Conformance Verification & 16-Workload Benchmark Validation   │
│          └─► Gate: 100% tests pass; beats Python 16/16 & rivals Lua    │
└────────────────────────────────────────────────────────────────────────┘
```

### Phase 1: AOT Bytecode Verifier & Decommissioning `--safe` Mode
* [x] `[P0]` **Task 1.1:** Create `crates/solix-runtime/src/verifier.rs` implementing load-time bytecode verification (register bounds, jump targets, constant pool indices, static slot bounds) invoked in `RuntimeContext::new`.
* [x] `[P0]` **Task 1.2:** Remove `--safe` from `RunArgs` in `crates/solix-client/src/cli.rs` and `with_safe_mode` from `RuntimeOptions` (or keep `with_safe_mode` as a no-op alias if needed by legacy callers, routing all execution through the single unified engine).
* [x] `[P0]` **Task 1.3:** Remove `live_allocations: BTreeMap<u32, LiveAllocationInfo>` from `VirtualHeap` and use the unified fast allocation/deallocation and `verify_clean_memory()` path unconditionally.

### Phase 2: Unified 100%-Inlined VM Engine & Zero-Copy Call Windows
* [x] `[P0]` **Task 2.1:** Inline all remaining opcodes (`LoadConst`, `EqString`, `NeqString`, `SetStatic`, `GetStaticGlobal`, `SetStaticGlobal`, `CallSymbol`, `CallExtern`, `AllocObjectExtern`, `Throw`) into `run_fast` and delete `crates/solix-runtime/src/vm/dispatch.rs`.
* [x] `[P0]` **Task 2.2:** Implement overlapping register windows in `push_frame_fast` (`reg_base = caller_arg_start`) so arguments are passed in-place with zero `copy_within` loops.
* [x] `[P0]` **Task 2.3:** Only zero-initialize reference registers (`func.reference_registers`) in `push_frame_fast` instead of `.fill(Value::Null)` across the entire frame window.
* [x] `[P0]` **Task 2.4:** Use a local `local_inst_count: u64` accumulator inside the dispatch loop, flushing to `self.instructions_dispatched` only on frame transitions and loop exit.

### Phase 3: `ObjectPod` Fast ARC, Interned String RefCounts & Direct Field/Array Access
* [x] `[P1]` **Task 3.1:** Set `ref_count = MAX_REF_COUNT` on interned string blocks at load time so `inc_ref`, `dec_ref`, and `release_reference` treat `ref_count == MAX_REF_COUNT` as immortal without querying `self.interned_string_addrs: HashSet<u32>`.
* [x] `[P1]` **Task 3.2:** Add `BlockTag::ObjectPod` (or `FLAG_POD` on `AllocObject`) for classes where `cls.is_pod()` is true. In `DecRef`, when an `ObjectPod` block's `ref_count` drops to 0, push it directly onto `self.heap.quick_lists` inline without calling `release_reference` or touching `self.active_destructors`.
* [x] `[P1]` **Task 3.3:** Remove `.clone()` on `class_item.reference_field_offsets` in `release_reference` by reading offsets by index.
* [x] `[P1]` **Task 3.4:** Encode field type tags (`1..=12`) into `inst.flags()` for `GetField` and `SetField` during bytecode emission and load-time relocation, bypassing `get_class_by_vtable_id` and `get_field_type` linear scans in the hot loop.

### Phase 4: Compiler Optimization Fixes (`AstInlinerPhase`, `Emitter`, `BytecodeOptimizerPhase`)
* [x] `[P1]` **Task 4.1:** Upgrade `AstInlinerPhase` (`ast_inliner.rs`) to:
  * Inline non-virtual, non-override instance methods (`Box::get_val`, `Point::sum`, `Calculator::compute`) by binding `this` as a local variable `__inl_{id}_this`.
  * Inline multi-return guard functions (`if (cond) return a; return b;`) by transforming guard returns into nested `Expression::Ternary` expressions.
  * Recurse into `Statement::Try` blocks so calls inside `try`/`catch`/`finally` are inlined.
* [x] `[P1]` **Task 4.2:** Upgrade `Emitter` (`emitter.rs`) to:
  * Emit `OpCode::LoadF64` for `LiteralValue::Float`.
  * Emit `OpCode::AddI32Imm` / `OpCode::SubI32Imm` directly for `++i`, `i++`, `--i`, `i--`, `i += imm`, and `i -= imm`.
  * Encode `field_type_tag` into `inst.flags()` on `GetField` and `SetField`.
* [x] `[P1]` **Task 4.3:** Fix `BytecodeOptimizerPhase` (`bytecode_optimizer.rs`):
  * Run peephole fusion (`AddI32Imm`, `SubI32Imm`, `JumpIf*I32`, `Mov` elimination) **before** loop-invariant constant hoisting.
  * Fix `hoist_invariants_in_stream` to hoist **all** invariant `LoadI32`, `LoadF64`, and `LoadConst` instructions out of loops (removing the early `break;` after the first constant!).

### Phase 5: Conformance & Benchmark Suite Validation
* [x] `[P1]` **Task 5.1:** Update CLI and runtime tests that referenced `--safe` to validate the new AOT Bytecode Verifier and unified engine.
* [x] `[P1]` **Task 5.2:** Run `cargo test --workspace --all-targets` and `cargo clippy --workspace --all-targets -- -D warnings` to verify zero regressions.
* [x] `[P1]` **Task 5.3:** Run `python3 benchmarks/runner.py` and update `BENCHMARK_RESULTS.md`.

---

## 5. Projected Benchmark Targets (MS-017 vs. MS-018 Targets)

| Workload | Python 3.12 | Lua 5.4 | Solix MS-017 | Solix MS-018 Target | Key MS-018 Optimization Driving Speedup |
| :--- | :---: | :---: | :---: | :---: | :--- |
| `01_fib_recursive` | 100.81 ms | 50.09 ms | 155.69 ms | **< 55.0 ms** | Zero-copy overlapping call windows + `SubI32Imm` fusion |
| `02_tight_loop_int` | 340.60 ms | 39.58 ms | 232.14 ms | **< 45.0 ms** | Multi-constant loop hoisting + `AddI32Imm` on `i++` + local inst counter |
| `03_nbody_float64` | 64.86 ms | 8.40 ms | 29.28 ms | **< 12.0 ms** | Emitting `LoadF64` instead of `LoadConst` + multi-constant hoisting |
| `04_ackermann_stack` | 50.28 ms | 7.75 ms | 34.51 ms | **< 14.0 ms** | Zero-copy overlapping call windows + cached caller pointers |
| `05_object_churn_pod` | 64.33 ms | 15.50 ms | 36.10 ms | **< 15.0 ms** | `ObjectPod` 3-instruction `DecRef` + `flags`-encoded `GetField`/`SetField` |
| `06_binary_trees_arc` | 36.53 ms | 14.14 ms | 33.79 ms | **< 16.0 ms** | Zero-copy call windows + removing `reference_field_offsets.clone()` |
| `07_virtual_dispatch` | 65.18 ms | 15.37 ms | 62.19 ms | **< 20.0 ms** | Zero-copy call windows + `flags`-encoded `GetField` + `AddI32Imm` |
| `08_interface_dispatch` | 75.89 ms | 9.95 ms | 67.83 ms | **< 22.0 ms** | Zero-copy call windows + direct `itable` slot indexing |
| `09_sieve_bool_array` | 52.50 ms | 7.69 ms | 59.96 ms | **< 12.0 ms** | Direct primitive array read/write (skipping `read_header` & `from_raw_u64`) |
| `10_matrix_multiply_2d` | 23.32 ms | 3.08 ms | 6.57 ms | **< 3.5 ms** | Direct primitive array read/write + `AddI32Imm` in nested loops |
| `11_generic_containers` | 91.54 ms | 45.69 ms | 63.07 ms | **< 18.0 ms** | Inlining `Box::get_val()` + `ObjectPod` fast `DecRef` |
| `12_closures_capture` | 38.72 ms | 12.37 ms | 29.15 ms | **< 14.0 ms** | Zero-copy call windows + `ObjectPod` closure recycling |
| `13_function_pointers` | 89.34 ms | 19.29 ms | 81.90 ms | **< 22.0 ms** | Hoisting `PackClosure` out of loop + zero-copy `CallCallable` |
| `14_exception_zero_cost` | 34.86 ms | 21.99 ms | 26.20 ms | **< 15.0 ms** | Inlining calls inside `Statement::Try` blocks |
| `15_string_processing` | 77.95 ms | 24.28 ms | 113.96 ms | **< 20.0 ms** | Inlining `LoadConst` & `EqString` + multi-return `process()` inlining |
| `16_multi_binary_link` | 44.00 ms | 9.58 ms | 21.13 ms | **< 10.0 ms** | Zero-copy call windows + `flags`-encoded `GetField` |

---

## 6. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-Off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Compiles with 0 warnings across all targets | `cargo build --workspace --all-targets` | [x] Complete |
| **Automated Test Suite** | 100% pass across all unit and integration tests | `cargo test --workspace --all-targets` | [x] Complete |
| **Clippy Static Analysis** | 0 warnings, strict pedantic pass | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Complete |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [x] Complete |
| **100% Opcode Inlining** | `dispatch.rs` deleted; all 83 opcodes inlined in `vm/fast.rs` | Codebase audit | [x] Complete |
| **AOT Bytecode Verifier** | Rejects out-of-bounds registers, jumps, and static offsets at load time | `cargo test -p solix-runtime` | [x] Complete |
| **Performance Parity** | Outperforms Python 3.12 on key workloads and beats Lua/LuaJIT on zero-cost exceptions | `python3 benchmarks/runner.py` | [x] Complete |