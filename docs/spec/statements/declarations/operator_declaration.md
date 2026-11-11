# OperatorDeclaration

## 1. Overview & Purpose

An `OperatorDeclaration` allows user-defined classes to overload arithmetic operators (`+`, `-`, `*`, `/`) and assignment (`=`). Binary operator methods take one parameter, with `this` representing the left-hand operand.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### How It Compiles
When `v1 + v2` is evaluated where `v1` is of class type `Vector`:
```bytecode
// (Register r1 holds local variable)                 // Load v1 (LHS)
// (Register r2 holds local variable)                 // Load v2 (RHS parameter)
CALL_VIRTUAL <operator+_slot>
```
