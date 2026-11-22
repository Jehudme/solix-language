# Changelog

All notable changes to the Solix C/C++ FFI SDK (`solix::ffi`) will be documented in this file.

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

### Features

- Introduce the official `ffi/c` CMake SDK (`solix_ffi` / `solix::ffi`) and `solix_ffi.h` supporting C11 and C++17 with full `SolixCValue` and `SolixVmContext` parity.
