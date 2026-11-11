# Formal Specification: Exceptions & Runtime Intrinsics

## Document Metadata

* **Specification ID:** SPEC-0016
* **Scope:** Exception Blocks & Built-in Language Intrinsics (`crates/solix-compilation/src/phases/parser/statements/control_flow.rs`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)

---

## 1. Syntax & EBNF Grammar

```ebnf
TryStatement    ::= "try" Block { CatchClause } [ FinallyClause ]
CatchClause     ::= "catch" "(" Type Identifier ")" Block
FinallyClause   ::= "finally" Block
ThrowStatement  ::= "throw" Expression ";"
AssertStatement ::= "assert" "(" Expression [ "," Expression ] ")" ";"
ExitStatement   ::= "exit" "(" Expression ")" ";"
PrintStatement  ::= ( "print" | "println" ) "(" [ Expression ] ")" ";"
```

---

## 2. Structured Exception Handling

```solix
try {
    perform_risky_operation();
} catch (IOException error) {
    Console::println("Caught I/O error: " + error.message());
} catch (Exception general_error) {
    Console::println("Caught generic exception: " + general_error.message());
} finally {
    cleanup_resources();
}
```

* Thrown objects **MUST** derive from nominal reference types (`class`). Attempting to throw primitive types (`throw 1;`) emits diagnostic `E0215`.
* Catch clauses are evaluated in top-to-bottom order using dynamic `instanceof` inspection.

---

## 3. Built-in Intrinsics

### 3.1 `assert(condition [, message])`
* Evaluates `condition: bool`. If false, raises a runtime assertion fault, prints the source file coordinate and optional message string, and exits with code `1`.

### 3.2 `exit(code)`
* Halts VM execution immediately and returns the 32-bit integer `code` as the process exit status.

### 3.3 `print(value)` and `println(value)`
* Built-in primitives for zero-overhead stdout writing, supporting all primitive types and object references implementing `IStringable`.
