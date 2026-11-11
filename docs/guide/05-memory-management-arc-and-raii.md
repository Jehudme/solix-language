# Guide 05: Memory Management, ARC & RAII

Solix delivers memory safety and deterministic reclamation without a tracing garbage collector by pairing Automatic Reference Counting (ARC) with scope-based RAII destructors.

---

## 1. How ARC Works in Solix

When an object is allocated via `new ClassName()`, its `strong_ref_count` is initialized to `1`.
* Passing an object to a function or assigning it to another variable increments its reference count (`RetainRef`).
* When a local variable exits its lexical scope, the runtime automatically emits a release instruction (`ReleaseRef`).
* When an object's strong reference count drops to zero, the object is destroyed immediately: its destructor (`~ClassName()`) runs and memory is returned to the `VirtualHeap`.

---

## 2. RAII Destructors (`~ClassName()`)

Destructors are ideal for releasing operating system resources, closing file handles, or freeing native buffers:

```solix
public class ScopedLock {
    private int32 lock_id;

    public ScopedLock(int32 initial_id) {
        this.lock_id = initial_id;
        acquire_native_lock(this.lock_id);
    }

    ~ScopedLock() {
        release_native_lock(this.lock_id);
        Console::println("Lock released deterministically.");
    }
}

public void critical_section() {
    ScopedLock lock = new ScopedLock(101);
    perform_concurrent_work();
    // lock goes out of scope here: ~ScopedLock() runs immediately!
}
```

---

## 3. Breaking Reference Cycles with `weak`

Circular strong references prevent reference counts from ever reaching zero, causing memory leaks:

```text
Node A ────── strong reference ─────► Node B
Node A ◄───── strong reference ────── Node B  (LEAK!)
```

Use the `weak` modifier to declare non-owning references:

```solix
public class TreeNode {
    public TreeNode left_child;
    public TreeNode right_child;
    public weak TreeNode parent_node; // Weak reference breaks cycle!

    public TreeNode() { }
}
```

---

## 4. Cross-References

* **Specification:** [Deterministic Memory Model & ARC](file:///home/jehud/Projects/solix-language/docs/spec/memory-model-and-arc.md)
* **Next Guide:** [Guide 06: Error Handling & Assertions](file:///home/jehud/Projects/solix-language/docs/guide/06-error-handling-and-assertions.md)
