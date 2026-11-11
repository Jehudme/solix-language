# ArrayLiteralExpression

## 1. Overview & Purpose

An `ArrayLiteralExpression` (`[1, 2, 3]` or `{1, 2, 3}`) constructs and populates an array inline. The element type is inferred by unifying the element expressions.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
int32[] primes = [2, 3, 5];
```

```bytecode
// Compiled VM Bytecode
LoadI32 r0, 3
ALLOC_STATIC 4              // Allocate array of length 3
DUP
LoadI32 r1, 2
LoadI32 r2, 0
SET_ARRAY                   // arr[0] = 2
DUP
LoadI32 r3, 3
LoadI32 r4, 1
SET_ARRAY                   // arr[1] = 3
DUP
LoadI32 r5, 5
LoadI32 r6, 2
SET_ARRAY                   // arr[2] = 5
Mov r1, r6                 // primes = array pointer
```
