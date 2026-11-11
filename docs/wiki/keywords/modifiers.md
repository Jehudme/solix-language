# Modifier Keywords

This section documents keywords modifying member visibility, dispatch, and storage:

* **`public`**: Unrestricted visibility across packages.
* **`private`**: Visible only within declaring class body.
* **`protected`**: Visible within declaring class and its subclasses.
* **`internal`**: Visible only within the declaring package boundary.
* **`static`**: Binds member to class scope rather than instances (accessed via `::`).
* **`const`**: Compile-time immutable constant value.
* **`virtual`**: Enables dynamic dispatch override via VTable.
* **`override`**: Explicitly overrides a virtual method declared in superclass.
* **`abstract`**: Declares an incomplete class or bodiless method requiring implementation.
* **`weak`**: Specifies a non-owning weak reference to prevent ARC reference cycles.
* **`native`**: Declares a method implemented in host native C/Rust shared libraries.
* **`inline`**: Designates a method as a candidate for AST-level call-site inlining.
* **`extern`**: Designates external dependency declarations resolved via package manager.
