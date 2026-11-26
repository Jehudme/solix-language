# Lexical Error Codes (`E0000`, `E0011` – `E0018`)

This section documents all lexical analysis diagnostics emitted during tokenization by [`crates/solix-compilation/src/phases/lexing/lexer.rs`](../../../crates/solix-compilation/src/phases/lexing/lexer.rs) and defined in [`LexingError`](../../../crates/solix-compilation/src/diagnostics/lexing.rs).

---

### `E0000`: Unknown Token
* **Cause:** Unknown or unrecognized token.
* **Help:** Remove or replace this unrecognized character with valid Solix syntax.
* **Bad Code:**
  ```solix
  int32 value = 10 @ 20;
  ```
* **Fixed Code:**
  ```solix
  int32 value = 10 + 20;
  ```

---

### `E0011`: Unterminated String Literal
* **Cause:** Unterminated string literal.
* **Help:** Add a closing quote '"' to terminate the string literal before end of line or file.
* **Bad Code:**
  ```solix
  string message = "hello world;
  ```
* **Fixed Code:**
  ```solix
  string message = "hello world";
  ```

---

### `E0012`: Unterminated Character Literal
* **Cause:** Unterminated character literal.
* **Help:** Add a closing quote '\\'' to terminate the character literal before end of line.
* **Bad Code:**
  ```solix
  char letter = 'a;
  ```
* **Fixed Code:**
  ```solix
  char letter = 'a';
  ```

---

### `E0013`: Unterminated Block Comment
* **Cause:** Unterminated block comment.
* **Help:** Add a closing '*/' before end of file.
* **Bad Code:**
  ```solix
  /* Comment started here
  public static int32 main() { return 0; }
  ```
* **Fixed Code:**
  ```solix
  /* Comment started here */
  public static int32 main() { return 0; }
  ```

---

### `E0014`: Malformed Hex Literal
* **Cause:** Malformed hexadecimal integer literal.
* **Help:** Hexadecimal literals must have one or more hex digits (0-9, a-f) after '0x'.
* **Bad Code:**
  ```solix
  int32 mask = 0x;
  ```
* **Fixed Code:**
  ```solix
  int32 mask = 0xFF;
  ```

---

### `E0015`: Malformed Binary Literal
* **Cause:** Malformed binary integer literal.
* **Help:** Binary literals must have one or more binary digits (0 or 1) after '0b'.
* **Bad Code:**
  ```solix
  int32 bits = 0b;
  ```
* **Fixed Code:**
  ```solix
  int32 bits = 0b1010;
  ```

---

### `E0016`: Leading Zero Integer Literal
* **Cause:** Decimal integer literal cannot have leading zeros.
* **Help:** Remove the leading zero or use '0x' for hexadecimal or '0b' for binary.
* **Bad Code:**
  ```solix
  int32 counter_value = 0123;
  ```
* **Fixed Code:**
  ```solix
  int32 counter_value = 123;
  ```

---

### `E0017`: Malformed Numeric Literal
* **Cause:** Numeric literal contains invalid underscore separators (leading after prefix, trailing, or consecutive underscores) or an exponent (`e`/`E`) without any exponent digits.
* **Help:** Numeric literals cannot have leading, trailing, or consecutive underscores, and exponents require at least one digit.
* **Bad Code:**
  ```solix
  int32 bad_sep = 1__0;
  float64 bad_exp = 1.0e+;
  ```
* **Fixed Code:**
  ```solix
  int32 good_sep = 1_000;
  float64 good_exp = 1.0e+2;
  ```

---

### `E0018`: Compilation Source File Limit Exceeded
* **Cause:** More than 65,536 source files were registered in a single compilation context (`Span::id` `u16` address space).
* **Help:** Reduce the number of source files or consolidate modules to stay within the 65,536-source index limit.
