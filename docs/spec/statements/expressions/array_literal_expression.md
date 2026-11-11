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
// Compiled Register VM Bytecode
NewArrayRef r0, array_i32_class_id, 3 // Allocate array of length 3 into r0
LoadI32 r1, 2
SetArrayElement r0, 0, r1            // arr[0] = 2
LoadI32 r2, 3
SetArrayElement r0, 1, r2            // arr[1] = 3
LoadI32 r3, 5
SetArrayElement r0, 2, r3            // arr[2] = 5
Mov r4, r0                           // primes = array pointer
```
