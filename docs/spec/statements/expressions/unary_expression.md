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
// Compiled Register VM Bytecode
// r0 holds 'value_a'
Mov r1, r0                             // r1 receives original value (to yield as result)
AddI32 r0, r0, 1                       // 'value_a' incremented in storage
Mov r2, r1                             // 'value_b' receives original value
```
