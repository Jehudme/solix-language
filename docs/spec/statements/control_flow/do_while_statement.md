# DoWhileStatement

## 1. Overview & Purpose

A `DoWhileStatement` implements post-test iteration. Unlike `while`, the body executes at least once before the condition is evaluated. If the condition evaluates to `true`, control loops back to the start of the body; if `false`, the loop terminates.

`break` jumps past the condition check to the loop exit, while `continue` jumps directly to the condition evaluation at the foot of the loop.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
int32 count = 0;
do {
    count++;
} while (count < 3);
```

```bytecode
// Compiled VM Bytecode
LoadI32 r0, 0
Mov r1, r0                 // count = 0

// --- Loop Start (<loop_head_ip>) ---
// (Register r1 holds local variable)
AddI32 r0, r0, 1
Mov r1, r0                 // count++

// --- Condition Evaluation (<condition_ip>) ---
// (Register r1 holds local variable)
LoadI32 r1, 3
LtI32 r2, r0, r1
JUMP_IF_TRUE <loop_head_ip> // Loop back if true

// --- Loop Exit ---
```
