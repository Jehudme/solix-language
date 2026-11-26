# Tripartite Documentation Architecture Specification: Guides, Specs & Atomic Wiki

## Document Metadata

* **Guideline ID:** GL-0012
* **Scope:** Universal Documentation Architecture (`docs/guide/`, `docs/spec/`, `docs/wiki/`)
* **Status:** Final Implementation Standard
* **Audience:** Core Systems Engineers, Compiler Architects, Documentation Authors, Language Contributors
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)

---

## 1. Executive Summary & Architectural Purpose

In the legacy Solix v1.x documentation, content suffered from severe structural entanglement: user tutorials explained low-level compiler memory layouts, language specifications included introductory syntax guides, and error code descriptions were scattered across unrelated documents. This lack of role boundaries caused massive duplication, maintenance fatigue, and rapid documentation drift.

This guideline formalizes the **Tripartite Documentation Architecture** for Solix v3:
1. **User Guides (`docs/guide/`) — "How to Use It":** Practical, task-oriented learning paths designed for software engineers writing Solix applications. Guides deliberately avoid deep VM internals and instead link directly to specifications and wiki articles.
2. **Formal Specifications (`docs/spec/`) — "How It Works Under the Hood":** Normative, authoritative engineering contracts detailing Rust compilation passes, 64-bit Register VM instructions, memory block headers, and binary container formats.
3. **Atomic Reference Wiki (`docs/wiki/`) — "Fast Dictionary Reference":** Granular, self-contained cheat sheets and lookup dictionaries for individual error codes, keywords, primitives, and CLI subcommands.

---

## 2. Normative Invariants & Core Principles

All documentation authors **MUST** adhere to the following normative invariants:

1. **Strict Role Boundary Invariant:**
   * A document in `docs/guide/` **MUST NOT** define low-level VM opcodes, ABI alignments, or raw binary container byte offsets. It **MUST** link to the authoritative specification in `docs/spec/`.
   * A document in `docs/spec/` **MUST NOT** serve as an introductory tutorial. It **MUST** define normative syntax, formal typing rules, AST representations, and VM lowering semantics.
   * A document in `docs/wiki/` **MUST** remain strictly atomic, documenting exactly one error code, keyword, primitive, or command with its minimal reproducible code example.
2. **Cross-Link Density Invariant:**
   * Every user guide **MUST** link to the corresponding formal specification for deep dive exploration and to the atomic wiki for keyword and primitive syntax.
   * Every wiki error code page **MUST** link to the specification pass that emitted the diagnostic (e.g., `E0105` links to `docs/spec/statements/declarations.md`).
3. **Zero Legacy Artifact Rule:**
   * Documentation **MUST NEVER** reference obsolete Solix v1.x concepts (`solix.json`, `.slxbin`, stack VM opcodes `PUSH_CONST`, dot-access for static members `Math.add`, or generic error tags `[E_BIND]`).
   * All documents **MUST** reflect modern Solix v3 reality: register-based 64-bit VM, scope resolution `::`, `manifest.yaml` (legacy `manifest.toml`), `.slxb` / `.slxp`, and granular error codes (`E0000`–`E0405`, `E9001`, `R0000`–`R0027`).
4. **Code Verification Mandate & Single-Letter Variable Prohibition (GL-0011):**
   * All Solix and Rust code snippets across guides, specs, and wiki articles **MUST** be valid, compilable code.
   * Single-letter variable names (`x`, `y`, `i`, `n`, `s`, `v`, `e`, etc.) are **STRICTLY FORBIDDEN** across all documentation and examples. Use descriptive, self-documenting identifiers (e.g., `coord_x`, `coord_y`, `index`, `element`, `item_count`, `error_value`).

---

## 3. Concrete Authoring Templates

Authors **MUST** copy and adapt these standard markdown templates when creating or updating documentation.

---

### 3.1 Wiki Keyword Template (`docs/wiki/keywords/<category>/<keyword>.md`)

Every keyword article **MUST** follow this exact 6-section structure:

````markdown
# Keyword: `<keyword_name>`

| Field | Detail |
| :--- | :--- |
| **Keyword** | `<keyword_name>` |
| **Category** | Control Flow / Declaration / Expression / Modifier |
| **Introduced** | Solix v3.x |
| **Related** | [`<related_keyword>`](<link_to_related>) |

---

## 1. Overview
[Brief 1-2 paragraph conceptual explanation of what the keyword represents, its purpose, and scope of use in Solix.]

---

## 2. Syntax & Grammar
[Formal syntactic production or EBNF snippet indicating where the keyword appears.]

```solix
<syntactic_form>
```

---

## 3. Formal Compiler Invariants
* **Parsing Invariant:** [Restrictions on where this keyword may appear]
* **Binding & Type Invariant:** [Type checking rules enforced by BinderPasses]
* **Scope Invariant:** [Visibility, lifetime, or shadowing constraints]

---

## 4. Correct Usage Example

```solix
class ExampleClass {
    public static int32 compute_total(int32 base_value, int32 multiplier_factor) {
        // Descriptive, multi-letter identifiers only (GL-0011)
        int32 total_result = base_value * multiplier_factor;
        return total_result;
    }
}
```

---

## 5. Invalid Usage & Diagnostic Errors

### Anti-Pattern: [Brief description of mistake]
```solix
// [Demonstrate incorrect code with error annotation]
```
* **Diagnostic Code:** [`E0XXX`](../../wiki/errors/binder-errors.md#e0xxx)
* **Compiler Remediation:** [Explain how to fix the error]

---

## 6. Related Specifications & Features
* Formal Spec: [`docs/spec/statements/<spec>.md`](../../spec/statements/<spec>.md)
* User Guide: [`docs/guide/<guide>.md`](../../guide/<guide>.md)
````

---

### 3.2 Wiki Diagnostic Error Template (`docs/wiki/errors/<category>-errors.md`)

Error codes in wiki error dictionaries **MUST** follow this standard schema:

````markdown
### `<EXXXX>`: `<Error Title>`

* **Diagnostic Code:** `E0XXX` (Compile-Time) or `R00XX` (Runtime)
* **Subsystem:** Lexer / Parser / Binder / Emitter / VM Runtime
* **Severity:** Fatal Error / Warning / Recoverable

#### Root Cause & Compiler Invariant
[Explain the exact semantic invariant that was violated and why the compiler rejected the construct.]

#### Erroneous Code Example
```solix
// ❌ FAILS with E0XXX: <error message>
class BadSample {
    public static void execute() {
        int32 count_value = "invalid_string_literal";
    }
}
```

#### Corrected Code Example
```solix
// ✅ PASSES: Valid typed initialization
class GoodSample {
    public static void execute() {
        int32 count_value = 42;
    }
}
```

#### Related Diagnostics & Formal Specification
* Related: [`E0YYY`](#e0yyy)
* Formal Specification: [`docs/spec/<spec_path>.md`](../../spec/<spec_path>.md)
````

---

### 3.3 Wiki Primitive Template (`docs/wiki/primitives/<primitive>.md`)

Every primitive type reference page **MUST** follow this structure:

````markdown
# Primitive Type: `<type_name>`

| Property | Value |
| :--- | :--- |
| **Type Identifier** | `<type_name>` |
| **Classification** | Signed Integer / Unsigned Integer / Floating Point / Boolean / Character |
| **Storage Width** | N bits (N bytes) |
| **Register Storage** | `u64` raw bit pattern in 64-bit VM register |
| **Value Range** | `<min_value>` to `<max_value>` |
| **Default Value** | `<default_literal>` |

---

## 1. Overview
[Detailed description of the primitive, IEEE 754 compliance, two's complement representation, or boolean logic role.]

---

## 2. Literal Syntax & Formatting
```solix
<type_name> decimal_value = 100;
<type_name> hex_value = 0x64;
```

---

## 3. Allowed Type Conversions
| Target Type | Conversion Mode | Semantics |
| :--- | :--- | :--- |
| `<wider_type>` | Implicit / Explicit | Value-preserving widening |
| `<narrower_type>` | Explicit Cast Required | Bit truncation |

---

## 4. Usage Example

```solix
class PrimitiveExample {
    public static <type_name> process_sample(<type_name> input_value) {
        return input_value;
    }
}
```

---

## 5. Related Specifications
* Formal Spec: [`docs/spec/types.md`](../../spec/types.md)
````

---

### 3.4 Wiki CLI Command Template (`docs/wiki/commands/<category>.md`)

Command reference entries **MUST** document commands using this format:

````markdown
## `solix <subcommand>`

[High-level summary of what the subcommand does.]

### Synopsis
```bash
solix <subcommand> [arguments...] [options]
```

### Positional Arguments
| Argument | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `<arg>` | Path / String | Yes / No | [Description] |

### Key Options & Flags
* `-o, --output <path>`: [Description and default]
* `--profile <mode>`: [Description and default]

### Practical Examples
```bash
# Common invocation 1
solix <subcommand> target

# Common invocation 2 with flags
solix <subcommand> target --flag
```

### Related Specifications
* Formal Spec: [`docs/spec/tools/cli/<subcommand>.md`](../../spec/tools/cli/<subcommand>.md)
````

---

### 3.5 Formal Language Specification Template (`docs/spec/<subsystem>/<feature>.md`)

Authoritative specification documents **MUST** follow this comprehensive engineering schema:

````markdown
# `<Feature Name>` — Formal Language Specification

## Document Metadata
* **Specification ID:** SPEC-XXXX
* **Scope:** Language Grammar, Type Binder, Codegen Lowering, VM ISA
* **Status:** Normative Standard

---

## 1. Syntax & EBNF Grammar

```ebnf
ProductionRule ::= Keyword Identifier ( ":" Type )? ";" ;
```

---

## 2. AST Representation (`solix-compilation`)

```rust
// In crates/solix-compilation/src/ast/nodes.rs
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureNode {
    pub span: TextSpan,
    pub identifier: SymbolIndex,
    pub target_type: Option<TypeSignature>,
}
```

---

## 3. Semantic Rules & Compiler Invariants
1. **Pass 1 (Symbol Registration):** [Invariant]
2. **Pass 2 (Hierarchy & Type Resolution):** [Invariant]
3. **Pass 3 (Body Type-Checking):** [Invariant]

---

## 4. Bytecode Lowering & VM ISA (64-Bit Register VM)

The construct lowers into the following register instructions:

```text
LoadConstImm64  r0, <constant_index>
AddI64          r2, r0, r1
ReturnVal       r2
```

---

## 5. Memory Model & ARC Invariants
* **Header Allocation:** [Object header layout]
* **Retain/Release Pairs:** [ARC increments and decrements]
* **Destructor Invariants:** [Deterministic cleanup on scope exit]

---

## 6. Verification Test Cases
* Positive Conformance: `crates/solix-compilation/tests/features/<feature>_tests.rs`
* Diagnostic Verification: `crates/solix-compilation/tests/diagnostics/e0xxx_tests.rs`
````

---

### 3.6 User Guide Template (`docs/guide/<guide_name>.md`)

User guides **MUST** provide a tutorial learning path:

````markdown
# <Guide Title>

[Introductory paragraph explaining what the reader will learn and build.]

---

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Core Concepts](#core-concepts)
3. [Step-by-Step Walkthrough](#step-by-step-walkthrough)
4. [Common Pitfalls & Best Practices](#common-pitfalls--best-practices)
5. [Summary & Next Steps](#summary--next-steps)

---

## Prerequisites
| Prerequisite | Minimum Version | Notes |
| :--- | :--- | :--- |
| Solix CLI | 3.0+ | `solix --version` |

---

## Core Concepts
[Conceptual explanation with diagram or architecture overview.]

---

## Step-by-Step Walkthrough
[Hands-on instructions with verified, compilable Solix snippets.]

```solix
class GuideExample {
    public static int32 main() {
        return 0;
    }
}
```

---

## Common Pitfalls & Best Practices
* **Pitfall:** [Description of common beginner mistake]
  * **Solution:** [How to avoid or fix it]

---

## Summary & Next Steps
* [Next Guide Link](<path_to_next_guide>)
* [Deep Dive Specification Link](<path_to_spec>)
````

---

## 4. Authoring Workflow for New Language Features

When adding or modifying a language feature:
1. **Author the Formal Spec First (`docs/spec/`):** Define the grammar, AST lowering, and bytecode emission.
2. **Author Wiki Reference Pages (`docs/wiki/`):** Document all new keywords in `docs/wiki/keywords/`, primitives in `docs/wiki/primitives/`, and any new diagnostic error codes in `docs/wiki/errors/`.
3. **Update or Author User Guides (`docs/guide/`):** Integrate practical examples into the relevant guide, cross-linking to the new spec and wiki entries.
4. **Audit per GL-0013:** Run the pre-merge Git-diff audit to verify zero documentation drift.
