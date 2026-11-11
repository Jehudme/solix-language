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
