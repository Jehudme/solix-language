# Solix Engineering Guidelines Suite

## Document Metadata
* **Guideline ID:** GL-0000
* **Scope:** Universal Architectural & Engineering Governance
* **Status:** Final Implementation Standard
* **Audience:** All Solix Engineering Contributors & Maintainers
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)

---

## 1. Overview & Master Index

The Solix engineering guidelines suite defines the architectural, structural, stylistic, and procedural invariants governing the Solix language compiler, virtual machine runtime, and toolchain in modern Rust (Edition 2024).

All contributors **MUST** adhere to these guidelines across all crates (`crates/*`), specifications (`docs/spec/`), guides (`docs/guide/`), and wiki pages (`docs/wiki/`).

---

## 2. Guideline Register

| Guideline ID | Title & Specification | Primary Scope | Status |
| :--- | :--- | :--- | :--- |
| **GL-0001** | [Conventional Commits & Release Specification](authoring-commits.md) | Universal Commit Grammar & SemVer | `FINAL` |
| **GL-0002** | [Trunk-Based & Milestone Branching Topology Specification](branching-conventions.md) | Git Topology & Milestone Branches | `FINAL` |
| **GL-0004** | [Meta-Guideline: Authoring Engineering Guidelines](authoring-guidelines.md) | Standard for Creating Guidelines | `FINAL` |
| **GL-0005** | [Language & Subsystem Specification Invariants & Schema](authoring-specs.md) | Grammar, Statements, Opcodes, Runtime, CLI | `FINAL` |
| **GL-0008** | [Milestone Planning & Delivery Specification](authoring-milestones.md) | Phased Milestones & Rust Definition of Done | `FINAL` |
| **GL-0009** | [Test Engineering & Verification Matrices Specification](authoring-test.md) | Unit Tests at Bottom of File & Dual Matrices | `FINAL` |
| **GL-0010** | [Living Documentation & Drift Prevention Specification](authoring-docs.md) | Zero-Drift, Rustdoc & CLI Sync | `FINAL` |
| **GL-0011** | [Rust Language & Security Conventions Specification](language-conventions.md) | No Single-Letter Names, Military Safety, Errors | `FINAL` |
| **GL-0012** | [Tripartite Documentation Architecture Specification](authoring-features-and-wiki.md) | Guides vs Specs vs Atomic Wiki Architecture | `FINAL` |
| **GL-0013** | [Mandatory Pre-Merge Git-Diff Documentation Audit Protocol](documentation-update-protocol.md) | Pre-Merge Cross-Update Trigger Matrix & Anti-Drift | `FINAL` |
| **GL-0014** | [CI, Sensitive-Path Governance & On-Demand Release Protocol](pr-and-release-governance.md) | `dev` CI, On-Demand `release.yml`, `CODEOWNERS` & Security CI | `FINAL` |

---

## 3. Core Architectural Pillars

1. **Military-Grade Security & Zero Panics:** `#![forbid(unsafe_code)]` by default; `.unwrap()` and `.expect()` are banned in production code; defensive bounds checks and checked arithmetic throughout.
2. **Descriptive Naming:** Absolute prohibition of single-letter variable names (`i`, `x`, `t`) and single-letter generic types (`T`, `E`); self-documenting code with clear types and multi-letter abbreviations.
3. **Structured Error Chaining:** Exhaustive error categorization with `thiserror` using `#[source]` error chaining, and rich compiler diagnostics with `miette`.
4. **Co-Located Unit Tests:** All unit tests placed at the bottom of the source file in `#[cfg(test)] mod tests` with dual positive/negative coverage.
5. **Dev Integration & On-Demand Release Isolation:** Contributor PRs integrate into protected `dev` with strict up-to-date checks before an on-demand maintainer-triggered release promotes tested commits to `master` ([GL-0002](branching-conventions.md), [GL-0014](pr-and-release-governance.md)).
6. **Zero Documentation Drift:** Code, compiler specifications, and user documentation are co-committed and kept in lockstep.
