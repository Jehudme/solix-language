# Trunk-Based & Milestone Branching Topology Specification

## Document Metadata
* **Guideline ID:** GL-0002
* **Scope:** Repository Git Topology, Milestone Integration & Release Management
* **Status:** Final Implementation Standard
* **Audience:** Core Systems Engineers, Compiler Architects, Runtime Maintainers, Release Engineers
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)

---

## 1. Executive Summary & Purpose

In a programming language repository comprising multiple tightly-coupled Rust crates (`solix-compilation`, `solix-runtime`, `solix-common`, `solix-toolchain`, `solix-lsp`, `solix-dbg`), branch mismanagement is a primary cause of compiler instability, syntax regressions, and release gridlock. Long-lived feature branches foster overwhelming merge conflicts, break incremental type checking assumptions, and obscure Git bisect investigations.

This specification formalizes the **Dev Integration & On-Demand Release Branching Model (`dev -> master`)** for the Solix project. It establishes **`dev`** as the protected contributor integration trunk, **`master`** as the release-only production trunk updated exclusively by the on-demand release workflow ([GL-0014](pr-and-release-governance.md)), strict up-to-date branch checks to eliminate merge skew, and auditable Git history.

---

## 2. Normative Invariants & Core Principles

All contributors and repository administrators **MUST** comply with the following normative rules:

1. **Dual-Trunk Invariant (`dev` -> `master`):**
   * **`dev` (Integration Trunk):** All contributor topic branches, bug fixes, Dependabot PRs, and documentation updates **MUST** target `dev` via pull request. Direct unreviewed pushes to `dev` are prohibited.
   * **`master` (Production Release Trunk):** Direct contributor pushes and feature/bugfix PRs targeting `master` are **STRICTLY PROHIBITED**. Only the on-demand [`release.yml`](../../.github/workflows/release.yml) workflow advances `master` from verified `dev`, cuts a SemVer release commit, tags `vX.Y.Z`, publishes crates and release binaries, and back-syncs the version bump commit to `dev`.
2. **Strict Up-to-Date Branch Protection (Zero Merge-Skew):** Repository rulesets on `dev` **MUST** enable *"Require branches to be up to date before merging"* so concurrent PRs are re-tested against the latest `dev` commit before landing.
3. **Short-Lived Topic Branches:** Individual feature and bugfix branches **MUST** branch from `origin/dev` and stay short-lived (target lifecycle 1–3 business days).
4. **Canonical Branch Naming Grammar:** Topic branch names **MUST** adhere to `<category>/<scope>-<short-description>` (`feat`, `fix`, `spec`, `refactor`, `docs`, `chore`, `test`, `milestone`).
5. **Admin & CODEOWNER Approval:** Pull requests touching protected paths in `.github/CODEOWNERS` display a concise sensitive-paths notice (`scripts/pr_risk_audit.py`) and require maintainer (`@Jehudme`) approval before merging.
6. **Immediate Post-Merge Branch Deletion:** Topic branches **MUST** be deleted after PR merge. Stale remote topic branches **MUST NOT** be retained.
7. **No Rewriting Shared Trunk History:** Force-pushing (`git push --force`) to `master`, `dev`, or shared milestone branches is strictly prohibited. Force-pushing with lease (`--force-with-lease`) is allowed *only* on private unmerged topic branches during rebase.

---

## 3. Structural Specification & Schema

### 3.1 Branch Category Taxonomy

| Branch Category | Target Workflow | SemVer Impact | Target Lifetime |
| :--- | :--- | :--- | :--- |
| `milestone/` | Dedicated integration branch for an entire milestone | Minor / Patch | Active milestone duration |
| `feat/` | New language feature, AST parser, opcode, or CLI command | Minor ($0.X.0$) | 1–3 days |
| `fix/` | Bug fix, parser recovery patch, VM stack underflow fix | Patch ($0.0.X$) | < 24 hours |
| `spec/` | Language specification, grammar BNF, opcode definition | None (Docs) | 1–2 days |
| `refactor/` | Internal Rust code restructuring without behavioral shifts | None | 1–2 days |
| `docs/` | Guidelines, language tutorials, wiki reference articles | None | < 24 hours |
| `chore/` | Cargo.toml dependency updates, CI workflows, tooling configs| None | < 24 hours |
| `test/` | Adding test fixtures, benchmarks, or verification suites | None | 1–2 days |

### 3.2 Dual-Trunk Branch Topology Diagram

```
master ──●────────────────────────────────────────────────────────●──► (Production Releases Only)
          \                                                      /  ^
           \ (On-demand release.yml merges dev -> master)       /   (Sync release tag/bump)
dev    ─────●─────────●──────────●────────────●────────────────●─────────► (Integration Trunk)
                       \        / \          /
                        ●──────●   ●────────●
                      feat/topic-a  fix/topic-b
```

### 3.3 Protected Branch Configuration Rules

Repository Rulesets **MUST** remain active on both **`dev`** and **`master`** ([GL-0014](pr-and-release-governance.md)):
* **`dev` Ruleset:** Require pull request before merging, require CODEOWNER (`@Jehudme`) approval on sensitive files, dismiss stale approvals, **require branches to be up to date before merging** (`strict`), require all CI + security status checks, and block force pushes.
* **`master` Ruleset:** Restrict direct pushes/merges exclusively to the maintainer/release workflow (`RELEASE_TOKEN`), require pull requests for any external changes, and block force pushes.

---

## 4. Standard Branching Commands

Contributors **MUST** execute the following sequence when creating, maintaining, and merging topic branches:

```bash
# 1. Synchronize local workspace with origin/dev
git fetch origin --prune
git checkout -b <category>/<scope>-<short-description> origin/dev

# 2. Work atomically with Conventional Commits
git add <files>
git commit -m "<type>(<scope>): <imperative description>"

# 3. Keep topic branch up to date with origin/dev
git fetch origin
git rebase origin/dev

# 4. Push topic branch and open PR targeting dev
git push -u origin <category>/<scope>-<short-description>

# 5. After PR merge into dev, delete local & remote topic branch
git branch -d <category>/<scope>-<short-description>
git push origin --delete <category>/<scope>-<short-description>
```

---

## 5. Concrete Production Example

### Lifecycle of `feat/compiler-tuple-primitive` targeting `dev`

#### 1. Inception & Branching
```bash
git fetch origin --prune
git checkout -b feat/compiler-tuple-primitive origin/dev
```

#### 2. Atomic Conventional Commits
```bash
# Commit 1: Update specification first (GL-0010 zero-drift)
git add docs/spec/types.md
git commit -m "docs(specs): specify tuple primitive syntax, AST nodes, and typing"

# Commit 2: Implement tuple parsing and tests in solix-compilation
git add crates/solix-compilation/
git commit -m "feat(compiler): parse tuple type declarations and add unit tests"
```

#### 3. PR Review & Merge to `dev`
Open a pull request targeting `dev`, pass all [`ci.yml`](../../.github/workflows/ci.yml) checks, obtain `@Jehudme` approval, merge into `dev`, and delete the topic branch.

---

## 6. Anti-Patterns & Counter-Examples (What NOT to Do)

### Anti-Pattern 1: Direct Pushes to `master` or Bypassing `dev`
```bash
# FLAWED:
git checkout master
git commit -am "quick fix for compiler parser panic"
git push origin master
```
* **Failure Mode:** Bypasses peer review, automated tests, clippy linter, and formatting checks.
* **Corrected Approach:** Always cut a topic branch from `origin/dev` (`fix/compiler-parser-panic`) and open a PR targeting `dev`.

### Anti-Pattern 2: The Stale 60-Day "Megabranch"
* **Flawed Practice:** A developer keeps a private branch `feat/complete-type-system` for two months without merging anything.
* **Failure Mode:** `dev` advances by hundreds of commits; merge conflicts and regressions pile up.
* **Corrected Approach:** Break large overhauls into discrete GitHub Milestone issues (`GL-0008`) that merge incrementally into `dev`.

### Anti-Pattern 3: Ambiguous Branch Names
```bash
# FLAWED:
git checkout -b my-patch
git checkout -b bugfix
git checkout -b testing
```
* **Failure Mode:** Branch intent is opaque to teammates and automated CI tooling.
* **Corrected Approach:** Use standard category and registered scope: `fix/runtime-stack-underflow`.

---

## 7. Verification & Compliance Checklist

All engineers and maintainers **MUST** enforce the following criteria:

- [ ] **Branch Taxonomy:** Branch name adheres to `<category>/<scope>-<short-description>`.
- [ ] **Dev Base & Target:** Topic branch cuts from `origin/dev` and opens PR targeting `dev`.
- [ ] **Rebased Prior to Merge:** Branch is up to date with `origin/dev` before merging.
- [ ] **No Force Pushes to Shared:** Shared `dev` and `master` trunks never experience forced history rewrites.
- [ ] **Remote Deletion:** Topic branches are deleted immediately after PR merge.
