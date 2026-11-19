# Foreign Function Interface (FFI) Wiki

The Solix Virtual Machine provides a stable C-ABI Foreign Function Interface allowing Solix programs to invoke compiled native libraries authored in **C (C11)**, **C++ (C++17)**, or **Rust**.

First-party FFI SDKs live at the repository root under [`ffi/`](../../../ffi/):
* **`ffi/c/` (`solix::ffi`)**: Header-only C11/C++17 CMake SDK ([C & C++ SDK Reference](c-cpp-sdk.md)).
* **`ffi/rust/` (`solix-ffi`)**: Safe Rust wrapper crate and macro DSL ([Rust SDK Reference](rust-sdk.md)).

---

## FFI SDK Pages

| Guide | Language / Backend | Key Symbols |
| :--- | :--- | :--- |
| [**C & C++ SDK Reference (`solix::ffi`)**](c-cpp-sdk.md) | C11 & C++17 (CMake) | `SolixCValue`, `SolixVmContext`, `solix_vm_*`, `SOLIX_REGISTER_MODULE`, `SOLIX_EXPORT_NATIVES_*` |
| [**Rust SDK Reference (`solix-ffi`)**](rust-sdk.md) | Rust (`cdylib` / `rlib`) | `SolixCValue`, `SolixVmContext`, `NativeRegistry`, `NativeCFunctionPtr`, `solix_export!` |

---

## Core ABI Architecture

Every Solix native module communicates through four shared ABI contracts:

### 1. Tagged Value Envelope (`SolixCValue`)
Arguments, return values, and errors use a uniform 16-byte `#[repr(C)]` tagged union struct:

| Field | C Type | Rust Type | Purpose |
| :--- | :--- | :--- | :--- |
| `tag` | `uint32_t` | `u32` | Discriminator (`0..=6`) identifying the active payload kind |
| `flags` | `uint32_t` | `u32` | Reserved ABI flags / alignment padding (`0`) |
| `payload` | `uint64_t` | `u64` | Scalar bit pattern or 32-bit VM heap address / function ID |

#### Value Type Tags

| Constant (C) | Constant (Rust) | Value | Solix Type | Payload Encoding |
| :--- | :--- | :--- | :--- | :--- |
| `SOLIX_TAG_NULL` | `TAG_NULL` | `0` | `null` | `0` |
| `SOLIX_TAG_BOOL` | `TAG_BOOL` | `1` | `bool` | `0` (`false`) or `1` (`true`) |
| `SOLIX_TAG_I32` | `TAG_I32` | `2` | `int32` | Sign-extended `int32_t` in `uint64_t` |
| `SOLIX_TAG_I64` | `TAG_I64` | `3` | `int64` | Bitcast `int64_t` to `uint64_t` |
| `SOLIX_TAG_F64` | `TAG_F64` | `4` | `float64` | IEEE-754 `double` bits |
| `SOLIX_TAG_REF` | `TAG_REF` | `5` | `string`, `class`, array | Low 32 bits hold VirtualHeap address (`uint32_t`) |
| `SOLIX_TAG_FNPTR` | `TAG_FNPTR` | `6` | Function pointer / closure | Low 32 bits hold VM function index (`uint32_t`) |

### 2. Array Element Type Tags

When allocating arrays via `solix_vm_alloc_array` / `SolixVmContext::alloc_array`, pass the element type tag matching the Solix target type:

| Constant (C) | Constant (Rust) | Value | Solix Element Type |
| :--- | :--- | :--- | :--- |
| `SOLIX_ARRAY_TAG_I8` | `SOLIX_ARRAY_TAG_I8` | `1` | `int8` / `byte` |
| `SOLIX_ARRAY_TAG_I64` | `SOLIX_ARRAY_TAG_I64` | `4` | `int64` |
| `SOLIX_ARRAY_TAG_F32` | `SOLIX_ARRAY_TAG_F32` | `9` | `float32` |
| `SOLIX_ARRAY_TAG_F64` | `SOLIX_ARRAY_TAG_F64` | `10` | `float64` |
| `SOLIX_ARRAY_TAG_CHAR` | `SOLIX_ARRAY_TAG_CHAR` | `12` | `char` |
| `SOLIX_ARRAY_TAG_REF` | `SOLIX_ARRAY_TAG_REF` | `13` | `string` / object references |

### 3. Standardized Entrypoint & Symbol Naming

All native functions receive the same 4-parameter C-ABI signature:

```c
SolixCValue native_impl(
    void* vm_context,
    const SolixCValue* args,
    uint32_t arg_count,
    SolixCValue* out_error
);
```

And register themselves into the VM via `solix_register_natives(RegisterNativeCallback register_fn)` using canonical Solix symbol strings:
* `"MathBridge::fast_add"` (class + `::` + method)
* Overloaded methods share the base signature resolution table when unambiguous, or disambiguate cleanly per class namespace.
* Assigning `*out_error` (`solix_vm_set_error` in C/C++, or writing `*out_error = SolixCValue::from_ref(...)` / scalar value in Rust) propagates a native failure into the Solix VM exception handler.

### 4. `SolixVmContext` Magic & Function Table

The opaque `void* vm_context` points to a VM-owned `SolixVmContext` validated with `SOLIX_VM_CONTEXT_MAGIC = 0x534F4C4958564D43` (`"SOLIXVMC"`). Always validate via `solix_vm_context_from_ptr(vm_context)` (C/C++) or `SolixVmContext::validate(vm_context)` (Rust) before invoking heap, string, array, object, metric, or cycle-collection helpers.

---

## Dual-Mode Compilation Architecture

Native subdirectories under `natives/` support compiling in two independent modes from the same source tree:

| Mode | CMake (`CMakeLists.txt`) | Cargo (`Cargo.toml`) | Output Artifact |
| :--- | :--- | :--- | :--- |
| **Standalone Mode** | `cmake -S ... -B ...` (`SOLIX_NATIVE_BUILD=OFF`) | `cargo build` / `cargo test` | Static/shared C/C++ library (`rlib` in Rust) + CTest / Cargo unit tests |
| **Solix Plugin Mode** | `solix build` (`-DSOLIX_NATIVE_BUILD=ON`, env `SOLIX_NATIVE_BUILD=1`) | `solix build` (`--features solix-native`, env `SOLIX_NATIVE_BUILD=1`) | Loadable shared object (`.so`, `.dylib`, `.dll`) exporting `solix_register_natives` |

### Repository Branch & Local Override Resolution
* **During development (`dev`):** Scaffolded CMake (`FetchContent`) and Cargo templates reference `GIT_TAG dev` / `branch = "dev"` in `https://github.com/Jehudme/solix-language.git`.
* **Local checkout override:** When `solix build` runs inside or alongside a Solix repository checkout (or when `SOLIX_FFI_SOURCE_DIR` / `SOLIX_FFI_RUST_DIR` is set), `solix build` automatically points CMake (`SOLIX_FFI_SOURCE_DIR`) and Cargo (`--config patch."https://github.com/Jehudme/solix-language.git".solix-ffi.path=...`) to the local `ffi/c` and `ffi/rust` trees.
* **During releases (`vX.Y.Z`):** `scripts/release/prepare_release_train.py` stamps `SOLIX_RELEASE_CHANNEL=release` so generated release toolchains default `DEFAULT_FFI_GIT_TAG` to `vX.Y.Z` while keeping `dev` synced cleanly on development branches.

---

## Complete Reference Example

See [`examples/ffi_dual_backend/`](../../../examples/ffi_dual_backend/README.md) for a side-by-side verified implementation sharing one Solix test matrix (`tests/test_ffi_matrix.slx`) across both a **C11/C++17 CMake backend** and a **Rust backend** (including standalone CTest and Cargo tests).

---

## Related Documentation
* [C & C++ SDK Reference (`solix::ffi`)](c-cpp-sdk.md)
* [Rust SDK Reference (`solix-ffi`)](rust-sdk.md)
* [Native Plugins Developer Guide](../../guide/09_native_plugins.md)
* [Normative Native FFI & ABI Specification](../../spec/native_interop.md)
* [Project Manifest (`manifest.yaml`) Wiki](../manifest.md)
