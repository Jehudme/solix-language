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
