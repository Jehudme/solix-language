# ArrayCreationExpression

## 1. Overview & Purpose

An `ArrayCreationExpression` (`new Type[size]`) allocates a contiguous array buffer on the heap. Arrays are reference types governed by ARC.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
int32[] arr = new int32[10];
```

```bytecode
// Compiled VM Bytecode
LoadI32 r0, 10
ALLOC_STATIC 4              // Allocates array buffer (element size 4)
Mov r1, r0                 // Stored into arr
```
