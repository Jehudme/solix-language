# Primitive Strings, Tuples & Void Reference

---

## 1. Primitive String (`string`)

The `string` primitive represents an immutable UTF-8 heap string allocated directly on the `VirtualHeap`:
* **Storage Width:** 64-bit object reference pointer (managed by ARC).
* **Literals:** Double quotes `"..."` or triple quotes `"""..."""` (raw multi-line).
* **Interoperability:** Seamlessly converts to and from standard library `solix.core.String`.

```solix
string greeting = "Hello, Solix!";
string combined = greeting + " Welcome.";
```

---

## 2. Anonymous Tuples (`tuple`)

Tuples are anonymous, strongly typed structural composite values combining heterogeneous fields:

```solix
(int32, string, bool) result_tuple = (200, "OK", true);

// Positional Member Access
int32 status_code = result_tuple.0;
string status_message = result_tuple.1;
bool is_successful = result_tuple.2;
```

---

## 3. Void (`void`)

The `void` type signifies the absence of a value:
* **Function Returns:** Indicates that a function returns no value (`void main()`).
* **Invariants:** Cannot be used as a variable type, struct field, or generic type argument (`void x` is compile error `E0245`).
