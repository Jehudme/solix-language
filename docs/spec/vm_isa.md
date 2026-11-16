# Solix Virtual Machine Instruction Set Architecture (Register VM)

## Document Metadata

* **Specification ID:** SPEC-0006
* **Scope:** 64-Bit Register Virtual Machine Architecture (`crates/solix-runtime/src/`, `crates/solix-common/src/bytecode/`)
* **Status:** Final Normative Standard
* **Implementation Reference:** [`crates/solix-common/src/bytecode/instruction.rs`](../../crates/solix-common/src/bytecode/instruction.rs), [`crates/solix-runtime/src/vm/engine.rs`](../../crates/solix-runtime/src/vm/engine.rs)

---

## 1. VM Architecture Overview

The Solix Virtual Machine operates as a high-throughput **64-bit register-based abstract machine** executing uniform 8-byte instruction words. Unlike legacy stack-based bytecode architectures, the register VM eliminates stack thrashing, minimizes memory traffic, and provides high cache locality.

```text
┌────────────────────────────────────────────────────────────────────────┐
│ Solix Call Frame Execution State                                      │
├────────────────────────────────────────────────────────────────────────┤
│ • Program Counter (pc: u32)                                            │
│ • Sliding Local Registers (r0 .. r[num_registers - 1])                 │
│   - r0 .. r[param_count - 1]: Parameter registers                      │
│   - r[param_count] .. r[N]:   Local variables & temporary registers    │
│ • Reference Register Bitmask (active registers tracked by ARC)         │
│ • Caller Return Target Register & Return Address                       │
└────────────────────────────────────────────────────────────────────────┘
```

The runtime enforces `#![forbid(unsafe_code)]` across all virtual machine execution loops and memory operations.

---

## 2. Instruction Formats (Fixed 64-Bit Words)

Every instruction in the bytecode stream occupies exactly 8 bytes (`u64`), encoded in little-endian byte order:

### Format A (Three Registers — Arithmetic & Logic)
Used for binary operations, comparisons, and register moves:
```text
┌───────────┬───────────┬──────────────┬──────────────┬──────────────┐
│ Opcode(8) │ Flags(8)  │ Reg_Dst(16)  │ Reg_Src1(16) │ Reg_Src2(16) │
└───────────┴───────────┴──────────────┴──────────────┴──────────────┘
 0         7 8        15 16          31 32          47 48          63
```

### Format B (Register + 32-Bit Immediate / Constant Index)
Used for loading constants, direct integer immediates, and memory offsets:
```text
┌───────────┬───────────┬──────────────┬─────────────────────────────┐
│ Opcode(8) │ Flags(8)  │ Reg_Dst(16)  │ Immediate / Index(32)       │
└───────────┴───────────┴──────────────┴─────────────────────────────┘
 0         7 8        15 16          31 32                         63
```

### Format C (Conditional Branch with Signed PC Offset)
Used for loops and conditional branching:
```text
┌───────────┬───────────┬──────────────┬─────────────────────────────┐
│ Opcode(8) │ Flags(8)  │ Reg_Cond(16) │ Signed PC Offset(32)        │
└───────────┴───────────┴──────────────┴─────────────────────────────┘
 0         7 8        15 16          31 32                         63
```

### Format Call (Windowed Call)
Used for static, direct, and virtual function dispatch:
```text
┌───────────┬──────────────┬──────────────┬──────────────┬───────────┐
│ Opcode(8) │ Reg_Dst(16)  │ Target_ID(16)│ Arg_Base(16) │ Arg_Count │
└───────────┴──────────────┴──────────────┴──────────────┴───────────┘
 0         7 8           23 24          39 40          55 56       63
```

---

## 3. Complete Register Opcode Matrix

### 3.1. Constants & Register Movement (`0x00` .. `0x0F`)
| Opcode Hex | Name | Format | Operands | Description |
|---|---|---|---|---|
| `0x00` | `Nop` | A | — | No operation. |
| `0x01` | `Mov` | A | `dst, src1` | Copies register `src1` to `dst`. |
| `0x02` | `LoadI32` | B | `dst, imm32` | Loads 32-bit signed immediate into `dst`. |
| `0x03` | `LoadI64` | B | `dst, const_idx` | Loads 64-bit integer from constant pool. |
| `0x04` | `LoadF64` | B | `dst, const_idx` | Loads 64-bit float from constant pool. |
| `0x05` | `LoadBool` | B | `dst, imm32` | Loads boolean (`0` or `1`) into `dst`. |
| `0x06` | `LoadNull` | B | `dst` | Sets register `dst` to `0` (null reference). |
| `0x07` | `LoadConst`| B | `dst, const_idx` | Loads string or object constant from pool. |

### 3.2. Arithmetic Operations (`0x10` .. `0x20`)
| Opcode Hex | Name | Format | Operands | Description |
|---|---|---|---|---|
| `0x10` | `AddI32` | A | `dst, src1, src2` | Signed 32-bit addition: `dst = src1 + src2`. |
| `0x11` | `AddI64` | A | `dst, src1, src2` | Signed 64-bit addition: `dst = src1 + src2`. |
| `0x12` | `AddF64` | A | `dst, src1, src2` | 64-bit floating addition: `dst = src1 + src2`. |
| `0x13` | `SubI32` | A | `dst, src1, src2` | Signed 32-bit subtraction: `dst = src1 - src2`. |
| `0x14` | `SubI64` | A | `dst, src1, src2` | Signed 64-bit subtraction: `dst = src1 - src2`. |
| `0x15` | `SubF64` | A | `dst, src1, src2` | 64-bit floating subtraction: `dst = src1 - src2`. |
| `0x16` | `MulI32` | A | `dst, src1, src2` | Signed 32-bit multiplication. |
| `0x17` | `MulI64` | A | `dst, src1, src2` | Signed 64-bit multiplication. |
| `0x18` | `MulF64` | A | `dst, src1, src2` | 64-bit floating multiplication. |
| `0x19` | `DivI32` | A | `dst, src1, src2` | Signed 32-bit division (traps on divide by zero; wrapping on `i32::MIN / -1`). |
| `0x1A` | `DivI64` | A | `dst, src1, src2` | Signed 64-bit division (traps on divide by zero; wrapping on `i64::MIN / -1`). |
| `0x1B` | `DivF64` | A | `dst, src1, src2` | 64-bit floating division. |
| `0x1C` | `ModI32` | A | `dst, src1, src2` | Modulo remainder (traps on divide by zero; wrapping on `i32::MIN % -1`). |
| `0x1D` | `ModI64` | A | `dst, src1, src2` | Modulo remainder (traps on divide by zero; wrapping on `i64::MIN % -1`). |
| `0x1E` | `NegI32` | A | `dst, src1` | Negation: `dst = -src1`. |
| `0x1F` | `NegI64` | A | `dst, src1` | Negation: `dst = -src1`. |
| `0x20` | `NegF64` | A | `dst, src1` | Negation: `dst = -src1`. |

### 3.3. Bitwise & Logic Operations (`0x21` .. `0x2F`)
| Opcode Hex | Name | Format | Operands | Description |
|---|---|---|---|---|
| `0x21` | `Band` | A | `dst, src1, src2` | Bitwise AND (preserves `I32` when inputs are `I32`). |
| `0x22` | `Bor` | A | `dst, src1, src2` | Bitwise OR (preserves `I32` when inputs are `I32`). |
| `0x23` | `Bxor` | A | `dst, src1, src2` | Bitwise XOR (preserves `I32` when inputs are `I32`). |
| `0x24` | `Bnot` | A | `dst, src1` | Bitwise NOT (preserves `I32` when input is `I32`). |
| `0x25` | `Shl` | A | `dst, src1, src2` | Bitwise left shift (preserves `I32` when inputs are `I32`). |
| `0x26` | `Shr` | A | `dst, src1, src2` | Arithmetic right shift (preserves `I32` when inputs are `I32`). |

### 3.4. Relational & Equality Comparisons (`0x30` .. `0x47`)
| Opcode Hex | Name | Format | Operands | Description |
|---|---|---|---|---|
| `0x30` | `EqI32` | A | `dst, src1, src2` | `dst = (src1 == src2)`. |
| `0x31` | `EqI64` | A | `dst, src1, src2` | `dst = (src1 == src2)`. |
| `0x32` | `EqF64` | A | `dst, src1, src2` | `dst = (src1 == src2)`. |
| `0x33` | `EqBool`| A | `dst, src1, src2` | `dst = (src1 == src2)`. |
| `0x34` | `EqString`| A | `dst, src1, src2` | String equality comparison. |
| `0x35` | `EqRef` | A | `dst, src1, src2` | Object address identity comparison. |
| `0x36` | `NeqI32`| A | `dst, src1, src2` | `dst = (src1 != src2)`. |
| `0x3C` | `LtI32` | A | `dst, src1, src2` | `dst = (src1 < src2)`. |
| `0x3D` | `LtI64` | A | `dst, src1, src2` | `dst = (src1 < src2)`. |
| `0x3E` | `LtF64` | A | `dst, src1, src2` | `dst = (src1 < src2)`. |
| `0x3F` | `LteI32`| A | `dst, src1, src2` | `dst = (src1 <= src2)`. |
| `0x42` | `GtI32` | A | `dst, src1, src2` | `dst = (src1 > src2)`. |
| `0x45` | `GteI32`| A | `dst, src1, src2` | `dst = (src1 >= src2)`. |

### 3.5. Memory & Object Allocation (`0x60` .. `0x6F`)
| Opcode Hex | Name | Format | Operands | Description |
|---|---|---|---|---|
| `0x60` | `NewObject` | B | `dst, class_id` | Allocates class instance on heap with 8-byte header. |
| `0x61` | `GetField` | A | `dst, obj, offset` | Reads field word from heap object. |
| `0x62` | `SetField` | A | `obj, offset, src` | Writes field word to heap object. |
| `0x63` | `GetStatic` | B | `dst, static_addr` | Reads from static partition. |
| `0x64` | `SetStatic` | B | `src, static_addr` | Writes to static partition. |
| `0x65` | `IncRef` | A | `obj` | Increments 28-bit reference count. |
| `0x66` | `DecRef` | A | `obj` | Decrements reference count; triggers cleanup at 0. |

### 3.6. Control Flow & Function Calls (`0x80` .. `0x8F`)
| Opcode Hex | Name | Format | Operands | Description |
|---|---|---|---|---|
| `0x80` | `Jump` | C | `offset` | Unconditional signed jump: `pc += offset`. |
| `0x81` | `JumpIfTrue` | C | `cond, offset` | Conditional jump if `cond != 0`. |
| `0x82` | `JumpIfFalse`| C | `cond, offset` | Conditional jump if `cond == 0`. |
| `0x83` | `CallDirect` | Call | `dst, fn_id, base, count` | Direct static function call with sliding window. |
| `0x84` | `CallVirtual`| Call | `dst, slot, base, count` | Virtual method dispatch via VTable. |
| `0x85` | `CallInterface`| Call | `dst, itable_slot, base, count` | Interface method dispatch via ITable. |
| `0x86` | `Ret` | A | `val` | Returns `val` to caller's destination register. |
| `0x87` | `RetVoid` | A | — | Returns `void`. |

### 3.7. Array Operations & Element Tagging
- `AllocArray`: Allocates typed `ArrayRef` on the heap with specified element capacity, stride, and element type tag.
- `GetArray`: Retrieves array element by index. For `float32` arrays (`type_tag == 9`), transparently handles both 4-byte native bit patterns and 8-byte VM register storage representations.
- `SetArray`: Writes array element at specified index, converting value to the primitive storage representation matching the array type tag.

---

## 4. Call Frame Layout & Sliding Windows

When a function executes, its activation record occupies a `CallFrame`:
- **Sliding Register Base**: When calling `CallDirect(dst, fn_id, arg_base, arg_count)`, registers `r[arg_base .. arg_base + arg_count]` become `r[0 .. arg_count]` in the callee frame.
- **Max Call Stack Depth**: The virtual machine limits recursion to `65,536` frames, raising `RuntimeError::StackOverflow` if exceeded.

---

## 5. Byte-Addressed Virtual Heap & 8-Byte Header

All heap blocks reside in a byte-addressed byte buffer (`Vec<u8>`) partitioned into:
1. **Static Partition** (`address < static_size`): Pre-allocated global and static class variables.
2. **Dynamic Heap Partition** (`address >= static_size`): Dynamically allocated objects and arrays.

Every dynamic block is preceded by an 8-byte packed header:
```text
Bytes 0..4 (32 bits): [ size (u32 byte capacity or element count) ]
Bytes 4..8 (32 bits): [ BlockTag (4 bits) | ref_count (28 bits)   ]
```

---

## 6. Zero-Cost Table-Driven Exception Handling

Exceptions use static metadata tables (`ExceptionHandlerEntry`) rather than inline setup instructions. When `Throw(reg)` executes, the runtime searches the static handler table in reverse, checks class inheritance via `instanceof`, decrements intermediate reference registers, and jumps directly to `catch_pc`.

---

## 7. Interactive Debug Hooks & Execution Monomorphization

The virtual machine supports zero-overhead interactive debugging via the `VmDebugHook` trait:

- **Hook Dispatch**: `VirtualMachine::run_debug(&mut hook)` checks `metadata.has_debug_symbols` per call frame. For functions compiled under `BuildProfile::Debug` with source maps, the hook's `before_instruction`, `on_exception_thrown`, and `on_uncaught_exception` methods are invoked.
- **Monomorphic Inner Loop**: Instructions execute through monomorphized `run_fast_step` to prevent dynamic dispatch overhead in tight instruction execution cycles.
- **Output Redirection**: Guest output streams (`println`, `print`) are intercepted non-destructively through thread-safe `OutputSink` callbacks for DAP stdio integration without process stream pollution.
- **Interactive Heap & Frame Introspection**:
  - `VirtualMachine::inspect_frame_locals(frame_index)`: inspects all active named parameters and local variables in a call frame using `LocalVariableDebugInfo`.
  - `VirtualMachine::inspect_object_fields(heap_address)`: inspects all declared fields of an object using `CompiledClass.field_names` looked up from `vtable_lookup`.
  - `VirtualMachine::inspect_array_elements(heap_address)`: inspects indexed elements of a heap-allocated array (`BlockTag::ArrayRef`), extracting elemental types and runtime values for DAP debugger visualization.

---

## 8. Fast-Path Dispatch & Execution Optimizations

The production interpreter loop (`run_fast_internal::<false>`) implements several structural optimizations while preserving exact ISA semantics:

- **Cross-Crate Opcode & Operand Inlining**: `Instruction` field accessors (`raw_opcode`, `dst`, `src1`, `src2`, `imm32`, `jump_offset`, `call_target`, `arg_base`, `arg_count`) and `OpCode::from_raw_u8` (`const fn` table decode) are `#[inline(always)]` in `solix-common`, alongside hot `Value` accessors in `solix-runtime`.
- **Stateless Closure Encoding (`PackClosure`)**: Closures with `cap_count == 0` encode directly as tagged `Value::FnPtr(target_lambda_id)` values without heap allocation or `DecRef` cleanup.
- **Direct Window Push & Lazy Errors**: `push_frame_window` pushes call frames and returns cached `(inst_ptr, inst_len, static_base, binary_id, reg_base)` directly without redundant frame lookups, and error constructors use lazy `.ok_or_else(...)` / `#[cold] #[inline(never)]` helpers to avoid `RuntimeError` drop glue in hot paths.
- **Cold-Path Outlining**: Infrequent or string/heap-heavy handlers (`CallSymbol`, `CallExtern`, `CallInterface`, `AllocObjectExtern`, `GetStaticGlobal`, `SetStaticGlobal`, `Throw`, tuple/cast/heap-closure operations, native callback marshaling, and `release_reference_cold`) execute out-of-line from the main dispatch loop to preserve L1 instruction cache locality and minimize interpreter stack frame size.
