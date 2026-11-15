# BinaryExpression

## 1. Overview & Purpose

A `BinaryExpression` combines two operands using an infix operator (arithmetic, relational, logical, bitwise). Solix executes typed ALU opcodes (`AddI64`, `AddF64`, `EqI64`, etc.) and short-circuits `&&` and `||`.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
int32 sum = a + b;
```

```bytecode
// Compiled Register VM Bytecode
// r0 holds local variable value_a
// r1 holds local variable value_b
AddI64 r2, r0, r1                     // r2 = value_a + value_b
Mov r3, r2                            // sum = r2
```
