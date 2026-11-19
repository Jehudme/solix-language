# Changelog

All notable changes to the Solix Programming Language will be documented in this file.

## [1.5.1] - 2026-11-18

### Bug Fixes

- *(ci,docs)* Filter .sha256 files when downloading previous release archive
- *(ci,docs)* Stage cached bundled libs before release post-bump tests
- *(ci)* Suppress informational trusted-publishing zizmor notice until crates.io OIDC setup
- *(ci)* Satisfy CodeQL cache-poisoning and shellcheck checks in release.yml
- *(benchmarks,release)* Eliminate CI host variance in release comparisons and remove all emojis

### Miscellaneous Tasks

- Ci,chore(workflows): add caching, same-host benchmarks, single release workflow & clean Actions UI
- Docs,chore: relocate guidelines, remove archived docs/benchmarks, document solix wiki
- *(deps)* Bump dashmap from 5.5.3 to 6.2.1

### Refactoring

- *(scripts,governance)* Simplify sensitive notice, strip emojis, fix Cargo.toml trigger

## [1.4.0] - 2026-11-17



## [1.3.1] - 2026-11-17

### Bug Fixes

- *(workspace)* Eliminate production unwrap, expect, panic, and unreachable calls

### Miscellaneous Tasks

- *(cargo)* Remove unused manifest fields and crate dependencies

### Style

- *(workspace)* Apply cargo fmt across all crates


