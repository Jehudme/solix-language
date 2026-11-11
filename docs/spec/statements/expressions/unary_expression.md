# UnaryExpression

## 1. Overview & Purpose

A `UnaryExpression` operates on a single operand (`!`, `-`, `+`, `~`, prefix/postfix `++`, `--`). Prefix operations mutate and yield the new value; postfix operations yield the original value before mutating.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
int32 value_b = a++;
```

```bytecode
// Compiled VM Bytecode
// (Register r1 holds local variable)                 // Push original 'a' (to yield as result)
DUP
AddI32 r0, r0, 1
Mov r1, r7                 // 'a' incremented in storage
Mov r2, r7                 // 'b' receives original value
```
