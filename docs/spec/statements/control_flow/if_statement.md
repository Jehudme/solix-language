# IfStatement

## 1. Overview & Scope

An `IfStatement` provides conditional branching based on the evaluation of a boolean predicate expression. If the condition evaluates to `true`, the `then` branch executes. If `false`, control transfers to the optional `else` branch (or bypasses the statement).

In Solix:
- The condition must be strictly of type `bool`. Numeric zero/non-zero truthiness is rejected.
- The compiler lowers the if-statement using conditional jump instructions (`JumpIfFalse`) with forward jump target patching.

---

## 2. Compilation & Runtime Mechanics (With Real Bytecode)

### Bytecode Disassembly Example
```solix
// Solix Code
if (score >= 50) {
    Console::println("Pass");
} else {
    Console::println("Fail");
}
```

```bytecode
// Compiled VM Bytecode
// (Register r1 holds local variable)                 // Load score
LoadI32 r0, 50
GREATER_EQ_I64              // Pushes bool (true/false)
JumpIfFalse r2, <else_ip>     // Jump to else branch if false

// --- Then Branch ---
LoadConst r1, 0         // "Pass"
CALL_NATIVE Console::println
Jump <end_ip>               // Jump over else branch

// --- Else Branch (<else_ip>) ---
LoadConst r2, 1         // "Fail"
CALL_NATIVE Console::println

// --- End (<end_ip>) ---
```
