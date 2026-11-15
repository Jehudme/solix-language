# Primitive Characters Reference

The `char` type represents a single 32-bit Unicode scalar value (U+0000 to U+10FFFF):
* **Storage Width:** 32 bits (4 bytes).
* **Enclosure:** Single quotes `'...'`.
* **Default Value:** `'\0'` (Null character).

```solix
char latin_letter = 'A';
char greek_lambda = 'λ';
char section_mark = '§';
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
