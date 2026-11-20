# `solix build` — Project Manifest & Container Build

The `build` subcommand orchestrates project building and packaging directly in Rust from `manifest.yaml` (with legacy `manifest.toml` compatibility). It runs `pre_compile` hooks, auto-detects Cargo (`Cargo.toml`) and CMake (`CMakeLists.txt`) native subprojects, compiles Solix source files, packages distributable `.slxp` containers (`build/<profile>/<name>.slxp`), and runs `post_compile` hooks.

---

## Synopsis

```bash
solix build [path] [options]
```

---

## Positional Arguments

| Argument | Type | Default | Description |
|---|---|---|---|
| `[path]` | Path | `.` (Current Directory) | Project root directory containing `manifest.yaml` (or `manifest.toml`). |

---

## Options & Flags

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--profile <mode>` | | String | `"release"` | Target build profile (e.g., `release`, `debug`, `test`, or custom declared profile). |
| `--debug` | | Flag | `false` | Shorthand for `--profile debug`. |
| `--release` | | Flag | `false` | Shorthand for `--profile release`. |
| `--test` | | Flag | `false` | Shorthand for `--profile test`. |
| `--platform <platform>` | | String | None | Filters build and native compilation to a designated target platform or OS family (e.g. `linux`, `windows`, `macos`). |
| `--pkg-dir <dir>` | | Path | `~/.solix/packages` | Root directory override for resolving package dependencies. |
| `--install` | | Flag | `false` | Automatically installs the built `.slxp` package into the package store. |
| `--force` | | Flag | `false` | Overwrites existing installed package when paired with `--install`. |
| `--check-env` | `--verify-only` | Flag | `false` | Runs Phase 0 pre-flight environment and toolchain verification without compiling or packaging. |
| `--target <triple>` | | String | `None` | Compiles native projects for a single target triple or `"host"` (also reads `SOLIX_TARGET_TRIPLE`). |

---

## Build Automation Pipeline

```mermaid
flowchart TD
    BuildEntry["solix build [path]"] --> ResolveProfile["Load manifest.yaml & Resolve Profile"]
    ResolveProfile --> ExpandPlatforms["Expand OS platforms -> All Architectures"]
    ExpandPlatforms --> PhaseZero["Phase 0: Verify Native Toolchains (cargo/rustc/cmake/zig)"]
    PhaseZero --> PhaseOne["Phase 1: Run pre_compile Command Hooks"]
    PhaseOne --> PhaseTwo["Phase 2: Build Auto-Detected Cargo (solix-native) & CMake (SOLIX_NATIVE_BUILD=ON) Natives"]
    PhaseTwo --> PhaseThree["Phase 3: Stage Resources, Root binaries.slxb, & Native Libs in bin/<target>/"]
    PhaseThree --> PhaseFour["Phase 4: Compile .slx Sources -> stage/binaries.slxb"]
    PhaseFour --> PhaseFive["Phase 5: Stamp manifest.yaml Checksums & Pack .slxp"]
    PhaseFive --> PhaseSix["Phase 6: Run post_compile Command Hooks"]
```

### Modernized Package Container Architecture (`.slxp`)
* **Package Root Bytecode (`binaries.slxb`):** The primary compiled Solix bytecode is placed at the package root as `binaries.slxb`.
* **Isolated Platform Directory (`bin/<target>/`):** Native modules targeting host or cross-compilation platforms are organized under `bin/<target_triple>/` (with fallback to `lib/<target_triple>/`).
* **Modern Container Emission:** Emits standard `.slxp` (Solix Package) format containing root `binaries.slxb` and target-tripled `bin/<target>/` native shared libraries.
* **Cross-Platform Tool Invocation:** Native build toolchains (`cargo`, `cmake`, `zig`) are located respecting host executable formats and `PATHEXT` on Windows, ensuring `.cmd` and `.bat` wrappers are cleanly dispatched without error 193.
* **Hermetic Zigbuild Toolchain Isolation:** When cross-compiling native modules via Zig (`cargo-zigbuild`), Solix generates isolated compiler wrappers (`zig-cc`, `zig-cxx`, `zig-ar`) and injects a dedicated CMake toolchain file configured with `CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY`. This prevents cross-compilation targets from leaking host `/usr/include` headers (such as host glibc and X11 definitions).

### Dual-Mode Native Compilation & Dependency Platform Validation
* **CMake (`CMakeLists.txt`):** `solix build` configures CMake with `-DSOLIX_NATIVE_BUILD=ON` and sets `SOLIX_NATIVE_BUILD=1`. When built outside `solix build`, the same CMake project builds cleanly as a standalone embeddable library.
* **Cargo (`Cargo.toml`):** When `[features]` declares `solix-native`, `solix build` passes `--features solix-native` and sets `SOLIX_NATIVE_BUILD=1`.
* **Native Dependency Platform Check:** When a profile requests target `platforms` (such as `[all]` or `[linux, windows]`), `solix build` verifies that every native-backed dependency `.slxp` bundles `.so`/`.dll`/`.dylib` binaries for all requested platforms, failing explicitly if any target platform is missing.

---

## Exit Codes

| Exit Code | Classification | Meaning |
|---|---|---|
| `0` | Success | Project built and `.slxp` package successfully created. |
| `1` | Error | Missing manifest, environment verification failure, hook error, native build error, or Solix compilation error. |

---

## Examples

### 1. Default Release Build
```bash
solix build
```

### 2. Debug Profile Build
```bash
solix build --debug
```

### 3. Host-Only Build & Auto-Install
```bash
solix build libraries/solixlib --profile release --target host --install --force
```
