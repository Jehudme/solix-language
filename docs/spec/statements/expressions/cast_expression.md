# CastExpression

## 1. Overview & Purpose

A `CastExpression` (`(TargetType)expr`) converts types. Primitive casts emit conversion opcodes (e.g. `CONV_F64`). Reference downcasts emit `CAST_CHECK` to validate the object's VTable at runtime.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
Dog d = (Dog)animal;
```

```bytecode
// Compiled VM Bytecode
// (Register r1 holds local variable)                 // Load animal reference
CAST_CHECK <Dog_vtable_id>  // Verifies inheritance; throws if mismatch!
Mov r2, r7                 // Stored into d
```
