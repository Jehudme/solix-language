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
// Compiled VM Bytecode
LoadI32 r0, 42
Mov r1, r0
PUSH_CONST_F64 3.14
Mov r2, r0
PUSH_TRUE
Mov r3, r0
LoadConst r1, 0         // Static string pool index
Mov r4, r1
PUSH_NULL
Mov r5, r1
```
