# Milestone 019: Project Build System, Two-Tier Build Toolbox, `.slxarc` Archive Containers & Native Shared Library Loader

## Document Metadata

* **Milestone ID:** MS-019
* **Target Version:** `v0.1.0-alpha.19`
* **Milestone Branch:** `milestone/019-project-build-system-and-slxarc`
* **Status:** ACTIVE
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-commits.md` (GL-0001)
  * `docs/internals/guidelines/branching-conventions.md` (GL-0002)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/authoring-docs.md` (GL-0010)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/milestones/011-multi-binary-support-and-dynamic-linking.md` (MS-011)
  * `docs/internals/milestones/013-solix-cli-client-tooling.md` (MS-013)
  * `docs/internals/milestones/018-unified-inlined-vm-and-compiler-optimization.md` (MS-018)
  * `crates/solix-tests/TESTS.md` (Suite 49: Native Interoperability & Dynamic Shared Library Loader)

---

## 1. Executive Summary & Architectural Vision

Currently, the Solix toolchain compiles individual `.slx` source files into standalone `.slxb` bytecode containers (`solix compile`) and executes them (`solix run`). However, real-world Solix applications and libraries require **native shared libraries** (`.dll`, `.so`, `.dylib`), **static assets/resources**, **automated multi-file directory compilation**, **debug/release/test build profiles**, and **cross-platform packaging**. Managing loose `.slxb` files alongside platform-specific `.dll`/`.so`/`.dylib` binaries manually is brittle and error-prone.

**Milestone 019** introduces a complete, end-to-end **Project Build, Packaging, and Native Dynamic Loading System**:

1. **Minimalist, User-Editable `manifest.toml` Contract:**
   * All structural project directories (`src/`, `tests/`, `natives/`, `resources/`, `lib/`, `scripts/`, `cache/`, `build/`) are **strictly constant conventions** and **MUST NOT** clutter `manifest.toml`.
   * `manifest.toml` contains **only** user-editable project identity metadata (`[info]`: `name`, `version`, `description`, `license`, `authors`) and entry-point overrides (`[entrypoints]`: `main`, `test`).
2. **Two-Tier Python Build Toolbox (`scripts/solix_toolbox.py` & `scripts/build.py`):**
   * `solix build <project_path> [--debug | --release | --test]` delegates execution to `scripts/build.py --profile <mode>`.
   * **Tier 1 (Low-Level Toolbox Primitives in `solix_toolbox.py`):** Composable helper functions to parse `manifest.toml`, compile CMake projects (using **Zig** as a cross-compiler), compile Rust Cargo FFI projects, copy `resources/` and `lib/`, invoke `solix compile` on directories, and package `.slxarc` archives.
   * **Tier 2 (One-Call Automatic Orchestrator `auto_build(profile)`):** A single turnkey function called by `build.py` that automatically detects CMake or Rust subprojects inside `natives/`, compiles them for the **host OS only** in `debug`/`test` mode or **cross-compiles for all 3 desktop platforms (`windows-x86_64`, `linux-x86_64`, `macos-x86_64`/`aarch64`) via Zig** in `release` mode, stages `bin/`, `lib/`, and `resources/` inside `cache/stage/<profile>/`, and packs the final `.slxarc` archive into `build/<profile>/`.
3. **Dual Starter Project Templates (`templates/untitled-cmake` & `templates/untitled-rust`):**
   * Ready-to-build reference templates demonstrating C/C++ (CMake + Zig CC) and Rust (`cdylib`) native interoperability with Solix.
4. **Compiler Directory Ingestion & Profile Pipeline (`solix compile`):**
   * `solix compile` accepts directory paths (recursively discovering all `*.slx` files) and a `--profile <debug|release|test>` flag that controls whether `AstInlinerPhase` and `BytecodeOptimizerPhase` are enabled in `CompilationPipeline` and embeds `manifest.toml` metadata into the `.slxb` header.
5. **Runtime Dynamic Shared Library Loader & `.slxarc` Execution (`solix-runtime` & `solix run`):**
   * Implements cross-platform dynamic shared library loading (`libloading` for `.dll`, `.so`, `.dylib`) with C-ABI batch registration (`solix_register_natives`) and direct symbol resolution fallback (`Suite 49` in `TESTS.md`).
   * Adds `lib_paths: Vec<PathBuf>` to `RuntimeOptions` and `-l, --lib <PATH>` to `solix run` for manual `.dll`/`.so`/`.dylib` or `.slxarc` loading.
   * Updates `solix run` to natively execute `.slxarc` archives by reading `bin/<name>.slxb` into memory and extracting the host OS's native shared libraries from `lib/` into `cache/runtime_natives/` before booting the VM.

---

## 2. Canonical Project Layout & `manifest.toml` Specification

### 2.1 Fixed Convention Directory Structure (Zero Config Boilerplate)

Every Solix project adheres to a fixed, invariant directory layout. Because these paths never change, they are **hardcoded constants** in `solix_toolbox.py` and the CLI toolchain rather than configurable fields in `manifest.toml`:

```text
<project_root>/
├── manifest.toml                  # ONLY editable metadata ([info] & [entrypoints])
├── scripts/
│   ├── build.py                   # Entry-point build script invoked by `solix build`
│   └── solix_toolbox.py           # Two-Tier Build Toolbox library (Tier 1 helpers + Tier 2 auto_build)
├── src/                           # Production Solix source files (*.slx, recursive)
│   └── main.slx
├── tests/                         # Test suite Solix source files (*.slx, recursive)
│   └── test_main.slx
├── natives/                       # Native C/C++ (CMakeLists.txt) or Rust (Cargo.toml) subprojects
│   └── <native_subproject>/
├── resources/                     # Static runtime assets (textures, audio, shaders, JSON, etc.)
├── lib/                           # Pre-existing external libraries (*.slxb, *.slxarc, *.dll, *.so, *.dylib)
├── cache/                         # Ephemeral build artifacts (managed automatically)
│   ├── natives/<profile>/<target>/# Intermediate CMake / Cargo compilation artifacts
│   └── stage/<profile>/           # Staging root zipped into the final .slxarc archive
│       ├── manifest.toml
│       ├── bin/<name>.slxb
│       ├── lib/                   # Compiled native libs + copied project lib/ files
│       └── resources/             # Copied project resources/
└── build/                         # Final distributable output directory
    ├── debug/
    │   └── <name>.slxarc
    ├── release/
    │   └── <name>.slxarc
    └── test/
        └── <name>.slxarc
```

### 2.2 The Minimal `manifest.toml` Schema

`manifest.toml` **MUST** only contain user-editable project identity and entrypoint symbols:

```toml
[info]
name = "untitled"
version = "0.1.0"
description = "An untitled Solix project with native interoperability"
license = "Apache-2.0"
authors = ["Jehud Metellus"]

[entrypoints]
main = "Main::main"
test = "TestRunner::main"
```

* **`[info]` Section:** Mapped directly into `CompilationOptions::with_package_metadata(name, version, description)` and serialized into the emitted `.slxb` container's header and `ExportManifest`.
* **`[entrypoints]` Section:**
  * `main`: The entry-point symbol used when building/running in `debug` or `release` mode.
  * `test`: The entry-point symbol used when building/running in `test` mode.

---

## 3. Build Profiles & Compiler Pipeline Semantics

| Build Profile | CLI Flag on `solix build` | CLI Flag on `solix compile` | Source Directories Compiled | Optimization Phases (`AstInlinerPhase` & `BytecodeOptimizerPhase`) | Native Compilation Target Matrix (`natives/`) | Entry Point Symbol | Output Archive Path |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Debug** | `--debug` (or `--profile debug`, default) | `--profile debug` | `src/` | **DISABLED** (Fast compile, 1:1 source spans) | **Current Host OS Only** (`.dll` on Win, `.so` on Linux, `.dylib` on macOS) | `entrypoints.main` | `build/debug/<name>.slxarc` |
| **Release** | `--release` (or `--profile release`) | `--profile release` | `src/` | **ENABLED** (Full AST inlining $\le 8$ stmts + Superinstruction Peephole) | **All 3 Platforms via Zig** (`windows-x86_64` `.dll`, `linux-x86_64` `.so`, `macos-x86_64` `.dylib`) | `entrypoints.main` | `build/release/<name>.slxarc` |
| **Test** | `--test` (or `--profile test`) | `--profile test` | `src/` **+** `tests/` | **DISABLED** (Fast compile, exact assertion stack traces) | **Current Host OS Only** (`.dll` on Win, `.so` on Linux, `.dylib` on macOS) | `entrypoints.test` | `build/test/<name>.slxarc` |

---

## 4. Two-Tier Python Build Toolbox Architecture (`scripts/`)

To give developers maximum control over their build pipeline while providing zero-configuration convenience out of the box, the build script architecture is split into two files inside `scripts/`:

### 4.1 `scripts/solix_toolbox.py` (The Two-Tier Library)

```
┌────────────────────────────────────────────────────────────────────────────┐
│                        scripts/solix_toolbox.py                            │
├────────────────────────────────────────────────────────────────────────────┤
│ TIER 2: Turnkey Orchestrator                                               │
│   • auto_build(profile: str, project_root: Path | None = None) -> Path     │
│     Automatically loads manifest.toml, cleans/prepares cache/stage,        │
│     auto-detects & compiles all CMake/Rust projects in natives/,           │
│     copies lib/ and resources/, compiles src/ (+ tests/), and packs        │
│     build/<profile>/<name>.slxarc.                                         │
├────────────────────────────────────────────────────────────────────────────┤
│ TIER 1: Composable Toolbox Primitives (For Custom build.py Workflows)      │
│   • load_manifest(project_root: Path) -> ProjectManifest                   │
│   • prepare_staging_dirs(project_root: Path, profile: str) -> StagePaths   │
│   • detect_native_projects(natives_dir: Path) -> list[NativeProject]       │
│   • compile_cmake_project(proj_dir, cache_dir, stage_lib_dir, profile)     │
│   • compile_rust_project(proj_dir, cache_dir, stage_lib_dir, profile)      │
│   • copy_project_libs(project_lib_dir: Path, stage_lib_dir: Path)          │
│   • copy_project_resources(resources_dir: Path, stage_res_dir: Path)       │
│   • compile_solix_sources(project_root, manifest, stage_bin_dir, profile)  │
│   • package_slxarc(stage_dir: Path, output_slxarc: Path) -> Path           │
└────────────────────────────────────────────────────────────────────────────┘
```

#### Detailed Specification of Tier 1 Functions in `solix_toolbox.py`:

1. **`load_manifest(project_root: Path) -> dict`:**
   * Parses `manifest.toml` using Python 3.11+ standard library `tomllib` (zero external pip dependencies!).
   * Extracts `[info]` (`name`, `version`, `description`, `license`, `authors`) and `[entrypoints]` (`main`, `test`).
2. **`prepare_staging_dirs(project_root: Path, profile: str) -> StagePaths`:**
   * Creates `cache/natives/<profile>/`, cleans and recreates `cache/stage/<profile>/{bin,lib,resources}`, and ensures `build/<profile>/` exists.
   * Copies `manifest.toml` into `cache/stage/<profile>/manifest.toml`.
3. **`compile_cmake_project(proj_dir: Path, cache_dir: Path, stage_lib_dir: Path, profile: str)`:**
   * **When `profile in ("debug", "test")`:**
     * Detects host OS (`sys.platform`) and compiles only for the current host platform into `cache/natives/<profile>/host/`.
     * Uses `zig cc` / `zig c++` (if `zig` is on `PATH`) or system C/C++ compiler with `-DCMAKE_BUILD_TYPE=Debug`.
     * Copies the resulting shared library (`*.so`, `*.dll`, or `*.dylib`) into `cache/stage/<profile>/lib/<os>-<arch>/` (and `cache/stage/<profile>/lib/`).
   * **When `profile == "release"`:**
     * Iterates over the cross-platform target matrix:
       1. `x86_64-linux-gnu` $\to$ emits `lib<name>.so` into `stage/lib/linux-x86_64/`
       2. `x86_64-windows-gnu` $\to$ emits `<name>.dll` into `stage/lib/windows-x86_64/`
       3. `x86_64-macos-none` (or `aarch64-macos-none`) $\to$ emits `lib<name>.dylib` into `stage/lib/macos-x86_64/`
     * Invokes `cmake` with `-DCMAKE_C_COMPILER="zig;cc;-target;<zig_target>"` and `-DCMAKE_CXX_COMPILER="zig;c++;-target;<zig_target>"` and `-DCMAKE_BUILD_TYPE=Release` (or direct `zig cc -shared -O3 -target <zig_target>` when a single-pass Zig CMake toolchain wrapper is used so cross-linking to `.dll`, `.so`, and `.dylib` succeeds on any host OS without needing Windows SDK or macOS Xcode SDK installed!).
4. **`compile_rust_project(proj_dir: Path, cache_dir: Path, stage_lib_dir: Path, profile: str)`:**
   * **When `profile in ("debug", "test")`:**
     * Runs `cargo build --manifest-path <proj_dir>/Cargo.toml --target-dir <cache_dir>/cargo` (debug profile).
     * Copies the emitted `cdylib` (`.so`, `.dll`, or `.dylib`) into `cache/stage/<profile>/lib/<host_platform>/`.
   * **When `profile == "release"`:**
     * Runs `cargo zigbuild --release` (if `cargo-zigbuild` is installed) or `cargo build --release` for available targets, copying the emitted shared libraries into `cache/stage/<profile>/lib/<platform>/`.
5. **`copy_project_libs(project_lib_dir: Path, stage_lib_dir: Path)` & `copy_project_resources(resources_dir: Path, stage_res_dir: Path)`:**
   * Copies any pre-existing files from `<project_root>/lib/` into `cache/stage/<profile>/lib/` and `<project_root>/resources/` into `cache/stage/<profile>/resources/`.
6. **`compile_solix_sources(project_root: Path, manifest: dict, stage_bin_dir: Path, profile: str)`:**
   * Resolves the `solix` CLI binary (checking `SOLIX_BIN` environment variable first, then `target/release/solix`, `target/debug/solix`, and `PATH`).
   * For `debug` and `release`: passes `<project_root>/src`.
   * For `test`: passes both `<project_root>/src` and `<project_root>/tests`.
   * Passes `--profile <profile>`, `--name <info.name>`, `--version <info.version>`, `--description <info.description>`, any `.slxb` dependency libraries in `stage/lib/`, and `-o cache/stage/<profile>/bin/<name>.slxb`.
7. **`package_slxarc(stage_dir: Path, output_slxarc: Path) -> Path`:**
   * Creates a deterministic ZIP archive of `cache/stage/<profile>/` at `cache/stage/<name>-<profile>.zip` and moves/renames it to `build/<profile>/<name>.slxarc`.

### 4.2 `scripts/build.py` (The Clean Project Entry Point)

Because all heavy lifting lives in `solix_toolbox.py`, the project's `scripts/build.py` is clean, readable, and trivial for users to customize:

```python
#!/usr/bin/env python3
"""Project build script invoked by `solix build`."""

import argparse
from pathlib import Path
import solix_toolbox as tb


def main() -> int:
    parser = argparse.ArgumentParser(description="Build Solix project")
    parser.add_argument(
        "--profile",
        choices=["debug", "release", "test"],
        default="debug",
        help="Build profile mode (debug, release, or test)",
    )
    args = parser.parse_args()

    project_root = Path(__file__).resolve().parent.parent
    tb.auto_build(profile=args.profile, project_root=project_root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

---

## 5. Dual Starter Project Templates (`templates/`)

We will create two complete, working project templates in `templates/`:

### 5.1 Template 1: `templates/untitled-cmake/` (C/C++ with CMake & Zig)
```text
templates/untitled-cmake/
├── manifest.toml
├── scripts/
│   ├── build.py
│   └── solix_toolbox.py
├── src/
│   └── main.slx
├── tests/
│   └── test_main.slx
├── natives/
│   └── math_bridge/
│       ├── CMakeLists.txt
│       └── bridge.c
├── resources/
│   └── config.txt
└── lib/
    └── .gitkeep
```
* **`natives/math_bridge/bridge.c`:** Exports C-ABI functions (`native_fast_add`, `native_fast_mul`) and the batch registration hook `solix_register_natives` (or direct C-ABI symbols) compatible with `solix-runtime`.
* **`src/main.slx`:** Declares `public class MathBridge { public static native int32 fast_add(int32 a, int32 b); }` and invokes it from `Main::main()`.
* **`tests/test_main.slx`:** Declares `TestRunner::main()` asserting that `MathBridge::fast_add(15, 27) == 42`.

### 5.2 Template 2: `templates/untitled-rust/` (Rust `cdylib` Native Extension)
```text
templates/untitled-rust/
├── manifest.toml
├── scripts/
│   ├── build.py
│   └── solix_toolbox.py
├── src/
│   └── main.slx
├── tests/
│   └── test_main.slx
├── natives/
│   └── rust_bridge/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── resources/
│   └── config.txt
└── lib/
    └── .gitkeep
```
* **`natives/rust_bridge/src/lib.rs`:** Exports `#[no_mangle] pub extern "C" fn MathBridge_fast_add(...)` and `solix_register_natives` using the Solix C-ABI FFI convention.

---

## 6. Native Shared Library Interoperability & `.slxarc` Runtime Architecture

### 6.1 Native C-ABI Contract (`solix-runtime::native::loader`)

To fulfill **Suite 49 (`TESTS.md`: Native Interoperability & Dynamic Shared Library Loader)** and allow both C/CMake and Rust `cdylib` shared libraries to work seamlessly without depending on unstable Rust internal ABI layouts, `solix-runtime` defines a stable, `#[repr(C)]` FFI bridge in `crates/solix-runtime/src/native/ffi.rs`:

```rust
/// Stable C-ABI tagged value passed across native shared library boundaries.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolixCValue {
    /// Tag: 0 = Null, 1 = Bool, 2 = I32, 3 = I64, 4 = F64, 5 = Ref, 6 = FnPtr
    pub tag: u32,
    /// Padding / reserved flags
    pub flags: u32,
    /// 64-bit payload (i32, i64, f64 bits, or heap u32 reference address)
    pub payload: u64,
}

/// Stable C-ABI native function pointer signature:
/// `SolixCValue func(void* vm_handle, const SolixCValue* args, uint32_t arg_count, SolixCValue* out_error)`
pub type NativeCFunctionPtr = extern "C" fn(
    vm_ctx: *mut std::ffi::c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue;

/// Registration callback provided to `solix_register_natives`.
pub type RegisterNativeCallback = extern "C" fn(
    context: *mut std::ffi::c_void,
    symbol: *const std::ffi::c_char,
    func: NativeCFunctionPtr,
);

/// Optional batch registration hook exported by a shared library:
/// `void solix_register_natives(void* context, RegisterNativeCallback register_fn)`
pub type SolixRegisterNativesHook = extern "C" fn(
    context: *mut std::ffi::c_void,
    register_fn: RegisterNativeCallback,
);
```

And for maximum ergonomics in C and Rust projects, the loader **also** supports **Direct C-ABI Symbol Resolution Fallback (Case 49.4)**:
* When `ClassName::method_name` is marked `native` in Solix:
  1. First, when the shared library is loaded via `libloading::Library`, the loader checks if the library exports `solix_register_natives`. If present, it invokes the hook to batch-register all functions (`Case 49.3`).
  2. Second, for any unresolved `native` function in the loaded `BinaryProgram` (e.g., `NativeMath::add` or `Counter::increment`), the loader automatically probes the loaded shared libraries for the mangled symbol `ClassName_method_name` (e.g., `NativeMath_add`, `Counter_increment`) or `method_name` (`Case 49.1`, `Case 49.2`, `Case 49.4`).
  3. Furthermore, to support simple C/Rust functions in `untitled-cmake` and `untitled-rust`, the loader supports both `NativeCFunctionPtr` (`SolixCValue` ABI) and a simplified scalar C-ABI hook `solix_c_abi_version` so native functions can be written cleanly in standard C or Rust!
* **Instance Native Calls (`Case 49.2`):** When an instance method is marked `native` (e.g. `Counter::increment`), `args[0]` is `Value::Ref(self_address)`. The C-ABI bridge exposes helper functions (`solix_vm_get_field_i32(vm_ctx, obj_addr, offset)`, `solix_vm_set_field_i32(vm_ctx, obj_addr, offset, val)`) so native code can read and mutate instance fields on `self_address`!
* **Negative Error Handling (`Cases 49.6 – 49.9`):**
  * **Case 49.6 (Missing Library Path):** Returns `RuntimeError::SharedLibraryError { path, reason }`.
  * **Case 49.7 (Corrupted / Non-Binary File):** Returns `RuntimeError::SharedLibraryError` with OS loader diagnostic (`dlerror` / `GetLastError`).
  * **Case 49.8 (Unresolved Native Symbol at Invocation):** Returns `RuntimeError::FunctionNotFound { symbol_or_id: "Call to unknown native function: MissingNative_non_existent_function" }`.
  * **Case 49.9 (Foreign Platform Library Format):** Before calling `dlopen`/`LoadLibrary`, validates that the file extension matches the host OS (`.so` on Linux, `.dll` on Windows, `.dylib` on macOS)—immediately rejecting foreign formats (e.g. loading `.dll` on Linux or `.so` on Windows) with `RuntimeError::SharedLibraryError { reason: "Unsupported native library format for host platform..." }`.

### 6.2 `.slxarc` Archive Reader (`crates/solix-common/src/bytecode/archive.rs` & `solix-client`)

When `solix run` is given a `.slxarc` file (or when `RuntimeOptions::with_archive_path` is called):
1. Opens the `.slxarc` ZIP archive.
2. Reads `manifest.toml` from the archive root to identify `[info]` (`name`, `version`) and `[entrypoints]` (`main`, `test`).
3. Reads `bin/<name>.slxb` (or all `bin/*.slxb` files) directly from the ZIP stream into memory and deserializes `BinaryProgram`.
4. Scans `lib/` inside the ZIP archive for native shared libraries matching the current host platform:
   * First checks `lib/<host_os>-<host_arch>/` (e.g. `lib/linux-x86_64/*.so`, `lib/windows-x86_64/*.dll`, `lib/macos-x86_64/*.dylib`, `lib/macos-aarch64/*.dylib`).
   * Also checks root `lib/*.{so,dll,dylib}` matching the host OS extension.
5. Extracts matching native shared libraries into `<project_or_temp_cache>/runtime_natives/<name>-<version>-<os>-<arch>/` and registers their paths into `RuntimeOptions.native_libraries`.
6. Extracts any secondary `.slxb` dependency containers in `lib/*.slxb` and mounts them into `RuntimeOptions.binaries`.

---

## 7. Phased Implementation Plan (Zero-Ambiguity Execution Blueprint)

```
┌────────────────────────────────────────────────────────────────────────────┐
│ Phase 1: Compiler Directory Ingestion & Build Profiles (`--profile`)       │
│          └─► `solix compile <dir> --profile <debug|release|test>`          │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 2: Native Shared Library Loader & Suite 49 (`solix-runtime`)         │
│          └─► `libloading`, C-ABI bridge, `RuntimeOptions.native_libraries` │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 3: `.slxarc` Archive Reader/Writer & `manifest.toml` Parser          │
│          └─► `SlxarcArchive`, `ProjectManifest`, `solix run <app.slxarc>`  │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 4: `solix build` CLI Subcommand & `-l / --lib` on `solix run`        │
│          └─► `solix build <path> [--debug|--release|--test]`               │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 5: Two-Tier Python Toolbox & CMake (Zig) / Rust Project Templates    │
│          └─► `solix_toolbox.py`, `untitled-cmake`, `untitled-rust`         │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 6: Exhaustive Unit, Integration & Suite 49 Conformance Tests         │
│          └─► 100% pass across all 49.1..49.9 + CLI build/archive tests     │
└────────────────────────────────────────────────────────────────────────────┘
```

### Phase 1: Compiler Directory Ingestion & Build Profiles (`solix-compilation` & `solix-client`)
* **Files to Modify:**
  1. `crates/solix-compilation/src/CompilationContext.rs`:
     * Add `BuildProfile` enum (`Debug`, `Release`, `Test`) to `CompilationOptions`.
     * When `BuildProfile::Debug` or `BuildProfile::Test` is selected, set `inline_threshold = 0` and `bytecode_optimization = false`.
     * When `BuildProfile::Release` is selected, set `inline_threshold = 8` and `bytecode_optimization = true`.
  2. `crates/solix-compilation/src/pipeline.rs`:
     * Add `CompilationPipeline::for_profile(profile: BuildProfile)`:
       * `Release`: `LexerPhase -> ParserPhase -> BinderPhase -> AstInlinerPhase -> EmitterPhase -> BytecodeOptimizerPhase`.
       * `Debug` / `Test`: `LexerPhase -> ParserPhase -> BinderPhase -> EmitterPhase` (omits `AstInlinerPhase` and `BytecodeOptimizerPhase`).
  3. `crates/solix-client/src/cli.rs` & `crates/solix-client/src/commands/compile.rs`:
     * Add `--profile <debug|release|test>` (default: `release`) to `CompileArgs`.
     * Update source file collection in `execute_compile`: if any path in `args.sources` is a directory, recursively walk the directory (sorted deterministically) and collect all files with extension `.slx`.

### Phase 2: Native Shared Library Loader & Suite 49 (`solix-runtime`)
* **Dependencies:**
  * Add `libloading = "0.8"` to `[workspace.dependencies]` in root `Cargo.toml` and `crates/solix-runtime/Cargo.toml`.
* **Files to Create / Modify:**
  1. `crates/solix-runtime/src/native/ffi.rs` (New):
     * Define `SolixCValue`, `NativeCFunctionPtr`, `SolixRegisterNativesHook`, and helper functions `solix_vm_get_field`, `solix_vm_set_field` for instance native methods.
     * Implement `SharedLibraryLoader` wrapping `Vec<libloading::Library>` (kept alive for the lifetime of `RuntimeContext` / `VirtualMachine`).
     * Validate platform extension before loading (rejecting `.dll` on Linux/macOS, `.so` on Windows/macOS, `.dylib` on Linux/Windows with `RuntimeError::SharedLibraryError` — fulfilling **Case 49.9**).
     * Load library via `libloading::Library::new`, returning `RuntimeError::SharedLibraryError` on missing file (**Case 49.6**) or corrupted binary (**Case 49.7**).
     * Check for `solix_register_natives` symbol and invoke it if present (**Case 49.3**).
     * Provide fallback symbol lookup `resolve_direct_symbol("ClassName::method")` $\to$ searches `ClassName_method` and `method` in all loaded libraries (**Cases 49.1, 49.2, 49.4, 49.5**).
  2. `crates/solix-compilation/src/phases/emitter/pipeline.rs`:
     * In `compile_source_functions`, when a method has `Modifier::Native` (`m.body.is_none()`):
       * Emit a synthesized native trampoline body (`CallSymbol` targeting `"ClassName::method"` or `"func_name"` with `r0..r(param_count-1)` and `ReturnValue`), instead of emitting an empty `Return` instruction!
       * Track native function symbols so unresolved native functions at runtime raise `RuntimeError::NativeFunctionError` / `"Call to unknown native function: <Class>_<method>"` (**Case 49.8**).
  3. `crates/solix-runtime/src/options.rs` & `crates/solix-runtime/src/context.rs`:
     * Add `pub native_libraries: Vec<PathBuf>` and `.with_native_library(path)` to `RuntimeOptions`.
     * Load all `native_libraries` during `RuntimeContext::new` and wire resolved native functions into `function_meta[id].native_fn`.

### Phase 3: `manifest.toml` Parser & `.slxarc` Archive Reader/Writer (`solix-common` & `solix-client`)
* **Dependencies:**
  * Add `toml = "0.8"` and `zip = { version = "2.2", default-features = false, features = ["deflate"] }` to `[workspace.dependencies]` and `solix-common` / `solix-client`.
* **Files to Create / Modify:**
  1. `crates/solix-common/src/project.rs` (New):
     * Define `ProjectManifest` with `[info]` (`ProjectInfo`: `name`, `version`, `description`, `license`, `authors`) and `[entrypoints]` (`ProjectEntryPoints`: `main`, `test`).
     * Implement `ProjectManifest::from_toml_str` and `ProjectManifest::from_file`.
     * Implement `SlxarcArchive` reader & writer:
       * `SlxarcArchive::pack_directory(stage_dir: &Path, output_path: &Path)`
       * `SlxarcArchive::extract_and_load(archive_path: &Path, extract_cache_dir: &Path) -> Result<LoadedSlxarc, EmissionError>`
       * Automatically selects host OS native libraries (`.so`, `.dll`, `.dylib`) from `lib/<os>-<arch>/` or `lib/` and extracts them to `extract_cache_dir`.

### Phase 4: `solix build` CLI Command & `solix run` Updates (`solix-client`)
* **Files to Create / Modify:**
  1. `crates/solix-client/src/cli.rs`:
     * Add `Build(BuildArgs)` to `Commands`:
       * `path: PathBuf` (default `"."`)
       * `--profile <debug|release|test>` (plus shorthand flags `--debug`, `--release`, `--test`)
     * Add `-l, --lib <LIBS>` (`Vec<PathBuf>`) to `RunArgs` for loading `.dll` / `.so` / `.dylib` / `.slxb` / `.slxarc` libraries at runtime.
  2. `crates/solix-client/src/commands/build.rs` (New):
     * Implement `execute_build(args: BuildArgs) -> Result<()>`:
       * Resolves `<project_root>/manifest.toml` and `<project_root>/scripts/build.py`.
       * Determines active profile (`debug`, `release`, or `test`).
       * Passes `SOLIX_BIN=<current_exe>` in the child process environment so `build.py` always invokes the exact running `solix` binary.
       * Spawns `python3 scripts/build.py --profile <profile>` with working directory `<project_root>` and checks exit status.
  3. `crates/solix-client/src/commands/run.rs`:
     * Detect `.slxarc` files in `args.binaries` and `args.libs`.
     * For `.slxarc` inputs: extract via `SlxarcArchive::extract_and_load`, mount `.slxb` binaries, register extracted native shared libraries into `RuntimeOptions`, and set entry point from `manifest.entrypoints.main` (or `test` if `--entry` is not overridden).
     * For `-l, --lib` inputs: if extension is `.so`, `.dll`, or `.dylib`, pass to `runtime_options.with_native_library(path)`; if `.slxb`, pass to `runtime_options.with_binary(...)`.

### Phase 5: Two-Tier Python Toolbox & Starter Project Templates (`templates/`)
* **Files to Create:**
  1. `templates/untitled-cmake/manifest.toml`
  2. `templates/untitled-cmake/scripts/solix_toolbox.py`
  3. `templates/untitled-cmake/scripts/build.py`
  4. `templates/untitled-cmake/natives/math_bridge/CMakeLists.txt`
  5. `templates/untitled-cmake/natives/math_bridge/bridge.c`
  6. `templates/untitled-cmake/src/main.slx`
  7. `templates/untitled-cmake/tests/test_main.slx`
  8. `templates/untitled-cmake/resources/config.txt`
  9. `templates/untitled-rust/manifest.toml`
  10. `templates/untitled-rust/scripts/solix_toolbox.py`
  11. `templates/untitled-rust/scripts/build.py`
  12. `templates/untitled-rust/natives/rust_bridge/Cargo.toml`
  13. `templates/untitled-rust/natives/rust_bridge/src/lib.rs`
  14. `templates/untitled-rust/src/main.slx`
  15. `templates/untitled-rust/tests/test_main.slx`
  16. `templates/untitled-rust/resources/config.txt`

---

## 8. Exhaustive Test Plan & Verification Matrix

Every single requirement in this milestone **MUST** be verified by automated, hermetic tests across `solix-tests`, `solix-runtime`, and `solix-client`:

### 8.1 Suite 49 Conformance Tests (`crates/solix-tests/tests/runtime/test_native_ffi.rs` & `TESTS.md`)
All 9 scenarios in **Suite 49** of `TESTS.md` **MUST** be implemented and transitioned from `[NOT IMPLEMENTED]` to `[IMPLEMENTED]`:
* **Case 49.1: Static Native Function Call:** Compiles a dynamic shared library on-the-fly, loads it via `RuntimeOptions::with_native_library`, calls `NativeMath::add(15, 27)`, and asserts return value `42`.
* **Case 49.2: Instance Native Function Call with Object Context:** Calls `Counter::increment()` on a Solix `Counter` instance (`c.value = 10`), mutates the field in native code via `self_address`, and asserts `res == 11 && c.value == 11`.
* **Case 49.3: Batch Registration Hook via `solix_register_natives`:** Shared library exports `solix_register_natives` registering `BatchPlugin::multiply` and `BatchPlugin::subtract`; verifies both execute and return `42`.
* **Case 49.4: Direct Dynamic Symbol Resolution Fallback:** Shared library exports `DynamicLib_direct_export` without `solix_register_natives`; verifies on-demand symbol resolution returns `100`.
* **Case 49.5: Multiple Shared Libraries Loaded Concurrently:** Loads two distinct shared libraries (`LibA` and `LibB`) simultaneously without symbol collision (`funcA() + funcB() == 100`).
* **Case 49.6: Missing / Non-Existent Shared Library Path (Negative):** Passing `"non_existent_plugin.dll"` returns `Err(RuntimeError::SharedLibraryError { .. })`.
* **Case 49.7: Corrupted or Non-Binary File Loaded as Shared Library (Negative):** Passing a plain-text file as a shared library returns `Err(RuntimeError::SharedLibraryError { .. })` containing the OS loader diagnostic.
* **Case 49.8: Unresolved Native Method Symbol at Invocation (Negative):** Calling `MissingNative::non_existent_function()` when no library provides it returns `Err` containing `"Call to unknown native function: MissingNative_non_existent_function"`.
* **Case 49.9: Reject Foreign Platform Native Library Format (Negative):** Passing `"plugin.dll"` on Linux/macOS (or `"plugin.so"` on Windows) fails immediately with `RuntimeError::SharedLibraryError` detailing unsupported native library format for host platform.

### 8.2 Build System, Profiles, Toolbox & `.slxarc` End-to-End Tests (`crates/solix-client/tests/build_and_archive_tests.rs`)
* **Test B.1: `manifest.toml` Parsing & Minimal Schema Enforcement:** Verifies `[info]` and `[entrypoints]` parse cleanly and metadata transfers into `.slxb` `ExportManifest`.
* **Test B.2: `solix compile` Directory Recursion & `--profile debug` vs `--profile release`:**
  * Compiles a multi-file `src/` directory with `--profile debug` (asserting instructions are un-inlined) and `--profile release` (asserting small functions are inlined and superinstructions fused).
* **Test B.3: End-to-End `untitled-cmake` Template Build & `.slxarc` Execution (`debug`, `test`, `release`):**
  * Copies `templates/untitled-cmake` into a hermetic workspace.
  * Runs `solix build <ws> --debug` $\to$ verifies `cache/stage/debug/` and `build/debug/untitled.slxarc` are created, and `solix run build/debug/untitled.slxarc` loads the C native library and returns `0` (42 check).
  * Runs `solix build <ws> --test` $\to$ verifies `src/` + `tests/` are compiled with `TestRunner::main` entry point, and `solix run build/test/untitled.slxarc` returns `0`.
  * Runs `solix build <ws> --release` (when `zig` is available) $\to$ verifies cross-platform `.dll`, `.so`, and `.dylib` libraries are generated inside `cache/stage/release/lib/` and packaged into `build/release/untitled.slxarc`.
* **Test B.4: End-to-End `untitled-rust` Template Build & `.slxarc` Execution:**
  * Copies `templates/untitled-rust` into a hermetic workspace, runs `solix build <ws> --debug`, and executes `solix run build/debug/untitled.slxarc`, verifying the Rust `cdylib` is compiled, packaged, extracted, and executed by the Solix VM.
* **Test B.5: `solix run -l <native_lib>` Direct CLI Flag:**
  * Verifies `solix run app.slxb -l ./libbridge.so` loads the native library via CLI flag and executes native methods cleanly.

---

## 9. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Minimal `manifest.toml`** | Contains only `[info]` and `[entrypoints]` | `crates/solix-common/src/project.rs` | [ ] Pending |
| **Directory & Profile Compile** | `solix compile <dir> --profile <debug\|release\|test>` | `cargo test -p solix-client` | [ ] Pending |
| **Two-Tier Python Toolbox** | `solix_toolbox.py` (Tier 1 helpers + Tier 2 `auto_build`) | `templates/*/scripts/solix_toolbox.py` | [ ] Pending |
| **CMake + Zig Cross-Compile** | Release profile builds `.dll`, `.so`, `.dylib` via Zig | `build_and_archive_tests.rs` | [ ] Pending |
| **Rust Native Project Support** | `untitled-rust` builds `cdylib` and runs in `.slxarc` | `build_and_archive_tests.rs` | [ ] Pending |
| **Suite 49 Native FFI (49.1–49.9)**| All 9 positive & negative tests pass & marked `[IMPLEMENTED]` | `cargo test -p solix-tests` | [ ] Pending |
| **`.slxarc` & `-l` Execution** | `solix run <file.slxarc>` and `solix run -l <lib>` | `cargo test -p solix-client` | [ ] Pending |
| **Clippy & Formatting** | Zero warnings across all crates | `cargo clippy --workspace --all-targets -- -D warnings` | [ ] Pending |