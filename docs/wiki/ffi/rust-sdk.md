# Rust FFI SDK Reference (`ffi/rust`, `solix-ffi`)

The Rust FFI SDK crate lives at [`ffi/rust/`](../../../ffi/rust/) (package name `solix-ffi`, library target `solix_ffi`). It provides safe `SolixCValue` conversions, `SolixVmContext` validation and heap helpers, and the declarative `solix_export!` registration macro.

---

## 1. Cargo Configuration & Dual-Mode Setup

### Scaffolding a Rust Native Library

```bash
solix new native fast_hash --project . --lang rust
```

### Dual-Mode `Cargo.toml` Pattern

Configure `crate-type = ["cdylib", "rlib"]` and an optional `solix-native` feature so pure Rust crates and unit tests (`cargo test`) have zero Solix FFI dependency until built as a Solix plugin:

```toml
[package]
name = "fast_hash"
version = "0.1.0"
edition = "2024"
license = "Apache-2.0"

[lib]
crate-type = ["cdylib", "rlib"]

[features]
default = []
solix-native = ["dep:solix-ffi"]

[dependencies]
solix-ffi = { git = "https://github.com/Jehudme/solix-language.git", branch = "dev", optional = true }
```

During `solix build`, Solix automatically detects `[features].solix-native`, passes `--features solix-native` plus `SOLIX_NATIVE_BUILD=1`, and injects a local Cargo `[patch]` pointing `solix-ffi` to `ffi/rust/` when working inside a Solix source checkout (or when `SOLIX_FFI_RUST_DIR` is set).

---

## 2. Constants & Core Types (`solix_ffi`)

Re-exported at crate root (`use solix_ffi::*;`):

### Constants

| Constant | Type | Value | Meaning |
| :--- | :--- | :--- | :--- |
| `SOLIX_VM_CONTEXT_MAGIC` | `u64` | `0x534F_4C49_5856_4D43` | Magic header verified by `SolixVmContext::validate` |
| `TAG_NULL` | `u32` | `0` | Null reference |
| `TAG_BOOL` | `u32` | `1` | Boolean scalar |
| `TAG_I32` | `u32` | `2` | 32-bit signed integer |
| `TAG_I64` | `u32` | `3` | 64-bit signed integer |
| `TAG_F64` | `u32` | `4` | 64-bit IEEE-754 float |
| `TAG_REF` | `u32` | `5` | VirtualHeap reference (`string`, object, array) |
| `TAG_FNPTR` | `u32` | `6` | Function pointer index |
| `SOLIX_ARRAY_TAG_I8` | `u8` | `1` | Element tag for `int8[]` |
| `SOLIX_ARRAY_TAG_I16` | `u8` | `2` | Element tag for `int16[]` |
| `SOLIX_ARRAY_TAG_I32` | `u8` | `3` | Element tag for `int32[]` |
| `SOLIX_ARRAY_TAG_I64` | `u8` | `4` | Element tag for `int64[]` |
| `SOLIX_ARRAY_TAG_U8` | `u8` | `5` | Element tag for `uint8[]` |
| `SOLIX_ARRAY_TAG_U16` | `u8` | `6` | Element tag for `uint16[]` |
| `SOLIX_ARRAY_TAG_U32` | `u8` | `7` | Element tag for `uint32[]` |
| `SOLIX_ARRAY_TAG_U64` | `u8` | `8` | Element tag for `uint64[]` |
| `SOLIX_ARRAY_TAG_F32` | `u8` | `9` | Element tag for `float32[]` |
| `SOLIX_ARRAY_TAG_F64` | `u8` | `10` | Element tag for `float64[]` |
| `SOLIX_ARRAY_TAG_BOOL` | `u8` | `11` | Element tag for `bool[]` |
| `SOLIX_ARRAY_TAG_CHAR` | `u8` | `12` | Element tag for `char[]` |
| `SOLIX_ARRAY_TAG_REF` | `u8` | `13` | Element tag for `string[]` / reference arrays |

### Callback & Registry Types

* `NativeCFunctionPtr`: `unsafe extern "C-unwind" fn(vm_context: *mut c_void, args: *const SolixCValue, arg_count: u32, out_error: *mut SolixCValue) -> SolixCValue`
* `RegisterNativeCallback`: `unsafe extern "C" fn(collector_context: *mut c_void, name: *const c_char, func: NativeCFunctionPtr)`
* `NativeRegistry`: safe wrapper around `RegisterNativeCallback` offering `register(&mut self, name: &str, func: NativeCFunctionPtr)` and null-checked `from_raw`
* `solix_export!`: emits `solix_register_natives` with null host-pointer guards and wraps every exported plugin function in a `std::panic::catch_unwind` `extern "C-unwind"` trampoline (`invoke_caught`) that converts any Rust panic into `*out_error` without unwinding past the ABI boundary

---

## 3. `SolixCValue` API Reference

`SolixCValue` is `#[repr(C)]`, `Copy`, `Clone`, `PartialEq`, `Eq`, and implements `Default` (`Self::null()`).

### Constructors
* `SolixCValue::null() -> Self`
* `SolixCValue::from_bool(boolean_value: bool) -> Self`
* `SolixCValue::from_i32(integer_value: i32) -> Self`
* `SolixCValue::from_i64(integer_value: i64) -> Self`
* `SolixCValue::from_f64(float_value: f64) -> Self`
* `SolixCValue::from_ref(heap_address: u32) -> Self`
* `SolixCValue::from_fn_ptr(func_id: u32) -> Self`

### Unpackers & Predicates
* `is_null(self) -> bool`
* `as_bool(self) -> bool`
* `as_i32(self) -> i32`
* `as_i64(self) -> i64`
* `as_f64(self) -> f64`
* `as_ref(self) -> u32`
* `as_fn_ptr(self) -> u32`

---

## 4. `SolixVmContext` Safe Wrapper Methods

Always validate `raw_context` first with `unsafe { SolixVmContext::validate(vm_context) }`:

| Method | Signature | Behavior |
| :--- | :--- | :--- |
| `validate` | `unsafe fn validate<'ctx>(raw_context: *mut c_void) -> Option<&'ctx Self>` | Checks non-null pointer, `SOLIX_VM_CONTEXT_MAGIC`, and non-null `vm_ptr` |
| `read_string` | `unsafe fn read_string(&self, heap_address: u32) -> Option<String>` | Reads full UTF-8 string bytes from `heap_address` (handles strings > 4 KiB automatically) |
| `read_bytes` | `unsafe fn read_bytes(&self, heap_address: u32, destination_buffer: &mut [u8]) -> usize` | Copies payload into caller slice (transcoding Unicode `char[]` elements to UTF-8) and returns full byte count |
| `alloc_string` | `unsafe fn alloc_string(&self, text: &str) -> u32` | Allocates a new guest `string` on the ARC heap |
| `alloc_chars` | `unsafe fn alloc_chars(&self, text: &str) -> u32` | Allocates a mutable guest `char[]` array storing decoded Unicode scalar code points |
| `alloc_byte_array` | `unsafe fn alloc_byte_array(&self, raw_bytes: &[u8]) -> u32` | Allocates a guest `int8[]` array |
| `alloc_array` | `unsafe fn alloc_array(&self, element_count: u32, type_tag: u8) -> u32` | Allocates a typed guest array using `SOLIX_ARRAY_TAG_*` |
| `array_len` | `unsafe fn array_len(&self, array_address: u32) -> u32` | Returns array element length (or string byte length) |
| `array_get_u64` | `unsafe fn array_get_u64(&self, array_address: u32, element_index: u32) -> u64` | Reads raw 64-bit bits of element slot `element_index` |
| `array_set_u64` | `unsafe fn array_set_u64(&self, array_address: u32, element_index: u32, raw_value: u64)` | Writes raw 64-bit bits into element slot `element_index` |
| `get_class_vtable_id` | `unsafe fn get_class_vtable_id(&self, class_name: &str) -> u32` | Resolves class VTable index (`0` if unknown) |
| `alloc_object` | `unsafe fn alloc_object(&self, class_vtable_id: u32, field_count: u32) -> u32` | Allocates an ARC-managed class instance |
| `get_field_i32` | `unsafe fn get_field_i32(&self, object_address: u32, field_index: u32) -> i32` | Legacy heuristic 32-bit signed integer field read |
| `set_field_i32` | `unsafe fn set_field_i32(&self, object_address: u32, field_index: u32, field_value: i32)` | Legacy heuristic 32-bit signed integer field write |
| `read_field_u64` | `unsafe fn read_field_u64(&self, object_address: u32, field_offset: u32) -> u64` | Legacy heuristic field slot read |
| `write_field_u64` | `unsafe fn write_field_u64(&self, object_address: u32, field_offset: u32, raw_value: u64)` | Legacy heuristic field slot write |
| `get_field_i32_by_index` | `unsafe fn get_field_i32_by_index(&self, object_address: u32, field_index: u32) -> i32` | Reads 32-bit signed integer field by explicit 0-based `field_index` |
| `set_field_i32_by_index` | `unsafe fn set_field_i32_by_index(&self, object_address: u32, field_index: u32, field_value: i32)` | Writes 32-bit signed integer field by explicit 0-based `field_index` |
| `get_field_i32_by_offset` | `unsafe fn get_field_i32_by_offset(&self, object_address: u32, byte_offset: u32) -> i32` | Reads 32-bit signed integer field by explicit `byte_offset` |
| `set_field_i32_by_offset` | `unsafe fn set_field_i32_by_offset(&self, object_address: u32, byte_offset: u32, field_value: i32)` | Writes 32-bit signed integer field by explicit `byte_offset` |
| `read_field_u64_by_index` | `unsafe fn read_field_u64_by_index(&self, object_address: u32, field_index: u32) -> u64` | Reads arbitrary field slot by explicit 0-based `field_index` |
| `write_field_u64_by_index` | `unsafe fn write_field_u64_by_index(&self, object_address: u32, field_index: u32, raw_value: u64)` | Writes arbitrary field slot by explicit 0-based `field_index` |
| `read_field_u64_by_offset` | `unsafe fn read_field_u64_by_offset(&self, object_address: u32, byte_offset: u32) -> u64` | Reads arbitrary field slot by explicit `byte_offset` |
| `write_field_u64_by_offset` | `unsafe fn write_field_u64_by_offset(&self, object_address: u32, byte_offset: u32, raw_value: u64)` | Writes arbitrary field slot by explicit `byte_offset` |
| `set_error` | `unsafe fn set_error(&self, out_error: *mut SolixCValue, message: &str)` | Allocates an error string in the VM heap and writes it into `*out_error` |
| `get_vm_metric` | `unsafe fn get_vm_metric(&self, metric_id: u32) -> u64` | Reads VM metric (`0..=55`) |
| `collect_cycles` | `unsafe fn collect_cycles(&self) -> u64` | Triggers Bacon & Rajan cycle collection |
| `set_cycle_threshold` | `unsafe fn set_cycle_threshold(&self, threshold: u32)` | Configures cycle collector candidate threshold |

---

## 5. `solix_export!` Macro & Dual-Mode Code Example

```rust
//! Standalone library core + gated Solix FFI bridge.

pub fn compute_sum(left_value: i32, right_value: i32) -> i32 {
    left_value.saturating_add(right_value)
}

#[cfg(feature = "solix-native")]
mod solix_bridge {
    use solix_ffi::{solix_export, NativeCFunctionPtr, SolixCValue, SolixVmContext};

    #[unsafe(no_mangle)]
    pub unsafe extern "C-unwind" fn MathBridge_add_checked(
        vm_context: *mut std::ffi::c_void,
        arguments: *const SolixCValue,
        argument_count: u32,
        out_error: *mut SolixCValue,
    ) -> SolixCValue {
        if argument_count < 2 || arguments.is_null() {
            if let Some(ctx) = unsafe { SolixVmContext::validate(vm_context) } {
                if !out_error.is_null() {
                    let msg_addr = unsafe { ctx.alloc_string("Expected 2 integer arguments") };
                    unsafe { *out_error = SolixCValue::from_ref(msg_addr); }
                }
            }
            return SolixCValue::null();
        }
        let left_value = unsafe { (*arguments.add(0)).as_i32() };
        let right_value = unsafe { (*arguments.add(1)).as_i32() };
        SolixCValue::from_i32(crate::compute_sum(left_value, right_value))
    }

    solix_export! {
        "MathBridge::add_checked" => MathBridge_add_checked,
    }
}
```

---

## Related Documentation
* [FFI Wiki Overview](README.md)
* [C & C++ SDK Reference (`solix::ffi`)](c-cpp-sdk.md)
* [Dual-Backend Example (`examples/ffi_dual_backend`)](../../../examples/ffi_dual_backend/README.md)
* [Normative Native FFI & ABI Specification](../../spec/native_interop.md)
