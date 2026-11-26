# TryCatchFinallyStatement

## 1. Overview & Purpose

A `TryCatchFinallyStatement` provides structured error recovery (`catch`) and guaranteed finalization (`finally`). Protected code in the `try` block routes exceptions to a typed catch matching table, while `finally` executes unconditionally even on early returns.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
try {
    dangerous();
} catch (std.Exception error) {
    recover();
}
```

```bytecode
// Compiled VM Bytecode
// --- Try Block ---
CALL dangerous()
Jump <try_end_ip>               // Normal exit jumps over catch table

// --- Catch Matching Table (<catch_start_ip>) ---
GET_EXCEPTION                   // Push active exception
INSTANCEOF <std.Exception_vtable_id>
JumpIfFalse r2, <unhandled_ip>    // Next catch or outer cleanup
GET_EXCEPTION
Mov r1, r7                     // Store into catch parameter 'e'
CLEAR_EXCEPTION                 // Clear active exception register
CALL recover()
// (Register r1 holds local variable)                     // Clean up catch parameter
DEC_REF
Jump <try_end_ip>

// --- End (<try_end_ip>) ---
```

### Nested `finally` Unwinding & Local Return Snapshot Preservation
- When a `return` statement inside `try` (or `catch`) returns a local variable while `finally` blocks are active, the emitter snapshots the return register into a temporary register (and emits `IncRef` for reference-typed returns) before running `finally` handlers so assignments or `null` overwrites inside `finally` cannot corrupt or prematurely release the already-evaluated return value.
- `emit_active_finally_blocks_up_to` pops one active `finally` frame per step rather than truncating all pending frames upfront, ensuring an early `return`, `break`, or `continue` inside a nested `finally` block still unwinds enclosing outer `finally` handlers cleanly.
```
