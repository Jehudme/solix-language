# Formal Specification: Standard Library Core (`solix.core`)

## Document Metadata

* **Specification ID:** SPEC-0018
* **Scope:** Standard Core Types, Interfaces, String & Monadic Wrappers
* **Status:** Final Normative Standard
* **Source Path:** [`libraries/solixlib/src/core/`](file:///home/jehud/Projects/solix-language/libraries/solixlib/src/core/)

---

## 1. Core Interfaces (`solix.core.Interfaces`)

| Interface | Method Signature | Purpose |
| :--- | :--- | :--- |
| `IStringable` | `string to_string()` | Converts the instance to a printable string representation |
| `IEquatable<TargetType>` | `bool equals(TargetType other)` | Tests equality against another instance of the same type |
| `IComparable<TargetType>`| `int32 compare_to(TargetType other)` | Returns negative if less, zero if equal, positive if greater |
| `ICloneable<TargetType>` | `TargetType clone()` | Creates a deep copy of the instance |
| `IHashable` | `int32 hash_code()` | Computes a 32-bit hash code for associative map keys |

---

## 2. Heap String (`solix.core.String`)

`solix.core.String` encapsulates a UTF-8 character buffer:
* **Constructors:** `new String()`, `new String(char[] characters)`, `new String(string text)`.
* **Inspection:** `int32 length()`, `bool is_empty()`, `bool is_blank()`, `char char_at(int32 index)`.
* **Transformations:** `String substring(int32 start, int32 end)`, `String trim()`, `String to_upper()`, `String to_lower()`.
* **Search & Replace:** `int32 index_of(String query)`, `bool contains(String query)`, `String replace(String pattern, String replacement)`.
* **Splitting & Formatting:** `String[] split(char delimiter)`, `static String format<args...>(string template_text, args... arguments)`.
* **Conversions:** `int32 to_int32()`, `int64 to_int64()`, `float64 to_float64()`, `bool to_bool()`.

---

## 3. Monadic Containers (`Optional<T>` & `Result<T, E>`)

### 3.1 `Optional<T>`
Type-safe container for optional values without `null` dereference risks:
* `static Optional<T> of(T value)`
* `static Optional<T> empty()`
* `bool is_present()`
* `T get()`
* `T or_else(T fallback_value)`

### 3.2 `Result<T, E>`
Error-handling monad representing either success (`Ok`) or failure (`Err`):
* `static Result<T, E> ok(T value)`
* `static Result<T, E> err(E error)`
* `bool is_ok()`
* `bool is_err()`
* `T unwrap()`
* `E unwrap_err()`

---

## 4. Standard Exception Hierarchy

* **`Exception`**: Base class for all catchable exceptions. Carries `message: string`.
  * **`RuntimeException`**: Base class for unchecked runtime errors.
    * **`IllegalArgumentException`**: Thrown when a parameter fails invariant validation.
    * **`NullPointerException`**: Raised when attempting to access a member of a `null` reference.
    * **`IndexOutOfBoundsException`**: Array or collection indexing out of valid bounds.
    * **`InvalidOperationException`**: State of the object does not permit the requested call.
  * **`IOException`**: Raised on filesystem, socket, or stream I/O failure.
