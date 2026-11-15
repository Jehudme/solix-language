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
solix-ffi = "1.1"
```

### 3.2. `natives/rust_bridge/src/lib.rs` (Using `solix-ffi`)
```rust
//! Rust native bridge extension crate for Solix.

use solix_ffi::{solix_export, NativeCFunctionPtr, SolixCValue};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MathBridge_fast_add(
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
    SolixCValue::from_i32(value_a + value_b)
}

solix_export! {
    "MathBridge::fast_add" => MathBridge_fast_add,
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

> [!NOTE]
> Native libraries are hermetically packaged inside `.slxarc` binaries. When executing or distributing your application, the Solix runtime automatically extracts and loads matching native shared libraries for the host operating system and CPU architecture.

> [!IMPORTANT]
> All native symbols declared with `native` must be exported and registered via `solix_register_natives`. Unresolved native symbols produce link errors during VM initialization.

---

## 5. Next Steps & Further Reading

- **Next Guide**: Learn how to bundle embedded assets and resources in [Packaging & Resources](packaging_and_resources.md).
- **Tooling Guide**: Configure your development environment in [VS Code Extension Setup](editors_vscode.md).
- **Previous Guide**: Review generics and monomorphization in [08 - Generics & Templates](08_generics_and_templates.md).
- **Further Reading**:
  - [Native FFI & ABI Specification](../spec/native-ffi-abi.md)
  - [native Keyword Modifier Reference](../wiki/keywords/modifiers/native.md)
  - [Binary Packaging Specification](../spec/binary-formats-slxb-and-slxarc.md)
