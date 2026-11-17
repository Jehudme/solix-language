# Formal Specification: Native Interoperability & C-FFI ABI

## Document Metadata

* **Specification ID:** SPEC-0011
* **Scope:** Native Interoperability, FFI Bridge & Dynamic Shared Libraries (`crates/solix-runtime/src/native/`)
* **Status:** Final Normative Standard
* **Implementation Source:** [`crates/solix-runtime/src/native/ffi.rs`](../../crates/solix-runtime/src/native/ffi.rs)

---

## 1. Overview & Architecture

The Solix Virtual Machine provides Foreign Function Interface (FFI) bindings for native code authored in Rust, C, or C++. Native libraries are distributed as platform-standard shared libraries:
- **Linux**: `.so` (ELF dynamic shared object)
- **macOS**: `.dylib` (Mach-O dynamic library)
- **Windows**: `.dll` (PE dynamic link library)

Shared libraries are loaded dynamically at runtime via `libloading` and registered into the VM's native dispatch table.

---

## 2. ABI Value Representation: `SolixCValue`

Values passed between the Solix virtual machine and native functions are encoded in a 16-byte aligned `SolixCValue` struct:

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
pub type NativeCFunctionPtr = unsafe extern "C" fn(
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

---

## 4. Host Memory Operations via `SolixVmContext`

Native functions can read and write heap structures via VM helper callbacks:
- `alloc_byte_array`: Allocates contiguous byte buffer on virtual heap.
- `alloc_string`: Allocates immutable raw UTF-8 string buffer (`BlockTag::String`) and tracks its address for clean teardown upon runtime shutdown.
- `alloc_chars`: Allocates unpacked `char[]` array buffer for Solix character slice operations.
- `alloc_array`: Allocates typed array of specified length.
- `array_len`: Returns element count of heap array.
- `array_get_u64` & `array_set_u64`: Accesses elements by index. For `float32[]` arrays (`type_tag == 9`), transparently normalizes between 32-bit heap bit-patterns and standard IEEE-754 64-bit float representations.
- `alloc_object`: Allocates an object instance on the virtual heap, reserving `class_desc.instance_size.max(8 + field_count * 8)` bytes for proper field layout alignment.
- `get_class_vtable_id`: Resolves the unique virtual table index for a class by name (`*const c_char`), enabling typed object instantiation via `alloc_object`.
- `read_field_u64`: Reads a field slot from a heap object using resolved field index or direct byte offset, adapting byte width and IEEE-754 bit-encoding according to class type tags (e.g. reading 32-bit `float32` bits without clobbering adjacent fields, and extracting untagged addresses from Tag 12 `Value::Ref`).
- `write_field_u64`: Writes a field value to a heap object's slot according to resolved type tags, writing only the defined field size (1, 2, 4, or 8 bytes), storing `float32` bits into 4-byte slots, and applying `Value::TAG_REF` boxing for Tag 12 reference fields.

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
