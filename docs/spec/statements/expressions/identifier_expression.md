# IdentifierNode

## 1. Overview & Purpose

An `IdentifierNode` represents a named reference to a local variable, function parameter, class field, global variable, or type.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
int32 y_coord = x;
```

```bytecode
// Compiled VM Bytecode
// (Register r1 holds local variable)                 // Resolves 'x' to slot 1 and loads value
Mov r2, r7                 // Stores into 'y'
```
