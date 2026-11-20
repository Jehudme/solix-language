# Primitive Characters Reference

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

## Escape Sequences

* `\n`: Line feed (U+000A)
* `\r`: Carriage return (U+000D)
* `\t`: Horizontal tab (U+0009)
* `\\`: Backslash (U+005C)
* `\'`: Single quotation mark (U+0027)
* `\"`: Double quotation mark (U+0022)
* `\0`: Null character (U+0000)

## Further Reading

* [Built-in Primitives Index](README.md): Overview of all primitive types.
* [Booleans Reference](booleans.md): Truth values in Solix.
* [Variables & Types Guide](../../guide/02_variables_and_types.md): Working with types and Unicode strings.
