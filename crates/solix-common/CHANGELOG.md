# Changelog

All notable changes to the Solix Programming Language will be documented in this file.

## [1.3.1] - 2026-11-17

### Bug Fixes

- *(runtime,common)* Fix call window overlap, worklist drop, and atomic cache
- *(workspace)* Eliminate production unwrap, expect, panic, and unreachable calls
- *(runtime,lsp)* Add canonical triple matching and hermetic LSP tests
- *(windows,macos,ci)* Normalize verbatim paths, link macOS frameworks, and expose build stderr
- *(common,lsp)* Isolate store cache, ignore .tmp native extraction files, and normalize LSP breakout paths
- *(common,runtime,build)* Match Windows host ABI deterministically and resolve exact native checksums

### Miscellaneous Tasks

- *(cargo)* Remove unused manifest fields and crate dependencies

### Performance

- *(common)* Inline instruction extractors and add OpCode::from_raw_u8
- *(compilation,runtime)* Add MulI32Imm/ModI32Imm, copy prop, and call-loop hoisting
- *(common,compilation,runtime)* Add ForLoopI32, AddBandI32, JumpIfI32Imm, and FLAG_MOVE ARC store fusion
- *(vm,optimizer)* Add tail-call, mul-add, array-fill superinstructions and save MS-061 baseline

### Refactoring

- *(workspace)* Replace single-letter identifiers with descriptive GL-0011 names

### Style

- *(workspace)* Apply cargo fmt across all crates


