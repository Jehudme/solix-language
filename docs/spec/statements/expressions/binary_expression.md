# BinaryExpression

## 1. Overview & Purpose

A `BinaryExpression` combines two operands using an infix operator (arithmetic, relational, logical, bitwise). Solix executes typed ALU opcodes (`ADD_I64`, `ADD_F64`, `EQ_I64`, etc.) and short-circuits `&&` and `||`.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
int32 sum = a + b;
```

```bytecode
// Compiled VM Bytecode
// (Register r1 holds local variable)                 // Load a
// (Register r2 holds local variable)                 // Load b
ADD_I64                     // Pops 2, pushes sum
Mov r3, r7                 // sum
```
