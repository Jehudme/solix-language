# Primitive Anonymous Tuples Reference

Tuples are anonymous, strongly typed structural composite values combining heterogeneous fields:

```solix
(int32, bool) result_tuple = (200, true);

// Positional Member Access
int32 status_code = result_tuple.0;
bool is_successful = result_tuple.1;
```

## Characteristics

* **Value Semantics:** Tuples are passed by value and laid out contiguously.
* **Heterogeneous Elements:** Tuple elements may be of any valid type (primitives or reference objects).
* **Positional Indexing:** Fields are accessed via zero-based numeric field selectors (`.0`, `.1`, `.2`).

## Further Reading

* [Built-in Primitives Index](README.md): Overview of all primitive types.
* [Void Reference](void.md): Absence of value.
* [Variables & Types Guide](../../guide/02_variables_and_types.md): Composite types in Solix.
