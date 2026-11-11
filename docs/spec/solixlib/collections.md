# Formal Specification: Standard Library Collections (`solix.collections`)

## Document Metadata

* **Specification ID:** SPEC-0019
* **Scope:** Generic Data Structures, Collections & Algorithms
* **Status:** Final Normative Standard
* **Source Path:** [`libraries/solixlib/src/collections/`](file:///home/jehud/Projects/solix-language/libraries/solixlib/src/collections/)

---

## 1. Collection Interface Hierarchy

```text
IIterable<ElementType>
       │
       ▼
ICollection<ElementType>
       ├── IList<ElementType>
       ├── IDeque<ElementType>
       └── ISet<ElementType>

IMap<KeyType, ValueType>
```

---

## 2. Linear Containers

| Class | Internal Structure | Time Complexity (Access / Insert) | Best Use Case |
| :--- | :--- | :--- | :--- |
| `List<ElementType>` | Dynamic contiguous array | $O(1)$ random access / $O(1)$ amortized append | General-purpose linear collection |
| `LinkedList<ElementType>` | Doubly-linked node list | $O(N)$ access / $O(1)$ head & tail insertion | Frequent insertions/removals at boundaries |
| `Deque<ElementType>` | Circular ring-buffer array | $O(1)$ access / $O(1)$ prepend & append | Double-ended queues, sliding work buffers |
| `Queue<ElementType>` | FIFO queue adapter | $O(1)$ enqueue & dequeue | Producer-consumer queues |
| `Stack<ElementType>` | LIFO stack adapter | $O(1)$ push & pop | Evaluation stacks, backtrack state |
| `CircularBuffer<ElementType>`| Fixed-capacity ring buffer | $O(1)$ read & write with overwrite semantics | Audio streaming, telemetry logs |

---

## 3. Associative Maps & Sets

### 3.1 `HashMap<KeyType, ValueType>` & `HashSet<ElementType>`
* **Architecture:** Hash table with power-of-two capacity, secondary hash perturbation, and automated resizing at load factor $\ge 0.75$.
* **Key Requirements:** Keys must implement `IHashable` (`hash_code()`) and `IEquatable<KeyType>` (`equals()`).
* **Performance:** Expected $O(1)$ lookup, insertion, and deletion.

### 3.2 `TreeMap<KeyType, ValueType>` & `TreeSet<ElementType>`
* **Architecture:** Self-balancing Red-Black Tree.
* **Key Requirements:** Keys must implement `IComparable<KeyType>` (`compare_to()`).
* **Performance:** Guaranteed $O(\log N)$ lookup, insertion, and deletion. Iteration yields keys in strictly ascending sorted order.

---

## 4. Generic Algorithms (`solix.collections.Algorithms`)

* `static void sort<ElementType : IComparable<ElementType>>(ElementType[] items)`: Quicksort with median-of-three pivot selection.
* `static int32 binary_search<ElementType : IComparable<ElementType>>(ElementType[] items, ElementType target)`: Returns index of target or `-1`.
* `static void reverse<ElementType>(ElementType[] items)`: Reverses elements in-place in $O(N)$.
* `static void fill<ElementType>(ElementType[] items, ElementType value)`: Overwrites all slots with the specified value.
