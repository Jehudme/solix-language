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

The bytecode crate (`solix-common`) enforces `#![forbid(unsafe_code)]`, while `solix-runtime/src/vm/` enforces `#![deny(unsafe_op_in_unsafe_fn)]` with documented `// SAFETY:` invariants on raw register/heap fast paths and JIT FFI trampolines.

---

## 2. Instruction Formats (Fixed 64-Bit Words)

Every instruction in the bytecode stream occupies exactly 8 bytes (`u64`), encoded in little-endian byte order:

### Arithmetic & Logic Operations

Arithmetic operations execute on Format A registers. Bitwise shift operations (`Shl`, `Shr`, `Lshr`) require non-negative shift counts; negative shift operands trap immediately with `RuntimeErrorCode::InvalidOperand` (exit code 26). Module static field initializers in `@init` evaluate in topological dependency order.

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
| `0x1E` | `NegI32` | A | `dst, src1` | Negation: `dst = -src1` (two's-complement wrapping on `i32::MIN`). |
| `0x1F` | `NegI64` | A | `dst, src1` | Negation: `dst = -src1` (two's-complement wrapping on `i64::MIN`). |
| `0x20` | `NegF64` | A | `dst, src1` | Negation: `dst = -src1`. |

> Signed arithmetic opcodes (`DivI32`, `DivI64`, `ModI32`, `ModI64`, `NegI32`, `NegI64`) trap strictly on division/modulo by zero and use defined two's-complement wrapping semantics on minimum-value overflow in both default and `--safe` execution modes. Set `SOLIX_TRACE_TRAPS=1` to emit rust backtraces when a `NullReferenceAccess` trap is constructed.

### 3.3. Bitwise & Logic Operations (`0x21` .. `0x2F`)
| Opcode Hex | Name | Format | Operands | Description |
|---|---|---|---|---|
| `0x21` | `Band` | A | `dst, src1, src2` | Bitwise AND (preserves `I32` when inputs are `I32`). |
| `0x22` | `Bor` | A | `dst, src1, src2` | Bitwise OR (preserves `I32` when inputs are `I32`). |
| `0x23` | `Bxor` | A | `dst, src1, src2` | Bitwise XOR (preserves `I32` when inputs are `I32`). |
| `0x24` | `Bnot` | A | `dst, src1` | Bitwise NOT (preserves `I32` when input is `I32`). |
| `0x25` | `Shl` | A | `dst, src1, src2` | Bitwise left shift (preserves `I32` when inputs are `I32`). |
| `0x26` | `Shr` | A | `dst, src1, src2` | Arithmetic right shift (preserves `I32` when inputs are `I32`). |
| `0x27` | `DivU32` | A | `dst, src1, src2` | Unsigned 32-bit division (traps on divide by zero). |
| `0x28` | `DivU64` | A | `dst, src1, src2` | Unsigned 64-bit division (traps on divide by zero). |
| `0x29` | `ModU32` | A | `dst, src1, src2` | Unsigned 32-bit modulo remainder (traps on divide by zero). |
| `0x2A` | `ModU64` | A | `dst, src1, src2` | Unsigned 64-bit modulo remainder (traps on divide by zero). |
| `0x2B` | `Lshr` | A | `dst, src1, src2` | Logical right shift (zero-fill right shift). |

### 3.4. Relational & Equality Comparisons (`0x30` .. `0x4F`, `0xEE`)
| Opcode Hex | Name | Format | Operands | Description |
|---|---|---|---|---|
| `0x30` | `EqI32` | A | `dst, src1, src2` | `dst = (src1 == src2)`. |
| `0x31` | `EqI64` | A | `dst, src1, src2` | `dst = (src1 == src2)`. |
| `0x32` | `EqF64` | A | `dst, src1, src2` | `dst = (src1 == src2)`. |
| `0x33` | `EqBool`| A | `dst, src1, src2` | `dst = (src1 == src2)`. |
| `0x34` | `EqString`| A | `dst, src1, src2` | String equality comparison. |
| `0x35` | `EqRef` | A | `dst, src1, src2` | Object address identity comparison. |
| `0x36` | `NeqI32` | A | `dst, src1, src2` | `dst = (src1 != src2)`. |
| `0x37` | `NeqI64` | A | `dst, src1, src2` | `dst = (src1 != src2)`. |
| `0x38` | `NeqF64` | A | `dst, src1, src2` | `dst = (src1 != src2)`. |
| `0x39` | `NeqBool` | A | `dst, src1, src2` | `dst = (src1 != src2)`. |
| `0x3A` | `NeqString` | A | `dst, src1, src2` | String inequality comparison. |
| `0x3B` | `NeqRef` | A | `dst, src1, src2` | Object address inequality comparison. |
| `0x3C` | `LtI32` | A | `dst, src1, src2` | `dst = (src1 < src2)`. |
| `0x3D` | `LtI64` | A | `dst, src1, src2` | `dst = (src1 < src2)`. |
| `0x3E` | `LtF64` | A | `dst, src1, src2` | `dst = (src1 < src2)`. |
| `0x3F` | `LteI32` | A | `dst, src1, src2` | `dst = (src1 <= src2)`. |
| `0x40` | `LteI64` | A | `dst, src1, src2` | `dst = (src1 <= src2)`. |
| `0x41` | `LteF64` | A | `dst, src1, src2` | `dst = (src1 <= src2)`. |
| `0x42` | `GtI32` | A | `dst, src1, src2` | `dst = (src1 > src2)`. |
| `0x43` | `GtI64` | A | `dst, src1, src2` | `dst = (src1 > src2)`. |
| `0x44` | `GtF64` | A | `dst, src1, src2` | `dst = (src1 > src2)`. |
| `0x45` | `GteI32` | A | `dst, src1, src2` | `dst = (src1 >= src2)`. |
| `0x46` | `GteI64` | A | `dst, src1, src2` | `dst = (src1 >= src2)`. |
| `0x47` | `GteF64` | A | `dst, src1, src2` | `dst = (src1 >= src2)`. |
| `0x48` | `Not` | A | `dst, src1` | Logical boolean negation (`!src1`). |
| `0x49` | `LtU32` | A | `dst, src1, src2` | `dst = (src1 < src2)` unsigned 32-bit. |
| `0x4A` | `LteU32`| A | `dst, src1, src2` | `dst = (src1 <= src2)` unsigned 32-bit. |
| `0x4B` | `GtU32` | A | `dst, src1, src2` | `dst = (src1 > src2)` unsigned 32-bit. |
| `0x4C` | `GteU32`| A | `dst, src1, src2` | `dst = (src1 >= src2)` unsigned 32-bit. |
| `0x4D` | `LtU64` | A | `dst, src1, src2` | `dst = (src1 < src2)` unsigned 64-bit. |
| `0x4E` | `LteU64`| A | `dst, src1, src2` | `dst = (src1 <= src2)` unsigned 64-bit. |
| `0x4F` | `GtU64` | A | `dst, src1, src2` | `dst = (src1 > src2)` unsigned 64-bit. |
| `0xEE` | `GteU64`| A | `dst, src1, src2` | `dst = (src1 >= src2)` unsigned 64-bit. |

### 3.5. Control Flow (`0x50` .. `0x53`)
| Opcode Hex | Name | Format | Operands | Description |
|---|---|---|---|---|
| `0x50` | `Jump` | C | `offset` | Unconditional signed jump: `pc += offset`. |
| `0x51` | `JumpIfTrue` | C | `cond, offset` | Conditional jump if `cond != 0`. |
| `0x52` | `JumpIfFalse` | C | `cond, offset` | Conditional jump if `cond == 0`. |
| `0x53` | `Switch` | B | `reg, table_idx` | Jump-table dispatch on integer/enum value. |

### 3.6. Function Invocations & Returns (`0x60` .. `0x69`)
| Opcode Hex | Name | Format | Operands | Description |
|---|---|---|---|---|
| `0x60` | `CallDirect` | Call | `dst, fn_id, base, count` | Direct static function call with sliding window. |
| `0x61` | `CallVirtual` | Call | `dst, slot, base, count` | Virtual method dispatch via VTable. |
| `0x62` | `CallInterface` | Call | `dst, itable_slot, base, count` | Interface method dispatch via ITable. |
| `0x63` | `CallSymbol` | Call | `dst, sym_idx, base, count` | Dynamic symbol dispatch. |
| `0x64` | `CallCallable` | Call | `dst, fn_reg, base, count` | Invokes a closure or function pointer register. |
| `0x65` | `Return` | A | — | Returns `void`. |
| `0x66` | `ReturnValue` | A | `val` | Returns `val` to caller's destination register. |
| `0x67` | `Exit` | A | `code` | Terminates VM execution with exit status `code`. |
| `0x68` | `CallExtern` | Call | `dst, sym_idx, base, count` | Invokes an external library function by symbol. |
| `0x69` | `CallNativeDirect` | Call | `dst, native_id, base, count` | Direct native FFI invocation by resolved index. |

### 3.7. Objects, Fields, Statics & ARC (`0x70` .. `0x79`, `0xC0` .. `0xC1`)
| Opcode Hex | Name | Format | Operands | Description |
|---|---|---|---|---|
| `0x70` | `AllocObject` | B | `dst, class_id` | Allocates class instance on heap with 8-byte header. |
| `0x71` | `GetField` | A | `dst, obj, offset` | Reads field word from heap object (`FLAG_WEAK` supported). |
| `0x72` | `SetField` | A | `obj, offset, src` | Writes field word to heap object (`FLAG_WEAK` / `FLAG_MOVE` supported). |
| `0x73` | `GetStatic` | B | `dst, static_addr` | Reads from static partition. |
| `0x74` | `SetStatic` | B | `src, static_addr` | Writes to static partition. |
| `0x75` | `GetStaticGlobal` | B | `dst, sym_idx` | Reads global static slot by symbol index. |
| `0x76` | `GetStaticDirect` | B | `dst, slot` | Reads resolved static slot directly. |
| `0x77` | `SetStaticGlobal` | B | `src, sym_idx` | Writes global static slot by symbol index. |
| `0x78` | `SetStaticDirect` | B | `src, slot` | Writes resolved static slot directly. |
| `0x79` | `AllocObjectExtern` | B | `dst, sym_idx` | Allocates external dependency class by symbol index. |
| `0x90` | `PackClosure` | Call | `dst, fn_id, base, count` | Allocates closure environment capturing `count` registers. |
| `0x91` | `UnpackCaptures` | A | `base, count` | Unpacks captured variables into registers starting at `base`. |
| `0xA0` | `Instanceof` | B | `dst, obj, vtable_id` | Runtime class/interface type check. |
| `0xA1` | `CastCheck` | B | `dst, obj, vtable_id` | Checked reference cast (throws `TypeCastException` on mismatch). |
| `0xA2` | `ConvIToF` | A | `dst, src` | Converts integer register to `float64`. |
| `0xA3` | `ConvFToI` | A | `dst, src` | Converts `float64` register to `int64`. |
| `0xA4` | `TupleInstanceof` | B | `dst, tuple_reg, idx` | Checks runtime type tag of tuple element at `idx`. |
| `0xB0` | `Throw` | A | `exception_reg` | Throws exception and initiates stack unwinding. |
| `0xC0` | `IncRef` | A | `obj` | Increments 26-bit reference count. |
| `0xC1` | `DecRef` | A | `obj` | Decrements reference count; triggers cleanup at 0. |

### 3.7. Array Operations & Element Tagging
- `AllocArray`: Allocates typed `ArrayRef` on the heap with element count capacity, exact storage byte stride (`1` byte for `int8`/`uint8`/`bool`/`char`, `2` bytes for `int16`/`uint16`, `4` bytes for `int32`/`uint32`/`float32`, and `8` bytes for 64-bit scalars and object references), and element type tag. For object reference arrays, Format A operand `src2` delivers the 16-bit target class `vtable_id` encoded into array header metadata bytes `5..7` to prevent truncated 7-bit vtable mismatches during monomorphized array store verification (`ArrayStoreMismatch`).
- `GetArray`: Retrieves array element by index using single-qword header validation and typed inline fast paths for `bool` (`(elem_size == 1, type_tag == 11)`), `int32` (`(elem_size == 4, type_tag == 3)`), `float64` (`(elem_size == 8, type_tag == 10)`), and 8-byte reference/`string` arrays (`elem_size == 8` with reference/string tags), delegating `float32`, `Tuple`, and cold fallback variants (`decode_array_element_slow`, which distinguishes unsigned byte arrays `type_tag == 5` masked to `u8` from 32-bit Unicode scalar `char[]` arrays `type_tag == 12` preserved as `Value::I32(raw_word as i32)`) out of line. All tuple and array indexing operations utilize checked arithmetic (`checked_mul`, `checked_add`), ensuring out-of-bounds or overflowing indices cleanly trigger `IndexOutOfBounds` (`R0004`) or `NullReferenceAccess` (`R0005`) traps rather than unhandled host panics.
- `SetArray`: Writes array element at specified index with specialized inline stores for `bool`, `int32`, `float32` (32-bit float conversion), `float64`, and reference arrays (including `FLAG_MOVE` `0x80` ownership transfer without extra `IncRef`/`DecRef`), delegating remaining variants to `execute_set_array_slow`.

### 3.8. Superinstructions (`0xD0` .. `0xEF`)
| Opcode Hex | Name | Format | Operands | Description |
|---|---|---|---|---|
| `0xD0` | `AddI32Imm` | A | `dst, src1, imm16` | Fused signed 32-bit immediate add: `dst = src1 + sign_extend(imm16)`. |
| `0xD1` | `SubI32Imm` | A | `dst, src1, imm16` | Fused signed 32-bit immediate sub: `dst = src1 - sign_extend(imm16)`. |
| `0xD2` | `JumpIfLtI32` | A | `lhs, rhs, offset` | Jumps by signed `offset` if `lhs < rhs` (`int32`). |
| `0xD3` | `JumpIfLteI32` | A | `lhs, rhs, offset` | Jumps by signed `offset` if `lhs <= rhs` (`int32`). |
| `0xD4` | `JumpIfGtI32` | A | `lhs, rhs, offset` | Jumps by signed `offset` if `lhs > rhs` (`int32`). |
| `0xD5` | `JumpIfGteI32` | A | `lhs, rhs, offset` | Jumps by signed `offset` if `lhs >= rhs` (`int32`). |
| `0xD6` | `JumpIfEqI32` | A | `lhs, rhs, offset` | Jumps by signed `offset` if `lhs == rhs`. |
| `0xD7` | `JumpIfNeqI32` | A | `lhs, rhs, offset` | Jumps by signed `offset` if `lhs != rhs`. |
| `0xD8` | `MulI32Imm` | A | `dst, src1, imm16` | Fused signed 32-bit immediate multiply: `dst = src1 * sign_extend(imm16)`. |
| `0xD9` | `ModI32Imm` | A | `dst, src1, imm16` | Fused 32-bit modulo by immediate using bitmasking for powers of two and compile-time `MOD_MAGIC_TABLE` reciprocal multiplication for positive divisors. |
| `0xDA` | `ForLoopI32` | A | `idx, bound, offset` (`flags`: step/cmp/fill) | Increments `reg[idx]` in place (`+1` when bit `7` clear, `+reg[flags & 0x1F]` when bit `7` set), tests `<` (bit `6` clear) or `<=` (bit `6` set) against `reg[bound]`, and branches back by `offset`. Bit `5` (`0x20`) marks tight 2-instruction `SetArray` primitive array fill loops for bulk completion (`try_complete_array_fill_loop`), and bit `4` (`0x10`) marks verified unit-stride scalar loop bodies (`try_complete_scalar_for_loop`). |
| `0xDB` | `AddBandI32` | A | `dst, src1, src2` (`flags`: `acc`) | Fused signed add and bitwise mask: `reg[dst] = (reg[acc] + reg[src1]) & reg[src2]`. |
| `0xDC` | `JumpIfI32Imm` | A | `lhs, imm16, offset` (`flags`: cmp kind) | Compares `reg[lhs]` (`int32`) directly against signed `imm16` (`Eq`/`Neq`/`Lt`/`Lte`/`Gt`/`Gte`) and branches by `offset`. |
| `0xDD` | `TailCallDirect` | Call | `dst, fn_id, arg_base, arg_count` | Tail-call replacement reusing the current caller `CallFrame` and register window directly when caller and callee hold no reference registers or exception handlers. |
| `0xDE` | `MulAddF64` | A | `dst, src1, src2` (`flags`: `addend`) | Fused IEEE-754 float64 multiply-add step: `reg[dst] = reg[src1] * reg[src2] + reg[addend]`. |
| `0xDF` | `GetArrayJumpIfFalse` | A | `arr, idx, offset` (`flags`: invert) | Loads primitive array element `reg[arr][reg[idx]]` and branches immediately on falsy (`flags == 0`) or truthy (`flags != 0`) without materializing an intermediate boolean register. |
| `0xE0` | `GetArrayJumpIfTrue` | A | `arr, idx, offset` (`flags`: invert) | Loads primitive array element `reg[arr][reg[idx]]` and branches immediately on truthy (`flags == 0`) or falsy (`flags != 0`) without materializing an intermediate boolean register. |
| `0xE1` | `AddF64Imm` | A | `dst, src1, const_idx` | Fuses float64 register addition with a constant pool immediate operand: `dst = reg[src1] + const_pool[const_idx]`. |
| `0xE2` | `SubF64Imm` | A | `dst, src1, const_idx` | Fuses float64 register subtraction with a constant pool immediate operand: `dst = reg[src1] - const_pool[const_idx]`. |
| `0xE3` | `MulF64Imm` | A | `dst, src1, const_idx` | Fuses float64 register multiplication with a constant pool immediate operand: `dst = reg[src1] * const_pool[const_idx]`. |
| `0xE4` | `JumpIfLtF64` | A | `lhs, rhs, offset` | Jumps by signed `offset` if `lhs < rhs` (`float64`). |
| `0xE5` | `JumpIfLteF64` | A | `lhs, rhs, offset` | Jumps by signed `offset` if `lhs <= rhs` (`float64`). |
| `0xE6` | `AddI64Imm` | A | `dst, src1, imm16` | Fused signed 64-bit immediate addition: `dst = src1 + sign_extend(imm16)`. |
| `0xE7` | `JumpIfLtI64` | A | `lhs, rhs, offset` | Jumps by signed `offset` if `lhs < rhs` (`int64`). |
| `0xE8` | `JumpIfLteI64` | A | `lhs, rhs, offset` | Jumps by signed `offset` if `lhs <= rhs` (`int64`). |
| `0xE9` | `GetFieldI32` | A | `dst, obj, offset` | Direct 32-bit integer field load from heap object payload: reads `int32` from `obj + offset` bypassing tag validation. |
| `0xEA` | `GetFieldF64` | A | `dst, obj, offset` | Direct 64-bit float field load from heap object payload: reads `float64` from `obj + offset` bypassing tag validation. |
| `0xEB` | `SetFieldI32` | A | `val, obj, offset` | Direct 32-bit integer field store into heap object payload: writes `int32` to `obj + offset` bypassing reference counting. |
| `0xEC` | `SetFieldF64` | A | `val, obj, offset` | Direct 64-bit float field store into heap object payload: writes `float64` to `obj + offset` bypassing reference counting. |
| `0xED` | `CallVirtualDirect` | Call | `dst, target_fn, arg_base, arg_count` | Monomorphic quickened virtual call: invokes direct function target `target_fn` if vtable ID matches the flat `Vec`-indexed monomorphic call site cache (`(func_id, caller_pc)`); on class miss, unquickens to `CallVirtual` with the polymorphic bit (0x8000) set in target slot to avoid repeated quickening overhead. |

---

## 4. Call Frame Layout & Sliding Windows

When a function executes, its activation record occupies a `CallFrame`:
- **Sliding Register Base**: When calling `CallDirect(dst, fn_id, arg_base, arg_count)`, registers `r[arg_base .. arg_base + arg_count]` become `r[0 .. arg_count]` in the callee frame.
- **Direct Dispatch Pointer Caching**: In `run_fast_trap`, `current_binary_id` and `current_static_base` are cached in local execution stack variables across frame transitions (avoiding extra pointer locals that would increase register pressure in `'dispatch_loop`), enabling constant pool loads (`LoadF64`, `LoadConst`, `AddF64Imm`, `SubF64Imm`, `MulF64Imm` via `const_pool_cache[current_binary_id]`) and static slot offsets (`GetStatic`, `SetStatic`) to execute with zero frame lookup overhead.
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

Exceptions use static metadata tables (`ExceptionHandlerEntry`) rather than inline setup instructions. When `Throw(reg)` executes, ownership of owned/temp reference exceptions transfers cleanly into the unwinder (while borrowed parameter throws retain an in-flight reference) without double-running `finally` blocks when `catch` handles or rethrows. The runtime selects the innermost (narrowest `try_start_pc..try_end_pc` span) matching handler first (preferring explicit typed `catch` handlers over `<any>` `finally` unwinders within the same span), cleans intermediate reference registers, and jumps directly to `handler_pc`. If an exception escapes the call stack entirely, `format_exception_message` resolves the object class symbol and string payload rather than emitting raw heap address tokens. Both `.slxb` codecs (`BinaryProgram::try_to_bytes`, `BinaryProgram::from_bytes`, `ByteReader::checked_capacity`, `ByteReader::read_flag_bool`, `ByteReader::read_u64_usize`, and strict JSON `Type` validation) and the AOT bytecode verifier (`verify_program` / `verify_program_with_classes`) enforce container bounds, `checked_add` branch-offset arithmetic, strict per-binary constant-pool isolation, `ForLoopI32` step-register and reserved-flag checks, `MAX_TUPLE_ELEMENTS` (`4096`) tuple limits, and relocated class metadata limits (`AllocObject` vtable/size, non-header 4-byte-aligned `GetField*`/`SetField*` bounds, `CallVirtual` slot bounds, and `CallInterface` table bounds) prior to execution.

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

The production interpreter loop (`run_fast_trap::<false, TRACK_METRICS>`) implements several structural optimizations while preserving exact ISA semantics:

- **Monomorphized Metrics Gating (`TRACK_METRICS`)**: `run_fast_internal` instantiates `<false, false>` when telemetry flags (`--metrics`) are omitted via `RuntimeOptions::track_metrics`, eliding per-instruction `local_inst_count` updates and peak depth comparisons across tight loops.
- **Compact `VmTrap` Error ABI**: Fast-path handlers return `Result<_, VmTrap>` (`VmTrap(Box<RuntimeError>)`) instead of passing 96-byte `RuntimeError` values by stack `sret`, eliminating stack drop glue and keeping interpreter state resident in machine registers.
- **Compact 32-Byte `CallFrame`, Cached `inst_ptr`, Sentinel `Return`, & Trivial Leaf Fast-Call**: `CallFrame` (`#[repr(C)]`, compile-time asserted 32 bytes) caches `inst_ptr: *const Instruction` directly so `Return`/`ReturnValue` restore caller instruction pointers without re-indexing `func_code`. `RuntimeContext::new` appends a sentinel `OpCode::Return` when needed so `run_fast_trap::<false, _>` omits per-instruction `pc >= inst_len` bounds checks, and `'call_dispatch` evaluates 1–2 instruction trivial leaf functions inline without pushing a `CallFrame`.
- **Zero-Spill Direct Register Writeback & Pair Pattern Matching**: Arithmetic, bitwise, comparison, branch, `GetField`, and `GetArray` arms write fast-path results directly to `*reg_base.add(dst)` inside each branch (eliminating 16-byte `Value` phi-node stack spills) and match `(Value::I32(left_val), Value::I32(right_val))` in a single check before calling `#[cold]` fallbacks (eliminating callee-saved `inst_ptr` spills).
- **Ownership-Transfer Stores (`FLAG_MOVE`) & Cross-Crate Inlining**: `SetField` and `SetArray` support `FLAG_MOVE` (`0x80`) to move fresh owned rvalues straight into heap slots and null out the dead source register without `IncRef`/`DecRef` cycles. `Instruction` field accessors (`raw_opcode`, `dst`, `src1`, `src2`, `imm32`, `jump_offset`, `call_target`, `arg_base`, `arg_count`) and `OpCode::from_raw_u8` (`const fn` table decode) are `#[inline(always)]` in `solix-common`, alongside in-place `Value` readers in `solix-runtime`.
- **Weak Field Guards (`FLAG_WEAK`) & Scalar Loop Tagging**: Both `GetField` and `SetField` encode `FLAG_WEAK` (`0x01`) for `weak` fields so `GetField` bypasses the unchecked inline path and validates target allocation generations via `VirtualHeap`, nullifying dangling slots on deallocation/reuse. `ForLoopI32` scalar-loop quick-tagging (`0x10`) excludes `JumpIfLtF64`/`JumpIfLteF64`/`JumpIfLtI64`/`JumpIfLteI64` bodies so non-`i32` comparisons execute cleanly on the general dispatch loop.
- **Stateless Closure Encoding (`PackClosure`) & Inline Unpack**: Closures with `cap_count == 0` encode directly as `Value::FnPtr(target_lambda_id)` without heap allocation or `DecRef` cleanup, and 1–2 capture `UnpackCaptures` executes inline.
- **`idiv`-Free Modulo (`ModI32Imm`) & Cold-Path Outlining**: `ModI32Imm` uses bitmasks for powers of two and compile-time `MOD_MAGIC_TABLE` reciprocal multiplies for positive constants, while `CallSymbol`, `CallExtern`, `AllocObjectExtern`, `GetStaticGlobal`, `SetStaticGlobal`, `Throw`, tuple/cast/heap-closure operations, native callback marshaling, and `release_reference_cold` execute out-of-line for L1 instruction cache locality. `CallSymbol` quickening to `OpCode::CallNativeDirect` is strictly guarded on `function_meta[global_id].native_fn.is_some()` so package-scoped native stub wrappers falling back to global builtins cannot self-quicken into infinite recursion. `CallExtern` emits parameter-signature-mangled symbols (`pkg::Type::method(t1,t2)`) for non-zero-arity external calls and `/0` for zero-arity calls, and `RuntimeContext::resolve_function_symbol` enforces deterministic parameter-signature and arity matching (while `execute_call_extern` strips signatures/arities when checking `native_registry`), preventing cross-package overload collisions.
- **Scalar Loop Bulk Execution, Post-LICM Fusion & Self-Recursion Accumulator Lowering**: `RuntimeContext` post-verification analysis (`mark_scalar_for_loops`) flags pure unit-stride scalar `ForLoopI32` loops with bit `4` (`0x10`), routed alongside array-fill loops through `try_complete_special_for_loop` (`try_complete_scalar_for_loop` with pre-decoded unboxed straight-line `i32` execution). In the compiler, direct `solix run` sets `emit_export_metadata = false` to skip unused C/header export generation, emitter line-offset indexing caches offsets once per source file, `fuse_post_licm_add_load_band` fuses hoisted mask additions into `AddBandI32`, and `transform_binary_additive_self_recursion` lowers binary self-recursive loops into hybrid accumulator loops.
- **Cranelift JIT Compilation (`vm::jit`, Target-Gated `jit` Feature)**: On `x86_64` and `aarch64` non-Wasm targets when compiled with `solix-runtime`'s default `jit` feature, `JitEngine` (`cranelift-jit` + `cranelift-frontend`) lazily compiles eligible bytecode functions (<= 4 parameters, no exception handlers, and CFG-aware per-instruction register type inference supporting register reuse via a typed `VarTable`) into native machine code using unboxed `i32`/`i64`/`f64`/heap-address registers and a C-ABI `JitContext`. `vm::jit` pre-compiles direct callees (`CallDirect`/`TailCallDirect`), lowers self-recursive `TailCallDirect` to zero-overhead loops, inlines compile-time constant-pool values (`LoadF64`, `LoadConst`, `AddF64Imm`/`SubF64Imm`/`MulF64Imm`, interned string addresses), lowers `ModI32Imm` via bitmasks and Granlund-Montgomery signed reciprocal multiplies, fast-paths `EqString`/`NeqString` with pointer and header-length checks, and lowers scalar, object-field (`GetField`/`SetField`), array/string (`AllocArray`, inline `GetArray`/`SetArray` `ArrayRef` fast paths with `String` fallback in `jit_get_array`, `GetArrayJumpIfFalse`/`GetArrayJumpIfTrue`, `ArrayLength` for both `ArrayRef` and `String`), `CallSymbol` (including native `Math::sqrt` -> `fsqrt`), `CallNativeDirect`, and inline `IncRef`/`DecRef` ARC fast paths (updating `BlockHeader` reference counts in Cranelift IR with `MAX_REF_COUNT` immortal-object skips and `JitContext::cycle_collector_enabled` guards, calling `rt_inc_ref`/`rt_dec_ref` only on overflow, zero-refcount release, or cycle-collector root buffering) while preserving cycle-collector purple-root buffering (`push_purple_candidate_cold`) and dead-return register safety. Execution mode is controlled via `RuntimeOptions::jit_mode` (`JitMode::Tiered` [default: compiles functions with loops on first call or non-loop functions after 50 calls], `JitMode::Eager`, or `JitMode::Off`), with compilation counts (`jit_compiled_functions`) and latency (`jit_compile_time_ns`) reported in `VmTelemetry`.

---

## 9. Modular Multi-Entrypoint Lifecycle & Symbol Hardening

For host embeddings, game engines, and long-running interactive processes, the Solix Virtual Machine supports a reusable multi-entrypoint lifecycle:

1. **Construction & One-Time Static Initialization (`VirtualMachine::new_initialized` / `VirtualMachine::try_new`):**
   - Allocates interned string constants on the `VirtualHeap`, propagating `RuntimeError::OutOfMemory` or memory-write failures immediately rather than substituting null pointers, and executes all module `@init` initializers once (`statics_initialized = true`), populating global/static slots without invoking `main()`.
2. **Successive Entrypoint Invocations (`invoke_entry`):**
   - Validates that the target entrypoint is `public` and uses primitive, `string`, or 1D primitive array parameter/return types, executing the entrypoint while preserving `static_slots` and `VirtualHeap` state across calls.
3. **Deterministic Shutdown & Cleanup (`shutdown` & `Drop`):**
   - Nullifies static reference slots, invokes class destructors (`~ClassName()`) on global singletons, and reclaims interned constant strings cleanly.
4. **Load-Time Duplicate Symbol Rejection (`R0024`):**
   - When loading multiple `BinaryProgram` modules into `RuntimeContext::new`, duplicate non-generic function, class, or static global symbols immediately raise `RuntimeError::DuplicateSymbolDefinition` (`R0024`).

---

## 10. Container Binary Serialization & Declared Dynamic Libraries

The bytecode serialization format preserves project-level dynamic library dependencies to support automated FFI linkage:

* **`BinaryProgram` Manifest Metadata:** Encodes `dynamic_libraries: Vec<String>` alongside exported functions, classes, and native checksum tables.
* **`ExportManifest` Integration:** Stores declared shared library identifiers into Section 6 (`Metadata & ExportManifest`), allowing standalone packages and container runners to verify and quicken external native symbols deterministically without external manifest lookups. Tooling and compiler dependency resolution can also decode Sections 6 and 7 directly via `BinaryProgram::from_bytes_manifest_only` without allocating Sections 2–5.

---

See [`.slxasm` Disassembly Format Specification](disassembly-format-slxasm.md) for reading emitted bytecode listings.
