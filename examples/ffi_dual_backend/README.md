# Dual-Backend FFI Showcase & Verification Matrix

This example project demonstrates authoring a single Solix application (`src/main.slx` and `tests/test_ffi_matrix.slx`) that targets **two interchangeable native backends** in `manifest.yaml`:

1. **`cmake_backend` profile (`natives/cmake_backend/`):** C11 core + C++17 `solix::ffi` bridge built via CMake (`-DSOLIX_NATIVE_BUILD=ON`).
2. **`rust_backend` profile (`natives/rust_backend/`):** Rust crate using `solix-ffi` built via Cargo (`--features solix-native`).

Both backends implement identical `FfiMatrix::*` symbols exercising scalar `SolixCValue` types (`null`, `bool`, `i32`, `i64`, `f64`), guest string & `char[]` / `int8[]` allocations, `int64[]` / `float64[]` / `string[]` arrays, `MatrixBox` class vtable allocation and field mutations, VM metrics & cycle collection, and `out_error` signaling.

## Running Either Backend

```bash
# Test via the CMake (C11/C++17) backend
solix build examples/ffi_dual_backend --profile cmake_backend
solix run examples/ffi_dual_backend/build/cmake_backend/ffi_dual_backend.slxp

# Test via the Rust backend
solix build examples/ffi_dual_backend --profile rust_backend
solix run examples/ffi_dual_backend/build/rust_backend/ffi_dual_backend.slxp
```

## Standalone Native Unit Tests (Without Solix Plugin Mode)

Both native subdirectories also build and pass standalone unit tests when built outside `solix build`:

```bash
# CMake standalone library + CTest
cmake -S examples/ffi_dual_backend/natives/cmake_backend -B /tmp/cmake_backend_standalone -DBUILD_TESTING=ON
cmake --build /tmp/cmake_backend_standalone --config Release
ctest --test-dir /tmp/cmake_backend_standalone -C Release --output-on-failure

# Rust standalone library + Cargo tests
cargo test --manifest-path examples/ffi_dual_backend/natives/rust_backend/Cargo.toml
```
