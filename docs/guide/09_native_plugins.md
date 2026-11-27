# Native Rust & C Extensions (`native`)

Solix provides high-throughput Foreign Function Interface (FFI) interoperability with compiled native shared libraries (`.so` on Linux, `.dylib` on macOS, and `.dll` on Windows). Native extensions enable compute-intensive algorithms, OS hardware bindings, and zero-overhead native integration.

---

## 1. Declaring Native Methods in Solix

Native methods are declared using the `native` modifier. Like abstract methods, native methods omit the method body:

```solix
package my_project;

public class MathBridge {
    // Static native method
    public static native int32 fast_add(int32 value_a, int32 value_b);
    public static native int32 fast_multiply(int32 value_a, int32 value_b);
}
```

Calls to static native methods use Solix's canonical scope resolution syntax:
```solix
int32 result = MathBridge::fast_add(15, 27);
```

---

## 2. Stable C-ABI: `SolixCValue`

Values are passed across the native boundary using the `SolixCValue` structure (exact size 16 bytes, 8-byte alignment, defined identically in [`ffi/rust`](../../ffi/rust/src/value.rs) and [`ffi/c/include/solix_ffi.h`](../../ffi/c/include/solix_ffi.h)). Both `solix-ffi` and `solix::ffi` maintain lockstep release versions synchronized by [`prepare_release_train.py`](../../scripts/release/prepare_release_train.py) while keeping `ffi/rust/Cargo.toml` standalone-mountable inside Docker cross containers:

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SolixCValue {
    pub tag: u32,     // 0=Null, 1=Bool, 2=I32, 3=I64, 4=F64, 5=Ref, 6=FnPtr
    pub flags: u32,   // Reserved flags / padding
    pub payload: u64, // Raw 64-bit value bits or heap reference address
}
```

### Native Function Signature
All native bridge functions conform to the 4-argument C-ABI signature (`NativeCFunctionPtr` in Rust, `SolixNativeFunctionPtr` in C/C++):

```rust
pub type NativeCFunctionPtr = unsafe extern "C" fn(
    vm_context: *mut std::ffi::c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue;
```

---

## 3. Scaffolding Dual-Mode Native Extensions

Both Rust (`Cargo.toml`) and C/C++ (`CMakeLists.txt`) native subprojects can be scaffolded directly with `solix new` or `solix new native`:

```bash
# Add a dual-mode Rust native bridge into an existing project
solix new native rust_bridge --project . --lang rust

# Add a dual-mode C++17 CMake native bridge targeting Linux, macOS, and Windows
solix new native cpp_bridge --project . --lang cpp --platforms linux,macos,windows

# Add a dual-mode C11 CMake native bridge
solix new native c_bridge --project . --lang c
```

Dual-mode layout ensures that:
* **Outside `solix build`:** `cargo build` / `cargo test` and `cmake` / `ctest` compile your C, C++, or Rust library cleanly with zero Solix dependency.
* **Inside `solix build`:** Solix automatically detects `Cargo.toml` (passing `--features solix-native` and `SOLIX_NATIVE_BUILD=1`) or `CMakeLists.txt` (passing `-DSOLIX_NATIVE_BUILD=ON` and `SOLIX_NATIVE_BUILD=1`) with zero manual `type` field in `manifest.yaml`.
* **Standalone `solix-ffi` Crate:** The `solix-ffi` SDK located in `ffi/rust` is packaged as an independent, self-contained crate (version 3.1.1, edition 2024). It can be consumed via Git dependency or local patch in containerized compilation pipelines.

### 3.1. Option A — Rust Native Bridge (`ffi/rust`, `solix-ffi`)

`natives/rust_bridge/Cargo.toml`:
```toml
[package]
name = "rust_bridge"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[features]
default = []
solix-native = ["dep:solix-ffi"]

[dependencies]
solix-ffi = { git = "https://github.com/Jehudme/solix-language.git", branch = "dev", optional = true }
```

`natives/rust_bridge/src/lib.rs`:
```rust
//! Dual-mode Rust library + Solix FFI bridge.

pub fn add_native(value_a: i32, value_b: i32) -> i32 {
    value_a + value_b
}

#[cfg(feature = "solix-native")]
mod bridge {
    use solix_ffi::{solix_export, NativeCFunctionPtr, SolixCValue};

    #[unsafe(no_mangle)]
    pub unsafe extern "C-unwind" fn MathBridge_fast_add(
        _vm_context: *mut std::ffi::c_void,
        arguments: *const SolixCValue,
        argument_count: u32,
        _out_error: *mut SolixCValue,
    ) -> SolixCValue {
        if argument_count < 2 || arguments.is_null() {
            return SolixCValue::null();
        }
        let value_a = unsafe { (*arguments.add(0)).as_i32() };
        let value_b = unsafe { (*arguments.add(1)).as_i32() };
        SolixCValue::from_i32(crate::add_native(value_a, value_b))
    }

    solix_export! {
        "MathBridge::fast_add" => MathBridge_fast_add,
    }
}
```

See the full [Rust FFI SDK Wiki](../wiki/ffi/rust-sdk.md) for `SolixVmContext` heap, string, array, object, and cycle-collector helpers.

### 3.2. Option B — C11 / C++17 CMake Bridge (`ffi/c`, `solix::ffi`)

`natives/c_bridge/CMakeLists.txt`:
```cmake
cmake_minimum_required(VERSION 3.20)
project(c_bridge VERSION 0.1.0 LANGUAGES C)

option(SOLIX_NATIVE_BUILD "Build as a Solix VM native plugin" OFF)

if(SOLIX_NATIVE_BUILD)
    include(FetchContent)
    if(DEFINED ENV{SOLIX_FFI_SOURCE_DIR} AND EXISTS "$ENV{SOLIX_FFI_SOURCE_DIR}/CMakeLists.txt")
        file(TO_CMAKE_PATH "$ENV{SOLIX_FFI_SOURCE_DIR}" SOLIX_FFI_LOCAL_DIR)
        FetchContent_Declare(solix_ffi SOURCE_DIR "${SOLIX_FFI_LOCAL_DIR}")
    else()
        FetchContent_Declare(
            solix_ffi
            GIT_REPOSITORY https://github.com/Jehudme/solix-language.git
            GIT_TAG        dev
            SOURCE_SUBDIR  ffi/c
            GIT_SHALLOW    TRUE
        )
    endif()
    FetchContent_MakeAvailable(solix_ffi)

    add_library(c_bridge SHARED src/bridge.c)
    target_link_libraries(c_bridge PRIVATE solix::ffi)
    target_compile_definitions(c_bridge PRIVATE SOLIX_NATIVE_BUILD=1)
else()
    add_library(c_bridge STATIC src/bridge.c)
endif()
```

In `src/bridge.c`:
```c
#ifdef SOLIX_NATIVE_BUILD
#include "solix_ffi.h"

static SolixCValue native_fast_add(
    void* vm_context,
    const SolixCValue* args,
    uint32_t arg_count,
    SolixCValue* out_error
) {
    (void)vm_context;
    (void)out_error;
    if (args == NULL || arg_count < 2) {
        return solix_cvalue_null();
    }
    return solix_cvalue_i32(solix_cvalue_as_i32(args[0]) + solix_cvalue_as_i32(args[1]));
}

SOLIX_REGISTER_MODULE(((const SolixNativeEntry[]) {
    { "MathBridge::fast_add", native_fast_add }
}))
#endif
```

See the full [C & C++ FFI SDK Wiki](../wiki/ffi/c-cpp-sdk.md) for complete `solix_vm_*` helpers, `out_error`, and `SOLIX_EXPORT_NATIVES_*` macros.

---

## 4. Building and Packaging Native Libraries

Register native subprojects under `profiles.<name>.natives` in `manifest.yaml` (no `type` field is needed; Solix auto-detects `Cargo.toml` vs `CMakeLists.txt`):

```yaml
profiles:
  release:
    entrypoint: Main::main
    platforms:
      - linux
      - windows
      - macos
    pre_compile: []
    natives:
      c_bridge:
        profile: release
```

Then run a single build command:

```bash
solix build --profile release
```

During `solix build`, the pure-Rust build orchestrator executes `pre_compile` hooks, compiles every native subproject under `natives/` across all expanded target triples (`bin/<target_triple>/__native_<name>`), embeds SHA-256 checksums in `manifest.yaml`, compiles Solix sources, packages the `.slxp` container with root `binaries.slxb`, and executes `post_compile` hooks.

> [!NOTE]
> Native libraries are hermetically packaged inside `.slxp` packages. When executing or distributing your application, the Solix runtime automatically extracts and loads matching native shared libraries starting with `__native_` or `lib__native_` for the host operating system and CPU architecture without searching host system PATH.

> [!IMPORTANT]
> All native symbols declared with `native` must be exported and registered via `solix_export!` (Rust) or `SOLIX_REGISTER_MODULE` (C/C++). Unresolved native symbols produce link errors during VM initialization.

---

### 4.1 Containerized Cross-Compilation (`strategy: docker`)

For native projects that require external host development libraries (such as X11, Wayland, ALSA, or Mesa OpenGL) that are unavailable in minimal cross-compilation toolchains, Solix supports containerized compilation via Docker:

```yaml
profiles:
  release:
    platforms:
      - all
    natives:
      heavy_native_module:
        profile: release
        docker: true # Containerized execution is mandated for native subprojects
```

* **Mandatory Containerization:** All native library subprojects (Cargo and CMake) mandate Docker containerization across all compilation targets, eliminating host cross-compiler prerequisites and preventing host instruction set leaks.
* **Rust Natives (`Cargo.toml`):** Solix compiles via `cross` using official `ghcr.io/cross-rs/<target>:edge` images for Linux/Windows and the automatically built OSXCross image (`solix/apple-darwin-cross:15.5-ubuntu`) for Apple Darwin (`x86_64-apple-darwin`, `aarch64-apple-darwin`), automatically injecting target `Cross.toml` and compiler configuration even when a project has no manual `Cross.toml`.
* **CMake Natives (`CMakeLists.txt`):** Solix compiles via `dockcross/<target>` images (e.g. `dockcross/linux-arm64`) for Linux/Windows and `solix/apple-darwin-cross:15.5-ubuntu` (OSXCross toolchain `/osxcross/toolchain.cmake`) for Apple Darwin.
* **Cache Isolation:** Solix automatically segregates container target directories (`build/.cache/cargo-docker` and `build/.cache/cmake-docker`) from host build caches, preventing host-compiled build scripts from failing inside container environments.

> [!NOTE]
> Containerized compilation ensures 100% reproducible and identical binary output across every host operating system, packaging multi-platform shared libraries directly from a single runner.

---

## 5. Next Steps & Further Reading

- **FFI SDK Wikis**:
  - [FFI Wiki Overview](../wiki/ffi/README.md)
  - [C & C++ SDK Reference (`solix::ffi`)](../wiki/ffi/c-cpp-sdk.md)
  - [Rust SDK Reference (`solix-ffi`)](../wiki/ffi/rust-sdk.md)
- **Live Reference Example**: [Dual-Backend FFI Matrix (`examples/ffi_dual_backend`)](../../examples/ffi_dual_backend/README.md)
- **Next Guide**: Learn how to bundle embedded assets and resources in [Packaging & Resources](packaging_and_resources.md).
- **Tooling Guide**: Configure your development environment in [VS Code Extension Setup](editors_vscode.md).
- **Previous Guide**: Review generics and monomorphization in [08 - Generics & Templates](08_generics_and_templates.md).
- **Further Reading**:
  - [Native FFI & ABI Specification](../spec/native_interop.md)
  - [native Keyword Modifier Reference](../wiki/keywords/modifiers/native.md)
  - [Binary Packaging Specification](../spec/binary-formats-slxb-and-slxp.md)
