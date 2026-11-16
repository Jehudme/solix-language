# Formal Specification: 64-Bit Register VM ISA & Opcode Matrix

## Document Metadata

* **Specification ID:** SPEC-0006
* **Scope:** Virtual Machine Instruction Set Architecture (`crates/solix-runtime/src/`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)
* **Rust Implementation:** [`crates/solix-common/src/bytecode/instruction.rs`](../../crates/solix-common/src/bytecode/instruction.rs)

---

## 1. Register Architecture & Execution Model

The Solix Virtual Machine operates as a register-based abstract machine with flat 64-bit little-endian instructions:

```text
┌────────────────────────────────────────────────────────┐
│ Solix Call Frame Structure                             │
├────────────────────────────────────────────────────────┤
│ • Program Counter (PC: u32)                            │
│ • Sliding Local Registers (R0 .. R[num_registers - 1]) │
│   - R0 .. R[param_count - 1]: Argument registers       │
│   - R[param_count] .. R[N]:   Local / scratch registers│
│ • Reference Register Bitmask (for ARC tracing)         │
│ • Caller Return Address & Destination Register         │
└────────────────────────────────────────────────────────┘
```

---

## 2. Instruction Formats (64-Bit Little-Endian)

Every instruction occupies exactly 8 bytes (`u64`):

```text
Format A (Three Registers):
┌───────────┬───────────────┬───────────────┬───────────────┬───────────┐
│ Opcode(8) │ Reg_Dst(16)   │ Reg_Src1(16)  │ Reg_Src2(16)  │ Flags(8)  │
└───────────┴───────────────┴───────────────┴───────────────┴───────────┘
 0         7 8            23 24           39 40           55 56       63

Format B (Register + 16-Bit Immediate):
┌───────────┬───────────────┬───────────────┬───────────────┬───────────┐
│ Opcode(8) │ Reg_Dst(16)   │ Reg_Src(16)   │ Imm16(16)     │ Flags(8)  │
└───────────┴───────────────┴───────────────┴───────────────┴───────────┘
 0         7 8            23 24           39 40           55 56       63

Format Call (Function Call & Windows):
┌───────────┬───────────────┬───────────────┬───────────────┬───────────┐
│ Opcode(8) │ Reg_Dst(16)   │ Target_ID(16) │ Arg_Base(8)   │ Arg_Count │
└───────────┴───────────────┴───────────────┴───────────────┴───────────┘
 0         7 8            23 24           39 40           47 48       63
```

---

## 3. Core Opcode Taxonomy

| Category | Opcodes | Description |
| :--- | :--- | :--- |
| **Loads & Stores** | `LoadConst`, `MoveReg`, `LoadStatic`, `StoreStatic`, `LoadField`, `StoreField` | Register and memory transfers |
| **Arithmetic** | `AddI32`, `SubI32`, `MulI32`, `DivI32`, `ModI32`, `AddI64`, `SubI64`, `AddF64`, `DivF64` | 32-bit and 64-bit checked arithmetic |
| **Superinstructions** | `AddI32Imm`, `SubI32Imm`, `CmpEqI32Imm`, `JumpIfZero`, `JumpIfNotZero` | Fused arithmetic and branch primitives |
| **Comparisons** | `CmpEqI32`, `CmpLtI32`, `CmpGtI32`, `CmpEqF64`, `CmpLtF64`, `CmpEqRef` | Evaluates boolean condition into register |
| **Control Flow** | `Jump`, `JumpIfTrue`, `JumpIfFalse`, `Return`, `ReturnVal` | Unconditional and conditional branching |
| **Object Allocation** | `AllocObject`, `AllocArray`, `AllocString`, `AllocObjectExtern` | VirtualHeap allocation |
| **Function Calls** | `CallDirect`, `CallVirtual`, `CallInterface`, `CallExtern`, `CallNative` | Sliding window function dispatch |
| **ARC & Memory** | `RetainRef`, `ReleaseRef`, `WeakRetain`, `WeakRelease`, `CheckNull` | Reference count management |
| **Exceptions** | `Throw`, `EnterTry`, `LeaveTry` | Table-driven unwinding triggers |

---

## 4. In-Place Dynamic Quickening

When executing multi-binary programs, the VM optimizes cross-binary and dynamic calls at runtime:
1. **`CallExtern` Quickening:** Upon first execution of a `CallExtern` instruction, the runtime resolves the destination symbol's absolute function index. It mutates the calling instruction in-place to `CallDirect`, eliminating future hash-table symbol lookups.
2. **`AllocObjectExtern` Quickening:** Overwritten with `AllocObject` once the class VTable ID is resolved.
3. **`GetStaticGlobal` Quickening:** Mutated to `GetStaticDirect` with the direct physical memory slot offset.

---

## 5. Modular Multi-Entrypoint Lifecycle & Symbol Hardening

For host embeddings, game engines, and long-running interactive host processes, the Solix Virtual Machine supports a modular, reusable multi-entrypoint lifecycle:

1. **Construction & One-Time Static Initialization:**
   - `VirtualMachine::new_initialized(context)` constructs the VM, allocates interned string constants on the `VirtualHeap`, and executes all module `@init` functions once.
   - Sets `statics_initialized = true`, populating global/static slots without executing `main()`.
2. **Successive Entrypoint Invocations (`invoke_entry`):**
   - `vm.invoke_entry(symbol, args)` validates that target function is `public` and all parameter/return types are primitive types, strings, or 1D primitive arrays.
   - Executes the entry function while preserving existing `static_slots` and `VirtualHeap` state across calls.
3. **Deterministic Shutdown & Resource Cleanup (`shutdown` & `Drop`):**
   - Replaces all static slots containing reference values with `null` and decrements their ARC counters (invoking class destructors `~Class()` on global singleton instances).
   - Frees interned string constants and verifies heap integrity.
4. **Load-Time Duplicate Symbol Rejection (`R0024`):**
   - When multiple `BinaryProgram` modules are loaded into `RuntimeContext::new`, any function, class, or static global with the same symbol defined in more than one module immediately triggers `RuntimeError::DuplicateSymbolDefinition` (`R0024`), excluding generic template instantiations.
