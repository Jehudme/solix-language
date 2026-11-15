# Lexical Error Codes (`E0000` – `E0015`)

This section documents the lexical analysis diagnostics emitted during tokenization by [`crates/solix-compilation/src/phases/lexing/lexer.rs`](file:///home/jehud/Projects/solix-language/crates/solix-compilation/src/phases/lexing/lexer.rs).

---

### `E0001`: Unterminated String Literal
* **Cause:** A string literal begins with a double quote `"` but reaches the end of the line or file without a closing quote.
* **Bad Code:**
  ```solix
  string message = "hello world;
  ```
* **Fixed Code:**
  ```solix
  string message = "hello world";
  ```

---

### `E0002`: Invalid Number Literal
* **Cause:** A number literal has malformed digits for its base, multiple decimal points, or improper underscore placement.
* **Bad Code:**
  ```solix
  int32 binary_value = 0b1020;
  float64 malformed = 3.14.15;
  ```
* **Fixed Code:**
  ```solix
  int32 binary_value = 0b1010;
  float64 valid_float = 3.1415;
  ```

---

### `E0003`: Invalid Character Escape Sequence
* **Cause:** A backslash in a string or character literal is followed by an unrecognized escape character.
* **Bad Code:**
  ```solix
  string path = "C:\programs\tools";
  ```
* **Fixed Code:**
  ```solix
  string path = "C:\\programs\\tools";
  ```

---

### `E0004`: Unexpected Character
* **Cause:** The source text contains an invalid character or non-Unicode byte that cannot form any valid token.
* **Bad Code:**
  ```solix
  int32 value = 10 @ 20;
  ```
* **Fixed Code:**
  ```solix
  int32 value = 10 + 20;
  ```

---

### `E0005`: Unterminated Multi-Line Comment
* **Cause:** A delimited comment `/*` is opened but never closed with `*/` before reaching the end of the file.
* **Bad Code:**
  ```solix
  /* Comment started here
  public int32 main() { return 0; }
  ```
* **Fixed Code:**
  ```solix
  /* Comment started here */
  public int32 main() { return 0; }
  ```
