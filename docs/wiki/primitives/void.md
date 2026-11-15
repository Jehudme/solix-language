# Primitive Void Reference

The `void` type signifies the absence of a value:
* **Function Returns:** Indicates that a function returns no value (`void main()`).
* **Invariants:** Cannot be used as a variable type, struct field, or generic type argument (`void x;` is compile error `E0245`).

```solix
public void log_event(string message) {
    solix::io::Console::println(message);
}
```

## Further Reading

* [Built-in Primitives Index](README.md): Overview of all primitive types.
* [Tuples Reference](tuples.md): Structural composite values.
* [Functions Guide](../../guide/07_functions_and_lambdas.md): Function declarations and return types.
