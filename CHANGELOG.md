# Changelog

All notable changes to the Solix programming language, compiler toolchain, runtime virtual machine, and ecosystem are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
