# Primitive Booleans Reference

The `bool` type represents logical truth values:
* **Storage Width:** 8 bits (1 byte).
* **Values:** `true` and `false`.
* **Default Value:** `false`.
* **Friendly Alias:** `boolean`.

```solix
bool is_active = true;
bool has_completed = false;
```

## Logical Operators

Booleans support standard logical operations:
* `&&`: Short-circuit logical AND.
* `||`: Short-circuit logical OR.
* `!`: Logical NOT.

## Further Reading

* [Built-in Primitives Index](README.md): Full listing of scalar and composite primitives.
* [Characters Reference](chars.md): Unicode character handling.
* [Control Flow Guide](../../guide/03_control_flow.md): Boolean conditions in loops and branching.
