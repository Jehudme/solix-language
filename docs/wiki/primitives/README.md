# Built-in Primitive Types Reference

Solix defines built-in scalar and structural primitive types designed for bit-exact reproducibility, predictable memory layouts, and zero-overhead VM dispatch.

> [!NOTE]
> Strings are not scalar primitives in Solix. `String` is a heap-allocated, reference-counted object type managed deterministically by Automatic Reference Counting (ARC). See the [Variables & Types Guide](../../guide/02_variables_and_types.md#strings) for string operations and standard library string utilities.

---

## Primitive Summary Table

| Category | Types | Storage Width | Default Value | Specification Reference |
| :--- | :--- | :---: | :---: | :--- |
| **Signed Integers** | `int8`, `int16`, `int32`, `int64` | 8, 16, 32, 64 bits | `0` | [Integers](integers.md) |
| **Unsigned Integers** | `uint8`, `uint16`, `uint32`, `uint64` | 8, 16, 32, 64 bits | `0` | [Integers](integers.md) |
| **Floating Point** | `float32`, `float64` | 32, 64 bits | `0.0f`, `0.0` | [Floats](floats.md) |
| **Logical** | `bool` | 8 bits (1 byte) | `false` | [Booleans](booleans.md) |
| **ASCII Character** | `char` | 8 bits (1 byte) | `'\0'` | [Characters](chars.md) |
| **Anonymous Tuple** | `(T1, T2, ...)` | Composite | Value fields | [Tuples](tuples.md) |
| **Void** | `void` | 0 bits | None | [Void](void.md) |

---

## 1. Signed Integers

| Type | Bit Width | Range | Default | Friendly Alias |
| :--- | :---: | :--- | :---: | :--- |
| `int8` | 8 bits | $-128$ to $127$ | `0` | `byte` |
| `int16` | 16 bits | $-32,768$ to $32,767$ | `0` | `short` |
| `int32` | 32 bits | $-2,147,483,648$ to $2,147,483,647$ | `0` | `int` |
| `int64` | 64 bits | $-2^{63}$ to $2^{63}-1$ | `0` | `long` |

```solix
int32 counter = 100;
int64 large_timestamp = 1762809000L;
```

---

## 2. Unsigned Integers

| Type | Bit Width | Range | Default | Friendly Alias |
| :--- | :---: | :--- | :---: | :--- |
| `uint8` | 8 bits | $0$ to $255$ | `0` | `ubyte` |
| `uint16` | 16 bits | $0$ to $65,535$ | `0` | `ushort` |
| `uint32` | 32 bits | $0$ to $4,294,967,295$ | `0` | `uint` |
| `uint64` | 64 bits | $0$ to $2^{64}-1$ | `0` | `ulong` |

```solix
uint8 byte_mask = 0xFF;
uint32 memory_offset = 4096U;
```

---

## 3. Floating-Point Numbers

| Type | Bit Width | Precision | Range | Default | Friendly Alias |
| :--- | :---: | :--- | :--- | :---: | :--- |
| `float32` | 32 bits | ~7 decimal digits | $\pm 1.18 \times 10^{-38}$ to $\pm 3.4 \times 10^{38}$ | `0.0f` | `float` |
| `float64` | 64 bits | ~15-17 decimal digits | $\pm 2.23 \times 10^{-308}$ to $\pm 1.8 \times 10^{308}$ | `0.0` | `double` |

```solix
float32 coordinate_x = 3.14f;
float64 high_precision_pi = 3.141592653589793;
```

---

## 4. Booleans

The `bool` type represents logical truth values:
* **Storage Width:** 8 bits (1 byte).
* **Values:** `true` and `false`.
* **Default Value:** `false`.
* **Friendly Alias:** `boolean`.

```solix
bool is_running = true;
bool has_completed = false;
```

---

## 5. ASCII Characters

The `char` type represents a single 8-bit ASCII character value (`0x00` to `0x7F`, 0–127):
* **Storage Width:** 8 bits (1 byte, 1-byte alignment).
* **Enclosure:** Single quotes `'...'`.
* **Default Value:** `'\0'` (Null character).

```solix
char latin_letter = 'A';
char digit_symbol = '9';
char hash_mark    = '#';
char newline_char = '\n';
```

---

## 6. Anonymous Tuples

Tuples are anonymous, strongly-typed structural values combining heterogeneous fields:

```solix
(int32, bool) result_tuple = (200, true);

int32 status_code = result_tuple.0;
bool is_successful = result_tuple.1;
```

---

## 7. Void

The `void` type signifies the absence of a value:
* **Function Returns:** Indicates that a function returns no value (`void main()`).
* **Invariants:** Cannot be used as a variable type, struct field, or generic type argument.

---

## Next Steps & Further Reading

* [Integers Reference](integers.md): Signed and unsigned integer representations.
* [Floats Reference](floats.md): IEEE-754 floating-point details.
* [Booleans Reference](booleans.md): Logical evaluation rules.
* [Characters Reference](chars.md): 8-bit ASCII character handling.
* [Tuples Reference](tuples.md): Anonymous structural tuples.
* [Void Reference](void.md): Function return conventions.
* [Variables & Types Guide](../../guide/02_variables_and_types.md): Complete guide to typing, casting, and ARC reference objects.
