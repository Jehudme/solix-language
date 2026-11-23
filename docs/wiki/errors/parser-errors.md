# Parser Error Codes (`E0101` – `E0200`)

This section documents the syntax diagnostics emitted by [`crates/solix-compilation/src/phases/parser/`](../../../crates/solix-compilation/src/phases/parser).

---

### `E0101`: Expected Semicolon
* **Cause:** A statement is missing a mandatory terminating semicolon `;`.
* **Bad Code:**
  ```solix
  int32 total = 42
  ```
* **Fixed Code:**
  ```solix
  int32 total = 42;
  ```

---

### `E0102`: Expected Identifier
* **Cause:** The parser expected a user-defined name but encountered a keyword, symbol, or delimiter.
* **Bad Code:**
  ```solix
  int32 class = 10;
  ```
* **Fixed Code:**
  ```solix
  int32 class_count = 10;
  ```

---

### `E0104`: Expected Closing Delimiter
* **Cause:** An opening parenthesis `(`, brace `{`, bracket `[`, or angle bracket `<` lacks its matching closing token.
* **Bad Code:**
  ```solix
  if (value > 10 {
      do_something();
  }
  ```
* **Fixed Code:**
  ```solix
  if (value > 10) {
      do_something();
  }
  ```

---

### `E0108`: Invalid Destructor Modifier
* **Cause:** A destructor declaration (`~ClassName()`) was declared with visibility or member modifiers.
* **Bad Code:**
  ```solix
  public class Connection {
      public ~Connection() { }
  }
  ```
* **Fixed Code:**
  ```solix
  public class Connection {
      ~Connection() { }
  }
  ```

---

### `E0111`: Package Statement Must Be First
* **Cause:** A `package` statement appeared after import statements or class declarations.
* **Bad Code:**
  ```solix
  import solix.io.Console;
  package my.app;
  ```
* **Fixed Code:**
  ```solix
  package my.app;
  import solix.io.Console;
  ```

---

### `E0113`: Import Statement After Declaration
* **Cause:** An `import` statement appeared after a class, interface, or function declaration.
* **Bad Code:**
  ```solix
  public class Application { }
  import solix.io.Console;
  ```
* **Fixed Code:**
  ```solix
  import solix.io.Console;
  public class Application { }
  ```

---

### `E0148`: Expression Nesting Depth Exceeded
* **Cause:** An expression exceeds the maximum parser recursion depth ceiling (32 levels of nested sub-expressions or parenthesized clauses).
* **Bad Code:**
  ```solix
  int32 result = (((((((((((((((((((((((((((((((((1))))))))))))))))))))))))))))))));
  ```
* **Fixed Code:**
  ```solix
  int32 intermediate_value = 1;
  int32 result = intermediate_value;
  ```

---

### `E0149`: Required Parameter After Default Parameter
* **Cause:** A required formal parameter follows an optional parameter with a default initializer in a function or method signature.
* **Bad Code:**
  ```solix
  public static int32 compute(int32 base_multiplier = 2, int32 value_operand) {
      return base_multiplier * value_operand;
  }
  ```
* **Fixed Code:**
  ```solix
  public static int32 compute(int32 value_operand, int32 base_multiplier = 2) {
      return value_operand * base_multiplier;
  }
  ```

---

### `E0153`: Multi-Character Character Literal
* **Cause:** A character literal contains more than a single character or has extra trailing characters after an escape sequence.
* **Bad Code:**
  ```solix
  char invalid_letter = 'ab';
  char invalid_escape = '\nabc';
  ```
* **Fixed Code:**
  ```solix
  char valid_letter = 'a';
  char valid_escape = '\n';
  ```
