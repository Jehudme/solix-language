# MemberAccessExpression

## 1. Overview & Purpose

A `MemberAccessExpression` (`receiver.member`) accesses fields or nested properties. Instance field offsets are resolved statically at compile time.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
int32 age = user.age;
```

```bytecode
// Compiled VM Bytecode
// (Register r1 holds local variable)                 // user
GET_PROPERTY 0              // Reads field at offset 0
Mov r2, r7                 // age
```
