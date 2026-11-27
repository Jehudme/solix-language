# Changelog

All notable changes to the Solix Programming Language will be documented in this file.

## [3.1.0] - 2026-11-27

### Bug Fixes

- *(vscode,lsp,dap)* Dynamic manifest profiles, v3 compatibility, formatter/hints/evaluator fixes, and v3 snippets (#316)
- *(lsp)* Enforce cached-dep import parity, filter enum completions, and expand quick fixes (#315)
- *(solixlib, raylib, compiler)* Harden stdlib net/json/collections, overload resolution, and Raylib OOP ergonomics (#314)
- *(ffi,stdlib)* Harden C-unwind FFI ABI, Raylib bindgen, and solixlib_native safety (#313)
- *(runtime,compiler)* VM/ARC hardening, weak generation checks, and compiler edge-case safety (#292, #300) (#312)
- *(optimizer,runtime,common)* CFG escape analysis, checked relocations, strict SLXB codecs & AOT verifier bounds (#290, #291) (#311)
- Harden lexer/escape validation, AST inliner & call normalization, parser sync & call recovery perf (#289, #303, #306) (#310)
- Resolve diagnostics & docs parity (#295, #298) with CI pipeline optimization (#283, #284, #286) (#308)

### Features

- *(bench)* Precompile Lua with luac, streamline cross-language & self-comparison tables, scale workloads, and render host telemetry (#318)
- *(cli,lsp,dap)* Fix fmt/dap/rename gaps, pure-Solix scaffolding, cwd run, and check/test/init/add (#288, #301, #305) (#317)

## [3.0.1] - 2026-11-26

### Bug Fixes

- *(release)* Limit pre-publish cargo package dry-run to leaf crates (#279) (#281)
- *(release)* Sync solix-ffi version and verify pre-promotion crate parity (#279) (#280)

## [3.0.0] - 2026-11-26

### Bug Fixes

- *(tests)* Handle windows-msvc host triple in multi-platform resolution test
- *(release)* Split macOS Xcode and Docker Linux/Windows bundled builds
- *(toolchain)* Resolve default solix dependency from bundled manifest
- *(manifest)* Provide line, column, and contextual diagnostics for manifest parse errors (#247) (#268)
- *(templates)* Migrate project templates and examples to clean manifest.yaml (#246) (#267)
- *(vscode)* Synchronize extension versioning and honor force flag (#264) (#265)

### Features

- *(toolchain)* Containerize apple darwin cross-compilation everywhere (#276) (#277)
- *(toolchain)* Mandate docker cross-compilation for native library builds (#253, #251) (#271)
- *(bench)* Add java runtime and compiler to cross-language benchmark suite (#245) (#270)
- *(compiler)* Improve parser recovery, vscode discovery, and conditional checksums (#248, #249, #250) (#269)
- *(toolchain)* Scaffold clean manifest with default solix dependency (#244) (#266)

### Refactoring

- *(raylib)* Emit root pascalcase functions and use solix-ffi (#272) (#273)

## [2.0.1] - 2026-11-25

### Bug Fixes

- *(tests)* Synchronize C-FFI header version assertions dynamically with package version (#262)
- *(compiler)* Resolve eleventh-sweep binder, LSP, DAP, VM, fmt, and stdlib edge cases (#251-#260) (#261)

## [2.0.0] - 2026-11-24

### Bug Fixes

- *(release)* Synchronize post-bump bundled library build and dynamic test assertion (#242)
- *(ci)* Fix dev release workflow binary targets and actionlint shellcheck warnings (#241)
- *(compiler)* Resolve tenth-sweep binder, LSP, DAP, VM, and stdlib edge cases (#229-#234) (#235)
- *(compiler)* Resolve ninth-sweep binder, LSP, VM, and stdlib edge cases (#223-#227) (#228)
- *(compiler)* Resolve eighth-sweep verifier, binder, DAP, and stdlib edge cases (#217-#221) (#222)
- *(compiler)* Resolve seventh-sweep verifier, binder, toolchain, DAP, and stdlib edge cases (#210-#215) (#216)
- *(compiler)* Resolve sixth-sweep verifier, binder, FFI, and stdlib edge cases (#203-#207) (#209)
- *(compiler)* Resolve fifth-sweep OOP, verifier, cycle collector, and stdlib edge cases (#194-#201) (#202)
- *(compiler)* Resolve fourth-sweep memory safety, arithmetic, and parser edge cases (#183-#192) (#193)
- *(compiler)* Resolve third-sweep compiler, runtime, and stdlib edge cases (#171-#180) (#181)
- *(compiler)* Resolve second-sweep architectural and safety edge cases (#170)
- *(compiler)* Resolve control flow, finally scoping, and lambda inference bugs (#160)

### Features

- *(ci)* Implement independent versioning, VS Code compatibility, PR telemetry, and dev releases (#236-#239) (#240)
- *(ci)* Synchronize PR discussion comments and labels for external forks (#182)
- *(toolchain)* Unify native platforms, eliminate lib/ directory, and add pre-compiled library verification (#151)

### Miscellaneous Tasks

- *(deps)* Bump @vscode/vsce from 4.1.0 to 4.1.1 in /editors/vscode (#208)

## [1.9.0] - 2026-11-22

### Features

- *(toolchain)* Platform compatibility visibility and universal release target alignment (#144) (#145)

## [1.8.0] - 2026-11-22

### Bug Fixes

- *(doom)* Add working directory fallback for asset loading (#141)
- *(doom)* Resolve player elevation, authentic sector floors/ceilings, and hitscan damage (#140)
- *(build)* Isolate container cache directories and constrain raylib native platforms (#133)

### Features

- *(doom)* Authentic Doom 1 asset pipeline, textured 3D rendering & natural controls (#139)
- *(examples)* Implement modern 3d retro doom fps showcase game (#137)
- *(examples)* Implement modern modular arcade space invaders showcase game with audio (#135)

### Miscellaneous Tasks

- *(examples)* Remove doom showcase game and retain compiler reachability fix (#142)

## [1.7.0] - 2026-11-20

### Bug Fixes

- *(build)* Isolate cross-compilation toolchain and prevent host header leaks in raylib zigbuild (#118)
- *(toolchain)* Resolve Windows executable resolution and .cmd spawning error 193 (#116)
- *(toolchain)* Fix pure-solix package cache check, uninstall prefix collision, and doc HTML escaping
- *(parser)* Reject trailing characters in escaped char literals (closes #101)
- *(lexer)* Enforce strict newlines, trailing decimal floats, and reject leading zeros (closes #100)
- *(runtime)* Allow clippy::expect_used on backward-compatible VirtualMachine::new constructor
- *(runtime)* Propagate string allocation and memory-write errors during vm init (closes #99)
- *(optimizer)* Disambiguate ast inliner methods across namespaces (closes #98)
- *(binder)* Isolate monomorphizer cache and fix namespaced constructor resolution
- *(binder)* Ensure root package hierarchy access in symbol resolution
- *(binder)* Prevent generic class cache collisions across namespaces in monomorphizer

### Documentation

- Migrate to .slxp package format and synchronize specifications (#94, #96)
- *(pipeline)* Document namespaced class extraction and collapse clippy conditions

### Features

- *(bench)* Modernize benchmark suite with pre-compilation, hardware telemetry & iteration controls (#120)
- *(build)* Support containerized cross-compilation for native modules via cross and dockcross (#122)
- *(toolchain)* Default new project template release profile to platforms: all (#117)
- *(optimizer)* Wire 0xE0-0xED superinstruction peephole fusion passes (#93) (#113)
- *(vm)* Implement Category 2 & 3 superinstructions 0xE6..=0xED (fixes #92)
- *(vm)* Implement Category 1 superinstructions 0xE0..=0xE5 (fixes #91)
- *(cli)* Restructure CLI subcommands, add export, and retire reinstall (fixes #89, #90)
- *(package)* Modernize container format to .slxp and enforce strict native loader filtering (#87, #88) (#109)

### Performance

- *(vm)* Eliminate frame lookup overhead in fast dispatch and optimize polymorphic virtual calls (#121)

### Testing

- *(ci)* Implement end-to-end multi-platform command lifecycle pipeline & maintenance mandate (#119) (#129)
- *(bench)* Make test_solix_bench_telemetry_and_scaling_flags hermetic with mock binary
- *(toolchain)* Specify host platform for CalcCpp native module in test (#117)
- Strengthen test coverage across register bounds, escape analysis, and ARC paths (#102)

## [1.6.3] - 2026-11-19

### Build System

- *(manifests)* Target all desktop platforms in release profiles (#85)

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


