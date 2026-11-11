# ExpressionStatement

## 1. Overview & Purpose

An `ExpressionStatement` evaluates an expression purely for its side effects—such as variable assignments, method invocations, or increment/decrement operations.

In the 64-bit register VM, expressions executed for side effects manage registers and references deterministically:
- If the expression produces a **primitive value** (e.g. `x + 1;`), no stack popping is required; the destination register is simply overwritten or reclaimed by the register allocator.
- If the expression produces an **unassigned reference object** (e.g. `new String("temp");` or calling a function that returns a new object without storing it), the VM emits `DecRef` on the destination register to immediately free the temporary object so memory is not leaked.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
x = 5;                      // Assignment side-effect
Console::println("log");     // Void method invocation
new String("discard_me");   // Temporary reference discard
```

```bytecode
// Compiled VM Bytecode
// 1. x = 5;
LoadI32 r0, 5
Mov r1, r0                 // Stored into x; operand stack is balanced

// 2. Console::println("log");
LoadConst r1, 0
CALL_NATIVE Console::println // Returns void; no pop needed

// 3. new String("discard_me");
LoadConst r2, 1
CallDirect r0, string_new_id, r2, 1 // Allocates string into r0
DecRef r0                           // Immediately decremented and freed!
```
