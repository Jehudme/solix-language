# WhileStatement

## 1. Overview & Purpose

A `WhileStatement` implements pre-test repetitive execution. Before every iteration, the condition is evaluated. If `true`, the loop body executes; if `false`, execution exits the loop.

The loop body supports `break` (to exit immediately) and `continue` (to jump back to the condition check). Any reference variables declared inside the loop body are cleaned up via ARC at the end of each iteration.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
int32 index = 0;
while (index < 3) {
    index++;
}
```

```bytecode
// Compiled VM Bytecode
LoadI32 r0, 0
Mov r1, r0                 // i = 0

// --- Loop Start (<loop_start_ip>) ---
// (Register r1 holds local variable)                 // Load i
LoadI32 r1, 3
LtI32 r2, r0, r1
JumpIfFalse r2, <loop_exit_ip>// Exit loop if condition is false

// --- Loop Body ---
// (Register r1 holds local variable)
AddI32 r0, r0, 1
Mov r1, r1                 // i++
Jump <loop_start_ip>        // Loop back to condition

// --- Loop Exit (<loop_exit_ip>) ---
```
