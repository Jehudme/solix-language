# Expression Specifications

Evaluation order, type coercion rules, and 64-bit register bytecode lowering for Solix expressions.

- [`assignment_expression.md`](assignment_expression.md) — Lvalue mutation and ARC ownership transitions
- [`binary_expression.md`](binary_expression.md) — Arithmetic, relational, logical, and bitwise operators
- [`unary_expression.md`](unary_expression.md) — Prefix and postfix unary operators
- [`ternary_expression.md`](ternary_expression.md) — Lazy conditional selection expressions
- [`cast_expression.md`](cast_expression.md) — Explicit numeric casts and runtime class downcasts
- [`instanceof_expression.md`](instanceof_expression.md) — Runtime VTable hierarchy checks
- [`new_instance_expression.md`](new_instance_expression.md) — Object heap allocation and constructor calls
- [`array_creation_expression.md`](array_creation_expression.md) — Typed array allocation on the virtual heap
- [`array_access_expression.md`](array_access_expression.md) — Bounds-checked element indexing
- [`array_literal_expression.md`](array_literal_expression.md) — Inline array literal initialization
- [`member_access_expression.md`](member_access_expression.md) — Instance field access and static `::` resolution
- [`method_call_expression.md`](method_call_expression.md) — Direct, virtual, and interface call lowering
- [`identifier_expression.md`](identifier_expression.md) — Lexical variable and symbol resolution
- [`literal_expression.md`](literal_expression.md) — Numeric, string, character, boolean, and `null` literals
- [`function_pointer_expression.md`](function_pointer_expression.md) — First-class function pointer values
- [`lambda_expression.md`](lambda_expression.md) — Anonymous functions, closure packing, and ARC captures
