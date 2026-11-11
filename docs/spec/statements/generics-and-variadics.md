# Formal Specification: Generics, Bounds & Variadic Templates

## Document Metadata

* **Specification ID:** SPEC-0014
* **Scope:** Parametric Polymorphism, Monomorphization & Variadics (`crates/solix-compilation/src/phases/binder/`)
* **Status:** Final Normative Standard
* **Governing Guideline:** `docs/internals/guidelines/authoring-specs.md` (GL-0005)

---

## 1. Syntax & Grammar

```ebnf
TypeParameters  ::= "<" TypeParameter ( "," TypeParameter )* ">"
TypeParameter   ::= Identifier [ ":" TypeBound ] | VariadicParameter
VariadicParameter ::= Identifier "..."
TypeBound       ::= Type ( "+" Type )*
```

---

## 2. Generic Classes & Interfaces

Generic declarations define parametric blueprints:

```solix
public interface ICollection<ElementType> {
    public int32 size();
    public void add(ElementType item);
}

public class ArrayList<ElementType> implements ICollection<ElementType> {
    private ElementType[] storage;
    private int32 count;

    public void add(ElementType item) {
        this.storage[this.count] = item;
        this.count = this.count + 1;
    }
}
```

---

## 3. Type Parameter Bounds

Type parameter bounds restrict argument types to those satisfying specific interface contracts:

```solix
public class Algorithms {
    public static void sort<ElementType : IComparable<ElementType>>(ElementType[] items) {
        // Monomorphized sorting algorithm guaranteed to have compare_to()
    }
}
```

Attempting to instantiate a generic template with a type that does not satisfy the declared bound emits diagnostic `E0212`.

---

## 4. Variadic Templates (`<args...>`)

Solix supports variadic parameter packs for type-safe arbitrary-argument methods:

```solix
public class Console {
    public static void print_formatted<args...>(string template_text, args... arguments) {
        // Lowered via tuple packing and formatting intrinsics
    }
}
```

### Monomorphization & Blueprint Storage
1. **Source Preservation:** Generic classes and variadic methods preserve their original source code in `ExportManifest.generic_templates`.
2. **On-Demand Specialization:** When an external consumer references a generic template from an installed package, the compiler specializes the blueprint with the consumer's concrete types and compiles it directly into the target binary.
