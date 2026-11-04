# Milestone 006: Register-Based Bytecode Emitter & Binary Container (`SLXB`)

## Document Metadata
* **Milestone ID:** MS-006
* **Target Version:** `v0.1.0-alpha.6`
* **Milestone Branch:** `milestone/006-register-bytecode-emitter`
* **Status:** DELIVERED
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-10-26
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/milestones/003-recursive-descent-parser.md` (MS-003)
  * `docs/internals/milestones/004-compiler-stabilization-and-incremental-architecture.md` (MS-004)
  * `docs/internals/milestones/005-semantic-analysis-type-binding-and-generics.md` (MS-005)

---

## 1. Executive Summary & Purpose

With lexical analysis (MS-002), syntactic parsing (MS-003), and semantic type binding (MS-005) complete, the Solix compilation context contains fully resolved type annotations, verified nominal and inheritance hierarchies, complete virtual method tables, and on-the-fly monomorphized generic ASTs. The next phase of the compiler pipeline is the **Code Generation / Bytecode Emission Phase** (`phases/emitter/`).

Milestone 006 replaces the legacy stack-based, monolithic bytecode architecture from the reference C++ compiler with an industrial-grade **Register-Based Virtual Machine Bytecode & Binary Container** architecture. This milestone incorporates seven key architectural pillars and systems optimizations:

1. **Uniform 64-Bit Aligned Instruction Words & 16-Bit Register IDs:**
   - Every instruction is encoded into an 8-byte aligned 64-bit instruction word, guaranteeing zero memory unaligned read penalties during VM dispatch.
   - Registers use 16-bit indices (`u16`), supporting up to $65,535$ registers per call frame without risk of overflowing an 8-bit limit.
   - Instructions operate directly on registers (`r0`, `r1`, `r2`, ...):
     - Parameters: $r_0 \dots r_{\text{param\_count} - 1}$ (with `this` at $r_0$ for instance methods).
     - Local variables: $r_{\text{param\_count}} \dots r_{\text{param\_count} + \text{local\_count} - 1}$.
     - Temporary scratch registers: $r_{\text{param\_count} + \text{local\_count}} \dots r_{\text{num\_registers} - 1}$.

2. **Contiguous Argument Register Windows for Zero-Allocation Calls:**
   - Replaces variable-length operand lists (`Vec<Reg>`) with contiguous argument windows:
     $$\text{CALL\_DIRECT } dst\_reg, func\_id, arg\_base\_reg, arg\_count$$
   - The caller evaluates arguments into contiguous registers ($r_{\text{base}} \dots r_{\text{base} + \text{count} - 1}$).
   - The callee's frame initialization executes as a single contiguous memory block copy (`copy_from_slice`) into parameter slots $r_0 \dots r_{\text{count} - 1}$ in sub-nanosecond time.

3. **Synthesized Module Static Initializer (`@init`):**
   - The compiler automatically synthesizes a dedicated module initialization function (`@init` / `__static_init__`).
   - Evaluates all global and static variable initializers (e.g. `public static int32 a = 3;`, `public static Logger log = new Logger();`) and stores them directly into static memory slots via `SET_STATIC`.
   - The runtime invokes `@init()` immediately after loading the binary and reserving static memory, but *before* invoking `main()`, decoupling binary loading from execution.

4. **Modular `CompiledFunction` Structures vs. Monolithic Bytecode Stream:**
   - The legacy monolithic flat bytecode vector is eliminated.
   - Each function, method, constructor, destructor, and synthesized lambda is compiled into a standalone `CompiledFunction` struct with its own local bytecode chunk, register allocation count, canonical symbol identifier, ARC reference register list, and zero-cost exception handler table.
   - All jumps inside a function are relative to the function's own start (`pc = 0`), making functions completely relocatable, linkable, and modular.

5. **Relocatable Module-Partitioned Static Memory with Inline Caching:**
   - Static variables are stored in an isolated, dedicated static memory space, completely decoupled from the dynamic heap.
   - Each compiled binary declares its required static memory capacity (`static_space_size`).
   - At load time, the runtime allocates a contiguous slice in global static memory:
     $$\text{ModuleStaticBase} = \text{RuntimeCurrentStaticAllocated}$$
     $$\text{RuntimeCurrentStaticAllocated} \mathrel{+}= \text{ModuleStaticSpaceSize}$$
   - Function metadata records this base offset. Intra-module static variable accesses use direct, O(1) indexed instructions:
     $$\text{EffectiveAddress} = \text{FrameStaticBase} + \text{LocalStaticIndex}$$
   - Cross-module static accesses use **Inline Caching**: resolved once from the exported static symbol hash table on first execution and patched to a direct absolute offset to avoid repetitive hash lookups in loops.

6. **Dual-Indexed Runtime Function Registry:**
   - **Intra-Binary Calls (Direct O(1) Vector Indexing):** Functions compiled within the same module are assigned local `FunctionId` indices (0, 1, 2, ...). Calls within the module emit `CALL_DIRECT dst, func_id, arg_base, arg_count`, executing via direct array lookup with zero hash overhead.
   - **Inter-Binary Calls (Global Symbol Resolution with Dynamic Patching):** Exported functions register their canonical symbols (e.g. `solix::models::UserService::find_user_by_email`) in a runtime hash table. Cross-module calls use `CALL_SYMBOL dst, symbol_idx, arg_base, arg_count`, which links dynamically and caches the target pointer.

7. **Zero-Cost Table-Driven Exception Handling & Sound Stack Unwinding:**
   - Replaces the legacy instruction-based cleanup patching (`REGISTER_RETURN_CLEANUP`, `JMP_TO_OUTER_CLEANUP`) with modern **Table-Driven Zero-Cost Exception Tables**.
   - The normal execution path incurs **zero instruction overhead** entering or exiting `try` blocks.
   - When `THROW reg` is encountered, the VM searches the function's `exception_handlers` table for matching PC ranges (`try_start_pc <= pc < try_end_pc`) and target exception types.
   - **Sound Unwinding & Destructors:** Call frame registers are zero-initialized (`Value::Null`). The unwinder inspects `reference_registers: Vec<u16>` and calls `DEC_REF` on live references; if ref counts reach 0, class destructors (`~Class()`) run safely.
   - **Dual-Emission `finally`:** `finally` blocks are emitted inline for normal exit paths and early jumps (`return`, `break`, `continue`), and registered in the exception table for unhandled exception unwinding.

8. **Compact Hermetic Binary Container Format (`SLXB`):**
   - Implements a high-performance binary serialization and container format (`Solix Executable Binary` - `SLXB`).
   - Encodes headers, static reservations, constant pool, class vtables/itables, and function bytecode chunks into an endian-safe, compact binary layout (replacing verbose text/JSON representations).

---

## 2. Scope & Crates Affected

### 2.1 `crates/solix-common`
* `src/bytecode/mod.rs`: Top-level bytecode submodule exports.
* `src/bytecode/opcodes.rs`: Register-based virtual machine `OpCode` enumeration.
* `src/bytecode/instruction.rs`: Strongly-typed 64-bit aligned register instruction representation:
  * Formats: Formats A, B, C, Call.
* `src/bytecode/function.rs`:
  * `CompiledFunction`: standalone modular function metadata and 64-bit instruction chunk.
  * `ExceptionHandlerEntry`: try PC range, catch type symbol, catch register, handler PC, finally PC.
  * `SourceMapEntry`: instruction PC to source code `Span` mapping.
* `src/bytecode/class.rs`:
  * `CompiledClass`: class symbol, vtable ID, base vtable ID, instance memory layout, reference field offsets, virtual method table, flat itable interface cache `Vec<(u32, Vec<u32>)>`, destructor function index.
* `src/bytecode/binary.rs`:
  * `BinaryProgram` & `BinaryUnit`: In-memory and serializable container.
  * Binary format reader & writer (`SLXB` format version 1.0).
* `src/diagnostics/emission.rs`:
  * `EmissionError` diagnostic enum deriving `thiserror::Error` and `miette::Diagnostic` with standardized codes `E0301`–`E0320`.

### 2.2 `crates/solix-compilation`
* `src/phases/emitter/mod.rs`: Emitter phase exports and entry point functions.
* `src/phases/emitter/emitter.rs`:
  * AST Visitor walking the semantically bound AST (`ProgramAst`) and generating 64-bit register instructions.
  * Statement & Expression visitors covering all Solix language constructs.
  * Module static initializer synthesizer emitting `@init`.
* `src/phases/emitter/register_allocator.rs`:
  * Tracks local variable register bindings and scratch temporary registers.
  * Allocates contiguous register windows for call arguments.
  * Computes peak frame size (`num_registers`) per function.
* `src/phases/emitter/constant_pool.rs`:
  * Deduplicates string literals, floating-point numbers, and 64-bit constants.
* `src/phases/emitter/control_flow.rs`:
  * Back-patching engine for conditional/unconditional jumps, loops, and switch jump tables.
* `src/phases/emitter/exceptions.rs`:
  * Zero-cost exception table builder and dual-emission finally compiler.
* `src/phases/emitter/disassembler.rs`:
  * Formats `CompiledFunction` and `BinaryProgram` into human-readable assembly text for debugging and console inspection.
  * Formats instructions with registers (`$r0`), constants, jump targets, line spans, and exception tables.
* `src/CompilationContext.rs`:
  * Added `emission_artifacts: Option<EmissionArtifacts>` storing `BinaryProgram`, `functions_by_source: Vec<Vec<CompiledFunction>>`, `init_function: Option<CompiledFunction>`, and disassembled text.
  * Added source mapping context helpers:
    * `get_source_name(&self, source_id: u16) -> Option<&str>`
    * `get_source_id(&self, name: &str) -> Option<u16>`
    * `get_source_code(&self, source_id: u16) -> Option<&str>`

---

## 3. Technical Architecture & Invariants

```
                             ┌─────────────────────────────────┐
                             │    Semantically Bound AST       │
                             │  (SymbolTable + Layouts + Maps) │
                             └────────────────┬────────────────┘
                                              │
                                              ▼
                             ┌─────────────────────────────────┐
                             │       Bytecode Emitter          │
                             │  (Register-Based AST Visitor)   │
                             └────────┬───────────────┬────────┘
                                      │               │
                 ┌────────────────────┴──┐         ┌──┴────────────────────┐
                 ▼                       ▼         ▼                       ▼
     ┌───────────────────────┐ ┌────────────────┐ ┌────────────────┐ ┌───────────────┐
     │  Register Allocator   │ │ Constant Pool  │ │  Control Flow  │ │ Zero-Cost     │
     │  - Locals (r0..rK)    │ │ - Strings      │ │  - Jump Patch  │ │ Exception     │
     │  - Temps  (rK+1..rN)  │ │ - Floats, i64  │ │  - Break/Cont  │ │ Handler Table │
     │  - Windowed Arg Alloc │ │ - Deduplication│ │  - Switch Tbl  │ │ - Try Ranges  │
     └───────────────────────┘ └────────────────┘ └────────────────┘ └───────────────┘
                                      │
                                      ▼
                             ┌─────────────────────────────────┐
                             │   Compiled Function Structs     │
                             │   - User Functions & Methods    │
                             │   - Synthesized @init Module    │
                             │   - 16-Bit Register Window      │
                             │   - 64-Bit Aligned Bytecode     │
                             │   - ARC Reference Registers     │
                             │   - Exception Handler Table     │
                             │   - Source Map Spans            │
                             └────────────────┬────────────────┘
                                              │
                                              ▼
                             ┌─────────────────────────────────┐
                             │      Binary Container           │
                             │      (SLXB Serializer)          │
                             │  - Header & Magic Bytes (SLXB)  │
                             │  - Static Memory Reservation    │
                             │  - Exported Static Symbols Map  │
                             │  - Class VTables & Flat ITables │
                             │  - Function Metadata Vector     │
                             └─────────────────────────────────┘
```

### 3.1 Register Memory System & Windowed Allocator

* A function frame has $N$ registers: $r_0, r_1, \dots, r_{N-1}$ where each register is a 64-bit `Value`.
* Registers are indexed linearly via 16-bit identifiers (`u16`), supporting up to $65,535$ registers per frame:
  * $0 \le r < \text{param\_count}$: Parameters passed by caller. If instance method, $r_0 = \text{this}$.
  * $\text{param\_count} \le r < \text{param\_count} + \text{local\_count}$: Local variables bound 1:1 to semantic scopes.
  * $\text{param\_count} + \text{local\_count} \le r < \text{num\_registers}$: Temporary evaluation scratch registers.
* **Windowed Call Allocation:** When preparing a function call with $K$ arguments, the allocator reserves a contiguous span of registers $[r_{\text{base}}, r_{\text{base}+1}, \dots, r_{\text{base}+K-1}]$, evaluates the argument expressions directly into this contiguous window, and emits the call instruction:
  $$\text{CALL\_DIRECT } dst, func\_id, r_{\text{base}}, K$$
  When the callee is invoked, its initial frame parameters $r_0 \dots r_{K-1}$ are initialized via a single `copy_from_slice` block copy from the caller's window.

### 3.2 Uniform 64-Bit Instruction Word Encoding

All instructions are encoded into fixed-width 8-byte (64-bit) aligned words:

```text
Format A (3-Register 3-Address Arithmetic & Logic):
+---------------+---------------+---------------+---------------+---------------+
| Opcode (8b)   | Flags (8b)    | Dst (16b)     | Src1 (16b)    | Src2 (16b)    |
+---------------+---------------+---------------+---------------+---------------+

Format B (Register + 32-Bit Immediate / Constant / Static Offset):
+---------------+---------------+---------------+-------------------------------+
| Opcode (8b)   | Flags (8b)    | Dst (16b)     | Immediate / ConstIdx (32b)    |
+---------------+---------------+---------------+-------------------------------+

Format C (Control Flow Branch & Jump):
+---------------+---------------+---------------+-------------------------------+
| Opcode (8b)   | Flags (8b)    | CondReg (16b) | Signed PC Offset (32b)        |
+---------------+---------------+---------------+-------------------------------+

Format Call (Windowed Call Instruction):
+---------------+---------------+---------------+---------------+---------------+
| Opcode (8b)   | Dst (16b)     | TargetId (32b)| ArgBase (16b) | ArgCount (16b)|
+---------------+---------------+---------------+---------------+---------------+
```

### 3.3 Relocatable Static Memory Architecture & Inline Caching

* Static variables are allocated in a contiguous runtime array `runtime.static_memory`.
* Each binary module declares `static_space_size: u32`.
* When a module is loaded into the runtime:
  1. The runtime records `base_offset = runtime.static_memory.len() as u32`.
  2. The runtime reserves `unit.static_space_size` slots in `runtime.static_memory`.
  3. Every `CompiledFunction` belonging to this module receives `static_base_offset = base_offset`.
* **Intra-Module Static Access:**
  - `GET_STATIC dst_reg, static_idx`:
    $$\text{TargetSlot} = \text{Frame}.\text{static\_base\_offset} + \text{static\_idx}$$
  - `SET_STATIC static_idx, src_reg`:
    $$\text{TargetSlot} = \text{Frame}.\text{static\_base\_offset} + \text{static\_idx}$$
  Both are immediate $O(1)$ operations with zero hash lookups.
* **Inter-Module Static Access with Inline Caching:**
  - The binary exports `static_symbols_map: Vec<(String, u32)>` mapping fully-qualified symbols to module-local offsets.
  - On first execution of `GET_STATIC_GLOBAL dst_reg, symbol_idx`, the runtime resolves the global symbol once, calculates its absolute static slot, and patches the instruction in-place to `GET_STATIC_DIRECT dst_reg, absolute_slot`, eliminating hash lookups in hot loops.

### 3.4 Synthesized Module Initializer (`@init`)

* Global/static variable declarations with initializers (e.g. `public static int32 count = 42;`, `public static Logger logger = new Logger();`) cannot run during static binary parsing.
* The emitter automatically synthesizes a special function `@init` (ID 0) for each compiled module.
* In `@init`:
  1. Evaluates all global variable initializers in dependency order.
  2. Emits `SET_STATIC slot_idx, value_reg` to populate the allocated static partition.
  3. Emits `RETURN` (void).
* Execution lifecycle in runtime:
  $$\text{Load SLXB} \longrightarrow \text{Allocate Static Space} \longrightarrow \text{Execute } @init() \longrightarrow \text{Execute } main()$$

### 3.5 Dual-Indexed Function Registry

1. **Intra-Binary Vector (Direct Indexing):**
   - Each function inside the binary unit is indexed by a 32-bit `FunctionId` (`0 .. functions.len() - 1`).
   - Calls within the unit emit `CALL_DIRECT dst_reg, func_id, arg_base, arg_count`.
   - The runtime invokes `unit.functions[func_id]` via direct array index ($O(1)$).
2. **Inter-Binary Global Hash Table (Dynamic Symbol Linking):**
   - Every function's canonical symbol (`symbol: String`) is registered into:
     $$\text{runtime.function\_registry} : \text{HashMap}<\text{String}, \text{Arc}<\text{CompiledFunction}>>$$
   - Cross-module calls emit `CALL_SYMBOL dst_reg, symbol_const_idx, arg_base, arg_count`, resolving via the global symbol table and caching the direct pointer for subsequent calls.

### 3.6 Zero-Cost Exception Handling & Sound Stack Unwinding

* Normal execution flows through `try` blocks with **no runtime setup or registration overhead**.
* Each `try-catch-finally` compiles into an entry in `exception_handlers`:
  ```rust
  #[derive(Debug, Clone, PartialEq)]
  pub struct ExceptionHandlerEntry {
      pub try_start_pc: u32,
      pub try_end_pc: u32,
      pub catch_type_symbol: Option<String>,
      pub catch_reg: u16,
      pub handler_pc: u32,
      pub finally_pc: Option<u32>,
  }
  ```
* **Dual-Emission `finally` Semantics:**
  - On normal execution flow, the emitter appends the `finally` code at the end of the `try` and `catch` blocks before jumping to the exit.
  - If a `return`, `break`, or `continue` occurs inside a `try` block, the emitter compiles the `finally` code immediately prior to emitting the return or jump instruction.
  - On exceptional execution flow, `finally_pc` is executed during stack unwinding.
* **Sound Unwinding & Destructor Safety Invariants:**
  1. All registers in a call frame are zero-initialized to `Value::Null` upon frame creation.
  2. `DEC_REF` on `Value::Null` is a safe, immediate no-op.
  3. During unwinding, the unwinder iterates `reference_registers: Vec<u16>` and calls `DEC_REF` on active references; if ref counts reach zero, user-defined class destructors (`~Class()`) run safely before the frame is popped.

### 3.7 Interface Dispatch Optimization (Flat ITable Cache)

* Classes typically implement 1 to 4 interfaces.
* Instead of a dynamic hash table lookup on every interface call, each `CompiledClass` stores a flat array of interface tuples:
  ```rust
  pub itable: Vec<(u32 /* interface_id */, Vec<u32> /* method_vtable_slots */)>
  ```
* When `CALL_INTERFACE` executes, it performs a linear integer scan over 1–4 interface IDs in CPU cache, which is orders of magnitude faster than string hashing and avoids cache misses.

### 3.8 Compact `SLXB` Binary Container Specification

The binary container uses a compact, hermetic format:

| Section | Type | Description |
| :--- | :--- | :--- |
| **Magic** | `[u8; 4]` | `b"SLXB"` |
| **Version** | `u16, u16` | Major (1), Minor (0) |
| **Flags** | `u32` | Binary feature flags (debug symbols, optimization) |
| **Entry Point** | `String` | Fully-qualified symbol of the `main` entry point |
| **Static Reservation** | `u32` | Number of static slots required |
| **Exported Statics** | `Vec<(String, u32)>` | Symbol name to local static slot map |
| **Constant Pool** | `ConstantPool` | Deduplicated integers, floats, strings |
| **Class Descriptors** | `Vec<CompiledClass>` | VTables, flat ITables, instance layouts, destructors |
| **Function Descriptors** | `Vec<CompiledFunction>` | Function metadata, register counts, 64-bit instruction chunks, exception tables, source maps |

### 3.9 Multi-File Function Partitioning & Context Source Mapping

To mirror the modular architecture of earlier compilation phases (lexing, parsing, binding) and enable incremental compilation and clear multi-file debugging:
* In `EmissionArtifacts`:
  - User functions and methods are partitioned by their originating file:
    ```rust
    pub struct EmissionArtifacts {
        pub program: BinaryProgram,
        /// Functions grouped by source file: `functions_by_source[source_id]` contains the
        /// compiled functions declared in that source file.
        pub functions_by_source: Vec<Vec<CompiledFunction>>,
        /// Special synthesized module static initializer function (@init, Function ID 0).
        pub init_function: Option<CompiledFunction>,
        pub disassembled_code: String,
    }
    ```
  - The special synthesized module static initializer `@init` (Function ID 0) runs module-wide static variable initialization and is maintained separately in `init_function` (and at index 0 of `program.functions`), while `functions_by_source` maps purely user/method functions declared per source file.
* In `CompilationContext`:
  - Added fast bidirectional source resolution helpers:
    ```rust
    impl CompilationContext {
        /// Resolves the human-readable source path/name from a `source_id`.
        #[must_use]
        pub fn get_source_name(&self, source_id: u16) -> Option<&str>;

        /// Resolves the numeric `source_id` from a source file name/path.
        #[must_use]
        pub fn get_source_id(&self, name: &str) -> Option<u16>;

        /// Retrieves the complete raw source code string for a given `source_id`.
        #[must_use]
        pub fn get_source_code(&self, source_id: u16) -> Option<&str>;
    }
    ```

### 3.10 Diagnostic Bytecode Disassembler & Pretty-Printer

To enable rapid developer inspection and pinpoint compiler bugs in the console during test failures and development:
* `solix-compilation` provides a dedicated disassembler module (`src/phases/emitter/disassembler.rs`) that pretty-prints functions, bytecodes, constants, and exception handling ranges:
  ```rust
  pub fn disassemble_instruction(instruction: &Instruction, pool: &ConstantPool) -> String;
  pub fn disassemble_function(func: &CompiledFunction, pool: &ConstantPool) -> String;
  pub fn disassemble_program(program: &BinaryProgram) -> String;
  ```
* Sample Disassembly Output:
  ```text
  .function solix::models::User::calculate_total(r0: this, r1: discount_pct) -> f64
    .frame_size: 6
    .params: 2
    .ref_regs: [0]
    .source: "models/user.slx": lines 42-58
    .handlers:
      [TRY: 0004..0010 -> CATCH 'std::Error' -> @0012 (reg: r3)]

    0000  GET_FIELD       r2, r0, 1                ; r2 = this.base_price
    0001  LOAD_CONST      r3, $c0                  ; r3 = 100.0 (f64)
    0002  DIV_F64         r4, r1, r3               ; r4 = discount_pct / 100.0
    0003  MUL_F64         r5, r2, r4               ; r5 = base_price * discount
    0004  SUB_F64         r0, r2, r5               ; r0 = base_price - discount
    0005  RETURN          r0                       ; return r0
  ```
* Supports `--dump-bytecode` CLI flag in `solix-client` and console dumps during failing test runs.

---

## 4. Sequential Feature-by-Feature Delivery & Exhaustive Testing Pipeline

To guarantee that zero subtle emitter bugs escape into the runtime, Milestone 006 is structured as a **10-phase vertical delivery pipeline**. Every single phase contains explicit edge-case tests, instruction bitfield assertions, register invariants, and failure mode verifications that **must pass 100% before the next phase begins**.

```
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 1: Instruction Representation & SLXB Binary Container Engine     │
│          └─► Test Gate: Bitfield packing roundtrips & SLXB byte match  │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 2: Windowed Virtual Register Allocator & Constant Pool           │
│          └─► Test Gate: Temp reuse, contiguous windows, dedup tests    │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 3: Expression Emitter (Literals, Arithmetic, Logic, Conversions) │
│          └─► Test Gate: 3-address opcode & register verification tests │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 4: Control Flow, Loops, Jump Backpatching & Disassembler         │
│          └─► Test Gate: If-else, loops with break/continue, switch     │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 5: Static Memory Partitioning & Synthesized Module @init         │
│          └─► Test Gate: @init synthesis & static read/write tests      │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 6: Object Model, Classes, VTables & Flat ITables                 │
│          └─► Test Gate: ALLOC_OBJECT, GET/SET_FIELD, VTable & ITable   │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 7: Arrays & Heterogeneous Tuples                                 │
│          └─► Test Gate: Array ops, multi-dim arrays, tuple packing     │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 8: First-Class Lambdas, Closures & Function Pointers             │
│          └─► Test Gate: Capture analysis, PACK_CLOSURE, UNPACK_CAPTURES│
├────────────────────────────────────────────────────────────────────────┤
│ Phase 9: Safe ARC Tracking, Destructor Wiring & Exception Tables       │
│          └─► Test Gate: INC/DEC_REF, destructors, try PC ranges        │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 10: Master End-to-End Fixtures & Milestone Integration           │
│          └─► Test Gate: master_all_statements_emission.slx, 100% DoD   │
└────────────────────────────────────────────────────────────────────────┘
```

---

### Phase 1: Instruction Representation & `SLXB` Binary Container Engine

#### 1. Code Deliverables (`crates/solix-common/src/bytecode/`)
* `opcodes.rs`: Register-based `OpCode` enum with 64-bit aligned discriminants.
* `instruction.rs`: 64-bit aligned `Instruction` word with bit-packing and unpacking:
  * `FormatA`: `[ Opcode (8) | Flags (8) | Dst (16) | Src1 (16) | Src2 (16) ]`
  * `FormatB`: `[ Opcode (8) | Flags (8) | Dst (16) | Immediate (32) ]`
  * `FormatC`: `[ Opcode (8) | Flags (8) | CondReg (16) | SignedOffset (32) ]`
  * `FormatCall`: `[ Opcode (8) | Dst (16) | TargetId (32) | ArgBase (16) | ArgCount (16) ]`
* `function.rs`: `CompiledFunction`, `ExceptionHandlerEntry`, `SourceMapEntry`.
* `class.rs`: `CompiledClass` with flat ITable cache `Vec<(u32, Vec<u32>)>`.
* `binary.rs`: `BinaryProgram` (`SLXB`) reader and writer.
* `src/diagnostics/emission.rs`: `EmissionError` diagnostic enum.

#### 2. Exhaustive Test Suite (`crates/solix-common/tests/bytecode_tests.rs`)
* **Case 1.1 (Instruction Bit-Packing Accuracy):**
  - Verify every single `OpCode` encodes and decodes with zero bit corruption.
  - Test register boundaries: $r_0$, $r_1$, $r_{255}$, $r_{256}$, $r_{65535}$.
  - Test immediate boundaries: $0$, $-1$, $1$, $\text{i32::MIN}$, $\text{i32::MAX}$, $\text{u32::MAX}$.
  - Test jump offset boundaries: backward jumps ($-8$, $-1024$), forward jumps ($+8$, $+65536$).
  - Assert that all serialized instructions are exactly 8 bytes long.
* **Case 1.2 (Container Corrupt Input Handling):**
  - Verify invalid magic bytes (e.g. `b"RIFF"`, `b"JAVA"`) return `Err(EmissionError::InvalidMagic)`.
  - Verify version mismatch (e.g. major version 2) returns `Err(EmissionError::UnsupportedVersion)`.
  - Verify truncated byte buffers return `Err(EmissionError::TruncatedBinary)`.
* **Case 1.3 (Full Container Round-Trip):**
  - Construct a complex `BinaryProgram` containing:
    - 3 functions with varied register counts and exception handlers.
    - 2 classes with VTable slots and flat ITable caches.
    - Constant pool containing ASCII strings, UTF-8 unicode (`"λ_closure"`), empty strings, floats (`3.14159`, `-0.0`, `f64::NAN`), and integers.
    - 15 static variable reservations and export mappings.
  - Serialize with `to_bytes()`, deserialize with `from_bytes()`, assert 100% field-for-field equality.

---

### Phase 2: Windowed Virtual Register Allocator & Constant Pool

#### 1. Code Deliverables (`crates/solix-compilation/src/phases/emitter/`)
* `register_allocator.rs`:
  * Scoped register allocator managing parameter slots, locals, and temporary recycling.
  * Contiguous argument window allocation for function calls.
  * Frame size (`num_registers`) calculation.
* `constant_pool.rs`:
  * Deduplicating constant pool for strings, floats, and 64-bit integers.

#### 2. Exhaustive Test Suite (`crates/solix-compilation/tests/emitter_allocator_tests.rs`)
* **Case 2.1 (Parameter Slot Assignment):**
  - Static method with 0 params $\implies$ first local at $r_0$.
  - Static method with 3 params $\implies$ params at $r_0, r_1, r_2$; first local at $r_3$.
  - Instance method with 2 params $\implies$ `this` at $r_0$, params at $r_1, r_2$; first local at $r_3$.
* **Case 2.2 (Disjoint Scope Local Reuse):**
  - For sequential blocks `{ int a; } { int b; }`: verify $a$ and $b$ allocate the same register index.
* **Case 2.3 (Temporary Register Recycling & Minimum Frame Size):**
  - Simulate deep expression tree: `((a + b) * (c + d)) / ((e - f) + (g * h))`.
  - Assert that temporaries allocated for sub-expressions are recycled immediately upon consumption.
  - Verify `num_registers` matches the exact optimal register ceiling.
* **Case 2.4 (Contiguous Windowed Argument Allocation):**
  - Request argument windows of sizes 1, 2, 5, 10. Assert that all allocated registers form a strictly contiguous sequence: $r_{\text{base}}, r_{\text{base}+1}, \dots, r_{\text{base}+K-1}$.
  - Test nested calls `f(g(1, 2), h(3))`: verify inner call argument window does not collide with outer call window.
* **Case 2.5 (Constant Pool Deduplication):**
  - Insert identical strings repeatedly (`"solix"`, `"solix"`, `"solix"`). Assert constant pool size is 1, and all queries return index 0.

---

### Phase 3: Expression Emitter (Literals, Arithmetic, Logic, Conversions)

#### 1. Code Deliverables (`crates/solix-compilation/src/phases/emitter/`)
* `emitter.rs`:
  * Expression visitor implementing:
    * Literals: `LOAD_I32`, `LOAD_I64`, `LOAD_F64`, `LOAD_BOOL`, `LOAD_NULL`, `LOAD_CONST`.
    * Variable identifiers and basic assignments (`MOV`).
    * Binary arithmetic: `ADD_I64`, `ADD_F64`, `SUB`, `MUL`, `DIV`, `MOD`.
    * Bitwise operators: `BAND`, `BOR`, `BXOR`, `SHL`, `SHR`.
    * Relational operators: `EQ`, `NEQ`, `LT`, `LTE`, `GT`, `GTE`, `EQ_STRING`.
    * Unary operators: `NEG`, `NOT`, `BNOT`, `INC`, `DEC`.
    * Type conversions: `CONV_I_TO_F`, `CONV_F_TO_I`, `TRUNC_I32`.

#### 2. Exhaustive Test Suite (`crates/solix-compilation/tests/emitter_expression_tests.rs`)
* **Case 3.1 (Literal Instruction Generation):**
  - Integer literal `42` $\implies$ `LOAD_I32 dst, 42`.
  - Large int `9223372036854775807` $\implies$ `LOAD_I64 dst, i64::MAX`.
  - Float `3.14` $\implies$ `LOAD_F64 dst, const_idx`.
  - String `"abc"` $\implies$ `LOAD_CONST dst, const_idx`.
* **Case 3.2 (3-Address Arithmetic & Register Mapping):**
  - `a + b * c`: verify precedence emits `MUL temp, b, c` first, then `ADD dst, a, temp`.
  - Floating point `x + y`: verify emits `ADD_F64`, not `ADD_I64`.
* **Case 3.3 (Short-Circuit Logical Evaluation):**
  - `a && b`: verify emits `JUMP_IF_FALSE a, skip_b`; if `a` is false, `b` is never evaluated.
  - `a || b`: verify emits `JUMP_IF_TRUE a, skip_b`; if `a` is true, `b` is never evaluated.
* **Case 3.4 (Relational & String Equality):**
  - `s1 == s2` (nominal string type): verify emits `EQ_STRING dst, s1, s2`.
  - `i < j`: verify emits `LT_I64 dst, i, j`.

---

### Phase 4: Control Flow, Loops, Jump Backpatching & Disassembler

#### 1. Code Deliverables (`crates/solix-compilation/src/phases/emitter/`)
* `control_flow.rs`:
  * Label management and back-patching engine.
  * Statement visitor for `IfStatement`, `WhileStatement`, `DoWhileStatement`, `ForStatement`, `SwitchStatement`, `BreakStatement`, `ContinueStatement`, `ReturnStatement`, `AssertStatement`, `ExitStatement`.
* `disassembler.rs`: Human-readable assembly text formatter.

#### 2. Exhaustive Test Suite (`crates/solix-compilation/tests/emitter_control_flow_tests.rs`)
* **Case 4.1 (If-Else Branching & Jump Offsets):**
  - Simple `if (c) { foo(); }`: verify `JUMP_IF_FALSE c, +offset` skips `foo()`.
  - `if (c) { foo(); } else { bar(); }`: verify `foo()` ends with unconditional `JUMP` over `bar()`.
  - Assert all jump offsets are positive, nonzero, and instruction-aligned (multiples of 8 bytes).
* **Case 4.2 (While & Do-While Loops):**
  - `while (c) { body; }`: verify backward jump points to condition test; condition `JUMP_IF_FALSE` points to exit.
  - `do { body; } while (c)`: verify condition `JUMP_IF_TRUE` jumps backward to body start.
* **Case 4.3 (For Loops):**
  - `for (int i = 0; i < 10; i++) { body; }`:
    - Verify initializer emits once before loop.
    - Verify step (`i++`) executes after body and jumps to condition.
* **Case 4.4 (Break & Continue Back-patching):**
  - `break` inside loop: verify jumps to instruction immediately following loop exit.
  - `continue` inside `for` loop: verify jumps to step increment, NOT condition test.
  - Nested loops: verify inner `break` exits only the inner loop.
* **Case 4.5 (Switch Case Jump Tables):**
  - Dense integer switch ($0, 1, 2, 3$): verify emits `SWITCH` jump table.
  - Verify default branch execution when no case matches.
* **Case 4.6 (Disassembler Rendering):**
  - Disassemble loop and if-else bytecode: assert clean labels (`L001:`, `L002:`), register names, and opcode formatting.

---

### Phase 5: Static Memory Partitioning & Synthesized Module Initializer (`@init`)

#### 1. Code Deliverables (`crates/solix-compilation/src/phases/emitter/`)
* Static field reservation: assign local static slots to static declarations.
* Synthesis of `@init` function (Function ID 0):
  * Compiles initializers for static variables in dependency order.
  * Emits `SET_STATIC slot_idx, value_reg`.
  * Emits `RETURN`.
* Emits `GET_STATIC` and `SET_STATIC` for static accesses within functions.
* Populates `static_symbols_map`.

#### 2. Exhaustive Test Suite (`crates/solix-compilation/tests/emitter_static_tests.rs`)
* **Case 5.1 (`@init` Synthesis & Execution Sequence):**
  - Class with `public static int32 count = 100;` and `public static int32 limit = count * 2;`.
  - Assert `@init` function is synthesized at function ID 0.
  - Assert `@init` emits `LOAD_I32 r0, 100`, `SET_STATIC 0, r0`, followed by computing `limit` and `SET_STATIC 1, r1`.
  - Assert `@init` ends with `RETURN`.
* **Case 5.2 (Intra-Module Static Access):**
  - Function reading static field emits `GET_STATIC dst_reg, 0`.
  - Function writing static field emits `SET_STATIC 0, src_reg`.
* **Case 5.3 (Cross-Module Static Access):**
  - Reading external static emits `GET_STATIC_GLOBAL dst_reg, symbol_idx`.
  - Verify `static_symbols_map` contains fully-qualified names and assigned static slots.

---

### Phase 6: Object Model, Classes, VTables & Flat ITables

#### 1. Code Deliverables (`crates/solix-compilation/src/phases/emitter/`)
* Translate bound class layouts into `CompiledClass` descriptors.
* Emits `ALLOC_OBJECT dst, vtable_id, size_bytes`.
* Emits constructor calls with `this` at $r_0$.
* Emits `GET_FIELD` and `SET_FIELD` with byte offsets.
* Emits `CALL_VIRTUAL` referencing VTable slot indices.
* Emits `CALL_INTERFACE` using flat ITable caches.
* Emits `INSTANCEOF` and `CAST_CHECK`.

#### 2. Exhaustive Test Suite (`crates/solix-compilation/tests/emitter_object_tests.rs`)
* **Case 6.1 (Object Allocation & Constructor Dispatch):**
  - `new User(1, "Alice")`:
    - Emits `ALLOC_OBJECT r_user, user_vtable_id, instance_size`.
    - Prepares window $[r_{\text{user}}, r_{\text{id}}, r_{\text{name}}]$.
    - Emits `CALL_DIRECT None, user_ctor_id, r_user, 3`.
* **Case 6.2 (Field Offsets & Alignment):**
  - Object with fields `int64 id` (offset 8) and `string name` (offset 16).
  - Assert `u.name` emits `GET_FIELD dst, r_user, 16`.
  - Assert `u.name = "Bob"` emits `SET_FIELD r_user, 16, r_val`.
* **Case 6.3 (Virtual Method Dispatch):**
  - `Base.foo()` at slot 0; `Derived` overrides `foo()`.
  - `obj.foo()` emits `CALL_VIRTUAL dst, obj_reg, 0, obj_reg, 1`.
  - Verify `slot_idx` matches the layout computed by binder.
* **Case 6.4 (Interface Dispatch with Flat ITables):**
  - Class implementing `IDescribable` (interface ID 5).
  - Assert `CompiledClass.itable` contains `(5, [vtable_slot])`.
  - Assert calling interface method emits `CALL_INTERFACE dst, obj_reg, 5, 0, obj_reg, 1`.
* **Case 6.5 (RTTI Cast & Instanceof):**
  - `obj instanceof User` emits `INSTANCEOF dst, obj, user_vtable_id`.
  - `(User)obj` emits `CAST_CHECK obj, user_vtable_id`.

---

### Phase 7: Arrays & Heterogeneous Tuples

#### 1. Code Deliverables (`crates/solix-compilation/src/phases/emitter/`)
* Emits `ALLOC_ARRAY`, `GET_ARRAY`, `SET_ARRAY`, `ARRAY_LENGTH`.
* Emits array literals.
* Emits `ALLOC_TUPLE`, `GET_TUPLE`.

#### 2. Exhaustive Test Suite (`crates/solix-compilation/tests/emitter_array_tuple_tests.rs`)
* **Case 7.1 (Array Allocation & Indexing):**
  - `new int32[10]`: emits `LOAD_I32 r_len, 10`, `ALLOC_ARRAY dst, TAG_INT32, r_len`.
  - `arr[3] = 42`: emits `SET_ARRAY arr, r_idx, r_val`.
  - `arr[3]`: emits `GET_ARRAY dst, arr, r_idx`.
  - `arr.length`: emits `ARRAY_LENGTH dst, arr`.
* **Case 7.2 (Array Literals):**
  - `[1, 2, 3]`: emits `ALLOC_ARRAY`, followed by 3 `SET_ARRAY` instructions for indices 0, 1, 2.
* **Case 7.3 (Tuple Construction & Access):**
  - `(10, "Solix", true)`: evaluates elements into contiguous window $[r_1, r_2, r_3]$, emits `ALLOC_TUPLE dst, 3, r_1`.
  - `t.1`: emits `GET_TUPLE dst, t, 1`.

---

### Phase 8: First-Class Lambdas, Closures & Function Pointers

#### 1. Code Deliverables (`crates/solix-compilation/src/phases/emitter/`)
* Capture analysis: identify free variables captured from enclosing functions.
* Synthesize hidden lambda method `<lambda_XX>`.
* Emits `PACK_CLOSURE dst_reg, lambda_func_id, [captured_regs...]`.
* Emits `UNPACK_CAPTURES [dst_regs...]` at lambda method entry.
* Emits `CALL_CALLABLE` to invoke closures and function pointers.

#### 2. Exhaustive Test Suite (`crates/solix-compilation/tests/emitter_closure_tests.rs`)
* **Case 8.1 (Free Variable Capture Analysis):**
  - Function declares `int32 factor = 5;`. Lambda `(x) => x * factor`.
  - Assert that `factor` is detected as a free variable and captured.
* **Case 8.2 (Closure Packing):**
  - Verify enclosing function emits `PACK_CLOSURE dst, lambda_func_id, r_factor`.
* **Case 8.3 (Synthesized Lambda Method):**
  - Assert synthesized lambda has `UNPACK_CAPTURES r_factor` at instruction 0.
  - Verify body evaluates `x * factor` and emits `RETURN`.
* **Case 8.4 (Callable Dispatch):**
  - Invoking closure variable `cb(10)` emits `CALL_CALLABLE dst, r_cb, r_arg, 1`.

---

### Phase 9: Safe ARC Tracking, Destructor Wiring & Exception Tables

#### 1. Code Deliverables (`crates/solix-compilation/src/phases/emitter/`)
* ARC Reference Tracking:
  * Emit `INC_REF` on reference copies and assignments.
  * Emit `DEC_REF` on scope exits, loop breaks, and function returns.
  * Populate `reference_registers: Vec<u16>` per function.
  * Wire user-defined class destructors (`~Class()`) into `CompiledClass.destructor_id`.
* Zero-Cost Exception Tables:
  * Emit `THROW` instructions.
  * Compile `try-catch-finally` into `ExceptionHandlerEntry` tables.
  * Implement dual-emission `finally`: inlined on normal flow and early returns (`return`, `break`, `continue`), while registered in exception tables for unwinding.

#### 2. Exhaustive Test Suite (`crates/solix-compilation/tests/emitter_exception_arc_tests.rs`)
* **Case 9.1 (ARC Retain / Release Placement):**
  - Reference assignment `User u1 = u2;`: emits `INC_REF r_u2`, `MOV r_u1, r_u2`.
  - Reassignment `u1 = null;`: emits `DEC_REF r_u1`, `LOAD_NULL r_u1`.
  - Block exit: verify all reference variables in block receive `DEC_REF`.
  - Loop `break`: verify reference variables declared inside loop receive `DEC_REF` prior to the jump.
* **Case 9.2 (Function Return ARC Cleanup):**
  - Function with reference variables `a` and `b` returning `a`:
    - Assert `DEC_REF r_b` is emitted.
    - Assert `r_a` is NOT decremented before return!
* **Case 9.3 (Zero-Cost Exception Tables & PC Bounds):**
  - `try { work(); } catch (Error e) { handle(); }`:
    - Assert zero instructions emitted at try entry.
    - Assert `try_start_pc` points exactly to the first instruction of `work()`.
    - Assert `try_end_pc` points exactly to the instruction immediately following `work()`.
    - Assert `catch_type_symbol == Some("Error")`.
    - Assert `catch_reg` is the register allocated for `e`.
    - Assert `handler_pc` points to `handle()`.
* **Case 9.4 (Dual-Emission Finally Semantics):**
  - Normal execution: verify `finally` block instructions execute after try block before jumping to exit.
  - Early `return` in try: verify `finally` block instructions are emitted BEFORE the `RETURN` instruction!
  - Early `break` in try: verify `finally` block instructions are emitted BEFORE the loop exit `JUMP`!
  - Unhandled exception: verify `finally_pc` is registered in `ExceptionHandlerEntry` for stack unwinding.

---

### Phase 10: Master End-to-End Fixtures & Milestone Integration

#### 1. Code Deliverables
* Master positive fixture: `tests/fixtures/emission/positive/master_all_statements_emission.slx`.
* Master negative fixture: `tests/fixtures/emission/negative/master_invalid_emission.slx`.
* Hook emitter into `CompilationContext`: implement `emit_context` and `emit_source`.
* Add `--dump-bytecode` CLI flag in `solix-client`.

#### 2. Exhaustive Test Suite (`crates/solix-compilation/tests/emitter_master_tests.rs`)
* **Case 10.1 (Comprehensive End-to-End Language Emission):**
  - Compile `master_all_statements_emission.slx` covering 100% of Solix language constructs:
    - Primitives, classes, interfaces, enums, inheritance, constructors, destructors.
    - Static fields, `@init` module synthesis, relocatable static memory accesses.
    - If-else, loops (while, do-while, for), breaks, continues, switch-case.
    - Arrays, multi-dim arrays, tuples, member accesses, virtual calls, interface calls.
    - First-class lambdas and closures.
    - Try-catch-finally with typed catch and rethrow.
  - Assert compilation and emission succeed with 0 errors.
* **Case 10.2 (SLXB Binary Round-Trip & Zero Degradation):**
  - Serialize emitted `BinaryProgram` to bytes.
  - Deserialize back with `BinaryProgram::from_bytes`.
  - Assert 100% bit-for-bit equivalence across all functions, instructions, classes, vtables, and exception tables.
* **Case 10.3 (Disassembly Inspection):**
  - Disassemble compiled program: verify all opcodes, register names, labels, and exception tables render cleanly without panics.
* **Case 10.4 (Full Definition of Done):**
  - `cargo test --workspace --all-targets` passes 100%.
  - `cargo clippy --workspace --all-targets -- -D warnings -D clippy::unwrap_used -D clippy::expect_used` zero defects.
  - `cargo fmt --all -- --check` clean.

---

## 5. Definition of Done (DoD) Verification

1. `cargo build --workspace --all-targets` compiles with **zero warnings**.
2. `cargo test --workspace --all-targets` passes **100%** across all 10 testing gates.
3. `cargo clippy --workspace --all-targets -- -D warnings -D clippy::unwrap_used -D clippy::expect_used` reports **zero defects**.
4. `cargo fmt --all -- --check` is **clean**.
5. `#![forbid(unsafe_code)]` preserved across all production crates.
6. Zero single-letter variable names in production code.
