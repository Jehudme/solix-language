# Solix Lexical Cheat Sheet

Quick atomic reference for Solix tokenization rules, literals, escape sequences, and operators. For the full normative specification, see [`docs/spec/lexical.md`](../spec/lexical.md). For diagnostic troubleshooting, see [`errors/lexer-errors.md`](errors/lexer-errors.md).

---

## Encoding & Comments

* **Encoding:** UTF-8 (optional leading BOM `0xEF 0xBB 0xBF` stripped transparently).
* **Whitespace:** Spaces, tabs, `\r`, and `\n` act solely as token separators.
* **Single-line comments:** `// ...` until newline or EOF.
* **Block comments:** `/* ... */` (non-nesting; terminated at the first `*/`).

---

## Identifiers & Keywords

* **Identifiers:** `[A-Za-z_][A-Za-z0-9_]*` (case-sensitive; cannot collide with reserved or primitive keywords).
* **Reserved keywords & types:** See [`keywords/README.md`](keywords/README.md) and [`primitives/README.md`](primitives/README.md).

---

## Numeric Literals

| Kind | Examples | Notes |
| :--- | :--- | :--- |
| **Decimal integer** | `0`, `42`, `1_000_000`, `10L` | Defaults to `int32` (or `int64` when `L`/`l` suffix is present or value exceeds `int32` range) |
| **Hexadecimal integer** | `0xFF`, `0xDEAD_BEEF`, `0x10L` | Prefixed with `0x` or `0X` |
| **Binary integer** | `0b1010`, `0b1111_0000`, `0b1L` | Prefixed with `0b` or `0B` |
| **Floating-point** | `3.14159`, `0.5f`, `1.0e-4`, `2e10` | Defaults to `float64`; suffix `f`/`F` selects `float32` |

---

## String, Character & Escape Literals

* **String literals:** `"Hello, Solix!"`, supporting escapes below.
* **Character literals:** `'A'`, `'\n'`, `'\x7F'` (single decoded scalar/escape).

| Escape | Value | Meaning |
| :--- | :--- | :--- |
| `\n` | `0x0A` | Line feed |
| `\t` | `0x09` | Horizontal tab |
| `\r` | `0x0D` | Carriage return |
| `\\` | `0x5C` | Backslash |
| `\'` | `0x27` | Single quote |
| `\"` | `0x22` | Double quote |
| `\0` | `0x00` | Null byte |
| `\xHH` | `0xHH` | Two-digit hexadecimal byte escape |

---

## Operators & Punctuation (Maximal Munch)

| Category | Tokens |
| :--- | :--- |
| **Arithmetic** | `+`, `-`, `*`, `/`, `%` |
| **Assignment** | `=`, `+=`, `-=`, `*=`, `/=`, `%=` |
| **Comparison** | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| **Logical** | `&&`, `\|\|`, `!` |
| **Bitwise** | `~`, `&` |
| **Increment / Decrement** | `++`, `--` |
| **Arrow / Conditional** | `=>`, `?`, `:` |
| **Punctuation & Scope** | `;`, `,`, `.`, `::`, `...` |
| **Delimiters** | `(`, `)`, `{`, `}`, `[`, `]` |
