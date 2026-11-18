## Pull Request Summary

<!-- Briefly describe what this PR changes and link the GitHub Issue / Milestone task (`Closes #<issue>`). -->

- **Target Branch Check:**
  - [ ] This PR targets `dev` (required for all features, bug fixes, docs, and dependency bumps).
  - [ ] *(Maintainer release train only)* This is the official `dev -> master` Release PR.

## Verification Checklist

- [ ] `cargo fmt --all -- --check` passes cleanly
- [ ] `cargo clippy --workspace --exclude solix-tests --lib --bins --no-deps -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::min_ident_chars` passes cleanly
- [ ] `cargo test --workspace --all-targets` passes cleanly
- [ ] `python3 scripts/doc_sync_check.py --base origin/dev` and `--all-docs` pass (`GL-0011` / `GL-0013`)
- [ ] `python3 scripts/verify_docs.py` passes cleanly

## Sensitive-Path Disclosure (GL-0014)

If this PR modifies `.github/`, `docs/internals/guidelines/`, `scripts/`, `Cargo.toml`/`Cargo.lock`, `deny.toml`, `**/build.rs`, `crates/solix-toolchain/assets/`, or runtime security/FFI files, check below and briefly explain why the change is safe:

- [ ] Touches sensitive CI, governance, supply-chain, or security files (maintainer `@Jehudme` review required)
- **Safety Rationale:**
