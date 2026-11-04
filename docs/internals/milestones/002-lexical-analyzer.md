# Milestone 002: High-Performance Lexical Analyzer

## Document Metadata
* **Milestone ID:** MS-002
* **Target Version:** `v0.1.0-alpha.2`
* **Milestone Branch:** `master`
* **Status:** COMPLETED
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-11-15
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/specs/lexical.md` (SPEC-0001)

---

## 1. Executive Summary & Objectives

Milestone 002 implements the **lexical analysis engine (Lexer)** for the Solix language compiler in modern Rust (Edition 2024), completely replacing the legacy C++ single-pass `LexerState` implementation (`core/src/processes/lexer.cpp`).

In the legacy C++ implementation, lexical analysis suffered from several architectural deficiencies:
1. **String-Embedded Errors:** Errors were bundled as string payloads inside an untyped `UNKNOWN_TOKEN` variant, preventing downstream tools and tests from programmatically asserting error codes.
2. **Heap-Allocated Values:** Every token carried a heap-allocated `std::variant<int64_t, double, std::string, ...>`, incurring significant memory overhead during tokenization.
3. **Raw String Slicing:** Literal extractions relied on multiple heap-allocated substrings (`std::string::substr`) rather than zero-copy byte coordinate spans.
4. **Permissive Panics:** Error handling relied on C++ standard library exceptions (`std::stoull`, `std::stod`) with loose `catch (...)` blocks.

Milestone 002 re-architects the lexical analyzer into a **zero-copy, high-throughput, military-grade Rust subsystem**:
* **Zero Heap Allocations for Tokens:** Tokens are compact 16-byte `Copy` structures (`Token { kind: TokenKind, span: Span }`), indexing directly into `CompilationContext.sources` via byte offsets.
* **Safe Cursor Architecture:** A robust `SourceCursor<'source>` operating over UTF-8 byte slices with checked lookahead (`peek()`, `peek_next()`, `match_byte()`), eliminating slicing panics and unsafe pointers.
* **Hierarchical Error Tree (`LexerError`):** Dedicated diagnostic enum implementing `thiserror::Error` and `miette::Diagnostic` with standardized error codes (`E0001` through `E0010`), source code spans, and actionable remedies.
* **Complete Terminal Grammar Coverage:** 103 terminal grammar tokens (56 keywords and primitives, 25 operators, 14 delimiters/punctuation, numeric literals across decimal/hex/binary/float radices, escape sequence decoding, and comment stripping).
* **Multi-Source CompilationContext Integration:** Seamless tokenization across multiple source units registered in `CompilationContext`.

---

## 2. Scope & Crates Affected

* **`crates/solix-compilation`**:
  * `src/phases/lexing/tokens.rs`: Standard terminal symbols, `Span`, `TokenKind` (103 variants), `Token`, and `KEYWORDS` hashmap (complete).
  * `src/phases/lexing/cursor.rs`: Safe zero-copy byte cursor tracking byte offset, line number, and column coordinates.
  * `src/phases/lexing/errors.rs`: Hierarchical `LexerError` enum with `E0001`–`E0010` diagnostic metadata.
  * `src/phases/lexing/lexer.rs`: Core lexical analyzer implementation driving tokenization from input buffers.
  * `src/phases/lexing/mod.rs`: Public re-exports and lexical pass entrypoints.
  * `src/CompilationContext.rs`: Multi-source registry (`sources: Vec<String>`) assigning unique `u16` source IDs and accumulating token streams (`tokens: Vec<Vec<Token>>`).
* **`crates/solix-compilation/tests/`**:
  * `tests/lexer_tests.rs`: Tier 1 & Tier 2 integration tests covering all token categories, cursor bounds, and keyword lookups.
  * `tests/fixtures/syntax/positive/*.slx`: Positive test programs verifying valid lexical grammar.
  * `tests/fixtures/syntax/negative/*.slx`: Negative test fixtures verifying exact error codes (`E0001`–`E0010`).

---

## 3. Prioritized Deliverables & Task Matrix

### 3.1 P0: Critical Blockers (Release Gate)

- [x] `[P0]` **Safe Zero-Copy Byte Cursor (`LexerState` in `lexer.rs`):**
  Implement safe slice scanner operating over `&'source [u8]`.
  * Track `cursor: usize` offset, and line/column via token spans.
  * Implement safe lookahead: `peek() -> Option<u8>`, `peek_next() -> Option<u8>`, `advance() -> Option<u8>`, and `match_byte(expected: u8) -> bool`.
  * `#![forbid(unsafe_code)]` with zero unchecked slice indexing.

- [x] `[P0]` **Diagnostic Error Handling for Lexing (`LexingError`):**
  Implement `LexingError` conforming to GL-0011 with `thiserror::Error` and `miette::Diagnostic`.
  * Unless an unknown or illegal token is encountered (`TokenKind::Unknown`), the lexer outputs no errors.
  * Granular literal validation (such as character count, numeric overflow, or escape syntax) is cleanly delegated to the parser phase when constructing typed AST literals.
  * Record `UnknownToken { token: Token, span: SourceSpan }` with code `solix::lexer::unknown_token` (`E0000`).
  * Include actionable `#[help("...")]` recommendation.

- [x] `[P0]` **Core Maximal-Munch Lexer Engine (`lexer.rs`):**
  Implement `lex(context: &mut CompilationContext) -> Result<MetaData, Vec<LexingError>>` and `lex_source(source_id: u16, source: &str) -> Result<Vec<Token>, Vec<LexingError>>`.
  * Match operators greedily (e.g. `==` before `=`, `=>` before `=`, `++` before `+`).
  * Classify keywords using `lookup_keyword()` in $O(1)$.
  * Return `TokenKind::Eof` as the terminating token.

### 3.2 P1: High Priority (Milestone Core)

- [x] `[P1]` **String & Character Literal Tokenization:**
  Port string and character literal processing from C++ `handle_string()` and `handle_character()`:
  * String delimiters (`"..."`) and character delimiters (`'...'`).
  * Yields `TokenKind::String` and `TokenKind::Char` with zero string copies.
  * Emit `TokenKind::Unknown` and `LexingError::UnknownToken` on unclosed quotes at EOF.
  * Escape sequence decoding and semantic character literal validation deferred to the parser.

- [x] `[P1]` **Multi-Radix Numeric Literal Lexing:**
  Port numeric parsing from C++ `handle_number()`:
  * Decimal integers (`42`, `1000`).
  * Hexadecimal integers (`0xFF`, `0xDEAD_BEEF`) with prefix `0x`/`0X`.
  * Binary integers (`0b1010`, `0b1111_0000`) with prefix `0b`/`0B`.
  * Integer suffix support: `L` or `l` for explicit `int64`.
  * Floating-point literals with decimal point (`3.14`), exponents (`1.0e-4`, `2e10`), and suffix `f`/`F` for `float32`.
  * Reject malformed prefixes without digits (e.g. standalone `0x`, `0b`) with `LexingError::UnknownToken`.

- [x] `[P1]` **Comment Stripping & Unterminated Block Comment Detection:**
  Port comment handling from C++:
  * Single-line comments (`// ...`) consumed up to newline without adding tokens.
  * Non-nesting multi-line block comments (`/* ... */`) consumed up to closing `*/`.
  * Emit `LexingError::UnknownToken` if EOF is reached before closing `*/`.

- [x] `[P1]` **Composite Operators & Delimiter Optimization:**
  Support all composite operators and punctuation:
  * Compound assignments: `+=`, `-=`, `*=`, `/=`, `%=`.
  * Equality and relational: `==`, `!=`, `<=`, `>=`.
  * Increments and arrows: `++`, `--`, `=>`.
  * Delimiters: `::`, `...`, and `[]` (`PunctuationArrayBrackets` optimization when `[` is directly followed by `]`).

- [x] `[P1]` **CompilationContext Multi-Source Integration:**
  Update `CompilationContext`:
  * `add_source(name: String, content: String) -> u16`: Registers source text, returns `source_id`.
  * Zero-copy `get_lexeme(span: Span) -> &str` indexing directly into sources vector.
  * `lex(&mut context)` runs the lexer across all registered sources and populates `context.tokens: Vec<Vec<Token>>`.

- [x] `[P1]` **Master Positive Fixture & Inlined Negative Tests:**
  * Master Positive Fixture: Complete `.slx` file in `tests/fixtures/syntax/positive/all_tokens.slx` covering all 103 terminal symbols.
  * Master Negative Fixture: `tests/fixtures/syntax/negative/invalid_tokens.slx` verifying accumulation of 8 distinct unrecognized tokens.
  * Inlined Negative Snippets: Concise 1-2 line error snippets inlined directly in `tests/lexer_tests.rs` without creating single-line file clutter.

### 3.3 P2: Medium Priority (Ergonomics & Performance)

- [x] `[P2]` **BOM & Line Ending Normalization:**
  * Normalize `CRLF`, `LF`, and lone `CR` line endings seamlessly.

- [x] `[P2]` **Diagnostic Rendering Integration with `miette`:**
  * Derive `thiserror::Error` and `miette::Diagnostic` on `LexingError` with source span label and actionable help message.

- [x] `[P2]` **Compiler Throughput & Zero-Allocation Target:**
  * Ensure lexer achieves maximal throughput on realistic Solix source code without heap allocation bottlenecks (16-byte `Copy` tokens, zero string cloning).

### 3.4 P3: Low Priority (Tooling & Benchmarking)

- [ ] `[P3]` **Lexer Criterion Benchmark (`benches/lexer_bench.rs`):**
  Create an optional criterion benchmark tracking tokenization throughput across large synthetic Solix source files (deferred to Milestone 003).

- [ ] `[P3]` **CLI `--emit-tokens` Subcommand:**
  Add a flag to `solix compile --emit-tokens` that prints formatted token streams to stdout for compiler debugging (deferred to CLI / Parser milestone).

---

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Compiles with 0 warnings across all targets | `cargo build --workspace --all-targets` | [x] Verified |
| **Automated Test Suite** | 100% pass across all unit and integration tests | `cargo test --workspace --all-targets` | [x] Verified |
| **Clippy Static Analysis** | 0 warnings, 0 unwraps, strict pedantic pass | `cargo clippy --workspace --all-targets -- -D warnings` | [x] Verified |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [x] Verified |
| **Safety Invariants** | `#![forbid(unsafe_code)]` declared in `lexer.rs`, `tokens.rs` | Workspace source audit | [x] Verified |
| **Error Code Coverage** | 100% coverage of `LexingError::UnknownToken` (`E0000`) | `cargo test --test lexer_tests` | [x] Verified |
| **Living Documentation** | SPEC-0001 (`lexical.md`) synchronized with implementation | Audit against GL-0010 | [x] Verified |

---

## 5. Architectural Blueprint & Legacy C++ Mapping

### 5.1 Architecture Pipeline Diagram

```
Raw Source String (&str)
        │
        ▼
┌──────────────────────────────────────┐
│  SourceCursor<'source> (cursor.rs)   │  <-- Checked lookahead, line/column tracking
└──────────────────┬───────────────────┘
                   │ byte stream
                   ▼
┌──────────────────────────────────────┐
│      Lexer Engine (lexer.rs)         │  <-- Maximal-munch pattern matching
├──────────────────────────────────────┤
│  • Identifiers & KEYWORDS Hashmap    │
│  • Escape Sequence Decoder           │
│  • Multi-Radix Numeric Parser        │
│  • Comment & Whitespace Stripper     │
└──────┬────────────────────────┬──────┘
       │                        │
  Ok(TokenStream)          Err(LexerError)
       │                        │
       ▼                        ▼
Vec<Token> (tokens.rs)     miette::Diagnostic (errors.rs)
  { kind, span }             [E0001 - E0010 with SourceSpan & Help]
```

### 5.2 Legacy C++ to Modern Rust Structural Mapping

| C++ `LexerState` (`core/src/processes/lexer.cpp`) | Modern Rust Implementation (`solix-compilation`) | Key Architectural Improvement |
| :--- | :--- | :--- |
| `std::string source_code` | `&'source [u8]` inside `SourceCursor` | Zero-copy scanning; no string cloning. |
| `char peek() const` | `cursor.peek() -> Option<u8>` | Bounds checked without index out of bounds panics. |
| `char advance()` | `cursor.advance() -> Option<u8>` | Automatic column and UTF-8 boundary tracking. |
| `handle_identifier()` + `keywords` map | `TokenKind::is_keyword()` + `KEYWORDS: LazyLock<HashMap>` | Constant-time static lookup; returns typed enum. |
| `handle_string()` with `substr` | Zero-copy slice + `Span` tracking | Escape decoding directly into verified token span. |
| `TokenType::UNKNOWN_TOKEN` with string message | `LexerError` with explicit `E000X` codes | Standardized `miette` diagnostic tree; no stringly-typed errors. |
| `Token` with `const Source*` & `Value` | `Token { kind: TokenKind, span: Span }` | 16-byte `Copy` structure with zero heap allocations. |

---

## 6. Diagnostic Error Code Specification Matrix & Parser Delegation

In accordance with compiler architecture guidelines, the lexical analyzer only emits errors when an unrecognized byte or malformed prefix is encountered that cannot form any valid token:

| Error Code | Diagnostic Variant | Trigger Condition | Recommended Help Message |
| :--- | :--- | :--- | :--- |
| **`E0000`** | `UnknownToken` | Unrecognized byte (e.g. `@`, `#`, `$`, standalone `\|`), incomplete `0x`/`0b`, or unclosed literal at EOF | `"Remove or replace this unrecognized character with valid Solix syntax."` |

### 6.1 Deferred Parser Validations
All higher-level literal validations (such as escape character validation, character literal count, or numeric literal overflow) are cleanly delegated to the AST parser phase.

---

## 7. Risk Assessment & Mitigation Strategies

* **Risk 1: UTF-8 Slicing Panics Across Code Point Boundaries**
  * *Impact:* Slicing `&str` directly by arbitrary byte indices when scanning non-ASCII characters inside comments or string literals triggers a panic (`byte index is not a char boundary`).
  * *Mitigation:* The `SourceCursor` operates on the underlying byte slice (`&[u8]`). When advancing across Unicode characters, it uses `std::str::from_utf8` or `char::from_u32` to step complete UTF-8 codepoints, guaranteeing safe byte boundary alignment.

* **Risk 2: Catastrophic Backtracking on Malformed Operators**
  * *Impact:* Parsing complex composite symbols (`...`, `[]`, `=>`, `+=`) could cause unbounded lookahead and performance degradation.
  * *Mitigation:* The cursor lookahead is strictly bounded to 2 bytes (`peek()` and `peek_next()`). Maximal munch is implemented via deterministic finite-state match statements without unbounded backtracking.

* **Risk 3: Loss of C++ Compatibility in Keyword and Escape Behavior**
  * *Impact:* Existing Solix code and test files written for the C++ version fail to tokenize in Rust.
  * *Mitigation:* All 56 keywords, primitive names, and escape characters from `core/src/processes/lexer.cpp` have been 100% mirrored in `tokens.rs` and SPEC-0001, verified by exhaustive automated regression tests.
