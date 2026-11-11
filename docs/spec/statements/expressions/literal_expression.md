# LiteralNode

## 1. Overview & Purpose

A `LiteralNode` represents a constant scalar or string literal embedded directly in source code (`42`, `3.14`, `'c'`, `"string"`, `true`, `false`, `null`).

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
int32 index = 42;
float64 factor = 3.14;
bool flag = true;
String message = "text";
Object target = null;
```

```bytecode
// Compiled Register VM Bytecode
LoadI32 r0, 42
Mov r1, r0                             // r1 = 42
LoadF64 r2, const_idx_pi               // r2 = 3.14
LoadBool r3, 1                         // r3 = true
LoadConst r4, const_idx_text           // r4 = "text"
LoadNull r5                            // r5 = null
```
