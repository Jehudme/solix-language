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

This specification formalizes the **Two-Wave Release Train & Milestone Branching Model (`dev -> master`)** for the Solix project. It establishes **`dev`** as the protected contributor integration trunk, **`master`** as the release-only production trunk fed exclusively by the authorized release PR from `dev` ([GL-0014](pr-and-release-governance.md)), strict up-to-date branch checks to eliminate merge skew, and auditable Git history.

---

## 2. Normative Invariants & Core Principles

All contributors and repository administrators **MUST** comply with the following normative rules:

1. **Two-Wave Trunk Invariant (`dev` -> `master`):**
   * **`dev` (Wave 1 Integration Trunk):** All contributor topic branches, bug fixes, Dependabot PRs, and documentation updates **MUST** target `dev` via pull request. Direct unreviewed pushes to `dev` are prohibited.
   * **`master` (Wave 2 Production Release Trunk):** Direct pushes and direct feature/bugfix PRs targeting `master` are **STRICTLY PROHIBITED**. Only same-repository PRs originating from `dev` (or sanctioned `release-plz/*` / `release/vX.Y.Z` release branches) pass `Validate Release PR Source`. Merging into `master` cuts a new SemVer release, tags `vX.Y.Z`, publishes crates and release binaries, and back-syncs the version bump commit to `dev`.
2. **Strict Up-to-Date Branch Protection (Zero Merge-Skew):** Repository rulesets on `dev` **MUST** enable *"Require branches to be up to date before merging"* so concurrent PRs are re-tested against the latest `dev` commit before landing.
3. **Short-Lived Topic Branches:** Individual feature and bugfix branches **MUST** branch from `origin/dev` and stay short-lived (target lifecycle 1–3 business days).
4. **Canonical Branch Naming Grammar:** Topic branch names **MUST** adhere to `<category>/<scope>-<short-description>` (`feat`, `fix`, `spec`, `refactor`, `docs`, `chore`, `test`, `milestone`).
5. **Admin & CODEOWNER Approval:** Pull requests touching protected paths in `.github/CODEOWNERS` trigger a non-blocking risk advisory (`scripts/pr_risk_audit.py`) and require maintainer (`@Jehudme`) approval before merging.
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

### 3.2 Two-Wave Branch Topology Diagram

```
master ──●────────────────────────────────────────────────────────●──► (Production Releases Only)
          \                                                      /  ^
           \ (Protected release train via dev -> master PR)     /   (Back-sync release tag/bump)
dev    ─────●─────────●──────────●────────────●────────────────●─────────► (Wave 1 Integration)
                       \        / \          /
                        ●──────●   ●────────●
                      feat/topic-a  fix/topic-b
```

### 3.3 Protected Branch Configuration Rules

Repository Rulesets **MUST** remain active on both **`dev`** and **`master`** ([GL-0014](pr-and-release-governance.md)):
* **`dev` Ruleset:** Require pull request before merging, require CODEOWNER (`@Jehudme`) approval on sensitive files, dismiss stale approvals, **require branches to be up to date before merging** (`strict`), require all Wave 1 CI + security status checks, and block force pushes.
* **`master` Ruleset:** Require pull request before merging, require **`Validate Release PR Source`** (`head == dev` or sanctioned internal release branch, non-fork), restrict direct pushes/merges to admin/release train only, and block force pushes.

---

## 4. Production-Ready Template

Contributors **MUST** execute the following sequence when creating, maintaining, and merging branches:

```bash
# ----------------------------------------------------------------------------
# Scenario A: Working on a Task within an Active Milestone
# ----------------------------------------------------------------------------

# 1. Synchronize local workspace with remote milestone branch
git checkout milestone/<XXX>-<slug>
git fetch origin --prune
git rebase origin/milestone/<XXX>-<slug>

# 2. Cut a new topic branch from the milestone branch
git checkout -b <category>/<scope>-<short-description>

# 3. Work atomically with Conventional Commits
git add <files>
git commit -m "<type>(<scope>): <imperative description>"

# 4. Keep topic branch up to date with milestone branch
git fetch origin
git rebase origin/milestone/<XXX>-<slug>

# 5. Push topic branch and open PR targeting milestone/<XXX>-<slug>
git push -u origin <category>/<scope>-<short-description>

# 6. Post-Approval Non-Fast-Forward Merge into Milestone Branch
git checkout milestone/<XXX>-<slug>
git merge --no-ff <category>/<scope>-<short-description> \
  -m "Merge pull request #<PR_NUMBER> from <category>/<scope>-<short-description>"
git push origin milestone/<XXX>-<slug>

# 7. Delete local and remote topic branch
git branch -d <category>/<scope>-<short-description>
git push origin --delete <category>/<scope>-<short-description>

# ----------------------------------------------------------------------------
# Scenario B: Closing Milestone and Merging into Master
# ----------------------------------------------------------------------------
git checkout master
git fetch origin
git rebase origin/master
git merge --no-ff milestone/<XXX>-<slug> \
  -m "Merge milestone/<XXX>-<slug> into master (MS-<XXX>: <Milestone Title>)"
git push origin master
git branch -d milestone/<XXX>-<slug>
git push origin --delete milestone/<XXX>-<slug>
```

---

## 5. Concrete Production Example

### Lifecycle of `feat/compiler-tuple-primitive` within `milestone/003-ast-binder`

#### 1. Inception & Branching
```bash
git checkout milestone/003-ast-binder
git pull --rebase origin milestone/003-ast-binder
git checkout -b feat/compiler-tuple-primitive
```

#### 2. Atomic Conventional Commits
```bash
# Commit 1: Update specification first (GL-0010 zero-drift)
git add docs/internals/specs/statements/declarations/tuple_primitive.md
git commit -m "docs(specs): specify tuple primitive syntax, AST nodes, and typing"

# Commit 2: Add AST node and token support in solix-common
git add crates/solix-common/src/ast/types.rs
git commit -m "feat(common): define TupleType AST node and token discriminants"

# Commit 3: Implement tuple parsing in solix-compilation
git add crates/solix-compilation/src/parser/types.rs
git commit -m "feat(compiler): parse tuple type declarations and destructuring"

# Commit 4: Add unit tests at bottom of types.rs
git add crates/solix-compilation/src/parser/types.rs
git commit -m "test(compiler): add unit tests for tuple type parsing"
```

#### 3. PR Review & Merge to Milestone Branch
The PR is opened against `milestone/003-ast-binder`, passes all CI checks, and is approved.
The branch is merged with `--no-ff`:

```bash
git checkout milestone/003-ast-binder
git pull --rebase origin milestone/003-ast-binder
git merge --no-ff feat/compiler-tuple-primitive \
  -m "Merge pull request #45 from feat/compiler-tuple-primitive"
git push origin milestone/003-ast-binder
git push origin --delete feat/compiler-tuple-primitive
```

---

## 6. Anti-Patterns & Counter-Examples (What NOT to Do)

### Anti-Pattern 1: Direct Pushes to `master`
```bash
# CATASTROPHIC FLAW:
git checkout master
git commit -am "quick fix for compiler parser panic"
git push origin master
```
* **Failure Mode:** Bypasses peer review, automated tests, clippy linter, and formatting checks. If the commit contains a subtle regression, all contributors on the team are blocked immediately.
* **Corrected Approach:** Always cut a topic branch (`fix/compiler-parser-panic`) and open a PR through CI verification.

### Anti-Pattern 2: The Stale 60-Day "Megabranch"
* **Flawed Practice:** A developer keeps a private branch `feat/complete-type-system` for two months without merging anything.
* **Failure Mode:** `master` and milestone branches advance by hundreds of commits. When finally merged, merge conflicts are insurmountable, and subtle semantic regressions are impossible to isolate.
* **Corrected Approach:** Break large architectural overhauls into discrete, incremental milestone tasks that merge every 24–72 hours into the milestone branch.

### Anti-Pattern 3: Ambiguous Branch Names
```bash
# FLAWED:
git checkout -b my-patch
git checkout -b bugfix
git checkout -b testing
```
* **Failure Mode:** Branch intent is opaque to teammates and automated CI tooling. Impossible to map to crates, issues, or milestones.
* **Corrected Approach:** Use standard category and registered scope: `fix/runtime-stack-underflow`.

---

## 7. Verification & Compliance Checklist

All engineers and maintainers **MUST** enforce the following criteria:

- [ ] **Branch Taxonomy:** Branch name adheres to `<category>/<scope>-<short-description>`.
- [ ] **Milestone Isolation:** Scoped milestone tasks target the active `milestone/XXX-...` branch.
- [ ] **Rebased Prior to Merge:** Branch is rebased cleanly on its target branch before merging.
- [ ] **Non-Fast-Forward Merge:** All merges execute with `--no-ff` (merge commit generated).
- [ ] **No Force Pushes to Shared:** Shared branches never experience forced history rewrites.
- [ ] **Remote Deletion:** Topic branches are deleted immediately upon PR merge.
- [ ] **Clean Git Graph:** `git log --graph` reveals clearly bounded merge commit envelopes.
