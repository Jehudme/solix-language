# Solix Lexical Grammar Specification

## Document Metadata
* **Specification ID:** SPEC-0001
* **Scope:** Lexical Analysis, Tokenization, and Character Encoding
* **Status:** Draft / Active
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)
* **Target Milestone:** `docs/internals/milestones/002-lexical-analyzer.md` (MS-002)

---

## 1. Executive Summary & Purpose

This specification defines the normative lexical grammar of the Solix programming language: how raw source text is decoded, validated, and tokenized into a stream of discrete terminal tokens (`Token { kind: TokenKind, span: Span }`) consumed by the recursive-descent parser.

Conforming implementations **MUST** satisfy all lexical invariants, character sets, escape sequences, number formats, keyword classifications, and diagnostic error codes defined in this document.

---

## 2. Character Encoding & Input Normalization

### 2.1 UTF-8 Encoding
Solix source files **MUST** be encoded in valid **UTF-8**. While non-ASCII Unicode characters are permitted inside string literals, character literals, and comments, identifier names and operators are restricted to the ASCII subset (`U+0000` through `U+007F`).

### 2.2 Byte Order Mark (BOM)
If the first three bytes of a source file are `0xEF 0xBB 0xBF` (the standard UTF-8 BOM), the lexer **MUST** silently consume them without altering line or column numbers. A UTF-8 BOM appearing anywhere else in the file is illegal and **MUST** trigger diagnostic error `E0003` (`UnrecognizedCharacter`).

### 2.3 Line Ending Normalization
The lexer recognizes both Unix line endings (`LF`, `U+000A`) and Windows line endings (`CRLF`, `U+000D U+000A`). A lone `CR` (`U+000D`) not followed by `LF` is also treated as a single newline. All line endings advance the line counter by `1` and reset the column counter to `1`.

---

## 3. Whitespace & Comments

### 3.1 Whitespace
Whitespace characters (` `, `\t`, `\r`, `\n`) serve solely to separate adjacent tokens and are discarded during compilation. Adjacent identifiers, keywords, or numeric literals **MUST** be separated by at least one whitespace character or comment.

### 3.2 Single-Line Comments (`//`)
A single-line comment begins with the two-character sequence `//` and extends up to the next line terminator (`\n`) or the end of the input stream. The newline character is not part of the comment.

### 3.3 Multi-Line Block Comments (`/* ... */`)
A multi-line block comment begins with `/*` and terminates at the first subsequent `*/`.
* **Non-Nesting:** Block comments **DO NOT** nest. An interior `/*` is treated as literal comment content.
* **Unterminated Comments:** If the end of the source stream is reached before a closing `*/` is encountered, the lexer **MUST** emit diagnostic error `E0010` (`UnterminatedBlockComment`) spanning from the initial `/*` to EOF.

---

## 4. Identifiers & Keywords

### 4.1 Identifiers
An identifier represents a user-defined symbol (variable, function, class, interface, package name).

```ebnf
identifier  ::= id_start id_continue*
id_start    ::= [A-Za-z_]
id_continue ::= [A-Za-z0-9_]
```

* Identifiers are strictly case-sensitive (`value`, `Value`, and `VALUE` are distinct).
* Identifiers **SHALL NOT** match any reserved keyword or primitive type name.

### 4.2 Reserved Keywords (42 Keywords)
The following 42 tokens are reserved keywords:
`if`, `else`, `for`, `while`, `do`, `return`, `break`, `continue`, `switch`, `case`, `default`, `const`, `class`, `enum`, `try`, `catch`, `finally`, `throw`, `static`, `inline`, `alias`, `import`, `public`, `native`, `protected`, `private`, `internal`, `package`, `new`, `operator`, `extends`, `super`, `virtual`, `override`, `weak`, `abstract`, `interface`, `implements`, `instanceof`, `sizeof`, `assert`, `exit`.

### 4.3 Primitive Type Keywords (14 Keywords)
The following 14 tokens are reserved as built-in primitive types:
`void`, `bool`, `int8`, `int16`, `int32`, `int64`, `uint8`, `uint16`, `uint32`, `uint64`, `float32`, `float64`, `char`, `tuple`.

---

## 5. Numeric Literals

The lexer supports decimal, hexadecimal, binary, and floating-point literals.

### 5.1 Decimal Integer Literals
A sequence of decimal digits (`0`–`9`). Leading zeros are permitted only if the number is exactly `0`.
* Default type is `int32` (or `int64` if the value exceeds 32-bit signed range).
* Suffix `L` or `l` explicitly designates a 64-bit integer (`int64`).
* Exceeding `u64::MAX` triggers diagnostic error `E0009` (`IntegerLiteralOverflow`).

### 5.2 Hexadecimal Integer Literals
Begins with prefix `0x` or `0X` followed by one or more hexadecimal digits (`0`–`9`, `a`–`f`, `A`–`F`) with an optional `L` / `l` suffix:
* `0xFF`, `0xDEAD_BEEF`, `0x10L`.
* Having no hex digits after `0x` triggers diagnostic error `E0008` (`MalformedNumericLiteral`).

### 5.3 Binary Integer Literals
Begins with prefix `0b` or `0B` followed by one or more binary digits (`0` or `1`) with an optional `L` / `l` suffix:
* `0b1010`, `0b1111_0000`, `0b1L`.
* Having no binary digits after `0b` triggers diagnostic error `E0008` (`MalformedNumericLiteral`).

### 5.4 Floating-Point Literals
A decimal number containing a fractional point `.`, an exponent part (`e` or `E`), or both:
* Standard: `3.14159`, `0.5`, `1.0e-4`, `2e10`.
* Optional suffix `f` or `F` explicitly narrows the literal to a 32-bit float (`float32`). Default is `float64`.

---

## 6. String & Character Literals

### 6.1 String Literals (`"..."`)
Enclosed in double quotes `"` and spans zero or more characters or escape sequences:
* `""`, `"Hello, Solix!"`, `"Line 1\nLine 2"`.
* Multi-line raw strings must have proper closing quotes before EOF. Reaching EOF without a closing quote emits `E0001` (`UnterminatedStringLiteral`).

### 6.2 Character Literals (`'...'`)
Enclosed in single quotes `'` and represents exactly one decoded character or escape sequence:
* `'A'`, `'\n'`, `'\x7F'`.
* Empty character literal `''` triggers `E0005` (`EmptyCharacterLiteral`).
* Multi-character literal `'ab'` triggers `E0006` (`MultiCharacterLiteral`).
* Missing closing quote triggers `E0004` (`UnterminatedCharacterLiteral`).

### 6.3 Escape Sequences
The following escape sequences are supported in strings and characters:

| Sequence | Decoded Value | Description |
| :--- | :--- | :--- |
| `\n` | `0x0A` | Line Feed (newline) |
| `\t` | `0x09` | Horizontal Tab |
| `\r` | `0x0D` | Carriage Return |
| `\\` | `0x5C` | Backslash |
| `\'` | `0x27` | Single Quote |
| `\"` | `0x22` | Double Quote |
| `\0` | `0x00` | Null Byte |
| `\xHH` | `0xHH` | 2-digit hexadecimal byte escape |

Any unrecognized escape character (e.g. `\q`) emits diagnostic error `E0002` (`InvalidEscapeSequence`). A malformed `\x` escape without 2 valid hex characters emits `E0007` (`InvalidHexEscape`).

---

## 7. Operators & Punctuation

The lexer applies **maximal munch** (longest match rule):

| Category | Terminal Tokens |
| :--- | :--- |
| **Arithmetic** | `+`, `-`, `*`, `/`, `%` |
| **Assignment** | `=`, `+=`, `-=`, `*=`, `/=`, `%=` |
| **Comparison** | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| **Logical** | `&&`, `\|\|`, `!` |
| **Bitwise** | `~`, `&` |
| **Increment/Decrement** | `++`, `--` |
| **Lambda / Arrow** | `=>` |
| **Ternary / Elvis** | `?`, `:` |
| **Punctuation** | `;`, `,`, `.`, `::`, `...` |
| **Delimiters** | `(`, `)`, `{`, `}`, `[`, `]`, `[]` (array brackets optimization) |

---

## 8. Diagnostic Error Codes & Parser Delegation

In accordance with compiler architecture guidelines, the lexical analyzer operates strictly to categorize source text into discrete terminal tokens with zero allocations. The lexer **ONLY** emits errors when an unrecognized character or malformed prefix is encountered that cannot form any valid token:

| Error Code | Name | Diagnostic Code | Trigger Condition & Remediation |
| :--- | :--- | :--- | :--- |
| **`E0000`** | `UnknownToken` | `solix::lexer::unknown_token` | Byte or character sequence not recognized in Solix grammar (e.g. `@`, `#`, `$`, standalone `\|`, unclosed string/comment reaching EOF, or incomplete `0x`/`0b` prefixes). |

### 8.1 Deferred Semantic & Literal Validation
Higher-level validation of literals is strictly deferred to the parser phase when constructing typed AST literals:
* **String & Character Escape Decoding:** Validating `\n`, `\xHH`, and rejecting invalid escape sequences (e.g. `\q`).
* **Character Literal Length:** Rejecting empty `''` or multi-character `'ab'`.
* **Numeric Literal Constraints:** Validating hex/binary digit ranges and integer overflow (> `u64::MAX`).
