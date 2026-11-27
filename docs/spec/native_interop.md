# Formal Specification: Native Interoperability & C-FFI ABI

## Document Metadata

* **Specification ID:** SPEC-0011
* **Scope:** Native Interoperability, FFI SDKs (`ffi/c`, `ffi/rust`), ABI & Dynamic Shared Libraries (`crates/solix-runtime/src/native/`)
* **Status:** Final Normative Standard (SDK Release `3.1.1`)
* **Implementation Source:** [`crates/solix-runtime/src/native/ffi.rs`](../../crates/solix-runtime/src/native/ffi.rs)
* **SDK Version Constant:** `SOLIX_FFI_VERSION` (`3.1.1`), `SOLIX_FFI_VERSION_MAJOR` (3)

---

## 1. Overview & Architecture

The Solix Virtual Machine provides Foreign Function Interface (FFI) bindings for native code authored in Rust, C, or C++. Native libraries are distributed as platform-standard shared libraries:
- **Linux**: `.so` (ELF dynamic shared object)
- **macOS**: `.dylib` (Mach-O dynamic library)
- **Windows**: `.dll` (PE dynamic link library)

Shared libraries are loaded dynamically at runtime via `libloading` and registered into the VM's native dispatch table.

---

## 2. ABI Value Representation: `SolixCValue`

Values passed between the Solix virtual machine and native functions are encoded in a `SolixCValue` struct (exact size 16 bytes, 8-byte alignment):

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolixCValue {
    /// Type classification:
    /// 0 = Null, 1 = Bool, 2 = I32, 3 = I64, 4 = F64, 5 = Ref, 6 = FnPtr
    pub tag: u32,
    /// Reserved flags / padding
    pub flags: u32,
    /// 64-bit payload bits or heap reference address
    pub payload: u64,
}
```

### Type Tag Mapping

| Tag Value | Type | Description |
|---|---|---|
| `0` | `Null` | Absence of value (`null`) |
| `1` | `Bool` | Boolean truth value (`1` = `true`, `0` = `false`) |
| `2` | `I32` | 32-bit signed integer |
| `3` | `I64` | 64-bit signed integer |
| `4` | `F64` | IEEE-754 64-bit double precision float bits |
| `5` | `Ref` | 32-bit byte offset address within dynamic virtual heap |
| `6` | `FnPtr` | Virtual machine function index |

---

## 3. Native Function Pointer Contracts

### 3.1. Standard Native Call Signature
Every native method called from Solix bytecode implements `NativeCFunctionPtr`:

```rust
pub type NativeCFunctionPtr = unsafe extern "C-unwind" fn(
    vm_context: *mut std::ffi::c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue;
```

#### Contract Parameters:
1. **`vm_context`**: Opaque pointer to the host `SolixVmContext` enabling memory queries and allocations.
2. **`args`**: Contiguous array of `SolixCValue` structures representing call arguments left-to-right.
3. **`arg_count`**: Number of passed arguments.
4. **`out_error`**: Error output slot. If set to non-zero `tag`, the runtime raises `RuntimeError::NativeFunctionError`.

Native invocations in `invoke_c_native` are protected via `std::panic::catch_unwind(std::panic::AssertUnwindSafe(...))`. If a foreign plugin written in Rust panics, the exception is caught across the boundary and converted cleanly into `RuntimeError::NativeFunctionError` without aborting the host runtime.

### 3.2. Batch Registration Hook & Registry Representation
`NativeRegistry` stores dynamic C callbacks under `NativeCallback::CAbi(NativeCFunctionPtr)` alongside host `NativeCallback::Rust(NativeFunction)` builtins, resolving `*mut c_void` runtime contexts safely through checked helpers in [`crates/solix-runtime/src/native/ffi.rs`](../../crates/solix-runtime/src/native/ffi.rs).

Shared libraries export an optional batch registration entry point:

```rust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_register_natives(
    collector_context: *mut std::ffi::c_void,
    register_callback: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_char, NativeCFunctionPtr),
);
```

### 3.3. `ffi/rust` (`solix-ffi`) & `ffi/c` (`solix::ffi`) Native Extension SDKs
- **Rust SDK ([`ffi/rust/`](../../ffi/rust/src/lib.rs)):** The `solix-ffi` crate provides typed wrappers over `SolixCValue`, `NativeCFunctionPtr = unsafe extern "C-unwind" fn(...)`, complete `SOLIX_ARRAY_TAG_*` constants (`I8`, `I16`, `I32`, `I64`, `U8`, `U16`, `U32`, `U64`, `F32`, `F64`, `BOOL`, `CHAR`, `REF`), explicit `*_by_index` / `*_by_offset` `SolixVmContext` field accessors, and null/panic-guarded `solix_export!` trampolines (`invoke_caught` wrapping user callbacks in `std::panic::catch_unwind`).
- **C11 / C++17 CMake SDK ([`ffi/c/`](../../ffi/c/include/solix_ffi.h)):** Exports interface target `solix::ffi` (`CMakeLists.txt` with `SOURCE_SUBDIR ffi/c`) and [`solix_ffi.h`](../../ffi/c/include/solix_ffi.h) providing exact ABI-matched `SolixCValue`, `SolixVmContext` (verified with `SOLIX_VM_CONTEXT_MAGIC == 0x534F4C4958564D43`), inline value constructors/unwrappers, `solix_vm_*` host heap and explicit `*_by_index` / `*_by_offset` field helpers, `solix_vm_set_error`, and cross-platform export macros (`SOLIX_REGISTER_MODULE` / `SOLIX_EXPORT_NATIVES_BEGIN`).

---

## 4. Host Memory Operations via `SolixVmContext`

Native functions can read and write heap structures via VM helper callbacks (all arithmetic uses `checked_add`/`checked_mul` and validates VM pointers via `resolve_vm`):
- `alloc_byte_array`: Allocates contiguous byte buffer on virtual heap using checked arithmetic.
- `alloc_string`: Allocates immutable raw UTF-8 string buffer (`BlockTag::String`) and tracks its address for clean teardown upon runtime shutdown.
- `alloc_chars` & `read_bytes`: Transcode seamlessly between UTF-8 host strings and Solix `char[]` arrays (`type_tag == 12`) holding Unicode scalar values (`alloc_chars` decodes UTF-8 scalar code points with an ASCII fast path; `read_bytes` encodes `char[]` code points back into UTF-8 and returns full byte count).
- `alloc_array`: Allocates typed array of specified length with checked capacity bounds.
- `array_len`: Returns element count of heap array.
- `array_get_u64` & `array_set_u64`: Accesses elements by index using checked slot offset math. For `float32[]` arrays (`type_tag == 9`), transparently normalizes between 32-bit heap bit-patterns and standard IEEE-754 64-bit float representations.
- `alloc_object`: Allocates an object instance on the virtual heap, reserving `class_descriptor.instance_size.max(8 + field_count * 8)` bytes via checked arithmetic to prevent integer overflow.
- `get_class_vtable_id`: Resolves the unique virtual table index for a class by name (`*const c_char`), enabling typed object instantiation via `alloc_object`.
- `get_field_i32_by_index` / `set_field_i32_by_index` & `get_field_i32_by_offset` / `set_field_i32_by_offset`: Unambiguous 32-bit integer field accessors indexed strictly by 0-based `field_index` or direct payload `byte_offset`, avoiding threshold collisions on classes with `>= 8` fields.
- `read_field_u64_by_index` / `write_field_u64_by_index` & `read_field_u64_by_offset` / `write_field_u64_by_offset`: Unambiguous width- and tag-aware field accessors indexed strictly by 0-based `field_index` or direct payload `byte_offset` (handling sub-64-bit widths, `float32` slots, and Tag 12 `Value::Ref` boxing without colliding on offsets `< 64`). Legacy heuristic `get_field_i32`, `set_field_i32`, `read_field_u64`, and `write_field_u64` remain preserved for existing plugins.

---

## 5. Standard Built-in Native Functions & Output Redirection Sink

The virtual machine registers essential built-in native functions into the standard dispatch table:
- `println` & `Console::println`: Writes string arguments with a trailing newline.
- `print` & `Console::print`: Writes string arguments without trailing newlines.
- `assert`: Evaluates a boolean invariant and raises `RuntimeError::AssertionFailed` upon violation.
- `clock`: Returns high-resolution timestamps in seconds.

### Output Redirection via `OutputSink`
When the virtual machine executes within an interactive Debug Adapter Protocol (`solix dap`) session or headless runner, `builtin_println` and `builtin_print` dynamically redirect text output:
- **Default Execution**: If `vm.output_sink` is `None`, strings are formatted and output to standard host console output (`println!`/`print!`).
- **DAP Supervised Execution**: If `vm.output_sink` is configured (`Some(sink_handler)`), standard output stream writes are routed into the thread-safe `OutputSink` callback closure, emitting DAP `Event::Output` packets with `category: OutputEventCategory::Stdout`. This prevents guest stdout pollution or pipe corruption across the JSON-RPC wire channel.

### Dynamic FFI String Allocation Memory Management
Dynamic native FFI string allocations (`solix_vm_alloc_string`) create heap-allocated `BlockTag::String` buffers without leaking entries into the immutable constant pool. Each dynamic string is managed directly by automatic reference counting and cycle collection on the guest heap.

---

## 6. Native Dynamic Library Loader Isolation & Filter Policy

To eliminate security risks and unpredictable host environment pollution, the Solix runtime enforces strict library discovery isolation:

1. **Restricted Target Platform Directory**: The runtime never searches arbitrary system directories or the host system environment search paths. Instead, native modules are resolved exclusively from the active platform target directory (`bin/<target_triple>/` with fallback to `lib/<target_triple>/`) inside the extracted package cache or explicit `--lib` command options.
2. **Solix Native Module Prefix & Suffix Filtering**: By default, dynamic loader discovery automatically loads only shared libraries following Solix native plugin naming conventions:
   - Module prefix: `__native_*` or `lib__native_*` (e.g., `lib__native_math.so`, `__native_math.dll`).
   - Module suffix: `*_native` or `lib*native` (e.g., `libsolixlib_native.so`).
3. **Explicit Declaration via `dynamic_libraries`**: Shared libraries that do not use standard `__native_*` naming prefixes are only loaded into the virtual machine symbol table if explicitly declared in the project's `manifest.yaml` (under `dynamic_libraries` or profile `natives`) or embedded within the binary container's `dynamic_libraries` metadata list.
4. **Transitive Support Library Isolation**: Non-Solix third-party companion libraries (such as system drivers, rendering libraries, or windowing dependencies like `libglfw.so` or `glfw3.dll`) placed alongside native plugins in target folders are extracted to disk for operating system dynamic linker resolution, but are strictly excluded from registration in the Solix VM FFI native dispatch table.

---

## 7. C-FFI Header Version Handshake & Synchronization

The C/C++ FFI SDK header (`solix_ffi.h`) defines compile-time SemVer preprocessor macros to ensure binary and structural interface compatibility:

```c
#define SOLIX_FFI_VERSION_MAJOR 3
#define SOLIX_FFI_VERSION_MINOR 1
#define SOLIX_FFI_VERSION_PATCH 1
#define SOLIX_FFI_VERSION       "3.1.1"
```

During initialization, native extension modules can verify that the runtime major and minor versions satisfy their structural expectations. Automated CTest suites dynamically validate that `SOLIX_FFI_VERSION` correctly formats `SOLIX_FFI_VERSION_MAJOR`, `SOLIX_FFI_VERSION_MINOR`, and `SOLIX_FFI_VERSION_PATCH` without requiring hardcoded patch numbers.

---

## Related Documentation
* [FFI Wiki Overview](../wiki/ffi/README.md)
* [C & C++ SDK Reference (`solix::ffi`)](../wiki/ffi/c-cpp-sdk.md)
* [Rust SDK Reference (`solix-ffi`)](../wiki/ffi/rust-sdk.md)
* [Native Plugins Developer Guide](../guide/09_native_plugins.md)
* [Dual-Backend FFI Example (`examples/ffi_dual_backend`)](../../examples/ffi_dual_backend/README.md)
