# Conventional Commits & Release Specification

## Document Metadata
* **Guideline ID:** GL-0001
* **Scope:** Universal / Workspace-Wide (`solix-common`, `solix-compilation`, `solix-runtime`, `solix-toolchain`, `solix-lsp`, `solix-dbg`, `docs`, `ci`)
* **Status:** Final Implementation Standard
* **Audience:** Core Systems Engineers, Compiler Architects, Runtime Engineers, Release Maintainers
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)

---

## 1. Executive Summary & Purpose

The Solix language ecosystem is composed of tightly coordinated workspace crates in Rust: the AST compiler (`solix-compilation`), the deterministic bytecode virtual machine (`solix-runtime`), shared ISA and diagnostic primitives (`solix-common`), the unified CLI toolchain (`solix-toolchain`), the Language Server Protocol implementation (`solix-lsp`), and debugging tools (`solix-dbg`). The integrity of automated package publishing, versioning, and change auditing depends directly upon deterministic, machine-parseable Git commit histories.

This specification mandates adherence to the **Conventional Commits v1.0.0** standard coupled with automated release pipelines. By formalizing commit grammar, the engineering team guarantees:
1. Automated semantic version bumps (Major, Minor, Patch) according to **SemVer 2.0.0**.
2. Automated, immutable generation and maintenance of `CHANGELOG.md` across the workspace and individual crates.
3. Automated drafting and publication of tagged GitHub Releases and binary toolchain artifacts.
4. Complete elimination of ambiguous, unsearchable, or monolithic commits that obfuscate regressions in compiler passes or runtime dispatch.

Any commit that violates this grammar halts the automated integration pipeline and **MUST NOT** be merged into the protected trunk branch (`master`).

---

## 2. Normative Invariants & Core Principles

All contributors **MUST** comply with the following normative rules:

1. **Strict Conventional Commit Formatting:** Every commit subject line **MUST** match the structure `<type>(<scope>): <description>`.
2. **Deterministic Version Bumping:**
   * Commits of type `fix` or `perf` **MUST** trigger a **Patch** version bump ($x.y.Z \to x.y.Z+1$).
   * Commits of type `feat` **MUST** trigger a **Minor** version bump ($x.Y.z \to x.Y+1.0$).
   * Any commit introducing a breaking contract or ISA change **MUST** trigger a **Major** version bump ($X.y.z \to X+1.0.0$).
3. **Mandatory Breaking Change Syntax:** A breaking change **MUST** be signaled by appending an exclamation mark (`!`) immediately prior to the colon in the header (e.g., `feat(runtime)!: ...`) **AND** including a `BREAKING CHANGE:` footer block in the commit body. Omitting either token when altering bytecode format, language syntax, or public APIs is strictly prohibited.
4. **Canonical Lowercase Scopes:** Scopes **MUST** be selected exclusively from the registered manifest of canonical lowercase scopes (Section 3.3). Custom or ad-hoc scopes **MUST NOT** be used without an approved update to this guideline.
5. **Imperative Mood & Punctuation Constraints:** The commit description **MUST** be written in the imperative, present tense (e.g., "add", "fix", "implement", "refactor", not "added", "fixes", "implementing"). The description **MUST NOT** begin with a capital letter and **MUST NOT** end with a trailing period.
6. **Atomic Concern Principle:** Each commit **MUST** encapsulate a single atomic change. Combining unrelated modifications (e.g., fixing a parser precedence bug while optimizing runtime opcode dispatch) within one commit **MUST NOT** occur.
7. **Body Line Length:** Commit body lines **MUST** wrap at **72 characters** to guarantee readability across Git terminal pagers (`git log`) and release interfaces.

---

## 3. Structural Specification & Schema

### 3.1 Backus-Naur Form (BNF) Grammar

```bnf
<commit-message> ::= <header> <newline> <newline> <body> [<newline> <newline> <footer>]
<header>         ::= <type> ["(" <scope> ")"] ["!"] ":" <space> <description>
<type>           ::= "feat" | "fix" | "perf" | "docs" | "refactor" | "style" | "test" | "chore" | "build" | "ci"
<scope>          ::= "compiler" | "runtime" | "common" | "client" | "lsp" | "dbg" | "specs" | "docs" | "deps" | "ci"
<description>    ::= <lowercase-letter> <any-character-except-period>*
<body>           ::= <wrapped-text-72-cols>
<footer>         ::= <footer-element> (<newline> <footer-element>)*
<footer-element> ::= <breaking-change> | <issue-ref>
<breaking-change>::= "BREAKING CHANGE:" <space> <explanation>
<issue-ref>      ::= ("Refs:" | "Fixes:" | "Closes:") <space> "#" <issue-number>
```

### 3.2 Commit Type Semantics & SemVer Impact

| Commit Type | SemVer Impact | Release Action | Purpose and Guidance |
| :--- | :--- | :--- | :--- |
| `feat` | **Minor** ($0.X.0$) | Appends entry to `### Features` in `CHANGELOG.md` | Introduces new language feature, statement, opcode, or CLI command. |
| `fix` | **Patch** ($0.0.X$) | Appends entry to `### Bug Fixes` in `CHANGELOG.md` | Resolves a compiler bug, runtime trap defect, or CLI crash. |
| `perf` | **Patch** ($0.0.X$) | Appends entry to `### Performance` in `CHANGELOG.md` | Improves memory consumption, opcode dispatch speed, or compile latency. |
| `docs` | None ($0.0.0$) | Excluded from release notes by default | Documentation updates, guideline modifications, spec edits. |
| `refactor` | None ($0.0.0$) | Excluded from release notes by default | Code restructuring without functional or contract changes. |
| `test` | None ($0.0.0$) | Excluded from release notes by default | Addition, refactoring, or correction of unit/integration tests. |
| `style` | None ($0.0.0$) | Excluded from release notes by default | Code formatting, rustfmt compliance, clippy cosmetic fixes. |
| `build` | None ($0.0.0$) | Excluded from release notes by default | Cargo workspace configurations, build scripts, profile tuning. |
| `ci` | None ($0.0.0$) | Excluded from release notes by default | GitHub Actions workflows, toolchain gates, Miri / ASan checks. |
| `chore` | None ($0.0.0$) | Excluded from release notes by default | Repository maintenance, license headers, metadata updates. |

### 3.3 Registered Component Scopes

| Scope Identifier | Target Component / Crate | Directory Boundaries |
| :--- | :--- | :--- |
| `(compiler)` | `solix-compilation` AST compiler | `crates/solix-compilation/` |
| `(runtime)` | `solix-runtime` bytecode VM | `crates/solix-runtime/` |
| `(common)` | `solix-common` shared ISA & diagnostics | `crates/solix-common/` |
| `(toolchain)` / `(client)` | `solix-toolchain` unified CLI toolchain | `crates/solix-toolchain/` |
| `(lsp)` | `solix-lsp` Language Server Protocol | `crates/solix-lsp/` |
| `(dbg)` | `solix-dbg` debugger & inspector | `crates/solix-dbg/` |
| `(specs)` | Architectural & language specifications | `docs/internals/specs/` |
| `(docs)` | User guides, language wiki, overview docs | `docs/guides/`, `docs/wikipedia/`, `docs/overview/` |
| `(deps)` | Cargo workspace dependencies | `Cargo.toml`, `Cargo.lock` |
| `(ci)` | CI/CD automation & verification scripts | `.github/workflows/`, `scripts/` |

---

## 4. Production-Ready Template

Engineers **MUST** structure non-trivial commits using the following production template:

```gitcommit
<type>(<scope>): <imperative description starting lowercase without period>

[Optional body: Detailed technical context explaining the problem, the root
cause, the selected architecture, and the implementation rationale. Lines
MUST wrap at 72 characters.]

[Optional additional paragraphs separated by double newlines, providing
benchmarks, memory measurements, or opcode dispatch details.]

[Optional Breaking Change Footer - MANDATORY if type contains '!']:
BREAKING CHANGE: <Detailed description of the breaking change, migration
path, affected bytecode binaries, and deprecated syntax or APIs.>

[Optional Issue References]:
Fixes: #<issue-number>
Refs: #<issue-number>
```

---

## 5. Concrete Production Examples

### Example 1: Breaking Bytecode Format Change (Major Version Bump)

```gitcommit
feat(common)!: transition bytecode binary header to 16-byte magic schema

Expand the bytecode file header from 8 bytes to 16 bytes to incorporate
a dedicated target architecture profile flag, minimum required VM runtime
version, and endianness indicator.

BREAKING CHANGE: Bytecode files compiled with previous versions lack the
16-byte header signature ('SOLX\x02\x00...') and will be rejected by the
virtual machine with 'BytecodeVersionMismatch' upon loading. All existing
.slxbin files must be recompiled using 'solix compile'.

Fixes: #104
Refs: #98
```

### Example 2: Critical Stack Underflow Fix in VM Runtime (Patch Bump)

```gitcommit
fix(runtime): prevent stack underflow during binary operator evaluation

Verify operand stack depth prior to popping left and right operands in
eval_binary_op dispatch loop. Previously, malformed bytecode containing
orphaned arithmetic opcodes caused undefined behavior by popping an empty
runtime operand stack.

Closes: #142
```

### Example 3: Performance Improvement in Lexer Token Streaming (Patch Bump)

```gitcommit
perf(compiler): eliminate string allocations for identifier tokens

Replace heap-allocated String instances in Token::Identifier with
zero-copy source spans referencing the parent SourceFile buffer.
Reduces compiler memory allocations by 38% on large translation units.

Refs: #187
```

---

## 6. Anti-Patterns & Counter-Examples (What NOT to Do)

### Counter-Example 1: Ambiguous & Colloquial Descriptions
```text
# FLAWED:
git commit -m "fix bug"
git commit -m "WIP on parser"
git commit -m "update stuff"
```
* **Failure Mode:** Completely unparseable by automated changelog and release generators. The generated changelog contains empty or useless entries. Git bisect archaeology becomes nearly impossible when tracking subtle compiler regressions.
* **Corrected Form:**
```text
# CORRECT:
git commit -m "fix(compiler): correct operator precedence for logical disjunction"
git commit -m "feat(compiler): parse tuple destructuring declarations"
```

### Counter-Example 2: Capitalization & Trailing Punctuation
```text
# FLAWED:
git commit -m "feat(runtime): Implemented Automatic Reference Counting."
```
* **Failure Mode:** Violates strict regex linting in commit verification CI. Capitalization and the trailing dot break changelog formatting consistency and automated release notes parsing.
* **Corrected Form:**
```text
# CORRECT:
git commit -m "feat(runtime): implement automatic reference counting"
```

### Counter-Example 3: Silent Breaking Changes
```text
# FLAWED:
git commit -m "refactor(common): change OpCode::Add to require type operand"
```
* **Failure Mode:** Categorized as a non-releasing refactoring commit. A new minor or patch version is cut without bumping the Major version. When distributed, existing bytecode executables fail to run on the new VM runtime without warning.
* **Corrected Form:**
```text
# CORRECT:
feat(common)!: require type discriminator operand for OpCode::Add

BREAKING CHANGE: OpCode::Add now expects an 8-bit type discriminator
operand immediately following the instruction byte. Existing compiled
bytecode binaries must be recompiled.
```

### Counter-Example 4: Monolithic Multi-Crate Commits
```text
# FLAWED:
git commit -m "feat: add tuple primitive, update vm and add cli pack command"
```
* **Failure Mode:** Binds unrelated subsystems across compiler, runtime, and client together. Reverting a regression in the CLI pack command forces reverting the core language feature in the compiler and VM.
* **Corrected Form:** Split into discrete, atomic commits:
  1. `feat(specs): specify tuple primitive syntax, AST nodes, and opcodes`
  2. `feat(common): add OpCode::TupleNew and OpCode::TupleGet instructions`
  3. `feat(compiler): implement parsing and type checking for tuple types`
  4. `feat(runtime): implement tuple allocation and field extraction in VM`
  5. `feat(client): expose --pack flag in solix compile command`

---

## 7. Verification & Compliance Checklist

All engineers and automated PR gates **MUST** enforce the following criteria prior to merge:

- [ ] **Type Legitimacy:** The commit type is one of: `feat`, `fix`, `perf`, `docs`, `refactor`, `style`, `test`, `chore`, `build`, `ci`.
- [ ] **Scope Legitimacy:** The scope is lowercase and registered in Section 3.3 (`compiler`, `runtime`, `common`, `client`, `lsp`, `dbg`, `specs`, `docs`, `deps`, `ci`).
- [ ] **Imperative Mood:** Subject starts lowercase, uses imperative mood ("add", not "added"), and omits trailing punctuation.
- [ ] **Breaking Change Conformance:** If breaking, both `!` in header and `BREAKING CHANGE:` in footer are present.
- [ ] **Line Wrapping:** All lines in the commit body and footer wrap at 72 characters or fewer.
- [ ] **Atomic Integrity:** Commit does not intermingle distinct concerns across multiple crates.
