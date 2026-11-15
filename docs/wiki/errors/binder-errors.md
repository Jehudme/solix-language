# Binder & Semantic Error Codes (`E0201` – `E0290`)

This section documents semantic binding diagnostics emitted by [`crates/solix-compilation/src/phases/binder/`](../../../crates/solix-compilation/src/phases/binder).

---

### `E0201`: Undefined Symbol
* **Cause:** A variable, function, class, or method name is referenced without prior declaration or import.
* **Bad Code:**
  ```solix
  int32 total = undefined_counter + 5;
  ```
* **Fixed Code:**
  ```solix
  int32 undefined_counter = 10;
  int32 total = undefined_counter + 5;
  ```

---

### `E0203`: Logical Condition Not Boolean
* **Cause:** An `if`, `while`, or `do`/`while` condition expression evaluated to a non-boolean type.
* **Bad Code:**
  ```solix
  int32 count = 5;
  if (count) { }
  ```
* **Fixed Code:**
  ```solix
  int32 count = 5;
  if (count > 0) { }
  ```

---

### `E0215`: Type Mismatch in Assignment
* **Cause:** Attempted to assign a value of an incompatible type to a variable or field without an explicit cast.
* **Bad Code:**
  ```solix
  int32 number = "hello";
  ```
* **Fixed Code:**
  ```solix
  string message = "hello";
  ```

---

### `E0226`: Static Access Via Dot Operator
* **Cause:** A static method or field was accessed using the instance member access operator (`.`) instead of scope resolution (`::`).
* **Bad Code:**
  ```solix
  import solix.math.Math;
  float64 root = Math.sqrt(16.0);
  ```
* **Fixed Code:**
  ```solix
  import solix.math.Math;
  float64 root = Math::sqrt(16.0);
  ```

---

### `E0233`: Duplicate Local Variable
* **Cause:** A local variable was declared with a name already in use within the current block scope.
* **Bad Code:**
  ```solix
  int32 index = 0;
  int32 index = 1;
  ```
* **Fixed Code:**
  ```solix
  int32 first_index = 0;
  int32 second_index = 1;
  ```

---

### `E0238`: Argument Count Mismatch
* **Cause:** A function or constructor was invoked with a different number of arguments than its declared parameter list.
* **Bad Code:**
  ```solix
  int32 add(int32 first, int32 second) { return first + second; }
  add(1);
  ```
* **Fixed Code:**
  ```solix
  add(1, 2);
  ```

---

### `E0288`: Bitwise Operator on Non-Integer
* **Cause:** A bitwise operator (`&`, `|`, `^`, `<<`, `>>`) was applied to a floating-point or reference type.
* **Bad Code:**
  ```solix
  float64 mask = 3.14 & 1.0;
  ```
* **Fixed Code:**
  ```solix
  int32 mask = 14 & 1;
  ```

---

### `E0291`: Dependency Checksum Mismatch
* **Cause:** A resolved package artifact's SHA-256 digest does not match the pinned checksum recorded at compile time.
* **Fix:** Rebuild the dependency or compile with `--no-verify` (`--ignore-checksum`) if intentionally hot-swapping code.
