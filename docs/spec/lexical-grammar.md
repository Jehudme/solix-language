# Formal Specification: Lexical Grammar & Tokenization

## Document Metadata

* **Specification ID:** SPEC-0001
* **Scope:** Lexical Analysis & Token Streaming (`crates/solix-compilation/src/phases/lexing/`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/guidelines/authoring-specs.md` (GL-0005)
* **Rust Implementation:** [`crates/solix-compilation/src/phases/lexing/lexer.rs`](../../crates/solix-compilation/src/phases/lexing/lexer.rs)

---

## 1. Source Text & Encoding

Solix source code is a stream of Unicode characters encoded strictly in **UTF-8**.
* **Byte Order Mark (BOM):** If a leading UTF-8 BOM (`0xEF, 0xBB, 0xBF`) is present, the lexer strips it transparently prior to column counting.
* **Line Terminators:** The lexer recognizes LF (`\n`, `0x0A`), CR (`\r`, `0x0D`), and CRLF (`\r\n`) as canonical line breaks. Lines and 1-based columns are tracked monotonically across all source coordinates.
* **Whitespace:** ASCII space (`0x20`), horizontal tab (`0x09`), and form feed (`0x0C`) are recognized as whitespace and discarded outside string and character literals.

---

## 2. Comments

Solix supports two comment styles:
1. **Single-Line Comments:** Begin with `//` and extend to the next newline terminator or end of file.
2. **Multi-Line Delimited Comments:** Begin with `/*` and terminate at `*/`. Solix lexers **MUST** support recursive nested multi-line comments (`/* outer /* inner */ still comment */`). An unclosed multi-line comment reaching end of file emits diagnostic `E0005`.

---

## 3. Literals & Numeric Encodings

### 3.1 Integer Literals
Integer literals are parsed into 64-bit unsigned representations (`u64`) and range-checked during semantic type binding:
* **Decimal:** Non-zero ASCII digits followed by digits (`142`, `987654321`).
* **Hexadecimal:** Prefixed by `0x` or `0X` followed by hexadecimal digits (`0xFF`, `0x1A2B_3C4D`).
* **Binary:** Prefixed by `0b` or `0B` followed by binary digits (`0b1010_1100`).
* **Digit Separators:** Underscores (`_`) are permitted anywhere between digits to improve readability (`1_000_000`, `0xCAFE_BABE`). An integer literal cannot start or end with a bare underscore.

### 3.2 Floating-Point Literals
Floating-point literals represent IEEE-754 64-bit double precision floats:
* Decimal point format: `3.14159`, `0.001`, `100.0`.
* Scientific exponent format: `1e6`, `2.5E-4`, `1.0e+9`.
* Suffixes: `f` or `F` designates a 32-bit `float32`; omitted suffix or `d`/`D` designates `float64`.

### 3.3 Character Literals
Enclosed in single quotes `'...'` and represent a single 32-bit Unicode scalar value:
* Single characters: `'a'`, `'Z'`, `'9'`, `'λ'`.
* Standard escape sequences: `\n` (newline), `\r` (carriage return), `\t` (tab), `\\` (backslash), `\'` (single quote), `\"` (double quote), `\0` (null).
* Hexadecimal escapes: `\x7F` (two hex digits).
* Unicode scalar escapes: `\u{1F600}` (up to six hex digits within curly braces).

### 3.4 String Literals
Enclosed in double quotes `"..."` representing immutable UTF-8 heap string constants:
* Support all character escape sequences.
* Multi-line raw string literals are enclosed in triple quotes `"""..."""` and preserve whitespace and line breaks without escaping.

---

## 4. Identifiers & Reserved Keywords

* **Identifier Grammar:** An identifier must begin with an ASCII letter (`a-z`, `A-Z`) or underscore (`_`), followed by any number of ASCII letters, digits, or underscores.
* **Case Sensitivity:** Solix is strictly case-sensitive (`variable` and `Variable` represent distinct symbols).
* **Reserved Keywords:** Solix reserves 45+ keywords across control flow, declarations, modifiers, and expressions. Attempting to use a reserved keyword as a variable or class name emits diagnostic `E0102`.

---

## 5. Token Enumeration & Coordinate Spans

Every token emitted by the lexer contains:
* **`kind: TokenKind`**: One of the 108 strongly typed lexical categories.
* **`span: Span`**: Byte offset coordinate `[start..end]` along with source file identifier `source_id`.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub source_id: u16,
    pub start_offset: u32,
    pub end_offset: u32,
}
```

### Lexical Error Codes

| Error Code | Identifier | Description |
| :--- | :--- | :--- |
| **`E0001`** | `UnterminatedString` | String literal reached end-of-line or end-of-file without closing quote |
| **`E0002`** | `InvalidNumberLiteral` | Malformed numeric literal (invalid radix digits or multiple decimal points) |
| **`E0003`** | `InvalidEscapeSequence`| Unrecognized character escape sequence within literal |
| **`E0004`** | `UnexpectedCharacter`  | Source character does not match any valid Solix token |
| **`E0005`** | `UnterminatedComment` | Delimited comment (`/* ...`) reached end-of-file without matching `*/` |
