# TernaryExpression

## 1. Overview & Purpose

A `TernaryExpression` (`cond ? true_expr : false_expr`) is an inline conditional operator. It is strictly short-circuiting: only the branch matching the evaluated condition is executed.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
int32 val = flag ? 10 : 20;
```

```bytecode
// Compiled VM Bytecode
// (Register r1 holds local variable)                 // Load flag
JumpIfFalse r2, <false_branch>
LoadI32 r0, 10           // True branch
Jump <end>
<false_branch>:
LoadI32 r1, 20           // False branch
<end>:
Mov r2, r1                 // Stored into val
```
