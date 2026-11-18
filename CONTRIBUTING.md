# Contributing to Solix

Thank you for contributing to the **Solix Programming Language**! We welcome bug fixes, language features, documentation improvements, tests, and tooling enhancements.

---

## Quick Start

1. **Branch from `dev`:** Create a short-lived topic branch (`feat/...`, `fix/...`, `docs/...`, `refactor/...`) from `origin/dev` and always open your pull request targeting **`dev`**.
   ```bash
   git fetch origin
   git checkout -b feat/short-description origin/dev
   ```
2. **Use Conventional Commits:** Prefix commits as `<type>(<scope>): <imperative description>` (e.g., `feat(compiler): ...`, `fix(runtime): ...`, `docs(wiki): ...`).
3. **Verify Locally Before Opening a PR:**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace --all-targets
   python3 scripts/doc_sync_check.py --base origin/dev
   python3 scripts/verify_docs.py
   python3 -m unittest discover -s scripts/tests -v
   ```
4. **Link Issues & Sensitive Files:** Reference tracked issues (`Closes #<issue-number>`) in [`.github/pull_request_template.md`](.github/pull_request_template.md). Changes to [`.github/CODEOWNERS`](.github/CODEOWNERS)-protected paths are welcome and automatically flag maintainer (`@Jehudme`) review.

---

## Engineering Guidelines

Detailed standards for commits, branching, testing, documentation, Rust conventions, and releases live in the [Engineering Guidelines Suite](docs/guidelines/README.md):

| Standard | Topic | Link |
| :--- | :--- | :--- |
| **`GL-0001`** | Commit Messages & SemVer | [Commit Conventions](docs/guidelines/authoring-commits.md) |
| **`GL-0002`** | Branching & Integration Topology | [Branching Conventions](docs/guidelines/branching-conventions.md) |
| **`GL-0008`** | Milestones & Definition of Done | [Milestones & DoD](docs/guidelines/authoring-milestones.md) |
| **`GL-0009`** | Test Matrices & Coverage | [Test Authoring](docs/guidelines/authoring-test.md) |
| **`GL-0011`** | Rust Safety & Naming (`GL-0011`) | [Language Conventions](docs/guidelines/language-conventions.md) |
| **`GL-0013`** | Code-to-Docs Sync Matrix | [Doc Sync Protocol](docs/guidelines/documentation-update-protocol.md) |
| **`GL-0014`** | CI, Security & Release Governance | [Release Governance](docs/guidelines/pr-and-release-governance.md) |
