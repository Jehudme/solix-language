# Test Engineering & Verification Matrices Specification

## Document Metadata
* **Guideline ID:** GL-0009
* **Scope:** Universal Quality Engineering & Verification (`crates/*/src/`, `crates/*/tests/`, `tests/`)
* **Status:** Final Implementation Standard
* **Audience:** Core Systems Engineers, Compiler Architects, VM Runtime Engineers, Quality Contributors
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)

---

## 1. Executive Summary & Purpose

In a language compiler and deterministic virtual machine, unverified code paths lead to catastrophic failures: silent generation of corrupt bytecode, unhandled runtime traps, operand stack corruption, or compiler panics on malformed user syntax. A language compiler requires verifying thousands of distinct syntactic variations, type rules, edge cases, and error conditions.

Embedding thousands of language test programs directly inside implementation files causes unmanageable source bloat, obscuring production logic. Conversely, moving internal unit tests outside of modules breaks access to private helper methods and cursor state invariants.

This specification formalizes the **Three-Tier Testing Architecture**, the **Data-Driven Fixture Standard**, the **Dual Test Requirement (Positive & Negative)**, the **Hermetic Isolation Mandate**, and the **Zero-Defect Quality Gate** for the Solix language project in Rust:
1. **Tier 1 (Internal Unit Tests):** Placed at the bottom of the source file inside `#[cfg(test)] mod tests` (or a companion `tests.rs` child module), testing private internals, cursors, and data structures.
2. **Tier 2 (Data-Driven Language Fixtures):** Standalone `.slx` files in `tests/fixtures/` executing the vast catalog of positive and negative language tests through automated runners.
3. **Tier 3 (End-to-End CLI & Runtime Integration):** Comprehensive CLI binary execution and multi-stage toolchain verification.

---

## 2. Normative Invariants & Core Principles

All test suites and automated test executions **MUST** adhere to the following binding invariants:

### 2.1 The Three-Tier Testing Architecture
1. **Tier 1: Component Unit Tests (Bottom of File / Child Module):**
   * Unit tests that verify private functions, struct fields, cursor arithmetic, or bitwise decoding **MUST** be co-located with the source file, written at the bottom inside:
     ```rust
     #[cfg(test)]
     mod tests {
         use super::*;
         // Unit tests with full private access...
     }
     ```
   * Alternatively, for larger modules, engineers **MAY** declare `#[cfg(test)] mod tests;` at the bottom of `src/foo.rs` and place the tests in a companion child module `src/foo/tests.rs`. Because `tests.rs` is a Rust child module, it retains 100% access to all private items via `use super::*;`.
   * Unit tests in Tier 1 **MUST NOT** be used to store hundreds of raw Solix language code strings; Tier 1 is strictly for Rust component logic.
2. **Tier 2: Data-Driven Language Fixtures (`tests/fixtures/`) & Inlined Snippets:**
   * **Full-Program & Multi-Statement Fixtures (`.slx`):** Complete language programs, multi-class architectures, standard library components, and large data-driven test suites **MUST** be authored as standalone `.slx` source files in `crates/<crate>/tests/fixtures/`.
   * **Concise Test Snippets Inlined Directly in `.rs`:** For short, targeted test cases spanning only a couple of lines (e.g., 1–5 lines testing an isolated token, operator lookahead, escape sequence, or specific negative error condition), engineers **SHOULD** write the test snippet inlined directly in the test file (`tests/*.rs`) using raw string literals. Creating a standalone `.slx` file for every single-line snippet is discouraged as it causes file system bloat and test fragmentation.
   * Storing massive multi-page language programs as raw string literals inside Rust source files remains strictly prohibited.
   * Fixtures are executed by automated runner scripts in `tests/` that call the public compiler and runtime entrypoints (`compile_source()`, `parse()`, `type_check()`, `execute()`).
3. **Tier 3: End-to-End Toolchain Integration (`crates/solix-client/tests/`):**
   * Multi-crate workflows, file generation, CLI flags (`--emit-asm`, `--opt-level`), and binary packaging tests **MUST** reside in `tests/` directories.

### 2.2 Dual Test Requirement & Negative Error Testing
1. **The Dual Test Requirement:** Every syntax statement, type rule, opcode, and CLI subcommand **MUST** have automated coverage for both:
   * **Positive Scenarios (Happy Path):** Valid Solix code compiles cleanly, expected AST nodes are formed, valid bytecode is assembled, and the VM executes successfully to exit code `0`.
   * **Negative Scenarios (Unhappy / Hostile Path):** Syntax errors, unexpected tokens, type mismatches, division by zero, stack underflow, corrupted opcode bytes, and invalid CLI flags.
2. **Exact Diagnostic Verification:** Every negative test **MUST** verify that the compiler or VM rejects the input gracefully with structured diagnostics (`miette`), asserting:
   * The exact expected diagnostic error code (e.g., `E0102: TypeMismatch`, `E0140: ConditionMustBeBool`).
   * The correct source line and column coordinates.
   * That the process terminates cleanly without unhandled panics (`unwrap` on `None`).

### 2.3 Hermetic Isolation & Environment Safety
1. **Absolute Test Isolation:**
   * Tests **MUST NOT** read or write to static shared paths (e.g., `/tmp/solix_test.slx`). All filesystem interactions **MUST** use ephemeral unique temporary directories via `tempfile::tempdir()` that are guaranteed to clean up on drop.
   * Tests **MUST NOT** depend on execution order or global mutable state. Because `cargo test` runs test functions in parallel across multiple worker threads by default, tests **MUST** be thread-safe and re-entrant.
2. **No Flaky Wall-Clock Sleeps:** Tests **MUST NOT** use arbitrary delays like `std::thread::sleep()` to await asynchronous conditions. Asynchronous workflows (such as in `solix-lsp`) **MUST** coordinate via channels (`tokio::sync::mpsc`), promises, or bounded event barriers.
3. **Zero Unjustified Ignored Tests:** Pull requests **MUST NOT** disable failing tests using `#[ignore]` without an accompanying link to an open `P0` or `P1` issue documenting the blocker.

---

## 3. Structural Specification & Schema

### 3.1 Three-Tier Testing Hierarchy Schema

```text
crates/solix-compilation/
├── src/
│   ├── lexer.rs                    # Production code
│   │   └── #[cfg(test)] mod tests  # Tier 1: Small unit tests for lexer internals
│   ├── parser.rs                   # Production code
│   │   └── #[cfg(test)] mod tests; # Tier 1: Child module declaration
│   └── parser/
│       └── tests.rs                # Tier 1: Child module with 100% private access
└── tests/                          # Integration & Fixture Tests
    ├── test_fixtures.rs            # Tier 2: Automated test runner for .slx files
    └── fixtures/                   # Tier 2: The bulk of language test programs
        ├── syntax/
        │   ├── positive/           # Valid language syntax programs
        │   │   ├── while_basic.slx
        │   │   └── class_inheritance.slx
        │   └── negative/           # Invalid syntax asserting compiler rejection
        │       ├── while_missing_cond.slx
        │       └── unclosed_delimiter.slx
        └── types/
            ├── positive/           # Conforming type assignments
            │   └── numeric_widening.slx
            └── negative/           # Hostile type mismatches
                ├── assign_str_to_i32.slx
                └── cyclic_inheritance.slx
```

### 3.2 Dual Test Matrix Schema

| Test Domain | Positive Verification Vectors | Negative Verification Vectors |
| :--- | :--- | :--- |
| **Lexer Tokenization** | * Valid keywords and identifiers<br>* String, char, and numeric literals<br>* Composite operators (`==`, `!=`, `<=`) | * Unterminated string literal (`E0010`)<br>* Invalid unicode escape (`E0012`)<br>* Unrecognized byte (`E0001`) |
| **Parser & AST** | * Canonical statement grammar<br>* Operator precedence resolution<br>* Nested blocks and expressions | * Missing delimiter `}` (`E0101`)<br>* Missing semicolon (`E0105`)<br>* Invalid statement header (`E0110`) |
| **Semantic Binder & Types** | * Correct type inference<br>* Valid variable scoping and shadowing<br>* Valid method signature matching | * Type mismatch in assignment (`E0201`)<br>* Undeclared identifier (`E0205`)<br>* Cyclic class hierarchy (`E0220`) |
| **VM Opcode Execution** | * Arithmetic and bitwise evaluation<br>* Successful method dispatch<br>* ARC reference count increment/decrement | * Division by zero (`Trap::DivisionByZero`)<br>* Stack underflow (`Trap::StackUnderflow`)<br>* Null pointer dereference (`Trap::NullPointer`) |
| **CLI Arguments (`clap`)** | * Valid flags and file paths<br>* Output path resolution<br>* Verbose logging flags | * Missing required input file (`ExitCode 1`)<br>* Unknown or misspelled option (`ExitCode 1`)<br>* Non-existent source file (`ExitCode 1`) |

---

## 4. Production-Ready Templates

### 4.1 Tier 1: Unit Test Template (Accessing Private Internals)

```rust
// In crates/solix-compilation/src/cursor.rs

pub struct SourceCursor<'source> {
    buffer: &'source [u8],
    byte_offset: usize, // private field
}

impl<'source> SourceCursor<'source> {
    fn peek_next_byte(&self) -> Option<u8> { // private method
        self.buffer.get(self.byte_offset).copied()
    }
}

// ============================================================================
// Tier 1 Unit Tests (At bottom of file - Full access to private members)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_peek_returns_correct_byte() {
        let cursor = SourceCursor {
            buffer: b"let",
            byte_offset: 0, // Direct private field initialization permitted
        };
        assert_eq!(cursor.peek_next_byte(), Some(b'l'));
    }

    #[test]
    fn test_negative_peek_beyond_buffer_returns_none() {
        let cursor = SourceCursor {
            buffer: b"",
            byte_offset: 0,
        };
        assert_eq!(cursor.peek_next_byte(), None);
    }
}
```

### 4.2 Tier 2: Automated Language Fixture Runner (`tests/test_fixtures.rs`)

```rust
use std::fs;
use std::path::{Path, PathBuf};
use solix_compiler::compile_source;

struct LanguageFixture {
    path: PathBuf,
    source_content: String,
    expected_error_code: Option<String>,
}

fn load_fixture(path: &Path) -> LanguageFixture {
    let source_content = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("Failed to read fixture '{:?}': {}", path, error));

    // Negative fixtures declare expected error in header: // EXPECT_ERROR: E0102
    let expected_error_code = source_content.lines()
        .find(|line| line.starts_with("// EXPECT_ERROR:"))
        .map(|line| line.replace("// EXPECT_ERROR:", "").trim().to_string());

    LanguageFixture {
        path: path.to_path_buf(),
        source_content,
        expected_error_code,
    }
}

#[test]
fn test_all_positive_syntax_fixtures_compile_cleanly() {
    let fixtures_dir = Path::new("tests/fixtures/syntax/positive");
    if !fixtures_dir.exists() { return; }

    for entry in fs::read_dir(fixtures_dir).expect("Read fixtures dir") {
        let path = entry.expect("Valid entry").path();
        if path.extension().is_some_and(|ext| ext == "slx") {
            let fixture = load_fixture(&path);
            let result = compile_source(&fixture.source_content);
            assert!(
                result.is_ok(),
                "Positive fixture '{:?}' failed compilation: {:?}",
                fixture.path,
                result.err()
            );
        }
    }
}

#[test]
fn test_all_negative_syntax_fixtures_emit_expected_diagnostics() {
    let fixtures_dir = Path::new("tests/fixtures/syntax/negative");
    if !fixtures_dir.exists() { return; }

    for entry in fs::read_dir(fixtures_dir).expect("Read fixtures dir") {
        let path = entry.expect("Valid entry").path();
        if path.extension().is_some_and(|ext| ext == "slx") {
            let fixture = load_fixture(&path);
            let result = compile_source(&fixture.source_content);

            assert!(
                result.is_err(),
                "Negative fixture '{:?}' was expected to fail, but compiled successfully",
                fixture.path
            );

            let diagnostic = result.unwrap_err();
            if let Some(expected_code) = &fixture.expected_error_code {
                assert_eq!(
                    diagnostic.code(),
                    expected_code,
                    "Fixture '{:?}' failed with wrong error code: expected {}, got {}",
                    fixture.path,
                    expected_code,
                    diagnostic.code()
                );
            }
        }
    }
}
```

---

## 5. Concrete Production Examples

### Example 1: Negative Language Fixture (`tests/fixtures/types/negative/assign_string_to_i32.slx`)

```solix
// EXPECT_ERROR: E0201

int32 main() {
    int32 count = "invalid string value";
    return count;
}
```

### Example 2: Negative Syntax Fixture (`tests/fixtures/syntax/negative/while_missing_condition.slx`)

```solix
// EXPECT_ERROR: E0110

void run() {
    while {
        // Syntax error: while requires conditional guard expression
    }
}
```

### Example 3: Tier 3 CLI Integration Test with Hermetic Isolation

```rust
use std::fs;
use tempfile::tempdir;
use solix_client::commands::compile::{execute_compile, CompileOptions};

#[test]
fn test_cli_compile_generates_bytecode_container() {
    let temp_workspace = tempdir().expect("Failed to create temporary directory");
    let input_source = temp_workspace.path().join("main.slx");
    let output_binary = temp_workspace.path().join("out.slxbin");

    fs::write(&input_source, "int32 main() { return 42; }")
        .expect("Failed to write fixture source");

    let options = CompileOptions {
        input_file: input_source,
        output_file: output_binary.clone(),
        optimization_level: 1,
        emit_asm: true,
    };

    let result = execute_compile(options);
    assert!(result.is_ok(), "CLI compilation failed: {:?}", result.err());
    assert!(output_binary.exists(), "Bytecode file out.slxbin was not created");
    assert!(temp_workspace.path().join("out.slxasm").exists(), "Assembly output was not created");
}
```

---

## 6. Anti-Patterns & Counter-Examples (What NOT to Do)

### Anti-Pattern 1: Monolithic Test Bloat in Production Files
* **Flawed Practice:** Pasting 200 different Solix program strings and assertions inside `parser.rs` or `type_checker.rs`.
* **Failure Mode:** Implementation files become 15,000 lines long, making code reviews exhausting and hiding production bugs behind walls of repetitive test boilerplate.
* **Corrected Approach:** Keep only small internal unit tests in the file. Store language programs as individual `.slx` files in `tests/fixtures/`.

### Anti-Pattern 2: Shared Host Disk Pollution in Tests
```rust
// FLAWED:
#[test]
fn test_compile() {
    let output_file = std::path::Path::new("/tmp/test_build.slxbin");
    compile_to(output_file);
    assert!(output_file.exists());
}
```
* **Failure Mode:** Running parallel test jobs (`cargo test -- --test-threads=8`) causes tests to overwrite each other's files, producing intermittent test failures and permissions defects on multi-user systems.
* **Corrected Approach:** Use RAII temporary directories: `let temp_dir = tempfile::tempdir().unwrap();`.

### Anti-Pattern 3: Happy-Path Exclusivity
* **Flawed Practice:** Testing only valid Solix programs, completely ignoring syntax errors, type mismatches, and runtime arithmetic traps.
* **Failure Mode:** A user writes code with a typo, and the compiler crashes with an unhandled Rust panic (`index out of bounds` or `.unwrap() on None`) instead of printing a clean `miette` diagnostic.
* **Corrected Approach:** Implement dual test coverage: every syntax construct and opcode must have dedicated negative test fixtures asserting graceful failure and diagnostic code match.

---

## 7. Verification & Compliance Checklist

Reviewers and automated CI gates **MUST** enforce the following criteria prior to merging:

- [ ] **Tier Separation:** Implementation files contain only concise internal unit tests; language test programs reside in `tests/fixtures/`.
- [ ] **Private Access Integrity:** Tests requiring private internal access use `#[cfg(test)] mod tests` or child module `tests.rs`.
- [ ] **Dual Coverage:** Both positive (valid) and negative (hostile/invalid) test cases are implemented.
- [ ] **Diagnostic Code Assertions:** Negative tests assert exact error codes (`E0XXX`) rather than mere boolean failure.
- [ ] **Hermetic Isolation:** No tests read or write to shared static paths; all disk I/O utilizes `tempfile::tempdir()`.
- [ ] **No Flaky Sleeps:** Asynchronous tests coordinate using channels or notifications, not `thread::sleep`.
- [ ] **Zero Unjustified `#[ignore]`:** No tests are disabled without a documented tracking issue.
- [ ] **Full Pass:** All workspace tests pass under `cargo test --workspace --all-targets`.
