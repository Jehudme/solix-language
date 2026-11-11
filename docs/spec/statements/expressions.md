# Formal Specification: Expressions, Operators & Precedence

## Document Metadata

* **Specification ID:** SPEC-0017
* **Scope:** Expression Semantics, Operator Precedence & Evaluation Order (`crates/solix-compilation/src/phases/parser/expressions/`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)

---

## 1. Operator Precedence & Associativity Table

Solix expressions are parsed using Pratt precedence climbing according to the following strict hierarchy:

| Level | Operator Category | Operators | Associativity |
| :---: | :--- | :--- | :---: |
| **1** | Primary / Postfix | `()`, `[]`, `.`, `::`, `new` | Left-to-Right |
| **2** | Unary Prefix | `+`, `-`, `!`, `~`, `sizeof`, `default`, `cast`, `(Type)` | Right-to-Left |
| **3** | Multiplicative | `*`, `/`, `%` | Left-to-Right |
| **4** | Additive | `+`, `-` | Left-to-Right |
| **5** | Bitwise Shifts | `<<`, `>>` | Left-to-Right |
| **6** | Relational | `<`, `<=`, `>`, `>=`, `instanceof` | Left-to-Right |
| **7** | Equality | `==`, `!=` | Left-to-Right |
| **8** | Bitwise AND | `&` | Left-to-Right |
| **9** | Bitwise XOR | `^` | Left-to-Right |
| **10**| Bitwise OR | `\|` | Left-to-Right |
| **11**| Logical AND | `&&` (Short-circuiting) | Left-to-Right |
| **12**| Logical OR | `\|\|` (Short-circuiting) | Left-to-Right |
| **13**| Ternary Conditional | `? :` | Right-to-Left |
| **14**| Lambda Closure | `=>` | Right-to-Left |
| **15**| Assignment | `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `\|=`, `^=`, `<<=`, `>>=` | Right-to-Left |

---

## 2. Type Operators & Intrinsics

1. **`instanceof(object_reference, TargetType)`:** Evaluates to `bool`, verifying nominal class hierarchy or implemented interface table membership.
2. **`sizeof(Type)`:** Returns the byte width of `Type` as a `uint32`.
3. **`default(Type)`:** Produces zeroed memory for primitives (`0`, `0.0`, `false`, `null`).
4. **`cast(expression, TargetType)`:** Explicit type conversion equivalent to `(TargetType)expression`.

---

## 3. First-Class Lambdas & Closures

Lambdas define anonymous functions that capture environment variables by value:

```solix
(int32 first_operand, int32 second_operand) => first_operand + second_operand;

(string message_text) => {
    Console::println("Log: " + message_text);
    return true;
};
```

Lambdas lower to anonymous closure objects storing captured reference registers in heap payloads.
