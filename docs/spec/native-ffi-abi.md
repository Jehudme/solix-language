# Formal Specification: Native C-FFI ABI & Dynamic Extensions

## Document Metadata

* **Specification ID:** SPEC-0011
* **Scope:** Native Interoperability, C-ABI Bridge & Dynamic Extensions (`crates/solix-runtime/src/native/`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)

---

## 1. Native FFI Architecture

Solix applications interface with native C and Rust shared libraries through a stable, C-compatible foreign function interface (`#[repr(C)]`). Native modules are compiled as platform dynamic shared libraries (`.so`, `.dll`, `.dylib`) and loaded dynamically via `libloading`.

---

## 2. ABI Representation & Type Primitives

### 2.1 Value Representation (`SolixCValue`)
All primitive values, floats, and object references passed across the native boundary are marshaled through a uniform 64-bit union:

```rust
#[repr(C)]
#[derive(Copy, Clone)]
pub union SolixCValue {
    pub int64_value: i64,
    pub uint64_value: u64,
    pub float64_value: f64,
    pub pointer_value: *mut std::ffi::c_void,
}
```

### 2.2 Host VM Context (`SolixVmContext`)
Native functions receive an opaque pointer to the running VM instance, granting access to memory allocation, string creation, and exception handling:

```rust
#[repr(C)]
pub struct SolixVmContext {
    pub allocate_object: extern "C" fn(context: *mut SolixVmContext, class_id: u32, size: usize) -> *mut std::ffi::c_void,
    pub allocate_string: extern "C" fn(context: *mut SolixVmContext, utf8_bytes: *const u8, length: usize) -> *mut std::ffi::c_void,
    pub throw_exception: extern "C" fn(context: *mut SolixVmContext, exception_object: *mut std::ffi::c_void),
}
```

---

## 3. Writing Native Plugins in Rust (`natives/`)

A Solix native extension is authored as a standard Rust `cdylib` subproject within the `natives/` directory:

```rust
// natives/crypto/src/lib.rs
use std::ffi::c_void;

#[repr(C)]
pub union SolixCValue {
    pub int64_value: i64,
    pub pointer_value: *mut c_void,
}

#[no_mangle]
pub extern "C" fn solix_crypto_sha256(
    _context: *mut c_void,
    data_pointer: *const u8,
    data_length: usize,
) -> SolixCValue {
    let input_bytes = unsafe { std::slice::from_raw_parts(data_pointer, data_length) };
    let hash_result = compute_sha256(input_bytes);
    SolixCValue { int64_value: hash_result }
}
```

The toolchain automatically compiles `natives/` subprojects via Cargo during `solix build` and packages the resulting `.so`/`.dll`/`.dylib` into `lib/<host_triple>/` inside the `.slxarc` container.
