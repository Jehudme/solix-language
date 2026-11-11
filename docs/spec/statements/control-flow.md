# Formal Specification: Control Flow Statements & Bytecode Lowering

## Document Metadata

* **Specification ID:** SPEC-0015
* **Scope:** Branching, Iteration & Jump Lowering (`crates/solix-compilation/src/phases/parser/statements/control_flow.rs`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)

---

## 1. Syntax & EBNF Grammar

```ebnf
IfStatement     ::= "if" "(" Expression ")" Statement [ "else" Statement ]
WhileStatement  ::= "while" "(" Expression ")" Statement
DoWhileStatement::= "do" Statement "while" "(" Expression ")" ";"
ForStatement    ::= "for" "(" [ ForInitializer ] ";" [ Expression ] ";" [ Expression ] ")" Statement
SwitchStatement ::= "switch" "(" Expression ")" "{" { SwitchCase } "}"
ReturnStatement ::= "return" [ Expression ] ";"
BreakStatement  ::= "break" ";"
ContinueStatement ::= "continue" ";"
```

---

## 2. Conditional Branching (`if` / `else`)

The condition expression must evaluate strictly to boolean type (`bool`). Non-boolean condition expressions emit diagnostic `E0203`.

### Bytecode Lowering Pattern
```text
    [Evaluate Condition into Register R_cond]
    JumpIfFalse R_cond, Else_Label
    [True Block Instructions]
    Jump End_Label
Else_Label:
    [Else Block Instructions]
End_Label:
```

---

## 3. Iteration Constructs (`while`, `do`/`while`, `for`)

### 3.1 `while` Loop
Evaluates the condition prior to entering the loop body:
```text
Loop_Head:
    [Evaluate Condition into Register R_cond]
    JumpIfFalse R_cond, Loop_Exit
    [Body Instructions]
    Jump Loop_Head
Loop_Exit:
```

### 3.2 `do`/`while` Loop
Executes the body at least once before testing the condition:
```text
Loop_Head:
    [Body Instructions]
Loop_Cont:
    [Evaluate Condition into Register R_cond]
    JumpIfTrue R_cond, Loop_Head
Loop_Exit:
```

### 3.3 `break` and `continue` Semantics
* `break` jumps immediately to `Loop_Exit`.
* `continue` jumps to the loop update/condition check (`Loop_Head` for `while`, step expression for `for`).
* Prior to jumping out of a loop, the emitter automatically emits `ReleaseRef` instructions for all local reference variables declared within inner blocks.
