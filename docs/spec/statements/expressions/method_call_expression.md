# MethodCallExpression

## 1. Overview & Purpose

A `MethodCallExpression` (`receiver.method(args)`) invokes a function or method. It performs overload resolution, prepares arguments on the operand stack, and dispatches statically or dynamically via VTables.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
calc.add(10, 20);
```

```bytecode
// Compiled VM Bytecode
// (Register r1 holds local variable)                 // Receiver calc (this)
LoadI32 r0, 10           // Arg 1
LoadI32 r1, 20           // Arg 2
CALL_VIRTUAL <add_slot>     // Dispatches via VTable
```
