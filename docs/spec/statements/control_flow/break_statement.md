# BreakStatement

## 1. Overview & Purpose

A `BreakStatement` (`break;`) immediately terminates the innermost enclosing loop (`while`, `do-while`, `for`) or `switch` construct.

Before jumping to the construct's exit address, the compiler traverses the AST parent hierarchy and emits inline `DEC_REF` instructions for all active reference variables in intermediate blocks, guaranteeing leak-free escapes.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
while (true) {
    String temp = new String("leak_test");
    break; // Exits loop
}
```

```bytecode
// Compiled VM Bytecode
<loop_head>:
LoadConst r0, 0
CALL String.new(String)
Mov r1, r0                 // Slot 1: temp

// --- Break Execution ---
// (Register r1 holds local variable)                 // Intermediate block cleanup
DEC_REF                     // Clean up temp before jumping!
Jump <loop_exit>            // Jump out of while loop

<loop_exit>:
```
