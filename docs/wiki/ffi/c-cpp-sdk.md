# C & C++ FFI SDK Reference (`ffi/c`, `solix::ffi`)

The C and C++ FFI SDK lives at [`ffi/c/`](../../../ffi/c/) and exposes the `solix::ffi` CMake `INTERFACE` target alongside the single-header [`include/solix_ffi.h`](../../../ffi/c/include/solix_ffi.h) header compatible with **C11** (`-std=c11`) and **C++17** (`-std=c++17`).

---

## 1. CMake Integration & Dual-Mode Setup

### Scaffolding a C or C++ Native Library

```bash
# Add a C11 CMake native subproject to an existing Solix project
solix new native audio_dsp --project . --lang c

# Add a C++17 CMake native subproject targeting Linux and Windows
solix new native physics_engine --project . --lang cpp --platforms linux,windows
```

### Dual-Mode `CMakeLists.txt` Pattern

```cmake
cmake_minimum_required(VERSION 3.20)
project(my_native_lib VERSION 0.1.0 LANGUAGES C CXX)

option(SOLIX_NATIVE_BUILD "Build as a Solix VM native shared library plugin" OFF)
option(BUILD_TESTING "Build standalone unit tests" ON)

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

    add_library(my_native_lib SHARED src/core.c src/solix_bridge.cpp)
    target_link_libraries(my_native_lib PRIVATE solix::ffi)
    target_compile_definitions(my_native_lib PRIVATE SOLIX_NATIVE_BUILD=1)
else()
    add_library(my_native_lib STATIC src/core.c)
    if(BUILD_TESTING)
        enable_testing()
        add_executable(my_native_lib_tests tests/test_core.c)
        target_link_libraries(my_native_lib_tests PRIVATE my_native_lib)
        add_test(NAME my_native_lib_tests COMMAND my_native_lib_tests)
    endif()
endif()
```

### Target & Option Reference (`ffi/c/CMakeLists.txt`)

| Identifier | Kind | Default | Purpose |
| :--- | :--- | :--- | :--- |
| `solix::ffi` | CMake Alias (`solix_ffi` INTERFACE) | — | Adds `ffi/c/include` and defines `SOLIX_FFI_SDK_VERSION` |
| `SOLIX_FFI_BUILD_TESTS` | CMake Option | `OFF` | Builds `test_solix_ffi_c` (C11) and `test_solix_ffi_cpp` (C++17) under CTest |
| `SOLIX_NATIVE_BUILD` | User CMake Option | `OFF` | Set to `ON` by `solix build` (`-DSOLIX_NATIVE_BUILD=ON` + env `SOLIX_NATIVE_BUILD=1`) |
| `SOLIX_FFI_SOURCE_DIR` | Environment Variable | Unset | Points `FetchContent` to a local `ffi/c` checkout (injected automatically by `solix build` when available) |

---

## 2. Constants & Core Types (`solix_ffi.h`)

### Type Tags & Magic Constant

| Macro | Value | Meaning |
| :--- | :--- | :--- |
| `SOLIX_VM_CONTEXT_MAGIC` | `0x534F4C4958564D43ULL` | Magic header (`"SOLIXVMC"`) verified in `SolixVmContext` |
| `SOLIX_TAG_NULL` | `0u` | Null / uninitialized reference |
| `SOLIX_TAG_BOOL` | `1u` | Boolean scalar |
| `SOLIX_TAG_I32` | `2u` | 32-bit signed integer (`int32`) |
| `SOLIX_TAG_I64` | `3u` | 64-bit signed integer (`int64`) |
| `SOLIX_TAG_F64` | `4u` | 64-bit IEEE-754 float (`float64`) |
| `SOLIX_TAG_REF` | `5u` | Heap reference (`string`, `class`, array) |
| `SOLIX_TAG_FNPTR` | `6u` | Function pointer identifier |
| `SOLIX_ARRAY_TAG_I8` | `1u` | Element tag for `int8[]` |
| `SOLIX_ARRAY_TAG_I16` | `2u` | Element tag for `int16[]` |
| `SOLIX_ARRAY_TAG_I32` | `3u` | Element tag for `int32[]` |
| `SOLIX_ARRAY_TAG_I64` | `4u` | Element tag for `int64[]` |
| `SOLIX_ARRAY_TAG_U8` | `5u` | Element tag for `uint8[]` |
| `SOLIX_ARRAY_TAG_U16` | `6u` | Element tag for `uint16[]` |
| `SOLIX_ARRAY_TAG_U32` | `7u` | Element tag for `uint32[]` |
| `SOLIX_ARRAY_TAG_U64` | `8u` | Element tag for `uint64[]` |
| `SOLIX_ARRAY_TAG_F32` | `9u` | Element tag for `float32[]` |
| `SOLIX_ARRAY_TAG_F64` | `10u` | Element tag for `float64[]` |
| `SOLIX_ARRAY_TAG_BOOL` | `11u` | Element tag for `bool[]` |
| `SOLIX_ARRAY_TAG_CHAR` | `12u` | Element tag for `char[]` |
| `SOLIX_ARRAY_TAG_REF` | `13u` | Element tag for `string[]` and object reference arrays |

### Structs & Callback Typedefs

| Symbol | Fields / Signature | Purpose |
| :--- | :--- | :--- |
| `SolixCValue` | `{ uint32_t tag; uint32_t flags; uint64_t payload; }` | 16-byte C-ABI value container |
| `SolixNativeFunctionPtr` | `SolixCValue (*)(void* vm_context, const SolixCValue* args, uint32_t arg_count, SolixCValue* out_error)` | Canonical native function pointer signature |
| `SolixRegisterCallback` | `void (*)(void* collector_context, const char* symbol_name, SolixNativeFunctionPtr fn_ptr)` | VM registration callback passed into `solix_register_natives` |
| `SolixNativeEntry` | `{ const char* symbol_name; SolixNativeFunctionPtr fn_ptr; }` | Symbol-to-function descriptor used by `SOLIX_REGISTER_MODULE` |
| `SolixVmContext` | `{ magic, vm_ptr, get_field_i32, set_field_i32, read_bytes, alloc_string, alloc_chars, alloc_byte_array, alloc_array, array_len, array_get_u64, array_set_u64, alloc_object, read_field_u64, write_field_u64, get_class_vtable_id, get_vm_metric, collect_cycles, set_cycle_threshold, get_field_i32_by_index, set_field_i32_by_index, get_field_i32_by_offset, set_field_i32_by_offset, read_field_u64_by_index, write_field_u64_by_index, read_field_u64_by_offset, write_field_u64_by_offset }` | Live VM callback vtable supplied by `vm_context` |

---

## 3. `SolixCValue` Helper Functions

All helpers are `static inline` and safe to include across multiple translation units in C11 and C++17:

### Constructors
* `SolixCValue solix_cvalue_null(void)`
* `SolixCValue solix_cvalue_bool(bool value)`
* `SolixCValue solix_cvalue_i32(int32_t value)`
* `SolixCValue solix_cvalue_i64(int64_t value)`
* `SolixCValue solix_cvalue_f64(double value)`
* `SolixCValue solix_cvalue_ref(uint32_t heap_address)`
* `SolixCValue solix_cvalue_fnptr(uint32_t func_id)`

### Predicates & Unpackers
* `bool solix_cvalue_is_null(SolixCValue value)` — returns `true` if `tag == SOLIX_TAG_NULL` or (`tag == SOLIX_TAG_REF && payload == 0`).
* `bool solix_cvalue_as_bool(SolixCValue value)`
* `int32_t solix_cvalue_as_i32(SolixCValue value)`
* `int64_t solix_cvalue_as_i64(SolixCValue value)`
* `double solix_cvalue_as_f64(SolixCValue value)`
* `uint32_t solix_cvalue_as_ref(SolixCValue value)`
* `uint32_t solix_cvalue_as_fnptr(SolixCValue value)`

---

## 4. `SolixVmContext` Runtime & Heap Functions

Always extract and validate `const SolixVmContext*` with `solix_vm_context_from_ptr(vm_context)` before calling any VM helper:

| Helper Function | Parameters | Return Value | Behavior |
| :--- | :--- | :--- | :--- |
| `solix_vm_context_from_ptr` | `(void* raw_context)` | `const SolixVmContext*` | Returns `NULL` if pointer is `NULL`, `magic != SOLIX_VM_CONTEXT_MAGIC`, or `vm_ptr == NULL` |
| `solix_vm_read_bytes` | `(const SolixVmContext* ctx, uint32_t heap_addr, uint8_t* dest, uint32_t max_len)` | `uint32_t` | Copies UTF-8/byte payload from `heap_addr` (transcoding Unicode `char[]` elements to UTF-8); returns full byte length |
| `solix_vm_read_string_alloc` | `(const SolixVmContext* ctx, uint32_t heap_addr, size_t* out_len)` | `char*` | Heap-allocates a NUL-terminated C string via `malloc` (**caller must `free()`**) |
| `solix_vm_alloc_string` | `(const SolixVmContext* ctx, const char* c_str)` | `uint32_t` | Allocates a VM guest `string` from NUL-terminated UTF-8 |
| `solix_vm_alloc_string_len` | `(const SolixVmContext* ctx, const char* utf8, uint32_t byte_len)` | `uint32_t` | Allocates a VM guest `string` with explicit length |
| `solix_vm_alloc_chars` | `(const SolixVmContext* ctx, const char* utf8, uint32_t byte_len)` | `uint32_t` | Allocates a mutable guest `char[]` array decoded as Unicode scalar code points from UTF-8 |
| `solix_vm_alloc_byte_array` | `(const SolixVmContext* ctx, const uint8_t* raw_bytes, uint32_t byte_len)` | `uint32_t` | Allocates a guest `int8[]` array |
| `solix_vm_alloc_array` | `(const SolixVmContext* ctx, uint32_t element_count, uint8_t type_tag)` | `uint32_t` | Allocates an `ArrayRef` with element tag `SOLIX_ARRAY_TAG_*` |
| `solix_vm_array_len` | `(const SolixVmContext* ctx, uint32_t array_addr)` | `uint32_t` | Returns array element length (or string byte length) |
| `solix_vm_array_get_u64` | `(const SolixVmContext* ctx, uint32_t array_addr, uint32_t index)` | `uint64_t` | Reads element at `index` as raw 64-bit bits (`float32[]` normalized to IEEE-754 `double` bits) |
| `solix_vm_array_set_u64` | `(const SolixVmContext* ctx, uint32_t array_addr, uint32_t index, uint64_t raw_value)` | `void` | Writes raw 64-bit value into element `index` |
| `solix_vm_get_class_vtable_id` | `(const SolixVmContext* ctx, const char* class_name)` | `uint32_t` | Looks up class VTable ID by qualified or simple name (`0` if not found) |
| `solix_vm_alloc_object` | `(const SolixVmContext* ctx, uint32_t vtable_id, uint32_t field_count)` | `uint32_t` | Allocates an instance of the target class on the ARC heap |
| `solix_vm_get_field_i32` | `(const SolixVmContext* ctx, uint32_t obj_addr, uint32_t offset)` | `int32_t` | Legacy heuristic 32-bit field read (prefer explicit `*_by_index` / `*_by_offset`) |
| `solix_vm_set_field_i32` | `(const SolixVmContext* ctx, uint32_t obj_addr, uint32_t offset, int32_t value)` | `void` | Legacy heuristic 32-bit field write (prefer explicit `*_by_index` / `*_by_offset`) |
| `solix_vm_read_field_u64` | `(const SolixVmContext* ctx, uint32_t obj_addr, uint32_t offset)` | `uint64_t` | Legacy heuristic 64-bit field read (prefer explicit `*_by_index` / `*_by_offset`) |
| `solix_vm_write_field_u64` | `(const SolixVmContext* ctx, uint32_t obj_addr, uint32_t offset, uint64_t raw_value)` | `void` | Legacy heuristic 64-bit field write (prefer explicit `*_by_index` / `*_by_offset`) |
| `solix_vm_get_field_i32_by_index` | `(const SolixVmContext* ctx, uint32_t obj_addr, uint32_t field_index)` | `int32_t` | Reads a 32-bit integer field using explicit 0-based `field_index` |
| `solix_vm_set_field_i32_by_index` | `(const SolixVmContext* ctx, uint32_t obj_addr, uint32_t field_index, int32_t value)` | `void` | Writes a 32-bit integer field using explicit 0-based `field_index` |
| `solix_vm_get_field_i32_by_offset` | `(const SolixVmContext* ctx, uint32_t obj_addr, uint32_t byte_offset)` | `int32_t` | Reads a 32-bit integer field using explicit payload `byte_offset` |
| `solix_vm_set_field_i32_by_offset` | `(const SolixVmContext* ctx, uint32_t obj_addr, uint32_t byte_offset, int32_t value)` | `void` | Writes a 32-bit integer field using explicit payload `byte_offset` |
| `solix_vm_read_field_u64_by_index` | `(const SolixVmContext* ctx, uint32_t obj_addr, uint32_t field_index)` | `uint64_t` | Reads a field using explicit 0-based `field_index` (handles sub-64-bit widths and reference tags) |
| `solix_vm_write_field_u64_by_index` | `(const SolixVmContext* ctx, uint32_t obj_addr, uint32_t field_index, uint64_t raw_value)` | `void` | Writes a field using explicit 0-based `field_index` |
| `solix_vm_read_field_u64_by_offset` | `(const SolixVmContext* ctx, uint32_t obj_addr, uint32_t byte_offset)` | `uint64_t` | Reads a field using explicit payload `byte_offset` (handles sub-64-bit widths and reference tags) |
| `solix_vm_write_field_u64_by_offset` | `(const SolixVmContext* ctx, uint32_t obj_addr, uint32_t byte_offset, uint64_t raw_value)` | `void` | Writes a field using explicit payload `byte_offset` |
| `solix_vm_get_metric` | `(const SolixVmContext* ctx, uint32_t metric_id)` | `uint64_t` | Reads a VM runtime metric (`0..=55`) |
| `solix_vm_collect_cycles` | `(const SolixVmContext* ctx)` | `uint64_t` | Runs the Bacon & Rajan cycle collector and returns reclaimed object count |
| `solix_vm_set_cycle_threshold` | `(const SolixVmContext* ctx, uint32_t threshold)` | `void` | Adjusts the cycle collector candidate threshold |
| `solix_vm_set_error` | `(void* raw_vm_context, SolixCValue* out_error, const char* message)` | `void` | Allocates `message` on the VM heap and writes it to `*out_error` (falls back to a non-null scalar tag if unavailable) |

---

## 5. Symbol Registration Macros & Examples

`solix_ffi.h` offers two cross-platform registration styles (both automatically handle `extern "C"` and `__declspec(dllexport)` / `visibility("default")`):

### Style A: Table Macro (`SOLIX_REGISTER_MODULE`) in C11 or C++17

```c
#include "solix_ffi.h"
#include <stdlib.h>
#include <string.h>

static SolixCValue native_greet(
    void* vm_context,
    const SolixCValue* args,
    uint32_t arg_count,
    SolixCValue* out_error
) {
    const SolixVmContext* ctx = solix_vm_context_from_ptr(vm_context);
    if (ctx == NULL || args == NULL || arg_count < 1) {
        return solix_cvalue_null();
    }

    uint32_t name_len = 0;
    char* guest_name = solix_vm_read_string_alloc(ctx, solix_cvalue_as_ref(args[0]), &name_len);
    if (guest_name == NULL) {
        solix_vm_set_error(ctx, out_error, "Failed to read guest string");
        return solix_cvalue_null();
    }

    char message_buf[256];
    size_t copy_len = name_len < 200 ? (size_t)name_len : 200;
    memcpy(message_buf, "Hello, ", 7);
    memcpy(message_buf + 7, guest_name, copy_len);
    message_buf[7 + copy_len] = '\0';
    free(guest_name);

    uint32_t result_addr = solix_vm_alloc_string(ctx, message_buf);
    return solix_cvalue_ref(result_addr);
}

static const SolixNativeEntry MY_EXPORTS[] = {
    { "Greeter::greet", native_greet }
};

SOLIX_REGISTER_MODULE(MY_EXPORTS)
```

### Style B: Block Macros (`SOLIX_EXPORT_NATIVES_*`)

```cpp
#include "solix_ffi.h"

static SolixCValue native_multiply(
    void* vm_context,
    const SolixCValue* args,
    uint32_t arg_count,
    SolixCValue* out_error
) {
    (void)vm_context;
    (void)out_error;
    if (args == nullptr || arg_count < 2) {
        return solix_cvalue_null();
    }
    int32_t left_value = solix_cvalue_as_i32(args[0]);
    int32_t right_value = solix_cvalue_as_i32(args[1]);
    return solix_cvalue_i32(left_value * right_value);
}

SOLIX_EXPORT_NATIVES_BEGIN()
    SOLIX_EXPORT_NATIVE("MathBridge::multiply", native_multiply)
SOLIX_EXPORT_NATIVES_END()
```

---

## Related Documentation
* [FFI Wiki Overview](README.md)
* [Rust SDK Reference (`solix-ffi`)](rust-sdk.md)
* [Dual-Backend Example (`examples/ffi_dual_backend`)](../../../examples/ffi_dual_backend/README.md)
* [Normative Native FFI & ABI Specification](../../spec/native_interop.md)
