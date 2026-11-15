# ArrayAccessExpression

## 1. Overview & Purpose

An `ArrayAccessExpression` (`arr[index]`) accesses or modifies an element in an array. The VM enforces runtime bounds checking, raising `IndexOutOfBoundsException` on violation.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
int32 val = arr[2];
arr[2] = 50;
```

```bytecode
// Compiled VM Bytecode
// Read
// (Register r1 holds local variable)                 // arr
LoadI32 r0, 2            // index
GET_ARRAY                   // Bounds checks & reads element
Mov r2, r0                 // val

// Write
LoadI32 r1, 50           // value
// (Register r1 holds local variable)                 // arr
LoadI32 r2, 2            // index
SET_ARRAY                   // Bounds checks & writes element
```
