# Formal Specification: Declarations & OOP Members

## Document Metadata

* **Specification ID:** SPEC-0013
* **Scope:** Object-Oriented Declarations, Modifiers & Operator Overloads (`crates/solix-compilation/src/phases/parser/statements/declarations.rs`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)

---

## 1. Syntax & Grammar

```ebnf
ClassDeclaration     ::= [Modifiers] "class" Identifier [TypeParameters]
                         [ "extends" Type ] [ "implements" TypeList ] "{" { ClassMember } "}"
InterfaceDeclaration ::= [Modifiers] "interface" Identifier [TypeParameters]
                         [ "implements" TypeList ] "{" { InterfaceMember } "}"
EnumDeclaration      ::= [Modifiers] "enum" Identifier "{" EnumVariantList "}"
```

---

## 2. Access & Member Modifiers

| Modifier | Target | Semantics |
| :--- | :--- | :--- |
| `public` | Class, Method, Field | Unrestricted visibility across packages |
| `private` | Method, Field | Accessible only within the declaring class body |
| `protected` | Method, Field | Accessible within declaring class and subclasses |
| `internal` | Class, Method, Field | Accessible only within the declaring package |
| `static` | Method, Field | Belongs to class scope; accessed via `::` |
| `const` | Field | Compile-time immutable constant value |
| `virtual` | Method | Overridable via dynamic VTable dispatch |
| `override` | Method | Explicitly overrides a virtual method from superclass |
| `abstract` | Class, Method | Incomplete declaration requiring subclass implementation |
| `weak` | Field | Weak reference preventing ARC ownership cycles |
| `native` | Method | Implemented in host native shared library (`natives/`) |
| `inline` | Method | Candidate for AST-level inlining at call sites |

---

## 3. Destructors (`~ClassName()`) & RAII Lifecycle

Classes may declare an explicit destructor without access modifiers:

```solix
public class FileHandle {
    private int32 descriptor;

    public FileHandle(int32 initial_descriptor) {
        this.descriptor = initial_descriptor;
    }

    ~FileHandle() {
        if (this.descriptor >= 0) {
            close_descriptor(this.descriptor);
            this.descriptor = -1;
        }
    }
}
```

* **Modifier Invariant:** Attaching access modifiers to a destructor (e.g. `public ~FileHandle()`) is a syntax error (`E0108`).
* **Execution Trigger:** Destructors are invoked deterministically by the runtime when an object's strong reference count drops to zero.

---

## 4. Operator Overloading

Classes can overload canonical operators by defining method signatures with the `operator` keyword:

```solix
public class Vector2D {
    public float64 horizontal;
    public float64 vertical;

    public Vector2D operator+(Vector2D operand) {
        return new Vector2D(this.horizontal + operand.horizontal, this.vertical + operand.vertical);
    }

    public bool operator==(Vector2D operand) {
        return this.horizontal == operand.horizontal && this.vertical == operand.vertical;
    }

    public float64 operator[](int32 index) {
        if (index == 0) return this.horizontal;
        return this.vertical;
    }
}
```

Overloaded operators are invoked seamlessly via normal infix syntax (`vector_a + vector_b`, `vector_a[0]`).
