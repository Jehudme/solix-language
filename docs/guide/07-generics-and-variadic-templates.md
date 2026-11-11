# Guide 07: Generics & Variadic Templates

Generics allow you to write reusable, type-safe data structures and algorithms without sacrificing runtime performance.

---

## 1. Generic Classes

```solix
public class Stack<ItemType> {
    private ItemType[] storage_elements;
    private int32 top_index;

    public Stack(int32 initial_capacity) {
        this.storage_elements = new ItemType[initial_capacity];
        this.top_index = 0;
    }

    public void push(ItemType item) {
        this.storage_elements[this.top_index] = item;
        this.top_index = this.top_index + 1;
    }

    public ItemType pop() {
        this.top_index = this.top_index - 1;
        return this.storage_elements[this.top_index];
    }
}
```

---

## 2. Type Parameter Bounds

Restrict generic parameters using interface bounds (`: InterfaceName`):

```solix
extern import solix.core.IComparable;

public class Sorter {
    public static void sort<ItemType : IComparable<ItemType>>(ItemType[] items) {
        // Safe to call items[i].compare_to(items[j])
    }
}
```

---

## 3. Variadic Templates (`<args...>`)

Variadic templates accept an arbitrary number of heterogeneous arguments:

```solix
public class Formatter {
    public static void log_info<args...>(string category_name, args... parameters) {
        // Arguments are packed into structural tuples during lowering
    }
}
```

---

## 4. Cross-References

* **Specification:** [Generics & Variadic Templates](file:///home/jehud/Projects/solix-language/docs/spec/statements/generics-and-variadics.md)
* **Next Guide:** [Guide 08: Packages, Dependencies & Linking](file:///home/jehud/Projects/solix-language/docs/guide/08-packages-dependencies-and-linking.md)
