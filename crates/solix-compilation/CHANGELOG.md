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

## [1.5.0] - 2026-11-18

### Bug Fixes

- *(security)* Update rustix to 1.1.6 and harden cargo-audit CI step
- *(security)* Upgrade zip to 9.7.0 and resolve cargo-deny wildcard and advisory gates
- *(common)* Harden slxarc extraction and staging against Zip-Slip traversal

### Continuous Integration

- *(governance)* Add bootstrap fallback to Validate Release PR Source
- *(security)* Resolve all zizmor findings, add dependabot cooldown, and drop workflow_run
- *(governance)* Fix shellcheck actionlint rules and remove PyYAML test dependency
- *(governance)* Add two-wave dev-to-master train, security checks, and risk audits

### Documentation

- *(milestone)* Mark MS-063 tasks and verification sign-off completed

### Features

- *(toolchain)* Add offline versioned solix wiki portal and static docs builder

## [1.4.0] - 2026-11-17

### Bug Fixes

- *(compilation,runtime,toolchain)* Fix static linking relocation, native bundling, and test profile entrypoints



## [1.3.1] - 2026-11-17

### Bug Fixes

- *(compiler)* Preserve reference registers across throwing ops
- *(workspace)* Eliminate production unwrap, expect, panic, and unreachable calls
- *(tests)* Remove redundant borrow in optimizer assertion
- *(compilation,runtime)* Prune borrowed reference_registers, track UnpackCaptures, and relocate multi-binary itables

### Features

- *(ms061)* Finish CLI fast-path, telemetry active_guest_objects, MS-061 tests, and sound ARC/array edge fixes

### Miscellaneous Tasks

- *(clippy)* Satisfy Rust 1.100 needless_range_loop and needless_bool lints
- *(cargo)* Remove unused manifest fields and crate dependencies

### Performance

- *(compiler)* Add liveness peephole fusion and loop constant hoisting
- *(compiler)* Eliminate assignment Movs and devirtualize concrete calls
- *(compiler)* Enable loop-scoped SRA, getter forwarding, and window safety
- *(compilation,runtime)* Add MulI32Imm/ModI32Imm, copy prop, and call-loop hoisting
- *(compilation,runtime)* Omit discarded assign Mov, specialize ArrayRef, flatten vtable/itable
- *(compilation,runtime)* Inline local closures/ctors, elide redundant ARC, inline fast ARC ops
- *(compilation)* Fix owned-local ARC return leak and loop ARC/copy redundancy
- *(compilation)* Rotate single-entry loops to bottom-tested conditional backedges
- *(compilation,runtime)* Pack typed arrays, eliminate Value phi spills, and expand MOD_MAGIC_TABLE
- *(common,compilation,runtime)* Add ForLoopI32, AddBandI32, JumpIfI32Imm, and FLAG_MOVE ARC store fusion
- *(compilation)* Add EBB CSE, entry const prop, loop hoisting, diamond specialization, and leaf inlining
- *(vm,optimizer)* Add tail-call, mul-add, array-fill superinstructions and save MS-061 baseline

### Refactoring

- *(workspace)* Replace single-letter identifiers with descriptive GL-0011 names

### Style

- *(ms059)* Format modified compiler and runtime files with rustfmt
- *(workspace)* Apply cargo fmt across all crates


