# Binder & Semantic Error Codes (`E0201` – `E0298`)

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

---

### `E0297`: Unsatisfied Generic Type Constraint
* **Cause:** A generic type argument does not implement or inherit the declared bound.
* **Fix:** Supply a type that satisfies the constraint. For example, a type argument for `T extends IPrintable`
  must implement or inherit `IPrintable`.

---

### `E0298`: Cannot Throw Null Literal
* **Cause:** A `throw` statement directly specifies `null` or an expression statically typed as `null`. Throw expressions must evaluate to an instance of an `Exception` class.
* **Bad Code:**
  ```solix
  void faulty() {
      throw null;
  }
  ```
* **Fixed Code:**
  ```solix
  class CustomException extends Exception {}

  void faulty() {
      throw new CustomException();
  }
  ```

---

### `E0204`: Cyclic Interface or Class Inheritance
* **Cause:** An inheritance cycle was detected in class or interface `extends` declarations (e.g. `interface A extends B` and `interface B extends A`).
* **Bad Code:**
  ```solix
  interface Alpha extends Beta {}
  interface Beta extends Alpha {}
  ```
* **Fixed Code:**
  ```solix
  interface Alpha {}
  interface Beta extends Alpha {}
  ```

---

### `E0264`: Abstract Method in Non-Abstract Class
* **Cause:** A method was declared `abstract` inside a non-abstract class declaration. Only abstract classes or interfaces may declare abstract methods.
* **Bad Code:**
  ```solix
  class Worker {
      public abstract void execute();
  }
  ```
* **Fixed Code:**
  ```solix
  abstract class Worker {
      public abstract void execute();
  }
  ```

---

### `E0297`: Generic Type Constraint Violation
* **Cause:** A type argument supplied for a generic parameter does not satisfy the declared constraint or base class bound.
* **Bad Code:**
  ```solix
  class Container<T extends Component> {}
  Container<int32> container;
  ```
* **Fixed Code:**
  ```solix
  class Container<T extends Component> {}
  Container<Widget> container;
  ```

---

### `E0299`: Negative Array Length
* **Cause:** An array creation expression specifies a compile-time constant negative dimension. Array dimensions must evaluate to non-negative integers.
* **Bad Code:**
  ```solix
  int32[] buffer = new int32[-10];
  ```
* **Fixed Code:**
  ```solix
  int32[] buffer = new int32[10];
  ```

---

### `E0301`: Assertion Message Expression Not String
* **Cause:** The custom failure message in an `assert` statement (`assert condition : message;`) does not evaluate to type `string`.
* **Bad Code:**
  ```solix
  assert value > 0 : 404;
  ```
* **Fixed Code:**
  ```solix
  assert value > 0 : "Value must be positive";
  ```
