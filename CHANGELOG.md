# Changelog

All notable changes to the Solix programming language, compiler toolchain, runtime virtual machine, and ecosystem are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

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

## [1.3.1] - 2026-11-17

### VM Optimization, Bytecode Superinstructions & CI/CD Release Automation
* **Cross-Crate Opcode Inlining & Peephole Optimization (MS-059):**
  * Enabled cross-crate inlining for instruction extractors and dispatch helpers, outlined cold opcodes to shrink the fast interpreter frame, and introduced frame-window slicing optimizations.
  * Preserved reference registers cleanly across throwing operations in the emitter.
* **Compact Trap ABI, NaN-Boxed Value Access & Production Hardening (MS-060):**
  * Compacted `CallFrame` to 32 bytes, eliminated `RuntimeError` trap stack slots, and added single-entry bottom-tested loop rotation, copy propagation, call-loop hoisting, and `MulI32Imm` / `ModI32Imm` superinstructions.
  * Eliminated production `unwrap`, `expect`, `panic`, and `unreachable` calls across non-test workspace crates and standardized identifier hygiene (`GL-0011`).
* **Superinstructions, Zero-Spill Dispatch, Typed Arrays & Local CSE (MS-061):**
  * Packed typed `Bool`/`I32` arrays, added `ForLoopI32`, `AddBandI32`, `JumpIfI32Imm`, `FLAG_MOVE` ARC store fusion, tail-call / mul-add / array-fill superinstructions, EBB CSE, leaf-call inlining, and cached frame instruction pointers.
  * Published historical Milestone 061 benchmark baselines.
* **Cross-Platform CI/CD Overhaul, Hermetic Tests & Release Automation (MS-062):**
  * Consolidated CI/CD into two workflows (`ci.yml` + tag-triggered `release.yml`), removed standalone `release-plz.yml`, fixed `release-plz` authentication with `RELEASE_TOKEN`, enabled ordered crates.io publishing without `continue-on-error` masking, and removed duplicate `|| true` publishing steps.
  * Made `solix-lsp` breakout tests hermetic across Linux, macOS, and Windows, normalized Windows verbatim (`\\?\`) paths and UTF-8 benchmark streams, linked required macOS Apple frameworks for `raylib`, added ABI-aware host checksum matching (`msvc` vs `gnu`), and built host-native bundled `.slxarc` archives per release target.
  * Fixed `doc_sync_check.py` `TRIGGER_MATRIX` + `--base` CI diff gating and eliminated all workspace Cargo manifest warnings.

---

## [1.3.0] - 2026-11-16

### Dynamic Release Benchmarking, Cycle Collection, Expanded Switch & Standard Library
* **Universal Target Syntax & Documentation Generator (MS-053):**
  * Modernized package and target reference format to `<name>[:<profile>][@<version>]` across all CLI commands (`run`, `install`, `uninstall`, `info`).
  * Removed runtime `--test` and execution flags in favor of compile-time profile target artifacts (e.g. `build/test/project.slxarc`).
  * Added compile-time and runtime entrypoint validation rejecting non-existent entrypoints with actionable diagnostics (`[E0310]`).
  * Implemented built-in `solix doc` documentation generator producing responsive HTML/CSS API documentation trees with full markdown parsing.
  * Enhanced Language Server Protocol (`solix lsp`) and VS Code extension to present full markdown documentation across hover hints and IntelliSense proposals.
* **Expanded Pattern Matching & Equality Operators (MS-054):**
  * Expanded `switch` statements to support all primitive types (`int32`, `int64`, `float64`, `string`, `bool`).
  * Added direct support for enum discriminants and qualified variant cases (`case Enum::Variant:` and `case Variant:`).
  * Added class instance dispatch via `==` and `!=` operator overloading (`[E0320]`).
  * Introduced tuple pattern matching and type-testing expressions via `instanceof`.
* **Flow-Sensitive Escape Analysis, SRA & ARC Elision (MS-055):**
  * Implemented static inter-procedural flow-sensitive escape analysis discovering variable lifecycle boundaries.
  * Added Call Frame Stack Inlining (Scalar Replacement of Aggregates) with threshold sizing for non-escaping structures.
  * Implemented automatic reference count (ARC) increment and decrement elision for variables guaranteed not to escape local function scopes.
* **Standard Library Expansion, C++ `std::format` & VM Telemetry (MS-056):**
  * Introduced C++ `std::format`-style formatting engine in `solixlib` supporting positional indices, hex, octal, binary, floating-point precision, fill, and text alignment (`[E0331]`).
  * Added standard library modules: `solix.json` (`JsonValue`, `[E0332]`), `solix.net` (`TcpListener`, `TcpStream`, `UdpSocket`, `HttpClient`, `[E0333]`), and `solix.compress` (`Gzip`, `Zlib`, `Deflate`, `[E0334]`).
  * Implemented 56 VM telemetry metric queries in `solix.system.Vm` and `VmMetrics`.
  * Added `mount://` virtual filesystem redirection routing read-only IO directly into packaged `.slxpac` container zip resources (`[E0330]`).
* **Bacon-Rajan (2001) ARC Cycle Collector & Zero-Cost Opt-Out (MS-057):**
  * Implemented the Bacon & Rajan (2001) four-color cycle collection state machine (`Black`, `Gray`, `White`, `Purple`) in VM heap memory.
  * Added 3-phase trial deletion (`mark_gray`, `scan`, `collect_white`) collecting circular self-referential and multi-node reference graphs.
  * Added zero-cost runtime opt-out flag `--cycle-collector <on|off>`.
  * Configured configurable collection thresholds (`--cycle-threshold <N>`, `[E0341]`) and bounded microsecond budgets (`--cycle-budget-us <N>`, `[W0340]`).
* **Dynamic Release Benchmarking & CI/CD Performance Gates (MS-058):**
  * Replaced static baseline files with a dynamic release-to-release benchmarking pipeline.
  * Added `benchmarks/compare_releases.py` calculating geometric mean speedups, execution deltas, and RSS memory differences against preceding release assets.
  * Automated generation and injection of GitHub Flavored Markdown comparison tables into release notes.
  * Added mandatory `python3 benchmarks/runner.py --verify` correctness gate in `.github/workflows/ci.yml`.

---

## [1.2.0] - 2026-11-16

### Unified Configuration Manifest Schema, Profile-Scoped Sources & Automated CI/CD
* **Unified Project Manifest Architecture:**
  * Replaced legacy fragmented sections (`[info]`, `[entrypoints]`, `[build]`, `[profile.<name>]`, and root `[dependencies]`) with a modern, clean `[configuration]` and `[profiles]` schema.
  * Made `[profiles.release]` the only mandatory profile; any profile omitting `inherits` defaults to `release` as its parent automatically.
  * Introduced profile-scoped `sources` discovery, allowing profiles (e.g. `test`) to accumulate inherited source paths and append test directories preserving discovery order.
  * Introduced profile-scoped `dependencies` with table notation (`solix = { version = "2.0.0", profile = "release" }`), supporting hierarchical dependency inheritance and override.
* **Scaffolding & Native Extension Modernization:**
  * Modernized `solix new` and `solix_scaffold.py` to generate `release`, `debug`, and `test` profiles pre-configured.
  * Updated `--native` scaffolding to automatically provision a Rust bridge library in `natives/` configured with a dependency on `solix-ffi` (v1.2.0).
* **Toolchain & Compiler Hardening:**
  * Enhanced module solver profile-qualified resolution and authenticated extraction cache validation.
  * Hardened profile inheritance resolution with cycle detection, profile validation, and descriptive diagnostics.
  * Upgraded first-party standard library (`solixlib`), graphics library (`raylib`), and game examples (`breakout`) to modern schema.
* **Automated Crates.io CI/CD Pipeline:**
  * Configured GitHub Actions release workflows to automatically publish public library crates (`solix-ffi`, `solix-common`, `solix-runtime`, `solix-compilation`) using `CARGO_REGISTRY_TOKEN`.

---

## [1.1.3] - 2026-11-15

### Visual Studio Code Tooling, Compiler Diagnostic Parity & Language Intelligence
* **LSP Autocompletion with Signature Label Details:**
  * Enabled LSP 3.17 `CompletionItemLabelDetails` in code completion (`textDocument/completion`), presenting complete parameter signatures (e.g. `(Rectangle rec, float32 roundness, int32 segments, Color color)`) and return types (`void`, `int32`) on method proposals.
* **Resilient Live Signature Help During Active Typing:**
  * Implemented backward lexical invocation analysis (`find_enclosing_call_lexical`) to resolve active callee signatures, parameter names, and argument indices even when unclosed parentheses (`(`) prevent AST statement creation.
  * Preserved semantic snapshot symbol tables across intermediate typing states containing syntax errors, preventing inlay hint and signature help drops.
* **Semantic Token Highlight Resilience & Coordinate Precision:**
  * Resolved mid-word character fragment coloring during syntax errors by prioritizing call invocation syntax (`ident(` and `::ident(`) in lexical classification over PascalCase class heuristics.
  * Sanitized carriage return (`\r`) handling in UTF-16 column positioning, guaranteeing consistent visual column alignment across Linux, macOS, and Windows CRLF documents.
* **Compiler Diagnostic & Package Import Parity:**
  * Enforced strict package boundary validation in compiler Pass 1 and LSP background analysis: wildcard imports targeting undeclared external packages (such as `solix.*` or `raylib.*`) consistently emit `CannotResolveImport` (`E0232`), reserving automatic package resolution strictly to standard library modules (`std::*`) and built-in intrinsics (`blt::*`).
* **Visual Studio Code Disassembly & Build Tooling:**
  * Added one-click **Inspect 64-bit VM Disassembly** action bar button providing interactive disassembly viewing directly in the editor with custom syntax highlighting.
  * Added top action bar **Solix Build** button to execute project builds in an integrated terminal.
  * Added upward `manifest.toml` directory discovery to `solix compile`, allowing compilation of nested source files with dependency resolution without requiring manual `--deps` flags.
* **Compiler & Optimizer Remediation:**
  * Eliminated unsound loop-invariant code motion hoisting expressions across conditional blocks, preventing corrupt bytecode branch offsets.
  * Preserved formal parameter names across all compilation passes on `SymbolEntry`, exposing true declared parameter names in inlay hints and hover tooltips.

---

## [1.1.0] - 2026-11-15

### Multi-Platform Release Distribution & Universal Compatibility
* **Universal Multi-Platform Support Across All Systems & Linux Distributions:**
  * **Windows 64-bit (`x86_64-pc-windows-msvc`):** Native Windows executable (`solix.exe`), distribution archive (`.zip`), and automated PowerShell installer (`solix-toolchain-installer.ps1`).
  * **macOS Apple Silicon (`aarch64-apple-darwin`):** Native release archive for M1/M2/M3/M4 Macs.
  * **macOS Intel (`x86_64-apple-darwin`):** Native release archive for Intel Macs.
  * **Universal Linux Statically Linked (`x86_64-unknown-linux-musl`):** 100% statically linked binary with zero glibc dependency, running on all Linux distributions including Alpine Linux, Void Linux, Ubuntu 16.04–24.04, Debian 10–12, CentOS/RHEL/Rocky/AlmaLinux, Arch Linux, Fedora, openSUSE, and NixOS.
  * **Linux x86_64 GNU (`x86_64-unknown-linux-gnu`):** Standard GNU glibc distribution archive.
  * **Linux ARM64 GNU (`aarch64-unknown-linux-gnu`):** Native Linux ARM64 binary for Raspberry Pi 4/5, AWS Graviton, and ARM-based servers.
* **Release Artifacts & Installers:**
  * Universal Unix shell script installer (`solix-toolchain-installer.sh`) supporting macOS and Linux auto-detection.
  * Windows PowerShell installer (`solix-toolchain-installer.ps1`).
  * Homebrew formula (`solix-toolchain.rb`).
  * Direct standalone bundled first-party packages (`raylib.slxarc`, `solix.slxarc`).
  * SHA-256 checksum manifests (`sha256.sum`).

---

## [1.0.0] - 2026-12-01

### Summary
Initial production release of the **Solix Programming Language (v1.0.0)**. Solix is a high-performance, statically-typed, modern systems and application programming language combining expressive object-oriented and functional ergonomics with a 64-bit register-based virtual machine, deterministic automatic reference counting (ARC), a hermetic multi-profile package manager, generic two-pass C-API FFI bindings, and production-grade IDE tooling.

---

### Compiler & Language (`solix-compilation`)
* **Multi-Pass Compilation Pipeline:**
  * **Lexer (`phases/lexing`):** High-throughput UTF-8 token stream generation with rich span tracking and precise error diagnostics.
  * **Parser (`phases/parsing`):** Full 108-token LL(k) recursive-descent grammar parser supporting classes, interfaces, generic templates, operator overloads, destructors, closures, pattern matching, and exceptions.
  * **4-Pass Semantic Binder (`phases/binder`):**
    * *Pass 1 (Symbol Declaration):* Registers packages, classes, interfaces, functions, methods, fields, and type aliases.
    * *Pass 2 (Type Hierarchy):* Resolves inheritance, implements interface dispatch tables, assigns deterministic layout offsets, and generates virtual method tables (VMT).
    * *Pass 3 (Type Checking):* Enforces static typing, generic instantiation, visibility rules, and operator contracts.
    * *Pass 4 (Semantic Validation):* Validates definite assignment, control-flow reachability, ARC borrow semantics, and exception dispatch.
* **Context-Typed Numeric Literal Coercion & Safe Widening:**
  * Numeric literals (`42`, `-1`, `3.14`) seamlessly adapt to target context types (`uint8`, `int16`, `int32`, `int64`, `float32`, `float64`) without requiring verbose manual casts when safe within value bounds.
  * Implemented safe widening conversions (`int8` $\to$ `int16` $\to$ `int32` $\to$ `int64`, `float32` $\to$ `float64`), while preserving strict type safety against lossy truncations.
* **Optimization & Verification:**
  * **AST Inliner (`AstInlinerPhase`):** Inline small static and instance methods with configurable heuristic thresholds (`--inline-threshold <THRESHOLD>`).
  * **Bytecode Verifier & Emitter (`phases/emitter`):** Generates optimized register bytecode instructions with automated register windowing, frame layout allocation, and pre-execution safety validation.

---

### Runtime & Virtual Machine (`solix-runtime`)
* **64-bit Register VM:**
  * Efficient NaN-boxed and direct 64-bit register representation (`Value::I64`, `Value::F64`, `Value::Ref`, `Value::Bool`).
  * Direct opcode dispatch loop with instruction quickening, branch prediction hints, and call frame windowing.
* **Deterministic ARC & Memory Architecture:**
  * Dual-partition virtual heap featuring a static partition for global symbols and an aligned dynamic partition for object payloads.
  * Plain-Old-Data (POD) fast-path deallocation bypassing destructor dispatch for leaf structures.
  * Intrusive O(1) quick-lists for allocations up to 256 bytes, backed by a coalescing free-list manager with zero heap fragmentation.
* **FFI & Native Extension Bridge:**
  * Zero-marshaling C-ABI FFI bridge (`natives/`) allowing Solix bytecode to invoke host native functions (`libloading`) across Linux (`.so`), macOS (`.dylib`), and Windows (`.dll`).
  * Bi-directional handle resolution with generational protection and automatic struct field read/write bindings.

---

### Package Manager & Build System (`solix-toolchain`)
* **Profile-Aware Package Store:**
  * Store directory structure isolates packages by profile and version: `~/.solix/packages/<name>-<profile>@<version>/`.
  * Support for `debug`, `release`, and `test` profiles with automatic profile fallback (`release` $\to$ `debug`).
* **Package Archive Container (`.slxarc`):**
  * Hermetic distribution archives combining executable bytecode container (`.slxb`), exported API manifest (`api.json`), and platform-specific native shared libraries (`natives/<platform>/`).
  * Cryptographic SHA-256 validation on archive unpacking and local cache caching.
* **Automated Hermetic Build Engine (`solix_build.py`):**
  * Automated orchestrator coordinating host CMake, C compiler (`zig cc` / `gcc` / `clang`), host code generators, Cargo native bridges, and Solix bytecode compiler.
  * Complete cache isolation: all intermediate artifacts reside strictly in `build/.cache/` leaving source trees 100% pristine.
* **Comprehensive CLI Tooling (`solix`):**
  * 13 primary subcommands: `new`, `compile`, `build`, `run`, `info`, `install`, `reinstall` (alias `sync`), `uninstall`, `list`, `lsp`, `dap`, `fmt`, `clean`.
  * Multi-format execution telemetry (`--metrics text` / `--metrics json`) and memory leak verification (`--gc-stats` / `-v`).

---

### First-Party Ecosystem
* **Standard Library (`solixlib` v2.0.0):**
  * Standard data structures: `List<T>`, `LinkedList<T>`, `HashMap<K, V>`, `HashSet<T>`, `TreeMap<K, V>`, `TreeSet<T>`, `PriorityQueue<T>`, `BitSet`.
  * High-performance string manipulation: `String`, `StringBuilder`, formatting, searching, splitting, and conversions.
  * Core subsystems: File I/O (`File`, `Directory`, `Path`), System & Environment (`Process`, `Environment`), DateTime & Duration (`DateTime`, `Duration`, `Stopwatch`), and Cryptography (`SHA256`, `MD5`, `Base64`).
* **Raylib 2D/3D Multimedia Bindings (`raylib` v5.5.0):**
  * 100% generic Two-Pass IR code generator producing complete 1-to-1 Solix bindings for Raylib 6.2 C API (`raylib.slx` and `natives/raylib`).
  * Full desktop cross-platform support across Linux, Windows, and macOS.
* **Showcase Application (`examples/breakout`):**
  * Complete, modern neon Breakout game built with Solix and Raylib featuring particle collisions, multi-tiered brick matrices, paddle deflection mechanics, and dynamic HUD scoring.

---

### IDE, Tooling & Diagnostics
* **Language Server Protocol (`solix-lsp`):**
  * Production-ready LSP server supporting autocomplete for keywords, identifiers, member dot-access (`.`), static scope resolution (`::`), and package imports (`import ...;`).
  * External package index caching with millisecond response latency.
  * Code actions, document formatting, semantic tokens, and live hover signatures.
* **Debug Adapter Protocol (`solix-dap`):**
  * Standard DAP implementation supporting line breakpoints, variable inspection, stack traces, step-in, step-over, step-out, and pause.
* **VS Code Extension (`editors/vscode`):**
  * Official VS Code extension published at version `1.0.0` with syntax highlighting, language configuration, LSP client, DAP launcher, and task integration.
* **Automated Drift & Quality Assurance:**
  * `scripts/verify_docs.py`: Continuous verification gate asserting 100% CLI command coverage, zero broken markdown links, and zero deprecated terminology.
  * Cross-platform release packaging pipelines with `cargo-dist` and `release-plz`.
