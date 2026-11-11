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

This guideline formalizes the **Tripartite Documentation Architecture** for Solix v2.0:
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
   * All documents **MUST** reflect modern Solix v2.0 reality: register-based 64-bit VM, scope resolution `::`, `manifest.toml`, `.slxb` / `.slxarc`, and granular error codes (`E0000`–`E0290`, `R0001`–`R0023`).
4. **Code Verification Mandate:**
   * All Solix code snippets across guides and wiki articles **MUST** be verified against the active compiler test suite or written to be fully valid, compilable Solix code.

---

## 3. Structural Specification by Document Tier

### 3.1 User Guides (`docs/guide/`)
* **Focus:** Pragmatic, step-by-step developer onboarding and operational usage.
* **Format:**
  * Title and metadata.
  * Conceptual introduction with real-world Solix code examples.
  * Practical usage patterns, best practices, and common pitfalls.
  * Cross-references: "Deep Dive Specifications" (`docs/spec/`) and "Language Reference" (`docs/wiki/`).

### 3.2 Formal Specifications (`docs/spec/`)
* **Focus:** Deep technical precision for compiler engineers, toolchain maintainers, and runtime developers.
* **Format:**
  * Formal syntax and EBNF grammar productions.
  * AST data structure definitions in Rust (`solix-compilation`).
  * Semantic type-checking and binding invariants (`BinderPass1`, `Pass2`, `Pass3`).
  * Bytecode lowering rules showing exact 64-bit Register VM opcodes (`Format A`, `Format B`, `Format Call`).
  * Runtime behavior and memory management implications (`VirtualHeap`, ARC, deterministic destructors).

### 3.3 Atomic Wiki (`docs/wiki/`)
* **Focus:** Rapid dictionary-style lookup for IDE tooltips, error explanations, and quick syntax reminders.
* **Format for Error Codes (`docs/wiki/errors/`):**
  * Diagnostic Code (`EXXXX` or `RXXXX`), Category, and Human-Readable Title.
  * Exact Root Cause & Compiler Invariant.
  * Erroneous Code Example (with error highlight).
  * Corrected Code Example.
  * Related Diagnostics and Specification Cross-Link.

---

## 4. Authoring Workflow for New Language Features

When adding or modifying a language feature:
1. **Author the Formal Spec First (`docs/spec/`):** Define the grammar, AST lowering, and bytecode emission.
2. **Author Wiki Reference Pages (`docs/wiki/`):** Document all new keywords in `docs/wiki/keywords/`, primitives in `docs/wiki/primitives/`, and any new diagnostic error codes in `docs/wiki/errors/`.
3. **Update or Author User Guides (`docs/guide/`):** Integrate practical examples into the relevant guide, cross-linking to the new spec and wiki entries.
4. **Audit per GL-0013:** Run the pre-merge Git-diff audit to verify zero documentation drift.
