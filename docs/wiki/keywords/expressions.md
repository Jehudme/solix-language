# Expression Keywords

This section documents keywords used within expressions and operator constructs:

* **`new`**: Allocates an object or array on the `VirtualHeap`.
* **`this`**: Self-referential pointer to the current class instance.
* **`super`**: Reference to the direct base class instance for constructor chaining and method overrides.
* **`instanceof`**: Tests whether an object conforms to a nominal class or interface contract.
* **`sizeof`**: Yields byte width of a type as `uint32`.
* **`default`**: Yields zero-initialized memory for a given type.
* **`cast`**: Explicit type conversion intrinsic (`cast(expr, TargetType)`).
* **`operator`**: Declares an operator overload method (`operator+`, `operator[]`).
* **`extends`**: Specifies the direct superclass of a class.
* **`implements`**: Specifies the interface contracts satisfied by a class.
* **`null`**: Literal representation of an uninitialized reference pointer.
* **`true` / `false`**: Canonical boolean literal values.
