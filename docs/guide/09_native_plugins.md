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

Values are passed across the native boundary using the 16-byte aligned `SolixCValue` structure:

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
All native bridge functions conform to the `NativeCFunctionPtr` signature:

```rust
pub type NativeCFunctionPtr = unsafe extern "C" fn(
    vm_context: *mut std::ffi::c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue;
```

---

## 3. Authoring a Native Extension in Rust

Every new Solix project created via `solix new` provides an out-of-the-box native bridge workspace in `natives/rust_bridge/`:

### 3.1. `natives/rust_bridge/Cargo.toml`
```toml
[package]
name = "rust_bridge"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
```

### 3.2. `natives/rust_bridge/src/lib.rs`
```rust
//! Rust native bridge extension crate for Solix.

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SolixCValue {
    pub tag: u32,
    pub flags: u32,
    pub payload: u64,
}

pub type NativeCFunctionPtr = unsafe extern "C" fn(
    *mut std::ffi::c_void,
    *const SolixCValue,
    u32,
    *mut SolixCValue,
) -> SolixCValue;

pub type RegisterNativeCallback = unsafe extern "C" fn(
    *mut std::ffi::c_void,
    *const std::ffi::c_char,
    NativeCFunctionPtr,
);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MathBridge_fast_add(
    _vm_context: *mut std::ffi::c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue {
            tag: 0,
            flags: 0,
            payload: 0,
        };
    }
    let value_a = unsafe { (*args.add(0)).payload as i32 };
    let value_b = unsafe { (*args.add(1)).payload as i32 };
    SolixCValue {
        tag: 2, // I32
        flags: 0,
        payload: ((value_a + value_b) as i64) as u64,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn solix_register_natives(
    collector_context: *mut std::ffi::c_void,
    register_callback: RegisterNativeCallback,
) {
    let add_symbol = c"MathBridge::fast_add";
    unsafe {
        register_callback(
            collector_context,
            add_symbol.as_ptr(),
            MathBridge_fast_add,
        );
    }
}
```

---

## 4. Building and Packaging Native Libraries

Native extensions are compiled directly with Cargo:

```bash
# Compile native companion library
cargo build --release --manifest-path natives/rust_bridge/Cargo.toml

# Build and package the Solix project with its companion library
solix build --profile release
```

During `solix build`, the build orchestrator (`scripts/build.py`) builds all native crates under `natives/`, copies the produced shared objects into `lib/`, and embeds them into the resulting `.slxarc` package.
