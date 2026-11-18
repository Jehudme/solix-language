# Contributing to the Solix Programming Language

Thank you for contributing to Solix! All contributions follow our **Two-Wave Release Train (`dev -> master`)** and engineering quality standards so every commit cleanly compiles, passes strict linting, keeps documentation in sync, and stays secure across Linux, macOS, and Windows.

---

## 1. Branching & Pull Request Workflow (`GL-0002`, `GL-0014`)

1. **Always Branch From `dev` (Never Target `master` Directly):**
   * `dev` is the active integration branch where all features, bug fixes, docs updates, and dependency bumps land first.
   * Pull requests opened directly against `master` from contributor branches or forks are automatically blocked by `Validate Release PR Source`. Only the maintained `dev -> master` release train merges into `master`.
2. **Fork / Clone & Cut a Topic Branch:**
   ```bash
   git fetch origin
   git checkout -b feat/short-description origin/dev
   ```
3. **Keep Your Branch Up to Date with `dev`:**
   * The `dev` ruleset requires branches to be up to date with `dev` before merging (preventing hidden integration conflicts when concurrent PRs land).
4. **Target `dev` & Link Issues:**
   * Reference any GitHub Milestone issue (`Closes #<issue-number>`) in your PR description using [`.github/pull_request_template.md`](.github/pull_request_template.md).

---

## 2. Commit Message Format (`GL-0001`)

Use **Conventional Commits** (`<type>(<scope>): <imperative lowercase description>`):
* `fix(...)`, `perf(...)`: increments **Patch** (`v1.X.Y+1`)
* `feat(...)`: increments **Minor** (`v1.X+1.0`)
* `<type>(...)!` + `BREAKING CHANGE:` footer: increments **Major** (`vX+1.0.0`)
* `docs(...)`, `test(...)`, `refactor(...)`, `ci(...)`, `build(...)`, `chore(...)`: non-breaking housekeeping updates

---

## 3. Required Local Pre-Flight Checks Before Opening a PR

Run these checks locally prior to pushing:

```bash
# 1. Rust formatting & strict Clippy (zero unwrap/expect/panic, no single-letter names in --lib --bins)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --exclude solix-tests --lib --bins --no-deps -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::min_ident_chars

# 2. Rebuild offline `solix wiki` static docs + run full workspace tests
python3 scripts/build_docs_site.py
cargo test --workspace --all-targets

# 3. Documentation sync, GL-0011 (zero single-letter code-block vars), and Python test suite
python3 scripts/doc_sync_check.py --base origin/dev
python3 scripts/doc_sync_check.py --all-docs
python3 scripts/verify_docs.py
python3 scripts/verify_doc_links.py
python3 -m unittest discover -s scripts/tests -v
```

> [!NOTE]
> Look closely at `GL-0011`: code snippets in Markdown docs and core Rust crates **MUST NOT** declare single-letter variable names (`index`, `counter`, `value`, `element`, etc. instead of `i`, `x`, `n`, `v`). Also **NEVER** add `unsafe` blocks outside strictly audited `crates/solix-ffi/`.

---

## 4. Sensitive Files & Maintainer Review (`.github/CODEOWNERS`)

Changes touching `.github/`, `docs/internals/guidelines/`, `Cargo.toml`/`Cargo.lock`, `deny.toml`, `scripts/`, `build.rs`, bundled toolchain assets, or archive/native/FFI security boundaries are **welcome**, but automatically display a non-blocking **Sensitive-Path Reviewer Advisory** and require maintainer (`@Jehudme`) CODEOWNER review before merging.

---

## 5. Authoring Tasks & Browsing Documentation

* **Milestones & Tasks (`GL-0008`):** Report bugs or propose milestone tasks using our GitHub Issue Forms under [`.github/ISSUE_TEMPLATE/`](.github/ISSUE_TEMPLATE/milestone_task.yml) linked to the active GitHub Milestone (`v1.5.0`, etc.).
* **Offline Docs (`solix wiki`):** Browse documentation matching your installed `solix` binary offline at any time:
  ```bash
  solix wiki
  ```
