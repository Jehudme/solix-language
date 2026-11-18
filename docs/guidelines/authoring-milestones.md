# Milestone Planning & Delivery Specification

## Document Metadata
* **Guideline ID:** GL-0008
* **Scope:** Engineering Management & Phased Delivery (GitHub Milestones & Issue Forms)
* **Status:** Final Implementation Standard
* **Audience:** Core Systems Engineers, Compiler Architects, Runtime Maintainers, Engineering Leads
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)

---

## 1. Executive Summary & Purpose

Building a modern programming language, optimizing compiler, and bytecode virtual machine in Rust requires coordinated, phased execution. Without disciplined milestone planning, compiler passes become entangled, syntax extensions merge before the type system can validate them, VM opcodes drift from the assembler, and developer tooling breaks.

This specification formalizes milestone authoring, priority taxonomy, execution lifecycle, branching strategy, and closure criteria for the Solix language project using native **GitHub Milestones** and structured `.github/ISSUE_TEMPLATE/` forms. It establishes an unambiguous **Definition of Done (DoD)** and an enforceable **Priority Blocker Taxonomy (`P0`–`P3`)** governing every compiler, runtime, and toolchain release.

---

## 2. Normative Invariants & Core Principles

All milestone plans and contributors **MUST** comply with the following binding invariants:

1. **GitHub Milestones & Issue Forms as Single Source of Truth:** Milestones and delivery tasks **SHALL** be authored and tracked natively as **GitHub Milestones** (e.g., `v1.6.0` / `MS-064`) backed by `.github/ISSUE_TEMPLATE/` YAML forms (`milestone_task.yml`, `bug_report.yml`, `feature_request.yml`). Multi-crate architectural design proposals belong in `docs/architecture/`.
2. **Dev Integration & On-Demand Release (`dev` -> `master`):** All pull requests implementing milestone issues **MUST** target protected **`dev`** and link tracked issues (`Closes #<number>`). Once all `P0` and `P1` milestone items land on `dev`, a maintainer triggers [`release.yml`](../../.github/workflows/release.yml) via `workflow_dispatch` to bump versions, validate post-bump checks, advance `master`, publish crates/installers, and sync `master -> dev`.
3. **Deterministic Blocker Classification:** Every issue within a milestone **MUST** carry an explicit priority tier (`P0`, `P1`, `P2`, or `P3`).
4. **The `P0`/`P1` Zero-Tolerance Rule:** A GitHub Milestone release **MUST NOT** ship on `master` while any `P0` or `P1` issue remains open. Open `P2` or `P3` issues **MAY** be deferred to the next GitHub Milestone.
5. **Non-Negotiable Rust Definition of Done (DoD):** A milestone issue cannot close until all five DoD pillars pass on `dev` CI:
   * **Clean Compilation:** `cargo build --workspace --all-targets`.
   * **100% Automated Test Pass:** `cargo test --workspace --all-targets`.
   * **Zero Clippy Defects:** `cargo clippy --workspace --exclude solix-tests --lib --bins --no-deps -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::min_ident_chars`.
   * **Formatting & Security:** `cargo fmt --check`, `cargo-deny`, `cargo-audit`, `actionlint`, and `zizmor`.
   * **Living Documentation:** User docs in `docs/guide/`, `docs/spec/`, `docs/wiki/` synchronized (`GL-0013`), verified by `scripts/verify_docs.py`, and compiled by `scripts/build_docs_site.py`.
6. **Milestone State Progression:** Issues and milestones follow `DRAFT` -> `ACTIVE` -> `FEATURE_COMPLETE` -> `VALIDATED` -> `CLOSED`.

---

## 3. Structural Specification & Schema

### 3.1 Blocker Priority Taxonomy

| Priority Tier | Classification | Definition & Criteria | Release Impact | SLA / Gate |
| :--- | :--- | :--- | :--- | :--- |
| **`P0`** | **Critical Blocker** | Halts release, causes VM panics, introduces undefined behavior, produces corrupted bytecode, or breaks CI compilation. | **Blocks all releases immediately.** | Must be fixed first. |
| **`P1`** | **High Priority** | Core milestone deliverable, essential language statement, parser AST node, opcode, or release blocker. | **Blocks milestone completion.** | Must resolve prior to `FEATURE_COMPLETE`. |
| **`P2`** | **Medium Priority** | Ergonomics, diagnostic polish, non-critical CLI flags, or workflow refinements. | Non-blocking if deferred cleanly. | Current milestone or next milestone. |
| **`P3`** | **Low Priority** | Minor documentation polish, internal refactoring, opportunistic cleanups. | Non-blocking. Deferrable. | Backlog or opportunistic cleanup. |

### 3.2 Milestone Lifecycle Workflow

```text
master ──●──────────────────────────────────────────●──► (On-Demand Release via release.yml)
          \                                        /
dev    ────●────●────●──────────●─────────●───────●────► (Integration Trunk)
                 \  /            \       /
             PR (#123 -> dev)   PR (#124 -> dev)
```

1. **Draft Issues:** Create scoped GitHub Issues using `.github/ISSUE_TEMPLATE/milestone_task.yml` and assign them to the target GitHub Milestone with priority (`P0`–`P3`) and type labels (`feature`, `bug`, `easy`, `medium`, `hard`).
2. **Implement Against `dev`:** Branch from `origin/dev` (either per issue or on an integration topic branch such as `milestone/064-ms064`), reference `Closes #<issue>` in PRs targeting `dev`, and pass all [`ci.yml`](../../.github/workflows/ci.yml) gates.
3. **Cut Release:** When all `P0`/`P1` issues in the milestone are closed on `dev`, run [`release.yml`](../../.github/workflows/release.yml) on demand to bump versions, validate tests, update `master`, publish artifacts, and close the GitHub Milestone.

---

## 4. Production Issue Checklist Template

Authors creating milestone tasks in GitHub Issues **SHOULD** supply:

```markdown
* **Priority Tier:** P1 - Core Milestone Deliverable
* **Affected Subsystems / Crates:** `crates/...` or `.github/workflows/`
* **Objective & Architectural Summary:** Clear problem statement and target behavior.
* **Acceptance Criteria / Verification Plan:** Specific commands (`cargo test`, `python3 -m unittest`, workflow run verification) confirming completion.
```

---

## 5. Anti-Patterns & Counter-Examples (What NOT to Do)

### Anti-Pattern 1: Direct PRs or Pushes to `master`
* **Flawed Practice:** Opening feature/task PRs directly against `master` instead of `dev`.
* **Failure Mode:** Skips integration testing alongside concurrent changes and conflicts with automated release versioning.
* **Corrected Approach:** Always target `dev`, then promote verified `dev` commits via [`release.yml`](../../.github/workflows/release.yml).

### Anti-Pattern 2: Premature Release with Unresolved `P0`/`P1` Issues
* **Flawed Practice:** Shipping a release while a `P0` runtime bug or `P1` core deliverable remains broken.
* **Failure Mode:** Deploys regressions to users and crates.io consumers.
* **Corrected Approach:** Resolve every `P0` and `P1` issue on `dev` before triggering a release, or explicitly defer lower-priority `P2`/`P3` items.

---

## 6. Verification & Compliance Checklist

- [ ] **GitHub Milestone Linked:** Every planned task is tracked as an issue in the active GitHub Milestone with explicit priority (`P0`–`P3`).
- [ ] **Zero Open `P0`/`P1` on Release:** All `P0` and `P1` milestone issues are merged and closed on `dev` before running `release.yml`.
- [ ] **Full DoD Verification:** All five DoD gates (`cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`, docs scripts) pass cleanly.
