# Guide 09: Native Rust Extensions

Solix applications can seamlessly call high-performance native code authored in Rust via the C-FFI bridge.

---

## 1. Project Layout with `natives/`

When a Solix project contains native extensions, place the Rust crates inside `natives/`:

```text
my_audio_project/
├── manifest.toml
├── src/
│   └── main.slx
└── natives/
    └── audio_driver/
        ├── Cargo.toml         # crate-type = ["cdylib"]
        └── src/
            └── lib.rs
```

---

## 2. Writing the Native Rust Bridge

In `natives/audio_driver/Cargo.toml`:

```toml
[package]
name = "audio_driver"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]
```

In `natives/audio_driver/src/lib.rs`:

```rust
use std::ffi::c_void;

#[repr(C)]
pub union SolixCValue {
    pub int64_val: i64,
    pub float64_val: f64,
    pub pointer_val: *mut c_void,
}

#[no_mangle]
pub extern "C" fn play_audio_tone(frequency_hertz: f64) -> SolixCValue {
    // Platform audio hardware interaction
    SolixCValue { int64_val: 0 }
}
```

---

## 3. Calling Native Functions from Solix

In your Solix source code, declare the method as `native`:

```solix
public class AudioEngine {
    public static native int32 play_audio_tone(float64 frequency_hertz);
}

public int32 main() {
    AudioEngine::play_audio_tone(440.0);
    return 0;
}
```

---

## 4. Cross-References

* **Specification:** [Native C-FFI ABI & Dynamic Extensions](file:///home/jehud/Projects/solix-language/docs/spec/native-ffi-abi.md)
* **Next Guide:** [Guide 10: Standard Library Tour](file:///home/jehud/Projects/solix-language/docs/guide/10-standard-library-tour.md)
