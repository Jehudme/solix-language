# Compiler Diagnostics & Error Reporting Specification

## 1. Overview & Purpose

The Solix compiler enforces uniform and deterministic error reporting across all compilation phases:
1. **Lexical Analysis (`Lexer`)**: Tokenization errors, unterminated literals, and unrecognized characters.
2. **Syntax Analysis (`Parser`)**: Grammar productions, missing tokens, and malformed statements.
3. **Semantic Analysis (`Binder`)**: Type checking, scope resolution, duplicate symbols, and inheritance contracts.
4. **Bytecode Emission (`Emitter`)**: Frame layout, vtable calculation, and entry point resolution.

---

## 2. Invariant Diagnostic Format

Every diagnostic message produced by the compiler follows the canonical format:

```text
[<file_path>:<line>:<column>] <Severity>: <Message>
```

When rendered to terminal / stdout via `Diagnostic::print_reports`, the diagnostic displays the source line and caret indicator (`^`) pointing to the offending character:

```text
/path/to/source.slx:4:17: error: [E0101]: Expected expression, got ';'
    int32 coord_x = ;
              ^
```

### Report Attributes

Each diagnostic report records the following properties:
- `source_path`: The file path or source identifier (never empty; defaults to `<unknown>` only when no source context exists).
- `line`: 1-indexed row number where the error was encountered.
- `column`: 1-indexed column number pointing to the beginning of the offending token or character.
- `code`: Structured diagnostic code (`E0001`–`E0099` Lexer, `E0100`–`E0199` Parser, `E0200`–`E0299` Binder, `E0300`–`E0399` Emission, `E0400`–`E0499` Manifest).
- `severity`: Severity level (`ERROR`, `WARNING`, `NOTE`).
- `message`: Clear, human-readable description of the error.

---

## 3. Log Levels & Pipeline Instrumentation

### 3.1 Logging Level Hierarchy
The compiler categorizes informational logs according to verbosity:
- **`INFO`**: High-level milestones (e.g. `Compiling N source file(s)...`, `Compilation completed in X ms`).
- **`DEBUG`**: Stage completion and duration summaries (e.g. `Lexer completed in X ms.`, `Parser completed in X ms.`, `Binder completed in X ms.`, `Assembler completed in X ms.`).
- **`TRACE`**: Detailed AST nodes, symbol definitions, template cloning, and local frame index allocations. Fine-grained compiler diagnostics are kept at `TRACE` to keep `DEBUG` logging uncluttered.

