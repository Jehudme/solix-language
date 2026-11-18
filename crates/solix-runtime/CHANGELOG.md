# Changelog

All notable changes to the Solix Programming Language will be documented in this file.

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

- *(runtime,common)* Fix call window overlap, worklist drop, and atomic cache
- *(compilation,runtime)* Prune borrowed reference_registers, track UnpackCaptures, and relocate multi-binary itables
- *(common,runtime,build)* Match Windows host ABI deterministically and resolve exact native checksums

### Features

- *(ms061)* Finish CLI fast-path, telemetry active_guest_objects, MS-061 tests, and sound ARC/array edge fixes

### Miscellaneous Tasks

- *(cargo)* Remove unused manifest fields and crate dependencies

### Performance

- *(runtime)* Eliminate dispatch spills, drop glue, and closure allocs
- *(runtime)* Outline cold opcodes to shrink fast interpreter frame
- *(runtime)* Inline Value hot helpers and return frame window from push_frame_window
- *(runtime)* Use compact VmTrap ABI, in-place Value access, and func_cache
- *(compilation,runtime)* Add MulI32Imm/ModI32Imm, copy prop, and call-loop hoisting
- *(compilation,runtime)* Omit discarded assign Mov, specialize ArrayRef, flatten vtable/itable
- *(compilation,runtime)* Inline local closures/ctors, elide redundant ARC, inline fast ARC ops
- *(runtime,toolchain)* Demand-page heap, shrink register stack, lazy run setup
- *(runtime)* Compact 32B CallFrame, L1i-fit dispatch loop, idiv-free modulo
- *(runtime)* Unify inline fast call dispatch across CallDirect/Virtual/Interface/Callable
- *(runtime)* Eliminate Vec::push call spills in inline call dispatch
- *(runtime)* Gate metrics tracking, remove RuntimeError trap slots, and specialize Bool/I32 array ops
- *(compilation,runtime)* Pack typed arrays, eliminate Value phi spills, and expand MOD_MAGIC_TABLE
- *(common,compilation,runtime)* Add ForLoopI32, AddBandI32, JumpIfI32Imm, and FLAG_MOVE ARC store fusion
- *(runtime)* Cache inst_ptr in CallFrame, add sentinel Return, and inline trivial leaf calls
- *(vm,optimizer)* Add tail-call, mul-add, array-fill superinstructions and save MS-061 baseline

### Refactoring

- *(workspace)* Replace single-letter identifiers with descriptive GL-0011 names

### Style

- *(ms059)* Format modified compiler and runtime files with rustfmt
- *(workspace)* Apply cargo fmt across all crates


