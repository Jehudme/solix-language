# Guide 02: Variables, Primitives & Strings

This guide explores Solix's static type system, fixed-width integer and floating-point primitives, array buffers, structural tuples, and strings.

---

## 1. Explicit Variables & Primitive Types

Solix requires explicit type annotations on variable declarations:

```solix
// Signed Integers
int8  small_byte = 127;
int16 audio_sample = 32000;
int32 user_id = 10042;
int64 epoch_nanos = 1762809000000000L;

// Unsigned Integers
uint8  pixel_alpha = 255;
uint32 memory_address = 0x8000_0000U;

// Floating-Point Numbers
float32 coordinate_x = 12.5f;
float64 high_precision = 3.141592653589793;

// Logical and Unicode Scalar
bool is_authenticated = true;
char status_glyph = '✓';
```

---

## 2. Fixed Arrays & Slices

Arrays in Solix represent contiguous heap-allocated buffers:

```solix
// Allocation with explicit size
int32[] numbers = new int32[5];

// Indexing and Modification
numbers[0] = 10;
numbers[1] = 20;

int32 first_item = numbers[0];
```

---

## 3. Structural Tuples

Tuples combine heterogeneous values without requiring a nominal class declaration:

```solix
// Declaring a 3-element tuple
(int32, string, bool) response_record = (200, "OK", true);

// Reading fields by positional index
int32 status_code = response_record.0;
string status_text = response_record.1;
bool is_success = response_record.2;
```

---

## 4. Primitive `string` vs `solix.core.String`

Solix provides two interoperable string types:
1. **Primitive `string`:** The built-in heap string primitive. Efficient, lightweight, and supports string concatenation via `+`:
   ```solix
   string first_name = "Alice";
   string full_greeting = "Hello, " + first_name;
   ```
2. **Standard Library `String` (`solix.core.String`):** A rich object wrapper providing manipulation methods (`substring`, `trim`, `replace`, `split`, `format`):
   ```solix
   extern import solix.core.String;

   String rich_text = new String("  solix language  ");
   String cleaned = rich_text.trim().to_upper();
   ```

---

## 5. Cross-References

* **Specification:** [Type System & Semantics](file:///home/jehud/Projects/solix-language/docs/spec/type-system-and-semantics.md)
* **Wiki Reference:** [Built-in Primitives](file:///home/jehud/Projects/solix-language/docs/wiki/primitives/README.md)
* **Next Guide:** [Guide 03: Control Flow & Functions](file:///home/jehud/Projects/solix-language/docs/guide/03-control-flow-and-functions.md)
