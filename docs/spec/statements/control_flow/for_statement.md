# ForStatement

## 1. Overview & Purpose

A `ForStatement` provides 3-clause iteration: an initialization clause (creating loop-scoped variables), a termination condition, and a step update expression.

The initialization variable is encapsulated inside an internal loop scope, meaning it does not leak into the outer function. `continue` jumps directly to the step expression, and ARC cleans up induction variables when the loop finishes.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
for (int32 index = 0; index < 2; index++) {
    Console::println(index);
}
```

```bytecode
// Compiled VM Bytecode
// --- Init (Loop Scope) ---
LoadI32 r0, 0
Mov r1, r0                 // i = 0

// --- Condition (<loop_start_ip>) ---
// (Register r1 holds local variable)
LoadI32 r1, 2
LtI32 r2, r0, r1
JumpIfFalse r2, <loop_exit_ip>

// --- Body ---
// (Register r1 holds local variable)
CALL_NATIVE Console::println

// --- Step Update (<step_ip>) ---
// (Register r1 holds local variable)
AddI32 r0, r0, 1
Mov r1, r1                 // i++
Jump <loop_start_ip>

// --- Exit (<loop_exit_ip>) ---
```
