# Milestone 035: Automated 1-to-1 Complete Raylib Ecosystem (`libraries/raylib`), Full Multi-Header (`raylib.h`, `raymath.h`, `rlgl.h`, `raygui.h`) Zig + CMake Extraction Pipeline & Hermetic `.cache` Packaging

## Document Metadata

* **Milestone ID:** MS-035
* **Target Version:** `v1.0.0-rc.2`
* **Milestone Branch:** `milestone/035-automated-raylib-1-to-1-pipeline`
* **Status:** DRAFT (Awaiting Architect Approval)
* **Lead Architect:** @jehudme
* **Target Delivery Date:** 2026-12-31
* **Normative Conformance:** RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, `OPTIONAL`)
* **Related Specifications:**
  * `docs/internals/guidelines/authoring-milestones.md` (GL-0008)
  * `docs/internals/guidelines/authoring-test.md` (GL-0009)
  * `docs/internals/guidelines/language-conventions.md` (GL-0011)
  * `docs/internals/guidelines/documentation-update-protocol.md` (GL-0013)
  * `docs/internals/milestones/016-native-interoperability-and-dynamic-shared-library-loader.md` (MS-016)
  * `docs/internals/milestones/019-build-system-and-slxarc-archive-container.md` (MS-019)
  * `docs/internals/milestones/033-final-v1-pre-release-audit-modular-entrypoints-import-intelligence-and-terminal-dap.md` (MS-033)

---

## 1. Executive Summary & Full Ecosystem Scope

The previous draft generated bindings solely from `raylib.h` (~270 functions). Raylib's true ecosystem comprises over **500+ functions** distributed across four primary headers:
1. **`raylib.h` (~270 functions, 32 structs, 21 enums):** Windowing, input polling, 2D/3D shapes, textures, font rendering, audio streaming, and high-level model loading.
2. **`raymath.h` (~70+ functions):** Vector math (`Vector2Add`, `Vector3CrossProduct`), matrix transformations (`MatrixMultiply`, `MatrixRotate`), and quaternions (`QuaternionSlerp`).
3. **`rlgl.h` (~90+ functions):** Low-level OpenGL immediate-mode abstraction (`rlBegin`, `rlEnd`, `rlVertex3f`, matrix stack manipulation, shader uniform buffers).
4. **`raygui.h` (~50+ functions):** The official immediate-mode GUI toolset (`GuiButton`, `GuiSlider`, `GuiTextBox`, `GuiListView`, etc.).

**Milestone 035** configures CMake, Zig, and `rlparser` to process **all four headers**, merge them into a single consolidated AST schema (`build/.cache/<profile>/api/raylib_complete_api.json`), and automatically emit **500+ 1-to-1 functions**, all structs, constants, and enums into `src/raylib.slx` and `natives/raylib/src/lib.rs`.

---

## 2. End-to-End Pipeline Architecture

```
┌────────────────────────────────────────────────────────────────────────────────────┐
│                   `solix build libraries/raylib --release`                         │
│                        (Driven by `scripts/build.py`)                              │
└─────────────────────────────────────────┬──────────────────────────────────────────┘
                                          │
                                          ▼
┌────────────────────────────────────────────────────────────────────────────────────┐
│ PHASE 1: CMake + Zig Toolchain (`build/.cache/<profile>/cmake/`)                   │
│   • Configures CMake with `zig cc` / `zig c++` (cross-platform C99/C++ compiler)   │
│   • `FetchContent` downloads Raylib tag & `raygui` into `.cache`                   │
│   • Compiles static `libraylib.a` with `-fPIC` (`BUILD_SHARED_LIBS=OFF`)           │
│   • Compiles `raylib_parser.c` -> `rlparser` executable in `.cache`                │
│   • Runs `rlparser` on all 4 headers:                                              │
│       - `raylib.h`  -> `build/.cache/<profile>/api/raylib_api.json`                │
│       - `raymath.h` -> `build/.cache/<profile>/api/raymath_api.json`               │
│       - `rlgl.h`    -> `build/.cache/<profile>/api/rlgl_api.json`                  │
│       - `raygui.h`  -> `build/.cache/<profile>/api/raygui_api.json`                │
│   • Merges all 4 JSON files -> `raylib_complete_api.json`                          │
└─────────────────────────────────────────┬──────────────────────────────────────────┘
                                          │
                       ┌──────────────────┴──────────────────┐
                       ▼                                     ▼
┌─────────────────────────────────────────┐ ┌────────────────────────────────────────┐
│ PHASE 2A: `scripts/generate_solix_api.py`│ │ PHASE 2B: `scripts/generate_rust_bridge.py`│
│   • Ingests `raylib_complete_api.json`  │ │   • Ingests `raylib_complete_api.json` │
│   • Emits `src/raylib.slx`:             │ │   • Emits `natives/raylib/src/lib.rs`: │
│     - Defines & Color constants         │ │     - All `#[repr(C)]` C structs       │
│     - All Enums & bitflag constants     │ │     - In-place `T*` mutation writeback │
│     - All Type Aliases (`Texture2D`...) │ │     - 500+ `extern "C"` FFI wrappers   │
│     - All Struct Classes (1-to-1)       │ │     - `solix_register_natives` table   │
│     - 500+ `public native` Functions    │ │       registering all symbols          │
└──────────────────────┬──────────────────┘ └────────────────┬───────────────────────┘
                       │                                     │
                       ▼                                     ▼
┌─────────────────────────────────────────┐ ┌────────────────────────────────────────┐
│ PHASE 3: Compile Native Bridge          │ │ PHASE 4: Compile Solix Bytecode        │
│   • Runs `cargo build` with             │ │   • Runs `solix compile src/raylib.slx`│
│     `--target-dir build/.cache/cargo`   │ │   • Emits `build/.cache/stage/<prof>/  │
│   • Statically links `libraylib.a` + OS │ │     bin/raylib.slxb`                   │
│     graphics libs (OpenGL/X11/Win32/Mac)│ └────────────────┬───────────────────────┘
│   • Copies `libraylib_native.{so,dll}`  │                  │
│     into `build/.cache/stage/<prof>/lib`│                  │
└──────────────────────┬──────────────────┘                  │
                       └──────────────────┬──────────────────┘
                                          ▼
┌────────────────────────────────────────────────────────────────────────────────────┐
│ PHASE 5: Cryptographic Checksum Pinning & `.slxarc` Packaging                      │
│   • Computes SHA-256 of `lib/libraylib_native.{so,dll,dylib}` into `manifest.toml` │
│   • Packs `build/.cache/stage/<profile>/` -> `build/<profile>/raylib.slxarc`       │
└────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Complete Multi-Header API Mapping Matrix

### 3.1 Subsystem Scope

| Module / Header | Functions | Structs & Types | Constants & Enums | Description |
| :--- | :---: | :---: | :---: | :--- |
| **`raylib.h`** | ~270 | 32 Structs, 6 Aliases | 55 Defines, 21 Enums | Core Window, Input, Audio, Shapes, Textures, Models |
| **`raymath.h`** | ~75 | Vector/Matrix math helpers | 2 Defines | Full 2D/3D math operations without external math libs |
| **`rlgl.h`** | ~95 | `rlVertexBuffer`, `rlRenderBatch` | OpenGL state enums | Low-level render pipeline control & custom mesh buffers |
| **`raygui.h`** | ~55 | `GuiStyleProp` | 30+ Control state enums | Complete immediate-mode UI controls (Buttons, Sliders, Boxes) |
| **Total** | **~495–510** | **35+ Structs** | **110+ Defines & Enums** | **Comprehensive, zero-omission Raylib environment** |

### 3.2 Pointer & Struct Marshaling Specification

1. **Pass-by-Value Structs (`Vector2`, `Vector3`, `Matrix`, `Color`, `Rectangle`):**
   * Solix represents these as standard classes (`class Vector2 { public float32 x; public float32 y; }`).
   * The Rust bridge reads fields via `SolixVmContext` into C-compatible `#[repr(C)]` stack variables and passes them directly to C.
2. **In-Place Mutation Struct Pointers (`Camera *`, `Image *`, `Mesh *`, `Wave *`, `Model *`):**
   * Functions such as `UpdateCamera(Camera *camera, int mode)` or `ImageResize(Image *image, int newWidth, int newHeight)` modify the struct in C memory.
   * **Writeback Protocol:** The bridge unpacks the Solix object into a local C struct, calls the C function with `&mut c_struct`, and **writes all updated fields back to the Solix heap object address** before returning.
3. **Contiguous Struct Arrays (`const Vector2 *points`, `int pointCount`):**
   * In Solix, signatures accept `Vector2[] points`.
   * The bridge reads the `ArrayRef` length and extracts each element into a contiguous `Vec<C_Vector2>` before passing `.as_ptr()` to C.
4. **Variadic String Formatting (`TraceLog`, `TextFormat`):**
   * Mapped in Solix to fixed-string signatures (`public static native void trace_log(int32 log_level, string text);`).
   * The bridge passes `c"%s".as_ptr()` as the format specifier to prevent format-string exploits while providing 1-to-1 functionality.

---

## 4. Multi-Header CMake Extraction Pipeline (`CMakeLists.txt`)

```cmake
cmake_minimum_required(VERSION 3.20)
project(solix_raylib_vendor C)

set(RAYLIB_VERSION_TAG "master" CACHE STRING "Upstream Raylib Git tag")
set(RAYGUI_VERSION_TAG "master" CACHE STRING "Upstream Raygui Git tag")

include(FetchContent)

set(BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set(BUILD_GAMES OFF CACHE BOOL "" FORCE)
set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)
set(CMAKE_POSITION_INDEPENDENT_CODE ON CACHE BOOL "" FORCE)

# 1. Fetch core Raylib
FetchContent_Declare(
    raylib
    GIT_REPOSITORY https://github.com/raysan5/raylib.git
    GIT_TAG        ${RAYLIB_VERSION_TAG}
    GIT_SHALLOW    TRUE
)
# 2. Fetch Raygui
FetchContent_Declare(
    raygui
    GIT_REPOSITORY https://github.com/raysan5/raygui.git
    GIT_TAG        ${RAYGUI_VERSION_TAG}
    GIT_SHALLOW    TRUE
)
FetchContent_MakeAvailable(raylib raygui)

# 3. Compile parser executable
add_executable(rlparser "${raylib_SOURCE_DIR}/parser/raylib_parser.c")

set(API_OUT_DIR "${CMAKE_BINARY_DIR}/api")
file(MAKE_DIRECTORY "${API_OUT_DIR}")

# 4. Parse all 4 headers
add_custom_command(
    OUTPUT "${API_OUT_DIR}/raylib_api.json"
           "${API_OUT_DIR}/raymath_api.json"
           "${API_OUT_DIR}/rlgl_api.json"
           "${API_OUT_DIR}/raygui_api.json"
    COMMAND rlparser -i "${raylib_SOURCE_DIR}/src/raylib.h"  -o "${API_OUT_DIR}/raylib_api.json"  -f JSON -d RLAPI
    COMMAND rlparser -i "${raylib_SOURCE_DIR}/src/raymath.h" -o "${API_OUT_DIR}/raymath_api.json" -f JSON -d RMAPI
    COMMAND rlparser -i "${raylib_SOURCE_DIR}/src/rlgl.h"    -o "${API_OUT_DIR}/rlgl_api.json"    -f JSON -d RLAPI
    COMMAND rlparser -i "${raygui_SOURCE_DIR}/src/raygui.h"  -o "${API_OUT_DIR}/raygui_api.json"  -f JSON -d RAYGUIAPI
    DEPENDS rlparser
            "${raylib_SOURCE_DIR}/src/raylib.h"
            "${raylib_SOURCE_DIR}/src/raymath.h"
            "${raylib_SOURCE_DIR}/src/rlgl.h"
            "${raygui_SOURCE_DIR}/src/raygui.h"
    COMMENT "Parsing raylib.h, raymath.h, rlgl.h, and raygui.h via rlparser"
)

add_custom_target(generate_raylib_api ALL
    DEPENDS "${API_OUT_DIR}/raylib_api.json"
            "${API_OUT_DIR}/raymath_api.json"
            "${API_OUT_DIR}/rlgl_api.json"
            "${API_OUT_DIR}/raygui_api.json"
)
```

---

## 5. Merging & Codegen Tooling

### 5.1 JSON Merger (`scripts/merge_apis.py`)
Combines the outputs of the four headers into a single cohesive AST definition:
* **Deduplication:** Structs defined across multiple headers (such as `Vector2`, `Vector3`, `Matrix`) are resolved once.
* **Namespace Partitioning:** Functions and constants are tagged with their origin header (`raylib`, `raymath`, `rlgl`, `raygui`) to allow both unified top-level access and modular class namespaces.

### 5.2 API Generators (`generate_solix_api.py` & `generate_rust_bridge.py`)
1. **`generate_solix_api.py`:**
   * Emits `src/raylib.slx`.
   * Organizes the API into clear classes:
     * `Raylib` (Core system, shapes, textures, models, audio)
     * `Raymath` (Vector, matrix, and quaternion math)
     * `Rlgl` (Render pipeline control, matrix stacks, framebuffers)
     * `Raygui` (Immediate-mode GUI widgets)
   * Re-exports all functions as top-level package functions for maximum convenience.
2. **`generate_rust_bridge.py`:**
   * Emits `natives/raylib/src/lib.rs`.
   * Compiles `#define RAYGUI_IMPLEMENTATION` into an auxiliary compilation unit to include GUI implementations inside the static library.
   * Maps all 500+ C function pointers into the `solix_register_natives` table.

---

## 6. Phased Implementation Plan

### Phase 1: Native FFI Struct Allocation Enhancements (`solix-runtime`)
* [ ] `[P0]` **Task 1.1:** Add `alloc_object`, `read_field_u64`, and `write_field_u64` to `SolixVmContext` in `crates/solix-runtime/src/native/ffi.rs`.
* [ ] `[P0]` **Task 1.2:** Validate struct allocation and field manipulation in `crates/solix-runtime/tests/unit_native_ffi_callbacks.rs`.

### Phase 2: CMake, Zig & Multi-Header Parser Setup (`libraries/raylib`)
* [ ] `[P0]` **Task 2.1:** Implement `libraries/raylib/CMakeLists.txt` fetching `raylib` and `raygui` and compiling `rlparser`.
* [ ] `[P0]` **Task 2.2:** Add `scripts/merge_apis.py` to consolidate `raylib_api.json`, `raymath_api.json`, `rlgl_api.json`, and `raygui_api.json` into `raylib_complete_api.json`.
* [ ] `[P0]` **Task 2.3:** Provide an offline fallback snapshot of `raylib_complete_api.json` in `scripts/` to ensure offline builds succeed without internet access.

### Phase 3: Complete Codegen Engines (`generate_solix_api.py` & `generate_rust_bridge.py`)
* [ ] `[P0]` **Task 3.1:** Implement `scripts/generate_solix_api.py` generating `src/raylib.slx` covering all 500+ functions, 35+ structs, and all enums/constants.
* [ ] `[P0]` **Task 3.2:** Implement `scripts/generate_rust_bridge.py` generating `natives/raylib/src/lib.rs` with complete C-ABI bindings, pointer mutation writebacks, and `solix_register_natives`.
* [ ] `[P0]` **Task 3.3:** Configure `natives/raylib/build.rs` to statically link `libraylib.a` along with host platform graphics dependencies.

### Phase 4: Staging, Checksum Pinning & Packaging
* [ ] `[P0]` **Task 4.1:** Verify that all intermediate CMake, Cargo, and JSON files are restricted to `build/.cache/<profile>/`.
* [ ] `[P0]` **Task 4.2:** Implement `scripts/build.py` to stage `manifest.toml`, `bin/raylib.slxb`, and `lib/libraylib_native.{so,dll,dylib}` and pack `build/<profile>/raylib.slxarc`.

### Phase 5: Verification & Automated Conformance Testing
* [ ] `[P0]` **Task 5.1:** Write headless integration tests in `libraries/raylib/tests/test_raylib_api.slx` verifying:
  * Math operations across `Raymath::Vector2Add`, `Raymath::MatrixMultiply`, and `Raymath::Vector3CrossProduct`.
  * In-place image mutation: `GenImageColor` $\to$ `ImageResize(&image, 32, 32)` $\to$ checking `image.width == 32`.
  * Collision detection functions: `CheckCollisionRecs`, `CheckCollisionCircles`, `GetCollisionRec`.
  * Text manipulation and color utility functions.
* [ ] `[P0]` **Task 5.2:** Build and install the package via `solix build libraries/raylib --release --sync` and verify successful compilation of an external game demo using `import raylib.*;`.

---

## 7. Definition of Done (DoD) Sign-Off Matrix

| DoD Requirement | Target Benchmark / Artifact | Verification Command | Sign-off Status |
| :--- | :--- | :--- | :--- |
| **Multi-Header CMake Pipeline** | Compiles `libraylib.a`, `rlparser`, and parses all 4 headers | `solix build libraries/raylib --release` | [ ] Pending |
| **Consolidated API Coverage** | Emits 500+ functions across `Raylib`, `Raymath`, `Rlgl`, and `Raygui` | `python3 libraries/raylib/scripts/merge_apis.py` | [ ] Pending |
| **Complete 1-to-1 `src/raylib.slx`** | Full typed declarations for all structs, enums, and 500+ functions | `cargo check --workspace` | [ ] Pending |
| **Complete 1-to-1 Rust FFI Bridge** | Native wrappers, pointer writebacks, and registration for all 500+ symbols | `cargo build -p raylib_native` | [ ] Pending |
| **Hermetic `.cache` Isolation** | All build intermediates in `build/.cache/`; only 3 artifacts in `.slxarc` | `solix info libraries/raylib/build/release/raylib.slxarc` | [ ] Pending |
| **Headless Conformance Suite** | Headless math, struct mutation, and collision tests pass | `solix run --test libraries/raylib/build/test/raylib.slxarc` | [ ] Pending |