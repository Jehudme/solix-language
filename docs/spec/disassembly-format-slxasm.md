# Solix Disassembly Format Specification (`.slxasm` / `.slxdasm`)

## Document Metadata

* **Specification ID:** SPEC-DISASM-001
* **Scope:** Compiler & Inspector Textual Disassembly Output (`crates/solix-compilation/src/phases/emitter/disassembler.rs`)
* **Status:** Final Normative Standard
* **Related Specs:** [64-Bit Register VM ISA](vm_isa.md), [`solix compile`](tools/cli/compile.md), [`solix info`](tools/cli/info.md)

---

## 1. Generating Disassembly Output

Solix produces human-readable register VM assembly listings from either source compilation or precompiled binary containers:

```bash
# Emit .slxasm alongside compiled .slxb during compilation
solix compile src/main.slx --emit-disassembly

# Inspect an existing .slxb or .slxarc archive on stdout
solix info build/release/app.slxb --disasm
```

Inside Visual Studio Code, **Solix: Inspect 64-Bit Register VM Disassembly** renders an ephemeral `.slxdasm` document backed by the exact same disassembler output.

---

## 2. Container Section Layout

A complete `.slxb` disassembly consists of four ordered sections:

```text
1. Binary Header Banner       ; Version, Flags, Entry Point, Static Reservation
2. .constant_pool:            ; $c00, $c01, ... (64-bit ints, floats, interned strings)
3. .classes:                  ; Class symbols, VTable slot tables, and ITable maps
4. .functions (N total):      ; Per-function activation layout, handlers, and .code instructions
```

### 2.1 Binary Header Banner

```text
; ================================================================
; Solix SLXB Binary Disassembly
; Version: 2.0, Flags: 0x00000000
; Entry Point: Some("CounterApp::main") (id: Some(2))
; Static Reservation: 0 slots
; ================================================================
```

* **`Version`:** Container major/minor format version.
* **`Flags`:** Bitmask encoding link topology and debug symbol flags.
* **`Entry Point`:** Default entrypoint symbol and its zero-based index inside `.functions`.
* **`Static Reservation`:** Number of pre-allocated global static slots reserved before dynamic heap addresses begin.

### 2.2 Constant Pool (`.constant_pool:`)

Immutable compile-time literals are indexed as `$c00`, `$c01`, and referenced inline by instructions (`LoadConst`, `LoadI64`, `LoadF64`):

```text
.constant_pool:
  $c00: i64 = 100000000000
  $c01: f64 = 3.141592653589793
  $c02: string = "Hello, Solix!"
```

### 2.3 Classes & Dispatch Tables (`.classes:`)

```text
.classes:
  .class Worker (vtable_id: 2, size: 16B, destructor: None)
    .vtable: [1, 4]
    .itable: [(256, [4])]
```

* **`vtable_id`:** Unique runtime class table identifier stored in heap object headers (`ALLOC_OBJECT` operand `vtable:2`).
* **`size`:** Total byte allocation size of each instance (`AllocObject` operand `size:16B`).
* **`destructor`:** Optional function index executed deterministically when ARC strong reference count reaches `0`.
* **`.vtable`:** Virtual method dispatch slots mapping slot indices to global function IDs.
* **`.itable`:** Interface dispatch pairs `(interface_id, [slot_func_ids...])` used by `CALL_INTERFACE`.

---

## 3. Function & Activation Record Syntax

Each function opens with a metadata header, followed by `.source_id`, optional `.handlers:`, and `.code:`:

```text
; Function #2
.function CounterApp::main(params: 0) -> frame_size: 8, refs: [0, 3]
  .source_id: 0, static_base: 0
  .handlers:
    [TRY: 0004..0009 -> CATCH 'solix.core.Exception' -> @0011 (reg: r5), finally: @0015]
  .code:
    0000  LOAD_I32         r1, 0
    ...
```

| Field | Meaning |
|---|---|
| `params: N` | Number of incoming arguments mapped to registers `r0 .. r(N-1)` upon call entry. |
| `frame_size: M` | Total registers `r0 .. r(M-1)` allocated on the VM call frame stack. |
| `refs: [...]` | Sorted list of register indices that may hold ARC-managed heap references cleaned up during unwinding. |
| `.source_id` | Index into the container source table used for error spans and DAP breakpoints. |
| `static_base` | Base offset applied to global/static slot accesses for this module when statically linked. |
| `.handlers:` | Zero-cost exception table mapping protected `[try_start..try_end)` PC spans to `catch` type symbols, catch register (`reg: rK`), target handler `@PC`, and optional `finally: @PC`. |

---

## 4. Reading Instruction Operands

Every line in `.code:` begins with a 4-digit zero-padded program counter (`0000`, `0001`, ...) and the instruction mnemonic:

| Syntax Pattern | Example | Interpretation |
|---|---|---|
| `dst, src` | `MOV r2, r0` | Copies register `r0` into `r2`. |
| `dst, imm32` | `LOAD_I32 r1, 100000` | Loads signed 32-bit immediate into `r1`. |
| `dst, $cN ; repr` | `LOAD_CONST r6, $c2 ; "OK"` | Loads constant pool slot `2` into `r6`. |
| `dst, src, imm+` | `MOD_I32_IMM r4, r1, +7` | Computes `r4 = r1 % 7` using reciprocal multiplication / bitmasking. |
| `dst, acc, src1, src2` | `ADD_BAND_I32 r0, r0, r3, r9` | Fused bitwise-masked accumulate: `r0 = (r0 + r3) & r9`. |
| `rA, rB, @PC (+off)` | `FOR_LOOP_I32 r1, r8, @0005 (-3)` | Loop step on index `r1` against bound `r8`, branching relative `-3` to PC `0005`. |
| `@PC (+off)` | `JUMP @0016 (+3)` | Unconditional relative branch to PC `0016`. |
| `dst, target:ID, args:rA..rB` | `CALL_DIRECT r4, target:1, args:r6..r6` | Calls function `#1` passing contiguous register slice `r6..r6` as callee `r0..r0`; writes return value into `r4`. |
| `dst, len:rL, elem_size:NB` | `ALLOC_ARRAY r0, len:r1, elem_size:8B` | Allocates array of length `r1` with 8-byte element slots into `r0`. |
| `dst, vtable:V, size:NB` | `ALLOC_OBJECT r0, vtable:2, size:8B` | Allocates class instance using VTable `#2` into `r0`. |

### Internal Instruction Flags

Certain instructions carry compact 8-bit `flags` evaluated at runtime:
* **`FLAG_MOVE` (`0x80` on `SET_FIELD` / `SET_ARRAY`):** Moves an owned reference straight into the heap slot and resets the dead source register to `null`, avoiding redundant `INC_REF`/`DEC_REF`.
* **`FLAG_SCALAR_FOR_LOOP` (`0x10` on `FOR_LOOP_I32`):** Stamped after bytecode verification when a unit-stride scalar loop body contains only pure registers and arithmetic/branch opcodes for fast scalar loop completion.
* **`FLAG_ARRAY_FILL_LOOP` (`0x20` on `FOR_LOOP_I32`):** Marks 2-instruction primitive `SET_ARRAY` loops eligible for contiguous native fill execution.

---

## 5. Worked Annotated Example

Given this Solix source loop:

```solix
class SumLoop {
    public static void main() {
        int32 running_total = 0;
        for (int32 index = 0; index < 1000; index = index + 1) {
            running_total = running_total + index;
        }
    }
}
```

Compiled `.slxasm` output:

```text
.function SumLoop::main(params: 0) -> frame_size: 4, refs: []
  .source_id: 0, static_base: 0
  .code:
    0000  LOAD_I32         r0, 0                 ; running_total = 0
    0001  LOAD_I32         r1, 0                 ; index = 0
    0002  LOAD_I32         r2, 1000              ; hoisted loop bound
    0003  JUMP_IF_GTE_I32  r1, r2, @0006 (+3)    ; exit immediately if 0 >= 1000
    0004  ADD_I32          r0, r0, r1            ; running_total += index
    0005  FOR_LOOP_I32     r1, r2, @0004 (-1)    ; ++index; if index < r2 jump to 0004
    0006  RETURN                                 ; exit frame
```

Reading top to bottom: `refs: []` confirms zero heap references exist in the frame, `@0006 (+3)` jumps past the loop if the initial guard fails, `0004` updates `running_total`, and `0005 FOR_LOOP_I32` handles induction increment, bound comparison, and backward branch in a single instruction.
