# Solix Diagnostic Error Code Encyclopedia

This encyclopedia catalogs all diagnostic error codes emitted by the Solix compiler (`E0000`–`E0350`) and runtime execution faults (`R0001`–`R0024`).

Per [`GL-0012`](../../internals/guidelines/authoring-features-and-wiki.md), each entry specifies the exact root cause, invariant violated, bad code example, and corrected code example.

---

## Diagnostic Categories

| Code Range | Subsystem | Description & Specification Link |
| :--- | :--- | :--- |
| [**`E0000` – `E0015`**](lexer-errors.md) | **Lexical Analyzer** | Unterminated literals, invalid escapes, unexpected characters. |
| [**`E0101` – `E0200`**](parser-errors.md) | **Recursive-Descent Parser** | Grammar syntax errors, mismatched delimiters, misplaced statements. |
| [**`E0201` – `E0290`**](binder-errors.md) | **Semantic Type Binder** | Type mismatches, undefined symbols, static `::` vs `.`, VTable conflicts. |
| [**`E0300` – `E0350`**](emission-errors.md) | **Bytecode Emitter** | Register overflow, corrupt container structure, pool limits. |
| [**`R0001` – `R0024`**](runtime-errors.md) | **Virtual Machine Runtime** | Runtime traps, null dereferences, invalid casts, process exit codes. |
