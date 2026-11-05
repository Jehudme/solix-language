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
   * **Tier 1 (Low-Level Toolbox Primitives in `solix_toolbox.py`):** Composable helper functions to parse `manifest.toml`, compile Rust Cargo FFI bridge projects (`compile_rust_project`), copy `resources/` and `lib/`, invoke `solix compile` on directories, and package `.slxarc` archives.
   * **Tier 2 (One-Call Automatic Orchestrator `auto_build(profile)`):** A single turnkey function called by `build.py` that automatically detects Rust `cdylib` subprojects inside `natives/`, compiles them via Cargo (`debug` vs `release`), stages `bin/`, `lib/`, and `resources/` inside `cache/stage/<profile>/`, and packs the final `.slxarc` archive into `build/<profile>/`.
3. **Rust Native Bridge Subprojects (`natives/`):**
   * Native extensions are standardized strictly as **Rust `cdylib` bridge crates**.
   * Rust bridge crates expose stable Solix C-ABI registration hooks via `solix_register_natives` (or direct export symbols).
   * If a project needs to integrate external C/C++ libraries, it does so within its Rust bridge crate using standard Rust ecosystem tooling (e.g. `cc`, `bindgen`, or `build.rs`), maintaining a unified, reliable toolchain boundary.
4. **Starter Project Template (`templates/untitled-rust`):**
   * Ready-to-build reference template demonstrating Rust (`cdylib`) native interoperability with Solix.
5. **Compiler Directory Ingestion & Profile Pipeline (`solix compile`):**
   * `solix compile` accepts directory paths (recursively discovering all `*.slx` files) and a `--profile <debug|release|test>` flag that controls whether `AstInlinerPhase` and `BytecodeOptimizerPhase` are enabled in `CompilationPipeline` and embeds `manifest.toml` metadata into the `.slxb` header.
6. **Runtime Dynamic Shared Library Loader & `.slxarc` Execution (`solix-runtime` & `solix run`):**
   * Implements cross-platform dynamic shared library loading (`libloading` for `.dll`, `.so`, `.dylib`) with C-ABI batch registration (`solix_register_natives`) and direct symbol resolution fallback (`Suite 49` in `TESTS.md`).
   * Adds `lib_paths: Vec<PathBuf>` to `RuntimeOptions` and `-l, --lib <PATH>` to `solix run` for manual `.dll`/`.so`/`.dylib` or `.slxarc` loading.
   * Updates `solix run` to natively execute `.slxarc` archives by reading `bin/<name>.slxb` into memory and extracting native shared libraries from `lib/` into `cache/runtime_natives/` before booting the VM.
7. **Container & Archive Inspection (`solix info`):**
   * Updates `solix info` to inspect `.slxarc` archive containers in addition to `.slxb` files, printing parsed `manifest.toml` metadata (`[info]`, `[entrypoints]`), internal file hierarchy, embedded `.slxb` bytecode summaries, and packaged native libraries.

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
├── natives/                       # Rust cdylib bridge subprojects (Cargo.toml + src/lib.rs)
│   └── <native_subproject>/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── resources/                     # Static runtime assets (textures, audio, shaders, JSON, etc.)
├── lib/                           # Pre-existing external libraries (*.slxb, *.slxarc, *.dll, *.so, *.dylib)
├── cache/                         # Ephemeral build artifacts (managed automatically)
│   ├── natives/<profile>/         # Intermediate Cargo compilation artifacts
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

| Build Profile | CLI Flag on `solix build` | CLI Flag on `solix compile` | Source Directories Compiled | Optimization Phases (`AstInlinerPhase` & `BytecodeOptimizerPhase`) | Native Cargo Target Mode (`natives/`) | Entry Point Symbol | Output Archive Path |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Debug** | `--debug` (or `--profile debug`, default) | `--profile debug` | `src/` | **DISABLED** (Fast compile, 1:1 source spans) | `cargo build` (debug cdylib) | `entrypoints.main` | `build/debug/<name>.slxarc` |
| **Release** | `--release` (or `--profile release`) | `--profile release` | `src/` | **ENABLED** (Full AST inlining $\le 8$ stmts + Superinstruction Peephole) | `cargo build --release` (release cdylib) | `entrypoints.main` | `build/release/<name>.slxarc` |
| **Test** | `--test` (or `--profile test`) | `--profile test` | `src/` **+** `tests/` | **DISABLED** (Fast compile, exact assertion stack traces) | `cargo build` (debug cdylib) | `entrypoints.test` | `build/test/<name>.slxarc` |

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
│     auto-detects & compiles all Rust cdylib projects in natives/,          │
│     copies lib/ and resources/, compiles src/ (+ tests/), and packs        │
│     build/<profile>/<name>.slxarc.                                         │
├────────────────────────────────────────────────────────────────────────────┤
│ TIER 1: Composable Toolbox Primitives (For Custom build.py Workflows)      │
│   • load_manifest(project_root: Path) -> ProjectManifest                   │
│   • prepare_staging_dirs(project_root: Path, profile: str) -> StagePaths   │
│   • detect_native_projects(natives_dir: Path) -> list[NativeProject]       │
│   • compile_rust_project(proj_dir, cache_dir, stage_lib_dir, profile)      │
│   • copy_project_libs(project_lib_dir: Path, stage_lib_dir: Path)          │
│   • copy_project_resources(resources_dir: Path, stage_res_dir: Path)       │
│   • compile_solix_sources(project_root, manifest, stage_bin_dir, profile)  │
│   • package_slxarc(stage_dir: Path, output_slxarc: Path) -> Path           │
└────────────────────────────────────────────────────────────────────────────┘
```

#### Detailed Specification of Tier 1 Functions in `solix_toolbox.py`:

1. **`load_manifest(project_root: Path) -> dict`:**
   * Parses `manifest.toml` using Python 3.11+ standard library `tomllib` (zero external pip dependencies).
   * Extracts `[info]` (`name`, `version`, `description`, `license`, `authors`) and `[entrypoints]` (`main`, `test`).
2. **`prepare_staging_dirs(project_root: Path, profile: str) -> StagePaths`:**
   * Creates `cache/natives/<profile>/`, cleans and recreates `cache/stage/<profile>/{bin,lib,resources}`, and ensures `build/<profile>/` exists.
   * Copies `manifest.toml` into `cache/stage/<profile>/manifest.toml`.
3. **`compile_rust_project(proj_dir: Path, cache_dir: Path, stage_lib_dir: Path, profile: str)`:**
   * Runs `cargo build --manifest-path <proj_dir>/Cargo.toml --target-dir <cache_dir>/cargo` (adding `--release` when `profile == "release"`).
   * Detects the host platform extension (`.so` on Linux, `.dll` on Windows, `.dylib` on macOS).
   * Discovers and copies the emitted `cdylib` into `stage_lib_dir`.
4. **`copy_project_libs(project_lib_dir: Path, stage_lib_dir: Path)` & `copy_project_resources(resources_dir: Path, stage_res_dir: Path)`:**
   * Copies any pre-existing files from `<project_root>/lib/` into `cache/stage/<profile>/lib/` and `<project_root>/resources/` into `cache/stage/<profile>/resources/`.
5. **`compile_solix_sources(project_root: Path, manifest: dict, stage_bin_dir: Path, profile: str)`:**
   * Resolves the `solix` CLI binary (checking `SOLIX_BIN` environment variable first, then `target/release/solix`, `target/debug/solix`, and `PATH`).
   * For `debug` and `release`: passes `<project_root>/src`.
   * For `test`: passes both `<project_root>/src` and `<project_root>/tests`.
   * Passes `--profile <profile>`, `--name <info.name>`, `--version <info.version>`, `--description <info.description>`, any `.slxb` dependency libraries in `stage/lib/`, and `-o cache/stage/<profile>/bin/<name>.slxb`.
6. **`package_slxarc(stage_dir: Path, output_slxarc: Path) -> Path`:**
   * Creates a deterministic ZIP archive of `cache/stage/<profile>/` and outputs to `build/<profile>/<name>.slxarc`.

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

## 5. Starter Project Template (`templates/untitled-rust/`)

The reference project template in `templates/untitled-rust/` demonstrates native Rust interoperability:

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

* **`natives/rust_bridge/src/lib.rs`:** Exports `#[no_mangle] pub extern "C" fn solix_register_natives(...)` registering `MathBridge::fast_add`, and direct fallback symbols using the Solix C-ABI FFI convention.
* **`src/main.slx`:** Declares `public class MathBridge { public static native int32 fast_add(int32 a, int32 b); }` and invokes it from `Main::main()`.
* **`tests/test_main.slx`:** Declares `TestRunner::main()` asserting that `MathBridge::fast_add(15, 27) == 42`.

---

## 6. Native Shared Library Interoperability & `.slxarc` Runtime Architecture

### 6.1 Native C-ABI Contract (`solix-runtime::native::loader`)

To fulfill **Suite 49 (`TESTS.md`: Native Interoperability & Dynamic Shared Library Loader)**, `solix-runtime` defines a stable, `#[repr(C)]` FFI bridge in `crates/solix-runtime/src/native/ffi.rs`:

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

### 6.2 Symbol Resolution Rules
* When `ClassName::method_name` is marked `native` in Solix:
  1. First, when the shared library is loaded via `libloading::Library`, the loader checks if the library exports `solix_register_natives`. If present, it invokes the hook to batch-register all functions (`Case 49.3`).
  2. Second, for any unresolved `native` function in the loaded `BinaryProgram` (e.g., `NativeMath::add` or `Counter::increment`), the loader automatically probes the loaded shared libraries for the mangled symbol `ClassName_method_name` (e.g., `NativeMath_add`, `Counter_increment`) or `method_name` (`Case 49.1`, `Case 49.2`, `Case 49.4`).
* **Instance Native Calls (`Case 49.2`):** When an instance method is marked `native` (e.g. `Counter::increment`), `args[0]` is `Value::Ref(self_address)`. The C-ABI bridge exposes helper functions (`solix_vm_get_field_i32(vm_ctx, obj_addr, offset)`, `solix_vm_set_field_i32(vm_ctx, obj_addr, offset, val)`) so native code can read and mutate instance fields on `self_address`.
* **Negative Error Handling (`Cases 49.6 – 49.9`):**
  * **Case 49.6 (Missing Library Path):** Returns `RuntimeError::SharedLibraryError { path, reason }`.
  * **Case 49.7 (Corrupted / Non-Binary File):** Returns `RuntimeError::SharedLibraryError` with OS loader diagnostic (`dlerror` / `GetLastError`).
  * **Case 49.8 (Unresolved Native Symbol at Invocation):** Returns `RuntimeError::FunctionNotFound { symbol_or_id: "Call to unknown native function: MissingNative_non_existent_function" }`.
  * **Case 49.9 (Foreign Platform Library Format):** Before calling `dlopen`/`LoadLibrary`, validates that the file extension matches the host OS (`.so` on Linux, `.dll` on Windows, `.dylib` on macOS)—immediately rejecting foreign formats (e.g. loading `.dll` on Linux or `.so` on Windows) with `RuntimeError::SharedLibraryError { reason: "Unsupported native library format for host platform..." }`.

### 6.3 `.slxarc` Archive Reader & Inspection (`solix-common` & `solix-client`)

When `solix run` is given a `.slxarc` file (or when `RuntimeOptions::with_archive_path` is called):
1. Opens the `.slxarc` ZIP archive.
2. Reads `manifest.toml` from the archive root to identify `[info]` (`name`, `version`) and `[entrypoints]` (`main`, `test`).
3. Reads `bin/<name>.slxb` into memory and deserializes `BinaryProgram`.
4. Extracts matching native shared libraries into `<project_or_temp_cache>/runtime_natives/<name>-<version>/` and registers their paths into `RuntimeOptions.native_libraries`.
5. Mounts secondary `.slxb` dependency containers in `lib/*.slxb` into `RuntimeOptions.binaries`.

When `solix info <file.slxarc>` is executed:
* Parses and outputs project metadata (`[info]` and `[entrypoints]`).
* Lists all files within the archive (with compressed and uncompressed sizes).
* Reads and displays the embedded `.slxb` binary header metrics (classes, functions, static partition).
* Identifies packaged native shared libraries.

---

## 7. Phased Implementation Plan

```
┌────────────────────────────────────────────────────────────────────────────┐
│ Phase 1: Compiler Directory Ingestion & Build Profiles (`--profile`)       │
│          └─► Completed (commit 53b3041)                                    │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 2: Native Shared Library Loader & Suite 49 (`solix-runtime`)         │
│          └─► `libloading`, C-ABI bridge, `RuntimeOptions.native_libraries` │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 3: `.slxarc` Archive Container & `manifest.toml` Parser              │
│          └─► `SlxarcArchive`, `ProjectManifest`, `solix-common`            │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 4: `solix build` CLI Subcommand, `-l / --lib`, & `solix info`        │
│          └─► `solix build`, `solix run -l`, `.slxarc` in `solix info`      │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 5: Two-Tier Python Toolbox & `templates/untitled-rust`               │
│          └─► `scripts/solix_toolbox.py`, `templates/untitled-rust`         │
├────────────────────────────────────────────────────────────────────────────┤
│ Phase 6: Exhaustive End-to-End Build & Archive Tests                       │
│          └─► Suite 49 conformance, CLI tests, clippy & formatting          │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## 8. Exhaustive Test Plan & Verification Matrix

### 8.1 Suite 49 Conformance Tests (`crates/solix-tests/tests/runtime/test_native_ffi.rs` & `TESTS.md`)
* **Case 49.1: Static Native Function Call:** Calls `NativeMath::add(15, 27)` $\to$ `42`.
* **Case 49.2: Instance Native Function Call with Object Context:** Mutates `c.value` from 10 to 11 via `self_address`.
* **Case 49.3: Batch Registration Hook via `solix_register_natives`:** Verifies `BatchPlugin::multiply` and `subtract`.
* **Case 49.4: Direct Dynamic Symbol Resolution Fallback:** Resolves `DynamicLib_direct_export` without init hook $\to$ `100`.
* **Case 49.5: Multiple Shared Libraries Loaded Concurrently:** Loads `LibA` and `LibB` simultaneously (`funcA() + funcB() == 100`).
* **Case 49.6: Missing / Non-Existent Shared Library Path (Negative):** Returns `RuntimeError::SharedLibraryError`.
* **Case 49.7: Corrupted or Non-Binary File Loaded as Shared Library (Negative):** Returns `RuntimeError::SharedLibraryError` with OS loader diagnostic.
* **Case 49.8: Unresolved Native Method Symbol at Invocation (Negative):** Throws `Call to unknown native function: MissingNative_non_existent_function`.
* **Case 49.9: Reject Foreign Platform Native Library Format (Negative):** Immediate rejection of foreign extensions (`.dll` on Linux/macOS, `.so` on Windows).

### 8.2 Build System, Profiles, Toolbox & `.slxarc` End-to-End Tests (`crates/solix-client/tests/build_and_archive_tests.rs`)
* **Test B.1: `manifest.toml` Parsing & Minimal Schema Enforcement.**
* **Test B.2: `solix compile` Directory Recursion & `--profile debug` vs `--profile release`.**
* **Test B.3: End-to-End `untitled-rust` Template Build & `.slxarc` Execution (`debug`, `test`, `release`).**
* **Test B.4: `solix run -l <native_lib>` Direct CLI Flag.**
* **Test B.5: `solix info <archive.slxarc>` Metadata & Container Display.**

---

## 9. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Tool | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Minimal `manifest.toml`** | Contains only `[info]` and `[entrypoints]` | `crates/solix-common/src/project.rs` | [ ] Pending |
| **Directory & Profile Compile** | `solix compile <dir> --profile <debug\|release\|test>` | `cargo test -p solix-client` | [x] Completed (`53b3041`) |
| **Two-Tier Python Toolbox** | `solix_toolbox.py` (Tier 1 helpers + Tier 2 `auto_build`) | `templates/untitled-rust/scripts/` | [ ] Pending |
| **Rust Native Bridge Support** | `untitled-rust` builds `cdylib` and runs in `.slxarc` | `build_and_archive_tests.rs` | [ ] Pending |
| **Suite 49 Native FFI (49.1–49.9)**| All 9 positive & negative tests pass & marked `[IMPLEMENTED]` | `cargo test -p solix-tests` | [ ] Pending |
| **`.slxarc` & `-l` Execution** | `solix run <file.slxarc>` and `solix run -l <lib>` | `cargo test -p solix-client` | [ ] Pending |
| **`.slxarc` in `solix info`** | Metadata and contents inspected via `solix info` | `cargo test -p solix-client` | [ ] Pending |
| **Clippy & Formatting** | Zero warnings across all crates | `cargo clippy --workspace --all-targets -- -D warnings` | [ ] Pending |
