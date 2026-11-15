# Variables and Types

Solix is a **statically typed** language — every variable has a type that is declared explicitly at the point of definition. There is no type inference; the compiler never infers a type from the right-hand side of an assignment. This section covers all primitive types, variable declaration syntax, immutability, arrays, and the `String` class.

---

## Table of Contents

1. [Primitive Types](#primitive-types)
2. [Variable Declaration](#variable-declaration)
3. [Constants (`const`)](#constants-const)
4. [No Type Inference](#no-type-inference)
5. [Arrays (`T[]`)](#arrays-t)
6. [The `String` Class](#the-string-class)
7. [Null and Nullability](#null-and-nullability)

---

## Primitive Types

Solix defines **13 primitive types** built into the compiler. They are never heap-allocated and are passed by value.

### Integer Types

| Type | Width | Signed | Minimum Value | Maximum Value |
|------|-------|--------|--------------|---------------|
| `int8` | 8 bits | Yes | −128 | 127 |
| `int16` | 16 bits | Yes | −32,768 | 32,767 |
| `int32` | 32 bits | Yes | −2,147,483,648 | 2,147,483,647 |
| `int64` | 64 bits | Yes | −9,223,372,036,854,775,808 | 9,223,372,036,854,775,807 |
| `uint8` | 8 bits | No | 0 | 255 |
| `uint16` | 16 bits | No | 0 | 65,535 |
| `uint32` | 32 bits | No | 0 | 4,294,967,295 |
| `uint64` | 64 bits | No | 0 | 18,446,744,073,709,551,615 |

### Floating-Point Types

| Type | Width | Precision | Range (approx.) |
|------|-------|-----------|-----------------|
| `float32` | 32 bits | ~7 significant digits | ±3.4 × 10³⁸ |
| `float64` | 64 bits | ~15 significant digits | ±1.8 × 10³⁰⁸ |

### Other Primitive Types

| Type | Description |
|------|-------------|
| `bool` | Boolean: `true` or `false` |
| `char` | A single Unicode code point (stored as an integer) |
| `void` | The absence of a value; valid only as a method return type |

### Literals

```solix
bool    flag     = true;
char    letter   = (char)65;     // 'A' — char literals use integer casts
int8    tiny     = -10;          // Context-typed numeric literal coercion
uint8   byte_val = 255;          // Implicitly coerced to uint8
int32   count    = 42;
int64   big      = 9000000000;
float32 ratio    = 3.14;         // Implicitly coerced to float32
float64 pi       = 3.141592653589793;
```

> [!NOTE]
> Solix does not have a dedicated character literal syntax like `'A'`. Characters are represented as their integer code point value cast to `char`.

---

## Variable Declaration

The syntax for declaring a variable is:

```
<type> <name> = <initial-value>;
```

or without initialization (value is zero/null by default):

```
<type> <name>;
```

**Examples:**

```solix
int32 x_coord = 10;
float64 temperature = 36.6;
bool active = false;
char first_char = (char)72;   // 'H'

// Without explicit initialization — defaults to 0 / false / null
int32 counter;                // 0
bool ready;                   // false
String label;                 // null
```

Variables are block-scoped. A variable declared inside a `{ }` block is only visible within that block.

```solix
{
    int32 inner = 5;
    // inner is visible here
}
// inner is NOT visible here — it has been destroyed
```

---

## Constants (`const`)

Declaring a variable with the `const` modifier makes it **immutable** after initialization. Attempting to reassign a `const` variable is a compile-time error.

```solix
const int32 MAX_SIZE = 100;
const float64 GRAVITY = 9.81;
const bool DEBUG_MODE = false;
```

`const` applies only to the **binding** — a `const` reference to a mutable object still allows mutation of the object's fields:

```solix
const List<String> names = new List<String>();
names.add("Alice");     // OK — mutates the List object
names = new List<String>();  // ERROR — reassigning a const variable
```

---

## No Type Inference

Unlike languages that support `var` or `let` with automatic type deduction, Solix **requires** an explicit type on every variable declaration. The compiler never infers a type.

```solix
// VALID — type is explicit
int32 result = compute();

// INVALID — Solix has no 'var' keyword
var result = compute();       // Compile error
```

This design provides immediate readability at the declaration site: the reader always knows the exact type without needing to trace the return type of the right-hand expression.

---

## Arrays (`T[]`)

An array is a fixed-length, contiguous sequence of elements of type `T`. Arrays of any type — primitive or class — are written as `T[]`.

### Creating Arrays

Use the `new T[length]` expression:

```solix
int32[] scores = new int32[5];           // [0, 0, 0, 0, 0]
float64[] prices = new float64[3];       // [0.0, 0.0, 0.0]
String[] names = new String[4];          // [null, null, null, null]
```

### Indexing

Arrays are **zero-indexed**. Accessing an index outside `[0, length-1]` throws an `IndexOutOfBoundsException` at runtime.

```solix
scores[0] = 95;
scores[4] = 87;
int32 first = scores[0];       // 95
```

### Array Length

Every array has a built-in `.length` property:

```solix
int32 count = scores.length;   // 5
```

### Iterating an Array

```solix
int32[] numbers = new int32[5];
for (int32 index = 0; index < numbers.length; index++) {
    numbers[index] = i * index;
}

// Print: 0 1 4 9 16
for (int32 index = 0; index < numbers.length; index++) {
    Console::print(numbers[index]);
    Console::print(" ");
}
```

### Multi-Dimensional Arrays

Solix supports arrays of arrays. Command-line arguments use this pattern:

```solix
char[][] args        // array of char[]
int32[][] matrix = new int32[][3];   // array of 3 int32[] references
matrix[0] = new int32[4];
```

### Array Literals (Initializer Lists)

```solix
int32[] primes = {2, 3, 5, 7, 11};
char[]  hello  = {'H', 'e', 'l', 'l', 'o'};
```

---

## The `String` Class

`String` is a first-class class in the `solix.core` package. It is **not** a primitive; it is a heap-allocated reference type managed by ARC.

Import it before use:

```solix
import solix.core.String;
```

Or use it unqualified if your package imports `solix.core.*`.

### Creating Strings

```solix
String empty  = new String();
String hello  = new String("Hello, World!");   // From a char[] literal
String copy   = new String(other_string);      // Copy constructor
```

> [!NOTE]
> String literals written with double quotes (e.g. `"Hello"`) are automatically converted to `char[]` by the compiler. Wrap them in `new String(...)` to obtain a `String` object.

### Common Operations

```solix
String message = new String("  Hello, Solix!  ");

int32  char_count = message.length();          // 17
char   target_ch  = message.char_at(2);        // 'H' (index 2)
bool   is_empty   = message.is_empty();        // false
String upper      = message.to_upper_case();   // "  HELLO, SOLIX!  "
String trimmed    = message.trim();            // "Hello, Solix!"
String sub_str    = message.substring(2, 7);   // "Hello"

bool is_equal = message.equals(new String("other")); // false

// Concatenation via + operator or concat()
String greeting = new String("Hello") + new String(", ") + new String("World!");
```

### Parsing and Formatting

```solix
// Parse a string to a number
String numStr = new String("42");
int32 val = numStr.to_int32();

// Convert a number to a string (static factory)
String formatted = String.from_int32(42);
String fmtFloat  = String.from_float64(3.14, 2);  // "3.14"
String fmtBool   = String.from_bool(true);          // "true"
```

---

## Null and Nullability

In Solix, **any reference type** (class instances and arrays) can be `null`. Primitive types (`int32`, `bool`, `char`, etc.) **cannot** be null.

```solix
String name = null;       // Valid — String is a reference type
int32  count = null;      // Compile error — int32 is primitive

if (name == null) {
    Console::println("Name is not set.");
}
```

Dereferencing a `null` reference at runtime throws a `NullPointerException`.

```solix
String message = null;
int32 char_count = message.length();   // Throws NullPointerException at runtime
```

> [!WARNING]
> Solix does not have nullable/non-nullable type annotations. Developers are responsible for null checks at points where null references may occur.

Use the `Optional<T>` class from `solix.core` to express optionality explicitly without raw null references:

```solix
import solix.core.Optional;

Optional<String> maybe_value = Optional.of_nullable(getUserInput());
if (maybe_value.is_present()) {
    String text_value = maybe_value.get();
    Console::println(text_value);
}
```

---

## Numeric Coercion & Widening Conversions

Solix features strong static typing combined with safe, ergonomic numeric conversions:

### 1. Context-Typed Numeric Literal Coercion
Unsuffixed integer and floating-point literals automatically coerce to the target type without explicit casts, provided the literal value fits within the target type's representable bounds:

```solix
uint8  byte_val = 255;       // Implicit coercion to uint8
int8   neg_val  = -10;       // Implicit coercion to int8
float32 ratio   = 3.14;      // Implicit coercion to float32
```

### 2. Lossless Widening Conversions (Implicit)
Values of smaller numeric types implicitly widen to larger numeric types along safe conversion paths without data loss:
- `int8` $\to$ `int16` $\to$ `int32` $\to$ `int64`
- `uint8` $\to$ `uint16` $\to$ `uint32` $\to$ `uint64`
- `float32` $\to$ `float64`

```solix
int16   small_num  = 500;
int32   medium_num = small_num;    // Implicit widening
int64   large_num  = medium_num;   // Implicit widening
float64 precise_pi = ratio;        // Implicit float32 -> float64
```

### 3. Narrowing Conversions (Explicit Cast Required)
Narrowing conversions that could lose precision or overflow require an explicit cast:

```solix
int64 timestamp = 1700000000;
int32 truncated = (int32)timestamp; // Explicit cast required
```

---

## The `tuple` Primitive Type

The `tuple` type in Solix is a built-in primitive representing an ordered, fixed-length heterogeneous collection of values.

```solix
tuple record = new tuple(42, "hello", true);
int32 element_count = record.length;              // 3
int32 first_val     = (int32)record[0];           // 42
String second_val   = (String)record[1];          // "hello"
bool third_val      = (bool)record[2];            // true
```

### Dynamic Operations on Tuples
- **Subscript Access**: `record[item_index]` retrieves the element at integer index `item_index`.
- **`instanceof`**: Evaluates whether an element matches a type (primitive or reference):
  ```solix
  if (record[0] instanceof int32) { ... }
  ```
- **Type Casting**: `(Type)record[item_index]` coerces the element to the given type, with runtime validation throwing `TypeCastException` on mismatch.
- **Automatic Cleanup**: Contained reference objects are managed by ARC and automatically destroyed via RAII when the tuple's ref count drops to zero.

