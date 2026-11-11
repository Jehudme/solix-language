# Formal Specification: Standard Library Math & Random (`solix.math`)

## Document Metadata

* **Specification ID:** SPEC-0021
* **Scope:** Mathematical Transcendental Functions & Pseudo-Random Number Generation
* **Status:** Final Normative Standard
* **Source Path:** [`libraries/solixlib/src/math/`](file:///home/jehud/Projects/solix-language/libraries/solixlib/src/math/)

---

## 1. Math Constants & Functions (`solix.math.Math`)

The `Math` class exposes mathematical constants and IEEE-754 floating-point operations:

### 1.1 Constants
* `public static const float64 PI = 3.14159265358979323846;`
* `public static const float64 E  = 2.71828182845904523536;`

### 1.2 Transcendental Functions
* **Trigonometric:** `static float64 sin(float64 angle_radians)`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2(y, x)`.
* **Hyperbolic:** `static float64 sinh(float64 val)`, `cosh`, `tanh`.
* **Power & Root:** `static float64 sqrt(float64 val)`, `cbrt`, `pow(float64 base_val, float64 exp_val)`.
* **Logarithmic:** `static float64 log(float64 val)` (natural), `log10`, `log2`, `exp`.

### 1.3 Rounding & Clamping
* `static float64 floor(float64 val)`, `ceil`, `round`, `trunc`.
* `static float64 abs(float64 val)`, `int32 abs(int32 val)`, `int64 abs(int64 val)`.
* `static float64 clamp(float64 val, float64 minimum_bound, float64 maximum_bound)`.

---

## 2. Deterministic PRNG (`solix.math.Random`)

The `Random` class implements the high-performance **`xoshiro256**`** 64-bit pseudo-random generator:
* `public Random()`: Seeded with current wall-clock monotonic timestamp.
* `public Random(int64 initial_seed)`: Seeded deterministically with user-specified seed.

### Generator Methods
* `int32 next_int()`: Non-negative 32-bit integer in $[0, 2^{31}-1]$.
* `int32 next_int(int32 upper_bound)`: Integer in $[0, \text{upper\_bound})$.
* `int32 next_range(int32 minimum_bound, int32 maximum_bound)`: Uniform integer in $[\text{min}, \text{max})$.
* `int64 next_int64()`: Uniform 64-bit integer across full range.
* `float64 next_double()`: Uniform floating-point value in $[0.0, 1.0)$.
* `bool next_bool()`: Uniform boolean choice (`true` or `false`).
* `void next_bytes(int8[] destination_buffer)`: Fills array with pseudorandom bytes.
* `float64 next_gaussian()`: Gaussian-distributed float with mean 0.0 and stddev 1.0 (Box-Muller transform).
