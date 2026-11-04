# Living Documentation & Drift Prevention Specification

## Document Metadata
* **Guideline ID:** GL-0010
* **Scope:** Universal Documentation (`docs/`, `crates/`)
* **Status:** Final Implementation Standard
* **Audience:** Core Systems Engineers, Compiler Architects, Documentation Stewards, Language Contributors
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)

---

## 1. Executive Summary & Purpose

In a modern programming language and compiler project, documentation is both the public specification contract and the user interface. When language documentation diverges from compiler and runtime reality, software engineers construct invalid mental models, write programs that fail type checking unexpectedly, execute incorrect build procedures, and misdiagnose runtime traps.

In the Solix language ecosystem, where an AST compiler (`solix-compilation`) lowers high-level language constructs to a deterministic bytecode format executed by a virtual machine (`solix-runtime`), obsolete or drifting documentation directly produces compiler regressions, broken developer tooling, and corrupted language semantics.

This specification establishes the **Living Documentation Invariant** and the **Zero-Drift Mandate** for Solix. Documentation is defined as a first-class engineering artifact possessing the same rigor, versioning, review process, and automated verification as executable Rust code. This standard formalizes non-negotiable rules for synchronizing code and documentation, compiling and verifying all language code examples, auditing CLI parameter accuracy against `clap` definitions, validating Rustdoc API coverage, and methodically archiving deprecated knowledge.

---

## 2. Normative Invariants & Core Principles

All contributors and documentation authors **MUST** adhere to the following binding rules:

1. **The Zero-Drift Mandate:** Code and documentation **MUST** be authored, updated, and committed within the exact same Git commit or pull request. Any PR that alters grammar syntax, AST nodes, bytecode opcodes, VM dispatch behavior, or CLI flags without updating the corresponding specifications and guides **SHALL NOT** be approved or merged.
2. **Compiler-Verified Language Snippets:** Every Solix (`.slx`) code snippet, grammar production, or compilation example in `docs/guides/` and `docs/wikipedia/` **MUST** be directly validated against the Solix compiler via automated test fixtures or integration tests. Hand-typed, uncompiled code snippets in documentation are strictly prohibited.
3. **Comprehensive Rustdoc API Documentation:** All public modules, structs, enums, traits, functions, and type aliases across all workspace crates (`crates/*`) **MUST** be documented using Rustdoc (`///`). Public API items **MUST** include `# Examples`, `# Errors`, and `# Panics` sections where applicable. All code examples in Rustdoc **MUST** compile cleanly under `cargo test --doc`.
4. **CLI Flag Synchronicity with `clap`:** Every documented CLI command, subcommand, flag, argument, and default value in `docs/internals/specs/cli/` or user guides **MUST** match the actual `clap` derive definitions in `crates/solix-client/src/` byte-for-byte.
5. **Diagnostic Code Synchronicity with `miette`:** Every documented compiler error code, diagnostic category, and suggested fix in documentation **MUST** match the structured diagnostic definitions in `solix-common` and `solix-compilation`.
6. **No Speculative or Unimplemented Features:** User guides (`docs/guides/`) and language wiki (`docs/wikipedia/`) **MUST NOT** describe future, planned, or unmerged language syntax as if currently functional. Features under active proposal or development **MUST** be confined to `docs/internals/specs/` or `docs/internals/milestones/` and prominently flagged with a `> [!WARNING] Experimental / Unimplemented` alert.
7. **Methodical Archival of Superseded Knowledge:** Obsolete language syntax, retired opcodes, and superseded architectural RFCs **MUST NOT** linger in active documentation directories. Superseded documents **MUST** be moved to `docs/archive/` with an authoritative deprecation header linking to the current standard.

---

## 3. Structural Specification & Schema

### 3.1 Documentation Directory Taxonomy

The `docs/` repository hierarchy is strictly partitioned by scope and target audience:

```text
docs/
├── overview/               # High-level architecture, compiler pipeline, VM design, memory model
│   ├── architecture.md     # System overview and crate dependency topology
│   └── memory_model.md     # Automatic Reference Counting (ARC) and memory layout
├── guides/                 # User-facing language tutorials and developer operational guides
│   ├── 01_getting_started.md
│   ├── 02_variables_and_types.md
│   ├── 03_control_flow.md
│   ├── 04_classes_and_oop.md
│   ├── 05_memory_management_and_arc.md
│   └── 06_error_handling.md
├── wikipedia/              # Formal encyclopedia of language keywords, statements, and expressions
│   ├── keywords/           # Individual keyword specifications (e.g., class, while, match)
│   ├── statements/         # Statement syntax, semantics, and AST representations
│   └── expressions/        # Expression syntax, evaluation order, and precedence
├── internals/              # Core engineering standards and architectural blueprints
│   ├── guidelines/         # Engineering guidelines suite (GL-0001 - GL-0011)
│   ├── milestones/         # Phased delivery plans, blocker tracking, and DoD sign-offs
│   └── specs/              # Subsystem specifications (statements, VM ISA, runtime, CLI)
└── archive/                # Historical, deprecated, or superseded specifications
```

### 3.2 Fenced Snippet Verification Annotations

When embedding Solix or Rust code snippets in documentation, authors **MUST** use explicit fenced code blocks with language identifiers (`solix`, `rust`, `bash`). Where snippets correspond to verified test cases or source files, authors **MUST** annotate the snippet with a clickable file link to the verifying test fixture:

````markdown
```solix
// Verified via crates/solix-compilation/tests/fixtures/control_flow/while_basic.slx
int32 counter = 0;
while counter < 10 {
    counter = counter + 1;
}
```
*Verifying Test:* [`tests/fixtures/control_flow/while_basic.slx`](file:///home/jehud/Projects/solix-language/crates/solix-compilation/tests/fixtures/control_flow/while_basic.slx)
````

### 3.3 Archival Header Specification

When moving superseded documents to `docs/archive/`, authors **MUST** prepend the following canonical deprecation alert:

```markdown
> [!CAUTION] ARCHIVED DOCUMENT
> This document was officially superseded on **YYYY-MM-DD**. It is preserved exclusively 
> for historical reference and compiler archaeology. It no longer represents active language semantics.
> **Current Authoritative Standard:** [Current Specification Name](file:///path/to/current.md)
```

---

## 4. Production-Ready Template

Engineers authoring or updating language guides (`docs/guides/`) **MUST** use the following canonical structure:

```markdown
# [Guide Title: Language Feature or Subsystem Name]

## Document Metadata
* **Document Path:** `docs/guides/[XX-name].md`
* **Target Audience:** Solix Developers / Language Contributors
* **Target Version:** `v[X.Y.Z]`
* **Last Verified Commit:** `git rev-parse HEAD`
* **Status:** Current Production Guide

---

## 1. Overview & Conceptual Model
<!-- High-level explanation of the language feature, rationale, and runtime semantics. -->
[Provide conceptual overview here]

## 2. Syntax & Grammar Formalism
<!-- Concise BNF grammar and syntax definition. -->
```bnf
[Insert Backus-Naur Form grammar here]
```

## 3. Concrete Language Examples
<!-- Fully compiling Solix code snippets with verified test references. -->
```solix
// Verified via crates/solix-compilation/tests/fixtures/...
[Insert verified Solix code here]
```

## 4. Bytecode Lowering & VM Semantics
<!-- Explanation of how this feature lowers to bytecode opcodes and behaves in the VM. -->
| Emitted Opcode | Operand Layout | Stack Effect | Description |
| :--- | :--- | :--- | :--- |
| `OP_EXAMPLE` | `[u8; 2]` | `[val] -> []` | [Opcode behavior description] |

## 5. Diagnostic Codes & Common Errors
<!-- Realistic compiler error messages, root causes, and remediation steps. -->
| Diagnostic Code | Diagnostic Message | Root Cause | Remediation Procedure |
| :--- | :--- | :--- | :--- |
| `E0102` | `type mismatch in condition` | Expression does not evaluate to `bool` | Cast or supply boolean expression |

## 6. Verification & Automated Test Linkage
<!-- Reference to the automated tests ensuring this documentation does not drift. -->
Verified via automated test suite `crates/solix-compilation/tests/syntax_fixtures_tests.rs`.
```

---

## 5. Concrete Production Example

### Verifying CLI Documentation Against `clap` Source Code

Below is an excerpt demonstrating how the `solix compile` CLI command is authoritatively documented in `docs/internals/specs/cli/compile.md`:

```markdown
### Command: `solix compile`

The `compile` subcommand invokes `solix-compilation` to parse, type-check, and assemble Solix source files into an executable `.slxbin` bytecode container.

#### Syntax & Options

```bash
solix compile <input_file> [-o <output_file>] [--optimization-level <level>] [--emit-asm]
```

| Flag / Option | Type | Default | `clap` Source Reference | Description |
| :--- | :--- | :--- | :--- | :--- |
| `<input_file>` | `PathBuf` | *(Required)* | `crates/solix-client/src/commands/compile.rs#L18` | Path to entrypoint Solix source file (`.slx`). |
| `-o, --output` | `PathBuf` | `out.slxbin` | `crates/solix-client/src/commands/compile.rs#L22` | Path to target bytecode binary output file. |
| `-O, --opt-level`| `u8` | `1` | `crates/solix-client/src/commands/compile.rs#L26` | Bytecode optimization tier: `0` (none), `1` (standard), `2` (aggressive). |
| `--emit-asm` | `bool` | `false` | `crates/solix-client/src/commands/compile.rs#L30` | Emit human-readable `.slxasm` disassembly alongside binary. |
| `--verbose, -v`| `bool` | `false` | `crates/solix-client/src/commands/compile.rs#L34` | Enable verbose compilation logs and timing metrics. |
```

#### Automated CLI Verification Test (`crates/solix-client/tests/test_cli_docs.rs`)
```rust
// Verified via crates/solix-client/tests/test_cli_docs.rs
#[test]
fn verify_compile_cli_options_match_documentation() {
    use clap::CommandFactory;
    use solix_client::cli::CommandLineArguments;

    let command_app = CommandLineArguments::command();
    let compile_subcommand = command_app.find_subcommand("compile")
        .expect("Subcommand 'compile' must exist in clap schema");

    assert!(compile_subcommand.get_arguments().any(|arg| arg.get_id() == "output"));
    assert!(compile_subcommand.get_arguments().any(|arg| arg.get_id() == "opt_level"));
    assert!(compile_subcommand.get_arguments().any(|arg| arg.get_id() == "emit_asm"));
}
```

---

## 6. Anti-Patterns & Counter-Examples (What NOT to Do)

### Anti-Pattern 1: The "I'll Document It Later" Fallacy
* **Flawed Practice:** An engineer adds a new `--emit-ast` flag to `solix-client` without updating `docs/internals/specs/cli/compile.md`, stating: "I'll open a separate docs PR later."
* **Failure Mode:** The separate PR is never opened. Tooling authors and language developers have no record of the flag. When users query CLI capabilities, documentation contradicts binary `--help` output, damaging project credibility.
* **Corrected Approach:** Reject the PR immediately at review time. Code changes, unit tests, and documentation updates **MUST** be submitted in the same PR.

### Anti-Pattern 2: Aspirational or Phantom Language Syntax
* **Flawed Practice:** Authoring a tutorial in `docs/guides/04_classes_and_oop.md` claiming that Solix supports multiple inheritance or traits before the compiler has implemented the syntax or AST validation.
* **Failure Mode:** Users attempt the documented syntax, encounter compiler syntax errors, and file bug reports. Engineering time is wasted resolving user confusion caused by fictional documentation.
* **Corrected Approach:** Only document features validated by green automated tests in `master`. For planned syntax, open a formal proposal in `docs/internals/specs/` tagged `[!WARNING] Unimplemented Proposal`.

### Anti-Pattern 3: Ghost Documentation & Stale Opcode Definitions
* **Flawed Practice:** When refactoring the VM to replace `OP_ADD_INT` with typed opcodes `OP_ADD_I32` and `OP_ADD_I64`, leaving the old opcode in `docs/internals/specs/vm_isa.md`.
* **Failure Mode:** Developers implementing the debugger (`solix-dbg`) or disassembler read the outdated specification and emit disassembler logic for non-existent bytecode opcodes, causing runtime decoder panics.
* **Corrected Approach:** Update `vm_isa.md` in the exact same commit that changes the Rust `enum OpCode` in `solix-common`. Move obsolete opcodes to the archival changelog table.

---

## 7. Verification & Compliance Checklist

Reviewers and documentation maintainers **MUST** enforce the following checklist on every PR modifying or adding documentation:

- [ ] **Co-Commit Compliance:** All code changes altering language grammar, opcodes, or CLI options are accompanied by documentation changes in the exact same PR.
- [ ] **Snippet Validation:** All Solix code examples compile and run cleanly against current compiler test fixtures.
- [ ] **Rustdoc Coverage:** All new or updated public items in Rust crates contain `///` doc comments with `# Examples` passing `cargo test --doc`.
- [ ] **Clap Option Synchronization:** CLI option names, short aliases, types, and defaults match `clap` source definitions byte-for-byte.
- [ ] **Diagnostic Code Match:** Error codes and messages match `miette` definitions in compiler crates.
- [ ] **No Aspirational Claims:** Unimplemented syntax is strictly excluded from active guides and wiki pages.
- [ ] **Archival Discipline:** Deprecated specifications are moved to `docs/archive/` with the mandatory `[!CAUTION]` alert banner.
