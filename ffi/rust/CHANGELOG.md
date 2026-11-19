# Changelog

All notable changes to the Solix Programming Language will be documented in this file.

## [1.6.2] - 2026-11-19

### Bug Fixes

- *(build,install)* Fix YAML merge corruption, embed universal libs, validate dep platforms & auto-override defaults (#83)

## [1.6.1] - 2026-11-19

### Bug Fixes

- *(release)* Skip incompatible host install during cross-target library staging

### Documentation

- *(ffi)* Note SDK 1.6.0+ scope in native_interop spec

## [1.6.0] - 2026-11-19

### Bug Fixes

- *(optimizer)* Preserve node_types across AST inliner hygienic renaming (#67)

### Build System

- *(packages)* Ship release-only bundled default libraries

### Continuous Integration

- Fix pre-tag FFI git tag in release verification and LSP RPC shutdown race (#78)
- *(coverage)* Fix SC2155 shellcheck warning on SOLIX_BIN export
- *(governance)* Enforce PR-only dev merges, skip CI on sync-dev, auto-merge passing dependabot PRs

### Documentation

- Add FFI SDK wiki, manifest.yaml wiki, new native docs, and build guide (#77)
- *(specs)* Deduplicate docs hierarchy, add .slxasm guide, upgrade wiki tree sidebar
- *(release,lsp)* Remove invalid external URLs, broken Homebrew instructions, and duplicate changelog header (#41)

### Features

- *(cli)* Add pure-Rust solix new and solix new native for C/C++/Rust (#75)
- *(build)* Migrate to declarative manifest.yaml and pure-Rust native builder (#69) (#74)
- *(ffi)* Create top-level ffi/ (ffi/rust + ffi/c CMake SDK) with release train integration (#73)
- *(toolchain)* Enforce release platform constraints, supported_platforms check, and pre-flight verification

### Miscellaneous Tasks

- *(vscode)* Isolate build artifacts and add 3-layer headless test suite
- *(deps)* Bump @types/vscode in /editors/vscode

### Performance

- *(vm,toolchain)* Pre-decode scalar loops, zip-compress wiki pack, fix strict clippy and doc sync
- *(toolchain)* Deduplicate unqualified native libs when target-qualified lib exists in .slxarc (#42)
- *(runtime,optimizer)* Add load-time scalar loop flag, unboxed i32 loop executor, and post-LICM mask fusion (#43)
- *(vm)* Add scalar ForLoopI32 executor, self-tail-call fast path, inline Null EqRef/NeqRef, and inline string compare (#43)
- *(optimizer)* Hybrid accumulator loop for binary self-recursion, extend AddBandI32 fusion, and refine ARC alias elision (#43)
- *(toolchain,compiler)* Pack embedded assets into target-filtered blobs and fast-path solix run compilation (#42, #43)

### Refactoring

- *(ci)* Organize scripts and add unified benchmark cli and pr automation

### Testing

- *(ffi)* Add dual-backend example showcase, FFI matrix suite runner, and CI gate (#76)
- *(qa)* Add differential proptest fuzzing, golden .slxasm snapshots, corruption fuzzers, ARC torture & coverage gate

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


