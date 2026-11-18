# Milestone Planning & Delivery Specification

## Document Metadata
* **Guideline ID:** GL-0008
* **Scope:** Engineering Management & Phased Delivery (`docs/internals/milestones/`)
* **Status:** Final Implementation Standard
* **Audience:** Core Systems Engineers, Compiler Architects, Runtime Maintainers, Engineering Leads
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)

---

## 1. Executive Summary & Purpose

Building a modern programming language, optimizing compiler, and bytecode virtual machine in Rust requires coordinated, phased execution. Without disciplined milestone planning, compiler passes become entangled, syntax extensions are merged before the type system can validate them, VM opcodes drift from the assembler, and developer tooling breaks.

This specification formalizes the authoring, priority taxonomy, execution lifecycle, branching strategy, and closure criteria for all engineering milestones within the Solix language project (`docs/internals/milestones/`). It establishes an unambiguous **Definition of Done (DoD)**, an enforceable **Priority Blocker Taxonomy (P0-P3)**, and mandates that **every milestone operates within its own dedicated Git milestone branch**. Every milestone file checked into Git serves as an authoritative, auditable delivery contract governing compiler and toolchain releases.

---

## 2. Normative Invariants & Core Principles

All milestone plans and contributors **MUST** comply with the following binding invariants:

1. **GitHub Milestones & Issue Forms for Active Planning (`MS-064+`):** Beginning after `MS-063`, active milestones and delivery checklists **SHALL** be authored and tracked natively as **GitHub Milestones** (`v1.5.0`, etc.) backed by structured `.github/ISSUE_TEMPLATE/` YAML forms (`milestone_task.yml`, `bug_report.yml`, `feature_request.yml`). Pull requests link issues (`Closes #<number>`) targeting **`dev`**, updating milestone progress automatically.
   * **Historical Archive & Architectural RFCs:** Existing milestone files `001` through `063` in `docs/internals/milestones/` remain preserved as a read-only historical architecture archive. Multi-crate architectural proposals (`ADR` / design specifications) live in `docs/architecture/`.
2. **Two-Wave Integration (`dev` -> `master`):** All PRs implementing milestone issues **MUST** target and merge into protected **`dev`**. Once a release milestone is ready, the curated `dev -> master` Release PR cuts the versioned release on `master`.
3. **Deterministic Blocker Classification:** Every work item, bug, or feature within a milestone **MUST** be labeled with an explicit priority tier: `P0`, `P1`, `P2`, or `P3`.
4. **The P0/P1 Zero-Tolerance Rule:** A GitHub Milestone release **MUST NOT** ship on `master` if a single `P0` or `P1` issue remains open. Open `P2` or `P3` issues **MAY** be deferred to the next GitHub Milestone.
5. **Non-Negotiable Rust Definition of Done (DoD):** A milestone issue cannot close until all five DoD pillars pass on `dev` CI:
   * **Clean Compilation:** `cargo build --workspace --all-targets`.
   * **100% Automated Test Pass:** `cargo test --workspace --all-targets`.
   * **Zero Clippy Defects:** `cargo clippy --workspace --exclude solix-tests --lib --bins --no-deps -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::min_ident_chars`.
   * **Formatting & Security:** `cargo fmt --check`, `cargo-deny`, `cargo-audit`, `actionlint`, and `zizmor`.
   * **Living Documentation:** User docs in `docs/guide/`, `docs/spec/`, `docs/wiki/` synchronized cleanly (`GL-0013`) and compiled cleanly by `scripts/build_docs_site.py`.
6. **Milestone State Progression:** Issues and milestones follow `DRAFT` $\to$ `ACTIVE` $\to$ `FEATURE_COMPLETE` $\to$ `VALIDATED` $\to$ `CLOSED`.

---

## 3. Structural Specification & Schema

### 3.1 Blocker Priority Taxonomy

| Priority Tier | Classification | Definition & Criteria | Release Impact | SLA / Gate |
| :--- | :--- | :--- | :--- | :--- |
| **`P0`** | **Critical Blocker** | Halts release, causes VM panics, introduces undefined behavior, produces corrupted bytecode emission, or breaks CI compilation. | **Blocks all merges to master immediately.** | Must be fixed within 24h; all other work stops. |
| **`P1`** | **High Priority** | Core milestone deliverable, essential language statement, parser AST node, opcode implementation, or type-checking rule. | **Blocks milestone completion.** Milestone cannot merge to `master`. | Must be resolved prior to `FEATURE_COMPLETE`. |
| **`P2`** | **Medium Priority** | Ergonomics, diagnostic warning formatting, disassembler output polish, non-critical CLI flags. | Non-blocking. Can release if maintainers approve deferral. | Scheduled for current milestone or pushed to next. |
| **`P3`** | **Low Priority** | Minor documentation polish, internal refactoring, non-functional code cleanups. | Non-blocking. Deferrable. | Backlog or opportunistic cleanup. |

### 3.2 Milestone Finite State Machine (FSM)

```
    ┌──────────┐
    │  DRAFT   │
    └────┬─────┘
         │ (Milestone approved by lead architect)
         ▼
    ┌──────────┐
    │  ACTIVE  │ ◄─────────────────────────────────────────┐
    └────┬─────┘                                           │
         │ (All P0 and P1 task PRs merged to milestone br) │ (Regression / test failure
         ▼                                                 │  found in validation suite)
    ┌──────────────────┐                                   │
    │ FEATURE_COMPLETE │ ──────────────────────────────────┘
    └────┬─────────────┘
         │ (DoD verified: cargo test, clippy clean, docs synced)
         ▼
    ┌───────────┐
    │ VALIDATED │
    └────┬──────┘
         │ (Milestone branch merged into master with --no-ff)
         ▼
    ┌──────────┐
    │  CLOSED  │
    └──────────┘
```

1. **`DRAFT`**: Scope and deliverables are being drafted. Not yet approved for implementation.
2. **`ACTIVE`**: Development in progress on the dedicated milestone branch (`milestone/XXX-...`).
3. **`FEATURE_COMPLETE`**: All scoped `P0` and `P1` functionality is merged into the milestone branch. Feature freeze in effect; only bug fixes, tests, and documentation permitted.
4. **`VALIDATED`**: Full multi-platform CI matrix passes on the milestone branch; all tests, clippy checks, and documentation verified.
5. **`CLOSED`**: Milestone branch successfully merged into `master` via `--no-ff` and officially retired.

### 3.3 Dedicated Milestone Branching Workflow

Every milestone follows a strict branch isolation protocol:

```text
master ──────────●──────────────────────────────────────────────●─────►
                  \                                            /
                   \ (Cut milestone branch)                   / (Merge milestone: --no-ff)
milestone/002-      ●───────────●───────────●────────────────●
lexer-parser         \         /             \              /
                      ●───────●               ●────────────●
                     feat/lexer-tokens       feat/parser-exprs
```

```bash
# 1. Cut the dedicated milestone branch from master
git checkout master
git pull --rebase origin master
git checkout -b milestone/002-lexer-parser
git push -u origin milestone/002-lexer-parser

# 2. Contributors cut topic branches FROM the milestone branch
git checkout milestone/002-lexer-parser
git checkout -b feat/compiler-lexer-tokens

# 3. PRs are submitted targeting 'milestone/002-lexer-parser' (NOT master)

# 4. Once validated, merge milestone branch into master
git checkout master
git pull --rebase origin master
git merge --no-ff milestone/002-lexer-parser \
  -m "Merge milestone/002-lexer-parser into master (MS-002: Lexer & Parser Implementation)"
git push origin master
```

---

## 4. Production-Ready Template

Engineers authoring milestone files (`docs/internals/milestones/XXX-[name].md`) **MUST** use the following canonical template:

```markdown
# Milestone [XXX]: [Milestone Title]

## Document Metadata
* **Milestone ID:** MS-[XXX]
* **Target Version:** `v[X.Y.Z]`
* **Milestone Branch:** `milestone/[XXX]-[slug]`
* **Status:** [DRAFT | ACTIVE | FEATURE_COMPLETE | VALIDATED | CLOSED]
* **Lead Architect:** [@github_username]
* **Target Delivery Date:** [YYYY-MM-DD]
* **Related Specifications:** [Spec 1, Spec 2]

---

## 1. Executive Summary & Objectives
<!-- High-level statement of the milestone's primary mission and delivered capabilities. -->
[Provide milestone summary and objectives here]

## 2. Scope & Crates Affected
* `crates/solix-common`: [Scope description]
* `crates/solix-compilation`: [Scope description]
* `crates/solix-runtime`: [Scope description]
* `crates/solix-toolchain`: [Scope description]
* `crates/solix-lsp`: [Scope description]
* `crates/solix-dbg`: [Scope description]
* `docs/`: [Scope description]

## 3. Prioritized Deliverables & Task Matrix

### 3.1 P0: Critical Blockers (Release Gate)
- [ ] `[P0]` **[Task Title]:** [Detailed task description and PR/Issue link]
- [ ] `[P0]` **[Task Title]:** [Detailed task description and PR/Issue link]

### 3.2 P1: High Priority (Milestone Core)
- [ ] `[P1]` **[Task Title]:** [Detailed task description and PR/Issue link]
- [ ] `[P1]` **[Task Title]:** [Detailed task description and PR/Issue link]

### 3.3 P2: Medium Priority (Ergonomics & Polish)
- [ ] `[P2]` **[Task Title]:** [Detailed task description and PR/Issue link]
- [ ] `[P2]` **[Task Title]:** [Detailed task description and PR/Issue link]

### 3.4 P3: Low Priority (Opportunistic Cleanups)
- [ ] `[P3]` **[Task Title]:** [Detailed task description and PR/Issue link]

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Zero warnings with `-D warnings` | `cargo build --workspace --all-targets` | [ ] Pending / Verified |
| **Automated Test Suite** | 100% pass across all unit, integration, and doc tests | `cargo test --workspace --all-targets` | [ ] Pending / Verified |
| **Clippy Static Analysis** | Zero warnings, zero `unwrap` / `expect` | `cargo clippy --workspace --all-targets` | [ ] Pending / Verified |
| **Rustfmt Formatting** | Zero formatting violations | `cargo fmt --check` | [ ] Pending / Verified |
| **Documentation Sync** | Specs and guides updated to GL-0010 | GL-0010 audit checklist | [ ] Pending / Verified |

## 5. Risk Assessment & Mitigation Strategies
* **Risk 1:** [Description of architectural or grammar ambiguity risk]
  * *Mitigation:* [Concrete engineering mitigation]
* **Risk 2:** [Description of VM runtime or opcode encoding risk]
  * *Mitigation:* [Concrete engineering mitigation]
```

---

## 5. Concrete Production Example

```markdown
# Milestone 002: Lexer & Recursive-Descent Parser Implementation

## Document Metadata

* **Milestone ID:** MS-002
* **Target Version:** `v0.1.0-alpha.2`
* **Milestone Branch:** `milestone/002-lexer-parser`
* **Status:** ACTIVE
* **Lead Architect:** @compiler-lead
* **Target Delivery Date:** 2026-11-20
* **Related Specifications:**
   * `docs/internals/specs/statements/control_flow.md`
   * `docs/internals/specs/statements/declarations.md`

---

## 1. Executive Summary & Objectives

Deliver the complete lexing token stream and recursive-descent AST parser for the Solix language in `solix-compilation`.
The parser converts raw source UTF-8 streams into strongly-typed Abstract Syntax Tree (AST) representations with exact
source span tracking, comprehensive error recovery via synchronization tokens, and rich diagnostics powered by `miette`.

## 2. Scope & Crates Affected

* `crates/solix-common`: Token definitions, source spans, diagnostic error codes.
* `crates/solix-compilation`: Lexer, token stream cursor, recursive-descent parser, AST node definitions.
* `crates/solix-toolchain`: CLI syntax check command (`solix check`).
* `docs/internals/specs`: Statement and expression grammar BNF definitions.

## 3. Prioritized Deliverables & Task Matrix

### 3.1 P0: Critical Blockers (Release Gate)

- [x] `[P0]` **Token Span Tracking:** Record exact line, column, and byte offset for all tokens in `solix-common`. (PR
  #12)
- [ ] `[P0]` **Parser Panic Recovery:** Implement panic-free error recovery with synchronization points on statement
  boundaries (`semicolon`, `closing brace`). (PR #18)
- [ ] `[P0]` **Zero Unwrapped Parsing:** Replace all `.unwrap()` invocations with structured `ParseError` types.

### 3.2 P1: High Priority (Milestone Core)

- [x] `[P1]` **Lexer Token Stream:** Implement zero-copy tokenization of keywords, literals, and operators. (PR #14)
- [ ] `[P1]` **Control Flow AST Parsing:** Parse `if`, `while`, `for`, `switch`, `return`, `throw`, and
  `try-catch-finally`. (PR #22)
- [ ] `[P1]` **Declaration AST Parsing:** Parse `class`, `interface`, `method`, `field`, and `enum` declarations. (PR
  #25)

### 3.3 P2: Medium Priority (Ergonomics & Polish)

- [ ] `[P2]` **Diagnostic Help Suggestions:** Add "did you mean" spelling suggestions for misspelled keywords in
  `miette` reports.
- [ ] `[P2]` **AST Pretty Printer:** Implement a debug tree formatter for `--emit-ast`.

### 3.4 P3: Low Priority (Opportunistic Cleanups)

- [ ] `[P3]` **Token Memory Footprint:** Optimize `Token` enum memory footprint using compact enum discriminants.

## 4. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Workspace Compilation** | Compiles with zero warnings | `cargo build --workspace --all-targets` | [ ] Pending |
| **Automated Test Suite** | 68 unit parser tests green | `cargo test --workspace --all-targets` | [ ] Pending |
| **Clippy Static Analysis** | Zero warnings, no unwrap in parser | `cargo clippy --workspace --all-targets` | [ ] Pending |
| **Rustfmt Formatting** | 100% rustfmt compliant | `cargo fmt --check` | [x] Verified |
| **Documentation Sync** | Grammar BNF specs updated | GL-0010 audit checklist | [ ] Pending |
```

---

## 6. Anti-Patterns & Counter-Examples (What NOT to Do)

### Anti-Pattern 1: Bypassing the Milestone Branch (Direct PRs to `master`)
* **Flawed Practice:** An engineer opens a PR for a half-finished type-checking pass directly against `master` because "they want early feedback."
* **Failure Mode:** If accidentally merged, `master` enters a broken state where language features are half-implemented, tests fail, and other developers cannot build or cut releases.
* **Corrected Approach:** Always target the dedicated `milestone/XXX-...` branch. The milestone branch isolates in-progress work until the full milestone is complete and validated.

### Anti-Pattern 2: Premature Closure with Unresolved Blockers
* **Flawed Practice:** Moving a milestone to `CLOSED` and merging to `master` while a `P0` parser crash on invalid syntax remains open, labeling it: "We'll fix it in a patch."
* **Failure Mode:** Deploying a compiler with known parser crashes causes the CLI to panic unhandled in production when users have syntax typos, destroying compiler reliability.
* **Corrected Approach:** A milestone **MUST NOT** close or merge to `master` with open P0 or P1 items. Either fix the blocker or formally down-scope the feature through maintainer consensus.

### Anti-Pattern 3: Incomplete Definition of Done
* **Flawed Practice:** Declaring a milestone complete because "the parser works on my sample program," skipping clippy checks, error-recovery unit tests, and documentation synchronization.
* **Failure Mode:** Code containing hidden `.unwrap()` calls panics on malformed inputs; compiler documentation remains out of sync with accepted syntax; CI fails on other developers' PRs.
* **Corrected Approach:** Enforce the full 5-pillar DoD matrix before merging the milestone branch to `master`.

---

## 7. Verification & Compliance Checklist

Milestone leads and release maintainers **MUST** verify the following checklist before modifying milestone states or merging to `master`:

- [ ] **Dedicated Branch Active:** Milestone work is isolated in `milestone/XXX-[slug]`.
- [ ] **Sequential Numbering:** Milestone file path matches `docs/internals/milestones/XXX-[name].md`.
- [ ] **Explicit Priorities:** Every listed task is explicitly tagged with `[P0]`, `[P1]`, `[P2]`, or `[P3]`.
- [ ] **Zero Open P0/P1s on Close:** No P0 or P1 tasks remain incomplete before transitioning to `CLOSED`.
- [ ] **Full DoD Verification:** All five DoD gates (`cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`, docs) pass cleanly on the milestone branch.
- [ ] **Non-Fast-Forward Merge:** Milestone branch is merged to `master` using `git merge --no-ff`.
