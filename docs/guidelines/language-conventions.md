# Rust Language & Security Conventions Specification

## Document Metadata
* **Guideline ID:** GL-0011
* **Scope:** Universal Workspace Codebase (`crates/*` [Rust Edition 2024])
* **Status:** Final Implementation Standard
* **Audience:** Core Systems Engineers, Compiler Architects, VM Runtime Engineers, Tooling Authors
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)

---

## 1. Executive Summary & Purpose

The Solix language compiler and deterministic virtual machine are authored exclusively in **modern Rust (Edition 2024)**. Because Solix operates as a programming language toolchain and runtime executing arbitrary user code, the codebase must uphold the highest standards of memory safety, determinism, readability, and defensive security.

Vague variable naming, single-letter identifiers, unhandled panics, careless error unwrapping, and comment bloat compromise codebase maintainability and introduce security vulnerabilities. This specification establishes the binding engineering standards for all Rust code in the Solix repository:
1. **Zero Single-Letter Identifiers:** Mandatory descriptive names or established multi-letter abbreviations for all variables, parameters, generic types, and lifetimes.
2. **Military-Grade Security & Safety:** Strict prohibition of `unsafe` code (`#![forbid(unsafe_code)]`), elimination of unhandled panics (`.unwrap()` banned), checked arithmetic, and defensive bounds validation.
3. **Structured Hierarchical Error Handling:** Proper error chaining using `thiserror` with `#[source]` attributes, enriched diagnostic context, and `miette` integration for rich compiler error reporting.
4. **Code Simplicity & Readability:** Straightforward, readable, and idiomatic architectures that minimize cognitive load.
5. **Strict Comment Discipline:** Code comments are reserved **exclusively for difficult, complex, or non-obvious logic**; trivial commentary is strictly prohibited.

---

## 2. Normative Invariants & Core Principles

All code authored in `crates/*` **MUST** adhere to the following binding invariants:

### 2.1 Identifier Naming & The Zero Single-Letter Rule
1. **Zero Single-Letter Variables:** Single-letter variable names (such as `i`, `j`, `k`, `x`, `y`, `n`, `s`, `c`, `v`, `t`, `e`, `p`) **SHALL NOT** be used in any loop, closure, match pattern, or function body.
   * In loop counters, engineers **MUST** use full descriptive names such as `index`, `counter`, `iteration_count`, `byte_offset`, or `line_number`.
   * In error handling closures or match arms, engineers **MUST** use `error`, `error_cause`, `failure`, or `diagnostic` instead of `e`.
2. **Zero Single-Letter Generics and Lifetimes:** Generic type parameters **SHALL NOT** be single capital letters (such as `T`, `E`, `R`, `U`, `V`).
   * Generic types **MUST** be named descriptively according to their role: `ItemType`, `ValueType`, `ErrorType`, `ContextType`, `StatePayload`.
   * Lifetimes **MUST** use descriptive labels (`'source`, `'arena`, `'ctx`, `'input`) instead of `'a` or `'b`.
3. **Established Multi-Letter Abbreviations Permitted:** Established multi-letter abbreviations are permitted where they are standard in compiler engineering: `expr` (expression), `stmt` (statement), `decl` (declaration), `ident` (identifier), `param` (parameter), `arg` (argument), `span` (source span), `vm` (virtual machine), `ast` (abstract syntax tree).

### 2.2 Military-Grade Security & Safety Invariants
1. **Forbid Unsafe Code:** Every workspace crate **MUST** declare `#![forbid(unsafe_code)]` at its root (`src/lib.rs` or `src/main.rs`). Any variance requiring `unsafe` **MUST** be formally approved by two core maintainers, isolated in a minimal leaf module, and preceded by a comprehensive `// SAFETY:` invariant explanation.
2. **Zero Panics in Non-Test Code:** Production library code **MUST NOT** invoke `.unwrap()`, `.expect()`, `panic!()`, `unreachable!()`, or `todo!()`.
   * The workspace enforces:
     ```rust
     #![deny(clippy::unwrap_used)]
     #![deny(clippy::expect_used)]
     #![deny(clippy::panic)]
     ```
   * All error conditions **MUST** be handled through explicit `Result<ValueType, ErrorType>` propagation.
   * `.unwrap()` and `.expect()` are permitted **exclusively** within unit test modules (`#[cfg(test)] mod tests`) and integration test suites (`tests/`).
3. **Checked Arithmetic & Defensive Bounds:** All arithmetic operations that evaluate user bytecode or perform buffer allocations **MUST** use checked arithmetic (`checked_add`, `checked_mul`), saturating operations, or trigger an explicit `RuntimeTrap::IntegerOverflow`.
4. **Defensive Slice & Stack Access:** The virtual machine operand stack and instruction stream **MUST NOT** perform raw unchecked indexing. Array and slice lookups **MUST** use `.get()` with explicit error handling when offsets originate from external bytecode.
5. **Defensive Memory Sanitization:** Sensitive memory structures, cryptographic key material, and execution frame buffers **SHOULD** implement zeroing on drop (e.g., via `Zeroize`) to prevent memory exfiltration in core dumps.

### 2.3 Structured Hierarchical Error Architecture (`thiserror` + `miette`)

1. **Hierarchical Error Tree (Enums of Sub-Enums):**
   * Compiler and virtual machine errors **MUST NOT** be authored as a flat monolithic enum where every error across the entire toolchain is bundled together.
   * Errors **MUST** be structured as a **hierarchical tree of enums**:
     * **Root Workspace Error (`SolixError`):** Top-level entrypoint error wrapping crate-level errors (`Compiler`, `Runtime`, `Client`, `Io`).
     * **Crate-Level / Subsystem Error (`CompilerError`, `RuntimeError`):** Composed of sub-enums representing compiler pipeline phases or runtime components (`Lexer`, `Parser`, `Binder`, `TypeCheck`, `Codegen`).
     * **Domain-Specific Leaf Enum (`LexerError`, `ParserError`, `TypeError`, `VirtualMachineError`):** Granular error code variants representing specific diagnostics with contextual fields.
   * This architecture guarantees complete modular decoupling, clean encapsulation, and precise diagnostic mapping.

2. **Structured Error Code Metadata in Every Variant:**
   * Every error variant in a domain leaf enum **MUST** represent a formally registered error code (e.g., `E0001`, `E0102`).
   * Each variant **MUST** provide comprehensive diagnostic metadata:
     * **Formal Error Code:** Machine-readable identifier string declared via `#[diagnostic(code(solix::<subsystem>::<code_name>))]` and returned by `fn error_code(&self) -> &'static str`.
     * **Descriptive Error Message:** Human-readable explanation interpolated with contextual data via `#[error("...")]`.
     * **Source Code Span:** Concrete source code coordinates (`SourceSpan`) annotated with a contextual label (`#[label("...")]`).
     * **Actionable Help / Remedy:** Helpful suggestion guiding the user on how to resolve the error (`#[help("...")]`).

3. **Diagnostic Authoring & Message Hygiene Rule:**
   * All user-facing error messages and formatting builders **MUST** format Solix types, AST literals, values, binary tags, and errors via human-readable `Display` implementations (`.to_string()`, `{return_type}`, `{err}`) — **NEVER** leak Rust `Debug` syntax (`{:?}`, such as `Primitive(Int32)`, `["a"]`, or raw byte slices `[80, 75]`) into `#[error(...)]`, `#[help(...)]`, or error builder strings.
   * Every diagnostic variant **MUST** allocate a globally unique `E`/`R` error code across the entire workspace within the subsystem ranges below, avoid hardcoded `main`-only or `manifest.toml` wording where configurable `manifest.yaml` entry points (`profiles.<profile>.entrypoint` or `--entry`) apply, and match `docs/wiki/errors/` (verified by `python3 scripts/ci/verify_docs.py`).
4. **Mandatory Unbroken Causal Chaining with `#[source]`:**
   * When any error variant wraps an inner error (whether a sub-enum, an underlying I/O error, or an operating system error), it **MUST** annotate the field with `#[source]` (or use `#[from]` which implies `#[source]`).
   * The causal chain from root `SolixError` down to the leaf cause **MUST NOT** be broken or erased.
   * The entire error hierarchy **MUST** implement `std::error::Error::source(&self)` so that tools, logs, diagnostics, and test harnesses can programmatically traverse the complete causal tree.

5. **Standard Error Code Allocation Table:**

| Error Code Range | Subsystem Domain | Description & Examples |
| :--- | :--- | :--- |
| **`E0000` – `E0099`** | **Lexer / Lexical Analysis** | Unterminated string/char literals, unterminated comments, invalid hex/binary/decimal literals, source index limits, unrecognized tokens (`E0000`, `E0011`–`E0018`). |
| **`E0101` – `E0200`** | **Parser / Syntax Analysis** | Missing delimiters, invalid type paths (`E0157`), unknown control-flow keywords (`E0158`), invalid modifiers (`E0160`–`E0200`). |
| **`E0201` – `E0299`, `E0315` – `E0323`** | **Type Binder & Semantics** | Undefined symbols, type mismatches, inheritance/interface cycles (`E0204`/`E0323`), switch/exception/const validation. |
| **`E0301` – `E0314`** | **Bytecode Emitter & Binary (`EmissionError`)** | Constant pool/register/argument/jump limits, invalid magic/version, corrupt `.slxb` containers, export manifests. |
| **`E0330` – `E0399`, `E9001`** | **Stdlib, Toolchain CLI & Custom Plugins** | `mount://` read-only writes, UTF-8/JSON/socket/compression errors, cycle collector threshold (`E0341`), custom lint phases (`E9001`). |
| **`E0401` – `E0499`** | **Package & Project Manifest (`ManifestError`)** | Schema validation, removed `inherits`, missing `profiles.release`, unknown root keys, manifest I/O errors. |
| **`R0000` – `R0027`** | **Virtual Machine & Runtime (`RuntimeErrorCode`)** | Execution traps, division by zero, null dereferences, stack overflow, array store mismatches, explicit exits. |

### 2.4 Code Simplicity & Comment Discipline
1. **Simplicity Over Cleverness:** Prefer straightforward, obvious code over convoluted macro metaprogramming or complex trait gymnastics. Code that is easy to read, audit, and reason about is inherently more secure.
2. **Comment Discipline (High Signal, Zero Noise):** Comments **MUST ONLY** be written on complex, non-obvious, or subtle pieces of code:
   * **Permitted:** Explaining bit-packing in bytecode formats, zero-copy span cursor mechanics, Pratt parsing precedence binding powers, or subtle borrow-checker workarounds.
   * **Prohibited:** Writing trivial narrative comments that merely restate what the code does (e.g., `// increments counter`, `// loop through items`, `// return the result`).
3. **Rustdoc for Public API:** All public items **MUST** have clean `///` documentation explaining purpose, arguments, return values, errors, and examples.

---

## 3. Structural Specification & Schema

### 3.1 Naming Convention Reference Table

| Code Construct | Convention | Good Example | Prohibited Anti-Pattern |
| :--- | :--- | :--- | :--- |
| **Local Variables** | `snake_case` | `instruction_pointer`, `counter` | `i`, `ip`, `x`, `tmp` |
| **Loop Indices** | `snake_case` | `index`, `iteration_count`, `row_index` | `i`, `j`, `k`, `idx` |
| **Error Bindings** | `snake_case` | `error`, `error_cause`, `failure` | `e`, `err` |
| **Generic Types** | `UpperCamelCase`| `ItemType`, `ValueType`, `ErrorType` | `T`, `E`, `R`, `U`, `V` |
| **Lifetimes** | `'snake_case` | `'source`, `'arena`, `'context` | `'a`, `'b`, `'x` |
| **Constants / Statics** | `SCREAMING_SNAKE` | `MAX_CALL_DEPTH`, `MAGIC_BYTES` | `MaxDepth`, `magic` |
| **Structs / Enums** | `UpperCamelCase`| `VirtualMachine`, `CallFrame` | `vm`, `call_frame` |
| **Functions / Methods**| `snake_case` | `execute_instruction`, `parse_statement`| `exec`, `do_parse` |

### 3.2 Hierarchical Error Tree Schema (`thiserror` + `miette`)

The following reference implementation illustrates the normative multi-tiered error architecture required across the Solix toolchain:

```rust
use miette::{Diagnostic, SourceSpan};
use std::path::PathBuf;
use thiserror::Error;

// ============================================================================
// Tier 1: Domain-Specific Leaf Error Enum (e.g., Lexer Subsystem)
// ============================================================================

/// Granular errors emitted specifically during lexical tokenization.
#[derive(Error, Diagnostic, Debug, PartialEq, Eq)]
pub enum LexerError {
    #[error("Unterminated string literal")]
    #[diagnostic(
        code(solix::lexer::E0001),
        help("Add a closing quote '\"' to terminate the string literal before end of line or file.")
    )]
    UnterminatedString {
        #[label("string literal starts here but is never closed")]
        span: SourceSpan,
    },

    #[error("Invalid escape sequence '\\{escape_char}' in string literal")]
    #[diagnostic(
        code(solix::lexer::E0002),
        help("Valid escape sequences are: \\n, \\r, \\t, \\\\, \\0, \\', \\\", \\xHH, \\u{{HHHH}}.")
    )]
    InvalidEscapeSequence {
        escape_char: char,
        #[label("unrecognized escape character '\\{escape_char}'")]
        span: SourceSpan,
    },

    #[error("Unrecognized character '{character}' (byte: 0x{byte_value:02X})")]
    #[diagnostic(
        code(solix::lexer::E0003),
        help("Remove or replace this character with valid Solix syntax.")
    )]
    UnrecognizedCharacter {
        character: char,
        byte_value: u8,
        #[label("unexpected character")]
        span: SourceSpan,
    },
}

impl LexerError {
    /// Returns the standardized diagnostic error code string.
    #[must_use]
    pub const fn error_code(&self) -> &'static str {
        match self {
            Self::UnterminatedString { .. } => "E0001",
            Self::InvalidEscapeSequence { .. } => "E0002",
            Self::UnrecognizedCharacter { .. } => "E0003",
        }
    }
}

// ============================================================================
// Tier 2: Crate-Level / Subsystem Error Enum (Enums of Sub-Enums)
// ============================================================================

/// High-level compiler error encompassing all pipeline phases.
#[derive(Error, Diagnostic, Debug)]
pub enum CompilerError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Lexer(#[from] #[source] LexerError),

    #[error("Source file I/O failure for '{file_path}': {source}")]
    #[diagnostic(code(solix::compiler::E0010))]
    SourceFileReadError {
        file_path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

// ============================================================================
// Tier 3: Root Workspace Error Enum
// ============================================================================

/// Top-level workspace error returned by the main driver and library entrypoints.
#[derive(Error, Diagnostic, Debug)]
pub enum SolixError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Compiler(#[from] #[source] CompilerError),

    #[error("Fatal toolchain environment error: {message}")]
    #[diagnostic(code(solix::toolchain::environment_failure))]
    Environment {
        message: String,
        #[source]
        source: Option<std::io::Error>,
    },
}

// ============================================================================
// Causal Chain Traversal Demonstration
// ============================================================================

/// Traverses and logs the full causal error tree from root to leaf cause.
pub fn log_error_causal_chain(root_error: &SolixError) {
    let mut current_cause: Option<&(dyn std::error::Error + 'static)> = Some(root_error);
    let mut depth: usize = 0;

    while let Some(cause) = current_cause {
        tracing::error!(depth, cause = %cause, "Error causal link in hierarchy");
        current_cause = cause.source();
        depth += 1;
    }
}
```

---

## 4. Production-Ready Templates

### 4.1 Production Module Template with Military-Grade Safety

```rust
//! Virtual machine execution engine module.
//!
//! Provides deterministic instruction evaluation, operand stack management,
//! and Automatic Reference Counting (ARC) lifecycle execution.

#![forbid(unsafe_code)]

use std::fmt::Debug;
use thiserror::Error;

/// Structured errors emitted during virtual machine execution.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum VirtualMachineError {
    #[error("Operand stack underflow: required {required_depth} elements, but stack depth was {actual_depth}")]
    StackUnderflow {
        required_depth: usize,
        actual_depth: usize,
    },

    #[error("Integer overflow detected during arithmetic evaluation")]
    IntegerOverflow,

    #[error("Division by zero in integer arithmetic")]
    DivisionByZero,
}

/// Strongly-typed virtual machine runtime.
#[derive(Debug)]
pub struct VirtualMachine<ValueType: Debug + Clone> {
    operand_stack: Vec<ValueType>,
    instruction_pointer: usize,
}

impl<ValueType: Debug + Clone> VirtualMachine<ValueType> {
    /// Creates a new virtual machine instance with empty operand stack.
    #[must_use]
    pub fn new() -> Self {
        Self {
            operand_stack: Vec::new(),
            instruction_pointer: 0,
        }
    }

    /// Pushes an evaluated value onto the operand stack.
    pub fn push_operand(&mut self, value: ValueType) {
        self.operand_stack.push(value);
    }

    /// Pops an evaluated value from the top of the operand stack.
    ///
    /// # Errors
    /// Returns [`VirtualMachineError::StackUnderflow`] if the operand stack is empty.
    pub fn pop_operand(&mut self) -> Result<ValueType, VirtualMachineError> {
        self.operand_stack.pop().ok_or_else(|| {
            VirtualMachineError::StackUnderflow {
                required_depth: 1,
                actual_depth: 0,
            }
        })
    }
}

// ============================================================================
// Unit Tests (MANDATORY: Placed at the bottom of the file)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_push_and_pop_operand() {
        let mut virtual_machine: VirtualMachine<i32> = VirtualMachine::new();
        virtual_machine.push_operand(42);

        let pop_result = virtual_machine.pop_operand();
        assert_eq!(pop_result, Ok(42));
    }

    #[test]
    fn test_negative_pop_empty_stack_returns_stack_underflow() {
        let mut virtual_machine: VirtualMachine<i32> = VirtualMachine::new();

        let pop_result = virtual_machine.pop_operand();
        assert_eq!(
            pop_result,
            Err(VirtualMachineError::StackUnderflow {
                required_depth: 1,
                actual_depth: 0,
            })
        );
    }
}
```

---

## 5. Concrete Production Examples

### Example 1: Robust Opcode Evaluation with Checked Arithmetic

```rust
use solix_common::instruction::OpCode;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum EvaluationError {
    #[error("Operand stack underflow: required 2 elements, found {actual_depth}")]
    StackUnderflow { actual_depth: usize },

    #[error("Arithmetic overflow occurred during integer addition")]
    IntegerOverflow,
}

pub struct ExecutionContext {
    pub operand_stack: Vec<i64>,
}

impl ExecutionContext {
    pub fn execute_add_i64(&mut self) -> Result<(), EvaluationError> {
        if self.operand_stack.len() < 2 {
            return Err(EvaluationError::StackUnderflow {
                actual_depth: self.operand_stack.len(),
            });
        }

        // Safe unwrap: stack length >= 2 checked immediately above
        let right_operand = self.operand_stack.pop().expect("Stack length verified");
        let left_operand = self.operand_stack.pop().expect("Stack length verified");

        // Checked arithmetic prevents undefined overflow behavior
        let computed_sum = left_operand.checked_add(right_operand)
            .ok_or(EvaluationError::IntegerOverflow)?;

        self.operand_stack.push(computed_sum);
        Ok(())
    }
}
```

### Example 2: Permitted Comment on Complex Bitwise Decoding

```rust
impl InstructionDecoder {
    pub fn decode_jump_offset(first_byte: u8, second_byte: u8) -> i16 {
        // Bitwise layout: Two's complement 16-bit signed integer encoded as
        // little-endian bytes. first_byte represents the low-order 8 bits,
        // second_byte represents the high-order sign-extended 8 bits.
        let low_order_bits = first_byte as u16;
        let high_order_bits = (second_byte as u16) << 8;
        let combined_word = high_order_bits | low_order_bits;
        combined_word as i16
    }
}
```

---

## 6. Anti-Patterns & Counter-Examples (What NOT to Do)

### Anti-Pattern 1: Single-Letter Variable Names
```rust
// FLAWED:
for i in 0..tokens.len() {
    let t = &tokens[i];
    match t {
        Token::Ident(s) => match_expr(s),
        _ => {}
    }
}
```
* **Failure Mode:** Decreases code comprehension, hides semantic roles of variables, and triggers immediate review rejection.
* **Corrected Form:**
```rust
// CORRECT:
for token_index in 0..tokens.len() {
    let current_token = &tokens[token_index];
    match current_token {
        Token::Identifier(identifier_name) => match_expression(identifier_name),
        _ => {}
    }
}
```

### Anti-Pattern 2: Careless `.unwrap()` in Production Code
```rust
// FLAWED:
pub fn parse_class_name(tokens: &[Token]) -> String {
    let first_token = tokens.first().unwrap(); // PANIC if tokens slice is empty!
    first_token.as_string().unwrap()          // PANIC if token is not string!
}
```
* **Failure Mode:** If a user passes an empty file or invalid token, the compiler crashes with a raw Rust panic message rather than a formatted diagnostic, destroying user trust.
* **Corrected Form:**
```rust
// CORRECT:
pub fn parse_class_name(tokens: &[Token]) -> Result<String, ParseError> {
    let first_token = tokens.first()
        .ok_or(ParseError::UnexpectedEndOfFile)?;
    first_token.extract_identifier_name()
        .ok_or(ParseError::ExpectedIdentifier)
}
```

### Anti-Pattern 3: Trivial Commentary Noise
```rust
// FLAWED:
// Increment the index by one
index += 1;

// Check if stack is empty
if stack.is_empty() {
    // Return an error
    return Err(Error::EmptyStack);
}
```
* **Failure Mode:** Clutters screen space with redundant explanations that merely echo the code syntax, distracting reviewers from actual logic bugs.
* **Corrected Form:**
```rust
// CORRECT: Code is self-documenting without narrative noise
index += 1;

if stack.is_empty() {
    return Err(Error::EmptyStack);
}
```

### Anti-Pattern 4: Flat Monolithic Error Enums & Broken Causal Chains
```rust
// FLAWED:
pub enum SolixError {
    UnterminatedString(usize),
    MissingSemicolon(usize),
    TypeMismatch(String, String),
    IoError(String), // Crushes std::io::Error into string, breaking causal chain!
}
```
* **Failure Mode:** Destroys subsystem modularity, creates monolithic coupling across the entire codebase, erases the underlying operating system error cause, and prevents test runners from asserting precise error codes.
* **Corrected Form:**
```rust
// CORRECT: Hierarchical tree of enums with unbroken #[source] chaining
#[derive(Error, Diagnostic, Debug)]
pub enum SolixError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Compiler(#[from] #[source] CompilerError),

    #[error("I/O failure: {source}")]
    #[diagnostic(code(solix::io::failure))]
    Io {
        #[source]
        source: std::io::Error,
    },
}
```

---

## 7. Verification & Compliance Checklist

Reviewers and automated CI gates **MUST** enforce the following criteria on every Rust PR:

- [ ] **No Single-Letter Variables:** Zero instances of `i`, `j`, `k`, `x`, `y`, `e`, etc., in variables or parameters.
- [ ] **Descriptive Generics:** Generic types use `ItemType`, `ValueType`, etc., rather than `T`, `E`, `R`.
- [ ] **Forbid Unsafe:** Root crate specifies `#![forbid(unsafe_code)]`.
- [ ] **Zero Unwraps in Production:** Production crates contain zero `.unwrap()`, `.expect()`, or `panic!()`.
- [ ] **Hierarchical Error Architecture:** Errors structured as enums of sub-enums (`SolixError` -> `CompilerError` -> `LexerError`).
- [ ] **Error Code Metadata & Spans:** Every error variant specifies a standardized diagnostic code (`E0XXX`), source span, and actionable help recommendation.
- [ ] **Unbroken Error Chaining:** Errors define `#[source]` attributes on all wrapped underlying errors.
- [ ] **Unit Tests at Bottom:** All unit tests are located inside `#[cfg(test)] mod tests` at the bottom of the file.
- [ ] **Zero Comment Noise:** Comments appear only on difficult-to-understand or non-obvious algorithms.
- [ ] **Full Static Pass:** `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` pass cleanly.
