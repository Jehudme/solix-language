# Raylib Bindings & OOP Ergonomics (`raylib`)

The first-party `raylib` package provides auto-generated FFI bindings for Raylib alongside object-oriented math types, geometry helpers, and deterministic RAII resource wrappers.

## 1. Vector, Matrix, Color, and Geometry Ergonomics

- **`Vector2`, `Vector3`, `Vector4`**:
  - Static unit/constant presets: `ZERO`, `ONE`, `UNIT_X`, `UNIT_Y` (plus `UNIT_Z` on `Vector3`/`Vector4` and `UNIT_W` on `Vector4`).
  - Operator overloads: `+`, `-`, `*`, `/` (component-wise and scalar overloads for `float32`, `float64`, `int32`), `==`, `!=`.
  - Methods: `length_squared()`, `length()`, `normalized()`, `dot(...)`, `distance_to(...)`, `lerp(...)`, plus `cross(Vector3)` on `Vector3`.
- **`Matrix`**:
  - `Matrix::identity()`, component-wise `+` / `-`, scalar `*`, and `4x4` matrix multiplication (`Matrix * Matrix`), plus `==` / `!=`.
- **`Quaternion`**:
  - `Quaternion::IDENTITY`, overloaded multi-precision constructors, and `==` / `!=`.
- **`Color`**:
  - `Color::from_hex(uint32)`, `with_alpha(...)`, `with_red(...)`, `with_green(...)`, `with_blue(...)`, and equality operators (`==`, `!=`).
- **`Rectangle`**:
  - `contains(Vector2)` / `contains(float32, float32)`, `overlaps(Rectangle)`, `center()`, `position()`, `size()`, and equality operators (`==`, `!=`).

## 2. Deterministic RAII Resource Wrappers & Scoped Draw Helpers

All GPU and audio resources provide ARC-managed wrappers with explicit `dispose()`, `is_valid()`, and automatic `~Managed*()` destructors:

- `ManagedTexture` (`UnloadTexture`)
- `ManagedRenderTexture` (`UnloadRenderTexture`)
- `ManagedImage` (`UnloadImage`)
- `ManagedShader` (`UnloadShader`)
- `ManagedFont` (`UnloadFont`)
- `ManagedModel` (`UnloadModel`)
- `ManagedSound` (`UnloadSound`)
- `ManagedMusic` (`UnloadMusicStream`)

Scoped callback helpers automatically pair begin/end render states around user lambdas using `try / finally`:

- `with_drawing(void(*)() draw_callback)`
- `with_mode_2d(Camera2D camera, void(*)() draw_callback)`
- `with_mode_3d(Camera3D camera, void(*)() draw_callback)`
- `with_texture_mode(RenderTexture2D target, void(*)() draw_callback)`

> [!NOTE]
> `libraries/raylib` is compiled before `solixlib` in topological dependency order, so `raylib.slx` remains self-contained and does not depend on `solix.*`.
