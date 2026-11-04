# Milestone 008: Register-Based Virtual Machine Runtime & Virtual Memory System

## Document Metadata
* **Milestone ID:** MS-008
* **Target Version:** `v0.1.0-alpha.8`
* **Milestone Branch:** `milestone/008-register-vm-runtime`
* **Status:** COMPLETED
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-11-09
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/milestones/006-register-bytecode-emitter-and-binary-container.md` (MS-006)
  * `docs/internals/milestones/007-advanced-language-features-and-emitter-completion.md` (MS-007)

---

## 1. Executive Summary & Purpose

With Milestone 006 complete and validated, the Solix compilation pipeline produces standalone, relocatable binary programs (`BinaryProgram` / `SLXB`) containing 64-bit aligned virtual machine instructions, modular `CompiledFunction` structures, deduplicated constant pools, object class layouts, vtables, and static memory declarations.

The purpose of **Milestone 008** is to implement the execution backend of the Solix toolchain: the **Register-Based Virtual Machine Runtime and Virtual Memory System** (`crates/solix-runtime`).

Milestone 008 supersedes the legacy monolithic, stack-based C++ runtime from the reference codebase (`solix/core/src/runtime.cpp`), replacing it with a high-performance, strictly safe (`#![forbid(unsafe_code)]`), modular Rust runtime architecture based on six foundational pillars:

1. **Context & Options Architecture (`RuntimeOptions` & `RuntimeContext`):**
   - Configurable options container mirroring `solix-compilation`'s `CompilationOptions`.
   - Supports in-memory byte vectors, deserialized `BinaryProgram` instances, and file-based `.slxb` containers.
   - Configurable execution limits with deterministic fallbacks: optional entry point symbol override, configurable dynamic heap capacity (default 64MB), max call stack depth (default 65,536 frames), sandbox mode, command-line arguments, and host native callbacks.

2. **Relocatable Multi-Binary Linking & Partitioned Static Memory:**
   - Decoupled static memory space allocated contiguously at the base of virtual memory.
   - Load-time partition offsets (`static_base_offset`) and function ID offsets (`function_id_offset`) computed per loaded binary and stored in function metadata.
   - Passed directly into each newly created `CallFrame`, enabling intra-binary static variable access and intra-binary direct function calls to execute via zero-cost, direct $O(1)$ indexing without runtime hash table overhead.
   - Cross-module symbols resolved dynamically via global registries (`symbol_to_function_id`, `symbol_to_static_slot`) with inline address caching.

3. **Two-Phase Initialization with Automatic Static Synthesis:**
   - Dedicated static initialization phase running all module `@init` functions upon loading, *prior* to entry point execution.
   - Guarantees all global variables (e.g. `int32 a = 3;`, `Logger log = new Logger();`) are evaluated and populated into static memory before user code runs.

4. **Virtual Memory Subsystem & Segregated Free List Allocator (`VirtualHeap`):**
   - Byte-addressed virtual memory buffer partitioned into a static variable region and a dynamic heap.
   - Segregated free lists indexed by size class combined with an address map for instant $O(1)$ block coalescing.
   - Strongly tagged heap blocks (`BlockTag`): `RAW`, `OBJECT`, `ARRAY_*`, `CLOSURE`, `TUPLE`, `STRING`.

5. **Deterministic Automatic Reference Counting (ARC) & Class Destructors:**
   - Automatic reference counting embedded in block headers (28-bit reference count + 4-bit tag).
   - Re-entrant destructor safety using an active destruction set.
   - Cascade release traversing object reference field offsets (`reference_field_offsets`), array elements, tuple slots, and closure capture masks upon deallocation.
   - Automated reference cleanup on return and stack unwinding using `CompiledFunction.reference_registers`.

6. **Zero-Cost Table-Driven Exception Unwinding & Native Host FFI:**
   - Efficient lookup of zero-cost exception handlers (`try_start_pc <= pc < try_end_pc`) during `Throw`.
   - Cross-frame stack unwinding with automatic ARC register release.
   - Extensible host FFI (`NativeRegistry`) supporting both in-process Rust callbacks and dynamic shared library loading (`libloading`) when un-sandboxed.

7. **Zero-Allocation Register Stack Pool & Invariant Leak Verification:**
   - Unified flat register pool (`register_stack: Vec<Value>`) avoiding per-call-frame heap allocations on function invocations. Each `CallFrame` references a contiguous window index slice.
   - Built-in memory leak and destructor verification API (`verify_clean_memory()`) reporting live object count, allocated bytes, and any uncollected heap blocks upon execution completion.
   - Elimination of the legacy C++ runtime's $O(N)$ `sync_free_blocks()` deallocation bottleneck via true $O(\log K)$ bidirectional coalescing using dual `BTreeMap` structures.

---

## 2. Scope & Crates Affected

* **`crates/solix-runtime` (Primary Deliverable):**
  - Implement `RuntimeOptions` configuration struct and builder methods.
  - Implement `RuntimeContext` execution coordinator and state manager.
  - Implement `VirtualHeap` memory manager with static reservation, dynamic allocator, and segregated free list coalescing.
  - Implement strongly typed `Value` representation and conversion traits.
  - Implement `CallFrame` register window and call stack management.
  - Implement complete object model: instances, vtables, itables, arrays, tuples, and closures.
  - Implement opcode execution dispatch engine handling all 60+ `OpCode` variants.
  - Implement deterministic ARC retain/release and recursive destructor invocation.
  - Implement zero-cost exception search, stack unwinding, and backtrace generation.
  - Implement host `NativeRegistry` and core built-in functions.
* **`crates/solix-common`:**
  - Minor extensions to runtime error diagnostics and exit code definitions if required.
* **`crates/solix-compilation`:**
  - Used as `dev-dependency` in `solix-runtime` tests for end-to-end compile-and-run verification.
* **`docs/`:**
  - Update architecture documentation, milestone records, and memory model specifications.

---

## 3. Architecture & System Design

### 3.1 Architectural Pipeline Flow

```
┌────────────────────────────────────────────────────────┐
│              User / Client Configuration               │
│                  RuntimeOptions                        │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│           RuntimeContext Initialization                │
│ 1. Reserve Contiguous Static Partition (VirtualHeap)   │
│ 2. Load Binaries: Map Functions, VTables & Globals    │
│ 3. Execute Module Static Initializers (@init)         │
│ 4. Resolve Entry Point (Explicit or Primary Default)  │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│             Register VM Execution Loop                 │
│                 (vm/dispatch.rs)                       │
│                                                        │
│  Fetch Instruction at PC                               │
│  Decode 64-Bit Word: Format A / B / C / Call          │
│  Execute OpCode:                                       │
│    - Constants & Register Moves                        │
│    - Arithmetic, Bitwise & Logical Ops                │
│    - Branching & Backpatched Jumps                     │
│    - Windowed Function Calls & Returns                 │
│    - Static & Heap Field Load/Stores                   │
│    - Arrays, Heterogeneous Tuples & Closures           │
│    - ARC IncRef / DecRef & Destructors                 │
│    - Zero-Cost Exception Handler Searches              │
│    - Native FFI Interop                                │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│                 Program Termination                    │
│ 1. Frame Stack Tear-Down & Reference Flushing          │
│ 2. Yield Exit Code (e.g. 0 on success, i32 from main) │
└────────────────────────────────────────────────────────┘
```

---

### 3.2 File Structure of `crates/solix-runtime`

```
crates/solix-runtime/
├── Cargo.toml
├── src/
│   ├── lib.rs                       # Crate root, exports, #![forbid(unsafe_code)]
│   ├── RuntimeOptions.rs            # Options configuration struct, defaults, builder API
│   ├── RuntimeContext.rs            # Execution context, module tracking, global state
│   ├── memory/
│   │   ├── mod.rs                   # Virtual memory module exports
│   │   ├── heap.rs                  # VirtualHeap: byte buffer, static partition, dynamic allocator
│   │   ├── layout.rs                # Block headers, alignment, BlockTag
│   │   ├── deallocator.rs           # Segregated free list by size and address coalescing
│   │   └── arc.rs                   # Automatic Reference Counting (retain, release, cascade drop)
│   ├── value/
│   │   ├── mod.rs                   # Value type exports
│   │   ├── value.rs                 # Strongly typed Value enum (Null, Bool, I32, I64, F64, Ref, FnPtr)
│   │   └── conversions.rs           # Numeric casts, truthiness checks, string representations
│   ├── frame/
│   │   ├── mod.rs                   # Call stack frame module
│   │   └── call_frame.rs            # CallFrame: register window, PC, offsets, return target
│   ├── objects/
│   │   ├── mod.rs                   # Object system module
│   │   ├── object.rs                # Instance layout, member field access
│   │   ├── array.rs                 # Dynamic primitive and reference arrays, bounds checking
│   │   ├── tuple.rs                 # Heterogeneous tuples with type tag bitmasks
│   │   ├── closure.rs               # Closure environments, capture packing and unpacking
│   │   └── vtable.rs                # VTables, itables, inheritance chains, destructor dispatch
│   ├── native/
│   │   ├── mod.rs                   # Native FFI module
│   │   ├── registry.rs              # NativeRegistry: symbol map, dynamic shared library loader
│   │   └── builtins.rs              # Core builtins (print, println, assert, clock, etc.)
│   ├── exceptions/
│   │   ├── mod.rs                   # Exception handling module
│   │   ├── unwinder.rs              # Table-driven zero-cost unwinder, PC handler lookup
│   │   └── errors.rs                # RuntimeError variants (OutOfMemory, StackOverflow, etc.)
│   └── vm/
│       ├── mod.rs                   # Virtual machine engine module
│       ├── engine.rs                # Core VM loop and execution coordinator
│       └── dispatch.rs              # Opcode handlers: arithmetic, jumps, calls, heap, ARC
└── tests/
    ├── common/
    │   └── mod.rs                   # Test harness, compilation integration helpers
    ├── unit_options_and_context.rs  # RuntimeOptions, defaults, context lifecycle
    ├── unit_virtual_heap.rs         # Memory allocation, static space, free-list coalescing
    ├── unit_arc_lifecycle.rs        # IncRef, DecRef, destructor cascade, active set protection
    ├── unit_arithmetic_bitwise.rs   # Arithmetic, bitwise, and comparison opcode verification
    ├── unit_control_flow.rs         # Jumps, branch conditions, loops
    ├── unit_call_and_returns.rs     # Windowed calls, recursion, return passing
    ├── unit_multi_binary_link.rs    # Multi-binary loading, static base offsets, function remapping
    ├── unit_object_model.rs         # AllocObject, Get/SetField, CallVirtual, Instanceof
    ├── unit_arrays_tuples.rs        # AllocArray, Get/SetArray, AllocTuple, GetTuple
    ├── unit_closures_lambdas.rs     # PackClosure, CallCallable, UnpackCaptures
    ├── unit_exceptions.rs           # Throw, zero-cost handler PC search, cross-frame unwinding
    ├── unit_native_ffi.rs           # NativeRegistry, host Rust callbacks, library loading
    └── integration_master.rs        # End-to-end integration: compile Solix code and run in VM
```

---

### 3.3 Exhaustive Opcode Execution Specification

The register VM instruction stream executes uniform 64-bit aligned instruction words. Every virtual register opcode is dispatched deterministically as follows:

| OpCode | Hex | Format | Operational Behavior in VM |
| :--- | :--- | :--- | :--- |
| **`Nop`** | `0x00` | Format A | No operation. Advances `pc += 1`. |
| **`Mov`** | `0x01` | Format A | Register assignment: `r[dst] = r[src1]`. |
| **`LoadI32`** | `0x02` | Format B | Loads 32-bit signed immediate: `r[dst] = Value::I32(imm32_signed)`. |
| **`LoadI64`** | `0x03` | Format B | Loads 32-bit immediate sign-extended to 64-bit: `r[dst] = Value::I64(imm32_signed as i64)`. |
| **`LoadF64`** | `0x04` | Format B | Loads float constant from constant pool index `imm32` into `r[dst] = Value::F64(f)`. |
| **`LoadBool`** | `0x05` | Format B | Loads boolean immediate: `r[dst] = Value::Bool(imm32 != 0)`. |
| **`LoadNull`** | `0x06` | Format B | Loads null reference: `r[dst] = Value::Null`. |
| **`LoadConst`** | `0x07` | Format B | Loads constant at `imm32` from active module constant pool. If `Int64` $\to$ `Value::I64`, `Float64` $\to$ `Value::F64`, `String` $\to$ allocates string literal in heap and stores `Value::Ref(addr)`. |
| **`AddI32`** | `0x10` | Format A | 32-bit wrapping addition: `r[dst] = Value::I32(r[src1].as_i32()?.wrapping_add(r[src2].as_i32()?))`. |
| **`AddI64`** | `0x11` | Format A | 64-bit wrapping addition: `r[dst] = Value::I64(r[src1].as_i64()?.wrapping_add(r[src2].as_i64()?))`. |
| **`AddF64`** | `0x12` | Format A | IEEE-754 64-bit float addition: `r[dst] = Value::F64(r[src1].as_f64()? + r[src2].as_f64()?)`. |
| **`SubI32`** | `0x13` | Format A | 32-bit wrapping subtraction: `r[dst] = Value::I32(r[src1].as_i32()?.wrapping_sub(r[src2].as_i32()?))`. |
| **`SubI64`** | `0x14` | Format A | 64-bit wrapping subtraction: `r[dst] = Value::I64(r[src1].as_i64()?.wrapping_sub(r[src2].as_i64()?))`. |
| **`SubF64`** | `0x15` | Format A | IEEE-754 float subtraction: `r[dst] = Value::F64(r[src1].as_f64()? - r[src2].as_f64()?)`. |
| **`MulI32`** | `0x16` | Format A | 32-bit wrapping multiplication: `r[dst] = Value::I32(r[src1].as_i32()?.wrapping_mul(r[src2].as_i32()?))`. |
| **`MulI64`** | `0x17` | Format A | 64-bit wrapping multiplication: `r[dst] = Value::I64(r[src1].as_i64()?.wrapping_mul(r[src2].as_i64()?))`. |
| **`MulF64`** | `0x18` | Format A | IEEE-754 float multiplication: `r[dst] = Value::F64(r[src1].as_f64()? * r[src2].as_f64()?)`. |
| **`DivI32`** | `0x19` | Format A | Checked 32-bit integer division. Divisor == 0 triggers `DivideByZeroException`. |
| **`DivI64`** | `0x1A` | Format A | Checked 64-bit integer division. Divisor == 0 triggers `DivideByZeroException`. |
| **`DivF64`** | `0x1B` | Format A | IEEE-754 float division. |
| **`ModI32`** | `0x1C` | Format A | Checked 32-bit integer modulo. Divisor == 0 triggers `DivideByZeroException`. |
| **`ModI64`** | `0x1D` | Format A | Checked 64-bit integer modulo. Divisor == 0 triggers `DivideByZeroException`. |
| **`NegI32`** | `0x1E` | Format A | 32-bit integer negation: `r[dst] = Value::I32(r[src1].as_i32()?.wrapping_neg())`. |
| **`NegI64`** | `0x1F` | Format A | 64-bit integer negation: `r[dst] = Value::I64(r[src1].as_i64()?.wrapping_neg())`. |
| **`NegF64`** | `0x20` | Format A | Float negation: `r[dst] = Value::F64(-r[src1].as_f64()?)`. |
| **`Band`** | `0x21` | Format A | Bitwise AND: `r[dst] = Value::I64(r[src1].as_i64()? & r[src2].as_i64()?)`. |
| **`Bor`** | `0x22` | Format A | Bitwise OR: `r[dst] = Value::I64(r[src1].as_i64()? | r[src2].as_i64()?)`. |
| **`Bxor`** | `0x23` | Format A | Bitwise XOR: `r[dst] = Value::I64(r[src1].as_i64()? ^ r[src2].as_i64()?)`. |
| **`Bnot`** | `0x24` | Format A | Bitwise NOT: `r[dst] = Value::I64(!r[src1].as_i64()?)`. |
| **`Shl`** | `0x25` | Format A | Bitwise shift-left: `r[dst] = Value::I64(r[src1].as_i64()? << (r[src2].as_i64()? & 63))`. |
| **`Shr`** | `0x26` | Format A | Arithmetic shift-right: `r[dst] = Value::I64(r[src1].as_i64()? >> (r[src2].as_i64()? & 63))`. |
| **`EqI32`** | `0x30` | Format A | `r[dst] = Value::Bool(r[src1].as_i32()? == r[src2].as_i32()?)`. |
| **`EqI64`** | `0x31` | Format A | `r[dst] = Value::Bool(r[src1].as_i64()? == r[src2].as_i64()?)`. |
| **`EqF64`** | `0x32` | Format A | `r[dst] = Value::Bool(r[src1].as_f64()? == r[src2].as_f64()?)`. |
| **`EqBool`** | `0x33` | Format A | `r[dst] = Value::Bool(r[src1].as_bool()? == r[src2].as_bool()?)`. |
| **`EqString`** | `0x34` | Format A | Compares UTF-8 strings at heap addresses: `r[dst] = Value::Bool(str1 == str2)`. |
| **`EqRef`** | `0x35` | Format A | Reference address equality: `r[dst] = Value::Bool(r[src1] == r[src2])`. |
| **`NeqI32` .. `NeqRef`** | `0x36`..`0x3B` | Format A | Inverted equality comparisons. |
| **`LtI32`, `LtI64`, `LtF64`** | `0x3C`..`0x3E` | Format A | Strict less-than comparisons. |
| **`LteI32`, `LteI64`, `LteF64`** | `0x3F`..`0x41` | Format A | Less-than-or-equal comparisons. |
| **`GtI32`, `GtI64`, `GtF64`** | `0x42`..`0x44` | Format A | Strict greater-than comparisons. |
| **`GteI32`, `GteI64`, `GteF64`** | `0x45`..`0x47` | Format A | Greater-than-or-equal comparisons. |
| **`Not`** | `0x48` | Format A | Boolean NOT: `r[dst] = Value::Bool(!r[src1].as_bool()?)`. |
| **`Jump`** | `0x50` | Format C | Unconditional PC jump: `pc = (pc as i32 + offset) as u32`. |
| **`JumpIfTrue`** | `0x51` | Format C | Conditional branch: if `r[cond_reg].is_truthy()`, jump relative `offset`. |
| **`JumpIfFalse`** | `0x52` | Format C | Conditional branch: if `!r[cond_reg].is_truthy()`, jump relative `offset`. |
| **`Switch`** | `0x53` | Format B/C | Multi-way table jump based on discriminator register. |
| **`CallDirect`** | `0x60` | Format Call | Calls intra-module function `target_id + frame.function_id_offset`. Passes register window slice `r[arg_base .. arg_base + arg_count]` into callee slots `r[0..arg_count]`. Pushes new `CallFrame`. |
| **`CallVirtual`** | `0x61` | Format Call | Dynamic dispatch: receiver `this` is `r[arg_base]`. Reads `vtable_id` from heap object header. Dispatches through `vtables[vtable_id][call_target]`. |
| **`CallInterface`** | `0x62` | Format Call | Interface dispatch: resolves method offset via `itables[vtable_id][interface_id][method_offset]`. |
| **`CallSymbol`** | `0x63` | Format Call | Inter-module or native symbol call: resolves canonical symbol in `symbol_to_function_id` or `native_registry`. |
| **`CallCallable`** | `0x64` | Format Call | First-class lambda / function pointer call: unpacks closure environment address and function pointer from callable object. |
| **`Return`** | `0x65` | Format A | Void return: cleans up current frame reference registers, pops `CallFrame`, writes `Value::Null` to caller's `r[call_dst]`. |
| **`ReturnValue`** | `0x66` | Format A | Value return: cleans up frame reference registers (except return value), pops `CallFrame`, writes `r[dst]` to caller's `r[call_dst]`. |
| **`Exit`** | `0x67` | Format A | Process termination: reads status code `code = r[dst].as_i32()?`, tears down runtime stack, and halts VM with `ExitCode(code)`. |
| **`AllocObject`** | `0x70` | Format Call | Allocates instance on dynamic heap with `vtable_id = target_id`. Sets header `tag = OBJECT`, `ref_count = 1`. Initializes fields to null/zero. Writes `Value::Ref(addr)` to `r[dst]`. |
| **`GetField`** | `0x71` | Format A | Reads member field at `offset = src2` from object `r[src1]`. Writes value to `r[dst]`. |
| **`SetField`** | `0x72` | Format A | Writes value `r[dst]` into member field at `offset = src2` of object `r[src1]`. Retains new reference and releases old reference if reference field. |
| **`GetStatic`** | `0x73` | Format B | Intra-module static read: reads from `frame.static_base_offset + imm32` into `r[dst]`. |
| **`SetStatic`** | `0x74` | Format B | Intra-module static write: writes `r[dst]` into `frame.static_base_offset + imm32`. |
| **`GetStaticGlobal`** | `0x75` | Format B | Cross-module static read: resolves static symbol from constant pool `imm32` via `symbol_to_static_slot`. |
| **`GetStaticDirect`** | `0x76` | Format B | Absolute static memory address read. |
| **`AllocArray`** | `0x80` | Format A | Dynamic array allocation: `src1` = length, `src2` = element byte size. Sets header `length`, `tag = ARRAY_*`, `ref_count = 1`. Writes `Value::Ref(addr)` to `r[dst]`. |
| **`GetArray`** | `0x81` | Format A | Array element read: `r[dst] = array[r[src2]]`. Bounds checked; throws `IndexOutOfBoundsException` on violation. |
| **`SetArray`** | `0x82` | Format A | Array element write: `array[r[src1]] = r[src2]`. Bounds checked; manages reference counts if reference array. |
| **`ArrayLength`** | `0x83` | Format A | Reads array length from heap header: `r[dst] = Value::I32(length)`. |
| **`AllocTuple`** | `0x84` | Format A | Allocates heterogeneous tuple of `src1` elements with tag mask header. |
| **`GetTuple`** | `0x85` | Format A | Reads tuple element at slot `src2` into `r[dst]`. |
| **`PackClosure`** | `0x90` | Format Call | Allocates closure object on dynamic heap containing `target_id` function ID and copies captured registers `r[arg_base .. arg_base + arg_count]`. Writes `Value::Ref(addr)` to `r[dst]`. |
| **`UnpackCaptures`** | `0x91` | Format Call | Unpacks captured variables from active closure environment into current frame registers. |
| **`Instanceof`** | `0xA0` | Format A | Nominal inheritance check: tests if object `r[src1]`'s vtable inherits from class `src2`. Writes `Value::Bool` to `r[dst]`. |
| **`CastCheck`** | `0xA1` | Format A | Validates type compatibility or throws `ClassCastException`. |
| **`ConvIToF`** | `0xA2` | Format A | Numeric conversion: signed integer to float64 `r[dst] = Value::F64(r[src1].as_i64()? as f64)`. |
| **`ConvFToI`** | `0xA3` | Format A | Numeric conversion: float64 to signed integer `r[dst] = Value::I64(r[src1].as_f64()? as i64)`. |
| **`TupleInstanceof`** | `0xA4` | Format A | Dynamic tuple element type inspection: extracts element at index `r[src2]` of tuple `r[src1]`. Compares dynamic type tag against expected tag `flags()`. Writes `Value::Bool` to `r[dst]`. |
| **`Throw`** | `0xB0` | Format A | Initiates table-driven exception search for `r[dst]`. Unwinds call frames until matching handler is found or stack exhausts. |
| **`IncRef`** | `0xC0` | Format A | Increments reference count of heap object at `r[dst]`. |
| **`DecRef`** | `0xC1` | Format A | Decrements reference count of heap object at `r[dst]`. If count hits 0: runs class destructor, cascades drops to child reference fields, and frees block. |

---

## 4. Prioritized Deliverables & Implementation Roadmap

Following the standard engineering protocol, each phase is implemented, committed, thoroughly tested with automated unit and integration tests, verified, committed, and advanced to the subsequent phase.

### Phase 1: `RuntimeOptions`, `RuntimeContext`, and Engine Scaffold
- [x] `[P0]` **Task 1.1:** Implement `RuntimeOptions` configuration struct with full builder pattern (`new()`, `with_entry_point()`, `with_binary()`, `with_heap_size()`, `with_max_call_stack_depth()`, `with_args()`, `with_sandboxed()`).
- [x] `[P0]` **Task 1.2:** Implement `RuntimeContext` lifecycle coordinator with module loading, binary deserialization (`BinaryProgram::from_bytes`), and symbol mapping.
- [x] `[P0]` **Task 1.3:** Implement initial unit tests verifying configuration options, defaults, error diagnostics, and context setup.

### Phase 2: Virtual Memory Subsystem & Segregated Free List Allocator
- [x] `[P0]` **Task 2.1:** Implement `VirtualHeap` byte buffer with segregated static and dynamic regions.
- [x] `[P0]` **Task 2.2:** Implement static partition reservation (`reserve_static_partition`) returning base addresses for loaded binaries.
- [x] `[P0]` **Task 2.3:** Implement dynamic block allocator with 8-byte headers (`count: u32`, `tag_and_ref: u32`), segregated size-indexed free lists, and address-indexed block coalescing.
- [x] `[P0]` **Task 2.4:** Implement `verify_clean_memory()` API auditing `live_objects_count == 0` and unallocated dynamic heap state with leak diagnostics.
- [x] `[P0]` **Task 2.5:** Write unit tests for static memory boundary isolation, dynamic allocation, block splitting, coalescing, and out-of-memory detection.

### Phase 3: Value System, Call Frame Stack, and Basic Dispatch
- [x] `[P0]` **Task 3.1:** Implement strongly typed `Value` enum (`Null`, `Bool`, `I32`, `I64`, `F64`, `Ref`, `FnPtr`) with `#![forbid(unsafe_code)]` compliance.
- [x] `[P0]` **Task 3.2:** Implement unified `register_stack: Vec<Value>` window pool and `CallFrame` referencing contiguous register slices without per-frame heap allocations.
- [x] `[P0]` **Task 3.3:** Implement basic instruction dispatch: `Nop`, `Mov`, `LoadI32`, `LoadI64`, `LoadF64`, `LoadBool`, `LoadNull`, and `LoadConst`.
- [x] `[P0]` **Task 3.4:** Write unit tests verifying constant loading, register moves, and call frame setup.

### Phase 4: Arithmetic, Bitwise, Logical, and Comparison Opcodes
- [x] `[P0]` **Task 4.1:** Implement 32-bit and 64-bit integer arithmetic (`AddI32`, `AddI64`, `SubI32`, `SubI64`, `MulI32`, `MulI64`, `DivI32`, `DivI64`, `ModI32`, `ModI64`, `NegI32`, `NegI64`).
- [x] `[P0]` **Task 4.2:** Implement IEEE-754 64-bit float arithmetic (`AddF64`, `SubF64`, `MulF64`, `DivF64`, `NegF64`).
- [x] `[P0]` **Task 4.3:** Implement bitwise operations (`Band`, `Bor`, `Bxor`, `Bnot`, `Shl`, `Shr`) and boolean logic (`Not`).
- [x] `[P0]` **Task 4.4:** Implement all comparison opcodes (`Eq*`, `Neq*`, `Lt*`, `Lte*`, `Gt*`, `Gte*`) for integers, floats, booleans, and strings.
- [x] `[P0]` **Task 4.5:** Write comprehensive unit test suite covering numeric edge cases, division by zero, float precision, and comparisons.

### Phase 5: Control Flow, Branching, Loops, and Backpatching
- [x] `[P0]` **Task 5.1:** Implement relative branching: `Jump`, `JumpIfTrue`, `JumpIfFalse`.
- [x] `[P0]` **Task 5.2:** Implement truthiness evaluation and signed PC offset updates.
- [x] `[P0]` **Task 5.3:** Implement multi-way switch branching dispatch (`Switch`).
- [x] `[P0]` **Task 5.4:** Write unit tests for nested loops, conditional branches, breaks, continues, and backward jumps.

### Phase 6: Function Calls, Two-Phase Static Init (`@init`), and Relocatable Linking
- [x] `[P0]` **Task 6.1:** Implement windowed parameter passing and call frame allocation for `CallDirect`.
- [x] `[P0]` **Task 6.2:** Implement return mechanics (`Return`, `ReturnValue`) passing results to caller register `call_dst`.
- [x] `[P0]` **Task 6.3:** Implement multi-binary relocatable linking: intra-module calls use `frame.function_id_offset + target_id`; intra-module statics use `frame.static_base_offset + slot`.
- [x] `[P0]` **Task 6.4:** Implement static initialization phase: automatically invoke all module `@init` functions upon load.
- [x] `[P0]` **Task 6.5:** Write unit tests for deep recursion, multiple function arguments, static variable persistence across calls, and multi-binary linking.

### Phase 7: Object Model, Instance Fields, VTables, and Dynamic Dispatch
- [x] `[P0]` **Task 7.1:** Implement `AllocObject`: allocates object payload on heap, initializes `vtable_id` at offset 0, returns reference.
- [x] `[P0]` **Task 7.2:** Implement `GetField` and `SetField` with field offset mapping.
- [x] `[P0]` **Task 7.3:** Implement `CallVirtual` and `CallInterface` dispatching methods via `vtables` and `itables`.
- [x] `[P0]` **Task 7.4:** Implement `Instanceof` and `CastCheck` verifying class hierarchy via `vtable_bases`.
- [x] `[P0]` **Task 7.5:** Write unit tests for single and multi-level inheritance, method overriding, field mutation, and instanceof checks.

### Phase 8: Dynamic Arrays, Heterogeneous Tuples, and Core Builtins
- [x] `[P0]` **Task 8.1:** Implement `AllocArray`, `GetArray`, `SetArray`, `ArrayLength` with strict bounds checking.
- [x] `[P0]` **Task 8.2:** Implement `AllocTuple` and `GetTuple` with type-tagged slot indexing.
- [x] `[P0]` **Task 8.3:** Implement core builtins (`print`, `println`, `assert`, `clock`).
- [x] `[P0]` **Task 8.4:** Write unit tests for primitive arrays, reference arrays, multi-dimensional arrays, tuples, and builtins.

### Phase 9: Closures, Function Pointers, and Host Native FFI
- [x] `[P0]` **Task 9.1:** Implement `PackClosure` capturing register windows into heap closure blocks.
- [x] `[P0]` **Task 9.2:** Implement `UnpackCaptures` and `CallCallable` executing closures with environment pointers.
- [x] `[P0]` **Task 9.3:** Implement `NativeRegistry` allowing host Rust applications to register custom native callbacks.
- [x] `[P0]` **Task 9.4:** Implement dynamic library loading (`libloading`) for shared objects (`.so`, `.dll`) when un-sandboxed.
- [x] `[P0]` **Task 9.5:** Write unit tests for nested closures, lambdas capturing outer locals, and host Rust FFI invocations.

### Phase 10: Automatic Reference Counting (ARC), Destructors, Zero-Cost Exceptions, and Master Fixtures
- [x] `[P0]` **Task 10.1:** Implement deterministic ARC: `IncRef`, `DecRef` with active destruction set protecting against cycles.
- [x] `[P0]` **Task 10.2:** Implement destructor invocation: automatically execute class destructor bytecode when `ref_count` drops to 0, followed by recursive child field cleanup.
- [x] `[P0]` **Task 10.3:** Implement zero-cost exception handling (`Throw`): search `CompiledFunction.exception_handlers` for matching PC ranges, unwinding call frames and running reference cleanup on `reference_registers`.
- [x] `[P0]` **Task 10.4:** Build master integration test suite: compile diverse Solix programs through `solix-compilation` and execute directly in `solix-runtime`.

---

## 5. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Command | Sign-Off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Zero warnings, `#![forbid(unsafe_code)]` | `cargo build --workspace --all-targets` | [x] Completed |
| **Automated Test Suite** | 100% test pass across all units and integration fixtures | `cargo test --workspace --all-targets` | [x] Completed |
| **Clippy Static Analysis** | Zero warnings with `-D warnings` | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Completed |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [x] Completed |
| **Documentation Sync** | Milestone specifications and architecture documents updated | GL-0010 audit checklist | [x] Completed |

---

## 6. Risk Assessment & Engineering Mitigations

1. **Risk: Memory Corruption or Unsoundness in Virtual Memory Allocation**
   - *Mitigation:* Preserve `#![forbid(unsafe_code)]` strictly across `solix-runtime`. All byte reads, writes, and slice operations use standard, checked Rust slices (`heap[start..end]`, `copy_from_slice`, `u32::from_le_bytes`, `to_le_bytes`), guaranteeing memory safety with zero undefined behavior.

2. **Risk: Stack Overflow or Infinite Loops during Exception Unwinding or Destructor Invocation**
   - *Mitigation:* `CallFrame` recursion depth is strictly capped by `max_call_stack_depth`. Destructor invocation tracks an `active_destructors` address set to prevent re-entrant destruction cycles.

3. **Risk: Performance Overhead in Multi-Binary Symbol Resolution**
   - *Mitigation:* Intra-module function calls and static variable accesses use direct indexed offsets (`frame.function_id_offset + target_id` and `frame.static_base_offset + slot`). Cross-module calls resolve once via `symbol_to_function_id` and can be inline-cached.
