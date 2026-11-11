# Primitive Booleans & Characters Reference

---

## 1. Boolean (`bool`)

The `bool` type represents logical truth values:
* **Storage Width:** 8 bits (1 byte).
* **Values:** `true` and `false`.
* **Default Value:** `false`.
* **Friendly Alias:** `boolean`.

```solix
bool is_active = true;
bool has_completed = false;
```

---

## 2. Character (`char`)

The `char` type represents a single 32-bit Unicode scalar value (U+0000 to U+10FFFF):
* **Storage Width:** 32 bits (4 bytes).
* **Enclosure:** Single quotes `'...'`.
* **Default Value:** `'\0'` (Null character).

```solix
char latin_letter = 'A';
char greek_lambda = 'λ';
char emoji_symbol = '🚀';
char newline_char = '\n';
```
